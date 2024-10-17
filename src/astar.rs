use crate::map::Map;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq)]
struct Node {
    position: (usize, usize),
    f_cost: usize,
    g_cost: usize,
}

// Implement ordering for the priority queue where lower costs are given higher priority.
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn a_star_search(map: &Map, start: (usize, usize), goal: (usize, usize)) -> Option<Vec<(usize, usize)>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

    let mut g_cost = HashMap::new();
    g_cost.insert(start, 0);
    let mut f_cost = HashMap::new();
    f_cost.insert(start, heuristic(start, goal));

    open_set.push(Node { position: start, f_cost: heuristic(start, goal), g_cost: 0 });

    while let Some(current) = open_set.pop() {
        if current.position == goal {
            return Some(construct_path(came_from, current.position));
        }

        for neighbor in map.grid[current.position.1][current.position.0].neighbors.clone() {
            if !map.grid[neighbor.1][neighbor.0].passable {
                continue;
            }
            let tentative_g_cost = g_cost[&current.position] + 1; // Assuming uniform cost

            if tentative_g_cost < *g_cost.get(&neighbor).unwrap_or(&usize::MAX) {
                came_from.insert(neighbor, current.position);
                g_cost.insert(neighbor, tentative_g_cost);
                f_cost.insert(neighbor, tentative_g_cost + heuristic(neighbor, goal));
                open_set.push(Node { position: neighbor, f_cost: tentative_g_cost + heuristic(neighbor, goal), g_cost: tentative_g_cost });
            }
        }
    }

    None // No path found
}

fn heuristic(start: (usize, usize), goal: (usize, usize)) -> usize {
    // Using Manhattan distance as heuristic
    (start.0 as isize - goal.0 as isize).abs() as usize + (start.1 as isize - goal.1 as isize).abs() as usize
}

fn construct_path(came_from: HashMap<(usize, usize), (usize, usize)>, mut current: (usize, usize)) -> Vec<(usize, usize)> {
    let mut path = vec![current];
    while let Some(next) = came_from.get(&current) {
        path.push(*next);
        current = *next;
    }
    path.reverse(); // Reverse path to start from the initial position
    path
}
