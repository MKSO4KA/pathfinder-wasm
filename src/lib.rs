// Импортируем необходимые модули из стандартной библиотеки
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

// Импортируем wasm-bindgen, чтобы "подружить" Rust и JavaScript
use wasm_bindgen::prelude::*;

// --- Структуры данных для алгоритма A* ---

// Простая структура для хранения координат (x, y).
// Атрибут 'derive' автоматически добавляет стандартные возможности,
// чтобы мы могли сравнивать точки, копировать их и использовать в коллекциях.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

// Структура для узла в графе поиска.
struct Node {
    position: Point,
    // g: стоимость пути от старта до этого узла
    g: i32,
    // h: эвристическая (приблизительная) стоимость от этого узла до цели
    h: i32,
}

// Реализуем методы для узла
impl Node {
    // f: общая стоимость узла (g + h). Чем она меньше, тем узел перспективнее.
    fn f(&self) -> i32 {
        self.g + self.h
    }
}

// --- Реализация трейтов для сравнения узлов ---
// Это нужно, чтобы наша очередь с приоритетом (BinaryHeap) знала,
// какой узел является "наилучшим" (с наименьшей стоимостью f).

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Сравниваем в обратном порядке, т.к. BinaryHeap по умолчанию - это max-heap,
        // а нам нужен узел с МИНИМАЛЬНОЙ стоимостью f.
        other.f().cmp(&self.f())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f() == other.f()
    }
}

impl Eq for Node {}


// --- Вспомогательные функции ---

// Эвристическая функция. Используем "Манхэттенское расстояние".
// Оно хорошо работает для сеток, где можно двигаться только по 4 направлениям.
fn heuristic(a: Point, b: Point) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

// Восстанавливает итоговый путь, двигаясь от цели к старту по "родителям".
fn reconstruct_path(parents: HashMap<Point, Point>, current: Point) -> Vec<Point> {
    let mut total_path = vec![current];
    let mut temp = current;
    // Пока у текущей точки есть родитель, добавляем его в путь
    while let Some(parent) = parents.get(&temp) {
        total_path.push(*parent);
        temp = *parent;
    }
    // Путь получился в обратном порядке (от цели к старту), поэтому разворачиваем его.
    total_path.reverse();
    total_path
}


// --- Основная функция A* ---
// Делаем коммит для публикации. СУПЕР!
// Находит путь от `start` до `goal`, избегая точек из `obstacles`.
fn find_path(start: Point, goal: Point, obstacles: &HashSet<Point>) -> Option<Vec<Point>> {
    // BinaryHeap - это эффективная очередь с приоритетом. Наш "open list".
    let mut open_list = BinaryHeap::new();
    // HashMap для хранения g-оценок и отслеживания "родителей" для восстановления пути.
    let mut g_scores = HashMap::new();
    let mut parents = HashMap::new();

    // Добавляем стартовый узел
    g_scores.insert(start, 0);
    open_list.push(Node {
        position: start,
        g: 0,
        h: heuristic(start, goal),
    });

    // Главный цикл алгоритма
    while let Some(current_node) = open_list.pop() {
        let current_pos = current_node.position;

        // Если мы достигли цели - ура, путь найден!
        if current_pos == goal {
            return Some(reconstruct_path(parents, current_pos));
        }

        // Обрабатываем соседей (вверх, вниз, влево, вправо)
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter() {
            let neighbor_pos = Point { x: current_pos.x + dx, y: current_pos.y + dy };

            // Пропускаем соседа, если это препятствие
            if obstacles.contains(&neighbor_pos) {
                continue;
            }

            // Стоимость пути до соседа (в нашем случае всегда +1)
            // ИСПРАВЛЕНИЕ: Исправлена опечатка ¤t_pos на ¤t_pos
            let tentative_g_score = g_scores.get(&current_pos).unwrap() + 1;

            // Если мы нашли более короткий путь до этого соседа
            if tentative_g_score < *g_scores.get(&neighbor_pos).unwrap_or(&i32::MAX) {
                // Запоминаем этот новый, лучший путь
                parents.insert(neighbor_pos, current_pos);
                g_scores.insert(neighbor_pos, tentative_g_score);
                // Добавляем соседа в очередь на рассмотрение
                open_list.push(Node {
                    position: neighbor_pos,
                    g: tentative_g_score,
                    h: heuristic(neighbor_pos, goal),
                });
            }
        }
    }

    // Если очередь опустела, а цель не найдена - пути нет.
    None
}


// --- "Обертка" для JavaScript ---
// Эта функция будет видна из нашего Tampermonkey скрипта.
// Она принимает простые типы (числа, массивы) и возвращает массив чисел.

#[wasm_bindgen]
pub fn find_path_wasm(start_x: i32, start_y: i32, goal_x: i32, goal_y: i32, obstacles_flat: &[i32]) -> Vec<i32> {
    // 1. Преобразуем "плоский" массив препятствий [x1, y1, x2, y2, ...] в удобный для Rust формат HashSet<Point>
    let mut obstacles = HashSet::new();
    for chunk in obstacles_flat.chunks_exact(2) {
        obstacles.insert(Point { x: chunk[0], y: chunk[1] });
    }

    let start = Point { x: start_x, y: start_y };
    let goal = Point { x: goal_x, y: goal_y };

    // 2. Вызываем нашу основную, "чистую" Rust-функцию
    let result = find_path(start, goal, &obstacles);

    // 3. Преобразуем результат обратно в "плоский" массив, понятный для JavaScript
    match result {
        Some(path) => {
            // Превращаем Vec<Point> в Vec<i32> вида [x1, y1, x2, y2, ...]
            path.into_iter().flat_map(|p| [p.x, p.y]).collect()
        }
        None => {
            // Если путь не найден, возвращаем пустой массив
            vec![]
        }
    }
}