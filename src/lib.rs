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
    if start == goal {
        return Some(vec![start]);
    }

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
            return Some(reconstruct_path_grid(parents, current_pos));
        }

        if let Some(exit_pos) = teleporters.get(&current_pos) {
            let tentative_g_score = current_g + 1;
            if tentative_g_score < *g_scores.get(exit_pos).unwrap_or(&i32::MAX) {
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
    None
}

// ================================================================================= //
// ===== WASM-ОБВЯЗКИ ДЛЯ JAVASCRIPT ============================================== //
// ================================================================================= //

#[wasm_bindgen]
pub fn find_path_on_grid_wasm(
    start_x: i32,
    start_y: i32,
    goal_x: i32,
    goal_y: i32,
    costs_flat: &[i32],
    zone_types_flat: &[i32],
    teleporters_flat: &[i32]
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

// Здесь должен быть ваш код для find_path_in_space_wasm и PhysicsParams, если вы его используете.
// Если нет, то этот код выше - это все, что нужно.