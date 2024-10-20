use crate::map::Map;
use super::common::{LowLevelNode, Constraint};
use std::collections::{BinaryHeap, HashMap, HashSet};

pub(super) fn a_star_search(
    map: &Map, 
    start: (usize, usize), 
    goal: (usize, usize), 
    constraints: &HashSet<Constraint>
) -> Option<Vec<(usize, usize)>> {
    let mut open = BinaryHeap::new();
    let mut trace: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut g_cost = HashMap::new();
    let mut f_cost = HashMap::new();

    g_cost.insert(start, 0);
    f_cost.insert(start, heuristic(start, goal));

    open.push(LowLevelNode { position: start, f_cost: heuristic(start, goal), g_cost: 0 });

    while let Some(current) = open.pop() {
        if current.position == goal {
            return Some(construct_path(trace, current.position));
        }

        // Time step increases as we move to the next node.
        let current_time = g_cost[&current.position] + 1;

        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            if !map.is_passable(neighbor.0, neighbor.1) {
                continue;
            }

            // Assuming uniform cost, typically 1 in a grid.
            let tentative_g_cost = g_cost[&current.position] + 1;

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: current_time
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            if tentative_g_cost < *g_cost.get(neighbor).unwrap_or(&usize::MAX) {
                trace.insert(*neighbor, current.position);
                g_cost.insert(*neighbor, tentative_g_cost);
                f_cost.insert(*neighbor, tentative_g_cost + heuristic(*neighbor, goal));
                open.push(LowLevelNode { position: *neighbor, f_cost: tentative_g_cost + heuristic(*neighbor, goal), g_cost: tentative_g_cost });
            }
        }
    }

    None
}

fn heuristic(start: (usize, usize), goal: (usize, usize)) -> usize {
    // Using Manhattan distance as heuristic
    (start.0 as isize - goal.0 as isize).unsigned_abs() + (start.1 as isize - goal.1 as isize).unsigned_abs()
}

fn construct_path(trace: HashMap<(usize, usize), (usize, usize)>, mut current: (usize, usize)) -> Vec<(usize, usize)> {
    let mut path = vec![current];
    while let Some(next) = trace.get(&current) {
        path.push(*next);
        current = *next;
    }
    path.reverse();
    path
}