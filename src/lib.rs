use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

// --- ЛОГИРОВАНИЕ ---
extern crate web_sys;
use web_sys::console;
macro_rules! log { ( $( $t:tt )* ) => { console::log_1(&format!( $( $t )* ).into()); } }

// --- ОБЩИЕ СТРУКТУРЫ ---
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point { x: i32, y: i32 }

// --- КОНСТАНТЫ ТИПОВ ЗОН (для Rust) ---
const ZONE_TYPE_NORMAL: i32 = 0;
const ZONE_TYPE_ONEWAY_UP: i32 = 1;


// ================================================================================= //
// ===== АЛГОРИТМ 1: ПОИСК ПУТИ НА 2D-СЕТКЕ (GRID) ================================= //
// ================================================================================= //

struct GridNode { position: Point, g: i32, h: i32 }
impl GridNode { fn f(&self) -> i32 { self.g + self.h } }
impl Ord for GridNode { fn cmp(&self, other: &Self) -> Ordering { other.f().cmp(&self.f()) } }
impl PartialOrd for GridNode { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl PartialEq for GridNode { fn eq(&self, other: &Self) -> bool { self.f() == other.f() } }
impl Eq for GridNode {}

fn heuristic_grid(a: Point, b: Point) -> i32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) * 10
}

fn reconstruct_path_grid(parents: HashMap<Point, Point>, current: Point) -> Vec<Point> {
    let mut total_path = vec![current];
    let mut temp = current;
    while let Some(parent) = parents.get(&temp) {
        total_path.push(*parent);
        temp = *parent;
    }
    total_path.reverse();
    total_path
}

fn find_path_grid(
    start: Point,
    goal: Point,
    costs: &HashMap<Point, i32>,
    zone_types: &HashMap<Point, i32>,
    teleporters: &HashMap<Point, Point>
) -> Option<Vec<Point>> {
    log!("[Rust/Grid] Запуск поиска на сетке.");
    let mut open_list = BinaryHeap::new();
    let mut g_scores = HashMap::new();
    let mut parents: HashMap<Point, Point> = HashMap::new();

    g_scores.insert(start, 0);
    open_list.push(GridNode { position: start, g: 0, h: heuristic_grid(start, goal) });

    while let Some(current_node) = open_list.pop() {
        let current_pos = current_node.position;
        let current_g = current_node.g;

        if current_g > *g_scores.get(&current_pos).unwrap_or(&i32::MAX) { continue; }
        if current_pos == goal {
            log!("[Rust/Grid] Цель достигнута!");
            return Some(reconstruct_path_grid(parents, current_pos));
        }

        if let Some(exit_pos) = teleporters.get(&current_pos) {
            let tentative_g_score = current_g + 1;
            if tentative_g_score < *g_scores.get(exit_pos).unwrap_or(&i32::MAX) {
                log!("[Rust/Grid] Найден путь через телепорт из {:?} в {:?}", current_pos, exit_pos);
                parents.insert(*exit_pos, current_pos);
                g_scores.insert(*exit_pos, tentative_g_score);
                open_list.push(GridNode { position: *exit_pos, g: tentative_g_score, h: heuristic_grid(*exit_pos, goal) });
            }
        }

        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter() {
            let neighbor_pos = Point { x: current_pos.x + dx, y: current_pos.y + dy };
            let neighbor_zone_type = *zone_types.get(&neighbor_pos).unwrap_or(&ZONE_TYPE_NORMAL);
            if neighbor_zone_type == ZONE_TYPE_ONEWAY_UP && *dy != 1 {
                continue;
            }

            let move_cost = *costs.get(&neighbor_pos).unwrap_or(&10);
            if move_cost == i32::MAX { continue; }

            let tentative_g_score = current_g + move_cost;
            if tentative_g_score < *g_scores.get(&neighbor_pos).unwrap_or(&i32::MAX) {
                parents.insert(neighbor_pos, current_pos);
                g_scores.insert(neighbor_pos, tentative_g_score);
                open_list.push(GridNode { position: neighbor_pos, g: tentative_g_score, h: heuristic_grid(neighbor_pos, goal) });
            }
        }
    }
    log!("[Rust/Grid] Путь не найден.");
    None
}

// ================================================================================= //
// ===== АЛГОРИТМ 2: ПОИСК ПУТИ С УЧЁТОМ ФИЗИКИ (SPACE) ============================ //
// ================================================================================= //

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PhysicsState { x: i32, y: i32, vx: i32, vy: i32 }

struct PhysicsNode { state: PhysicsState, g: i32, h: i32 }
impl PhysicsNode { fn f(&self) -> i32 { self.g + self.h } }
impl Ord for PhysicsNode { fn cmp(&self, other: &Self) -> Ordering { other.f().cmp(&self.f()) } }
impl PartialOrd for PhysicsNode { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl PartialEq for PhysicsNode { fn eq(&self, other: &Self) -> bool { self.f() == other.f() } }
impl Eq for PhysicsNode {}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct PhysicsParams {
    pub gravity_y: i32,
    pub jump_impulse_y: i32,
    pub max_velocity_y: i32,
    pub time_step_ms: i32,
    pub velocity_granularity: i32,
}

enum Action { Wait, Jump }

fn apply_physics(state: PhysicsState, action: &Action, params: &PhysicsParams, grid_size: i32) -> PhysicsState {
    let mut next_vy = state.vy;
    if let Action::Jump = action { next_vy += params.jump_impulse_y; }
    next_vy += (params.gravity_y * params.time_step_ms) / 1000;
    next_vy = next_vy.clamp(-params.max_velocity_y, params.max_velocity_y);
    let avg_vy = (state.vy + next_vy) / 2;
    let next_y = state.y + (avg_vy * params.time_step_ms) / (1000 * grid_size);
    let gran = params.velocity_granularity;
    let rounded_vy = (next_vy as f32 / gran as f32).round() as i32 * gran;
    PhysicsState { x: state.x, y: next_y, vx: 0, vy: rounded_vy }
}

fn heuristic_physics(state: PhysicsState, goal: Point, params: &PhysicsParams) -> i32 {
    let dy = (goal.y - state.y).abs();
    if params.max_velocity_y == 0 { return i32::MAX; }
    (dy * 1000) / params.max_velocity_y
}

fn find_path_physics(
    start_state: PhysicsState,
    goal_pos: Point,
    params: &PhysicsParams,
    obstacles: &HashMap<Point, i32>,
    grid_size: i32,
) -> Option<Vec<Point>> {
    log!("[Rust/Physics] Запуск поиска с учётом физики.");
    let mut open_list = BinaryHeap::new();
    let mut g_scores = HashMap::new();
    let mut parents: HashMap<PhysicsState, PhysicsState> = HashMap::new();

    g_scores.insert(start_state, 0);
    open_list.push(PhysicsNode { state: start_state, g: 0, h: heuristic_physics(start_state, goal_pos, params) });

    while let Some(current_node) = open_list.pop() {
        let current_state = current_node.state;
        let current_g = current_node.g;

        if current_g > *g_scores.get(&current_state).unwrap_or(&i32::MAX) { continue; }
        if current_state.x == goal_pos.x && current_state.y == goal_pos.y {
            log!("[Rust/Physics] Цель достигнута!");
            let mut path = vec![Point{x: current_state.x, y: current_state.y}];
            let mut temp = current_state;
            while let Some(parent_state) = parents.get(&temp) {
                path.push(Point{x: parent_state.x, y: parent_state.y});
                temp = *parent_state;
            }
            path.reverse();
            return Some(path);
        }

        for action in [Action::Wait, Action::Jump].iter() {
            let next_state = apply_physics(current_state, action, params, grid_size);
            if obstacles.contains_key(&Point{x: next_state.x, y: next_state.y}) { continue; }
            let tentative_g_score = current_g + params.time_step_ms;
            if tentative_g_score < *g_scores.get(&next_state).unwrap_or(&i32::MAX) {
                parents.insert(next_state, current_state);
                g_scores.insert(next_state, tentative_g_score);
                open_list.push(PhysicsNode {
                    state: next_state,
                    g: tentative_g_score,
                    h: heuristic_physics(next_state, goal_pos, params),
                });
            }
        }
    }
    log!("[Rust/Physics] Путь не найден.");
    None
}

// ================================================================================= //
// ===== WASM-ОБВЯЗКИ ДЛЯ JAVASCRIPT ============================================== //
// ================================================================================= //

#[wasm_bindgen]
pub fn find_path_on_grid_wasm(
    start_x: i32, start_y: i32,
    goal_x: i32, goal_y: i32,
    costs_flat: &[i32],
    zone_types_flat: &[i32],
    teleporters_flat: &[i32],
    _player_skill: &str,
) -> Vec<i32> {
    let start = Point { x: start_x, y: start_y };
    let goal = Point { x: goal_x, y: goal_y };

    let mut costs = HashMap::new();
    for chunk in costs_flat.chunks_exact(3) { costs.insert(Point { x: chunk[0], y: chunk[1] }, chunk[2]); }

    let mut zone_types = HashMap::new();
    for chunk in zone_types_flat.chunks_exact(3) { zone_types.insert(Point { x: chunk[0], y: chunk[1] }, chunk[2]); }

    let mut teleporters = HashMap::new();
    for chunk in teleporters_flat.chunks_exact(4) { teleporters.insert(Point { x: chunk[0], y: chunk[1] }, Point { x: chunk[2], y: chunk[3] }); }

    let result = find_path_grid(start, goal, &costs, &zone_types, &teleporters);
    result.map_or(vec![], |path| path.into_iter().flat_map(|p| [p.x, p.y]).collect())
}

#[wasm_bindgen]
pub fn find_path_in_space_wasm(
    start_x: i32, start_y: i32,
    goal_x: i32, goal_y: i32,
    obstacles_flat: &[i32],
    params: &PhysicsParams,
    grid_size: i32,
) -> Vec<i32> {
    let start_state = PhysicsState { x: start_x, y: start_y, vx: 0, vy: 0 };
    let goal_pos = Point { x: goal_x, y: goal_y };

    let mut obstacles = HashMap::new();
    for chunk in obstacles_flat.chunks_exact(3) {
        if chunk[2] == i32::MAX {
            obstacles.insert(Point { x: chunk[0], y: chunk[1] }, chunk[2]);
        }
    }

    let result = find_path_physics(start_state, goal_pos, params, &obstacles, grid_size);
    result.map_or(vec![], |path| path.into_iter().flat_map(|p| [p.x, p.y]).collect())
}