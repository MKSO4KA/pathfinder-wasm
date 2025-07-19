use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

// --- БЛОК ДЛЯ ЛОГИРОВАНИЯ В КОНСОЛЬ БРАУЗЕРА ---
extern crate web_sys;
use web_sys::console;

macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}
// --- КОНЕЦ БЛОКА ЛОГИРОВАНИЯ ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point { x: i32, y: i32 }

struct Node { position: Point, g: i32, h: i32 }
impl Node { fn f(&self) -> i32 { self.g + self.h } }
impl Ord for Node { fn cmp(&self, other: &Self) -> Ordering { other.f().cmp(&self.f()) } }
impl PartialOrd for Node { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl PartialEq for Node { fn eq(&self, other: &Self) -> bool { self.f() == other.f() } }
impl Eq for Node {}

fn heuristic(a: Point, b: Point) -> i32 { (a.x - b.x).abs() + (a.y - b.y).abs() }

fn reconstruct_path(parents: HashMap<Point, Point>, current: Point) -> Vec<Point> {
    let mut total_path = vec![current];
    let mut temp = current;
    while let Some(parent) = parents.get(&temp) {
        total_path.push(*parent);
        temp = *parent;
    }
    total_path.reverse();
    total_path
}

fn find_path(start: Point, goal: Point, obstacles: &HashSet<Point>) -> Option<Vec<Point>> {
    let mut open_list = BinaryHeap::new();
    let mut g_scores = HashMap::new();
    let mut parents = HashMap::new();
    g_scores.insert(start, 0);
    open_list.push(Node { position: start, g: 0, h: heuristic(start, goal) });
    while let Some(current_node) = open_list.pop() {
        let current_pos = current_node.position;
        if current_pos == goal { return Some(reconstruct_path(parents, current_pos)); }
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter() {
            let neighbor_pos = Point { x: current_pos.x + dx, y: current_pos.y + dy };
            if obstacles.contains(&neighbor_pos) { continue; }
            let tentative_g_score = g_scores.get(&current_pos).unwrap() + 1;
            if tentative_g_score < *g_scores.get(&neighbor_pos).unwrap_or(&i32::MAX) {
                parents.insert(neighbor_pos, current_pos);
                g_scores.insert(neighbor_pos, tentative_g_score);
                open_list.push(Node { position: neighbor_pos, g: tentative_g_score, h: heuristic(neighbor_pos, goal) });
            }
        }
    }
    None
}

#[wasm_bindgen]
pub fn find_path_wasm(start_x: i32, start_y: i32, goal_x: i32, goal_y: i32, obstacles_flat: &[i32]) -> Vec<i32> {
    log!("[Rust] Функция find_path_wasm вызвана.");
    log!("[Rust] Старт: ({}, {}), Цель: ({}, {})", start_x, start_y, goal_x, goal_y);
    log!("[Rust] Получено {} точек препятствий.", obstacles_flat.len() / 2);

    let mut obstacles = HashSet::new();
    for chunk in obstacles_flat.chunks_exact(2) {
        obstacles.insert(Point { x: chunk[0], y: chunk[1] });
    }

    let start = Point { x: start_x, y: start_y };
    let goal = Point { x: goal_x, y: goal_y };

    let result = find_path(start, goal, &obstacles);

    match result {
        Some(path) => {
            log!("[Rust] Путь найден! Длина: {} точек.", path.len());
            let flat_path: Vec<i32> = path.into_iter().flat_map(|p| [p.x, p.y]).collect();
            log!("[Rust] Возвращаем плоский массив длиной {}.", flat_path.len());
            flat_path
        }
        None => {
            log!("[Rust] Путь не найден. Возвращаем пустой массив.");
            vec![]
        }
    }
}