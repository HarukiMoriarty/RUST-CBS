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

    open.push(LowLevelNode { position: start, f_cost: heuristic(start, goal), g_cost: 0, h_open_cost: heuristic(start, goal), h_focal_cost: 0 });

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

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: current_time
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            // Assuming uniform cost, typically 1 in a grid.
            let tentative_g_cost = g_cost[&current.position] + 1;
            if tentative_g_cost < *g_cost.get(neighbor).unwrap_or(&usize::MAX) {
                trace.insert(*neighbor, current.position);
                g_cost.insert(*neighbor, tentative_g_cost);
                f_cost.insert(*neighbor, tentative_g_cost + heuristic(*neighbor, goal));
                let h_open_cost = heuristic(*neighbor, goal);
                open.push(LowLevelNode { position: *neighbor, f_cost: tentative_g_cost + h_open_cost, g_cost: tentative_g_cost, h_open_cost, h_focal_cost: 0});
            }
        }
    }

    None
}

pub(super) fn focal_a_star_search(
    map: &Map, 
    start: (usize, usize), 
    goal: (usize, usize), 
    subopt_factor: f64, 
    constraints: &HashSet<Constraint>,
    current_agent: usize,
    paths: &HashMap<usize, Vec<(usize, usize)>>
) -> Option<Vec<(usize, usize)>> {
    let mut open_list = BinaryHeap::new();
    let mut focal_list = BinaryHeap::new();
    let mut trace = HashMap::new();
    let mut g_cost = HashMap::new();

    let start_h_open_cost = heuristic(start, goal);
    let mut start_node = LowLevelNode {
        position: start,
        f_cost: start_h_open_cost,
        g_cost: 0,
        h_open_cost: start_h_open_cost,
        h_focal_cost: 0
    };

    open_list.push(start_node.clone());

    start_node.f_focal();
    focal_list.push(start_node);

    g_cost.insert(start, 0);

    while let Some(current) = focal_list.pop() {
        if current.position == goal {
            return Some(construct_path(trace, current.position));
        }

        // Time step increases as we move to the next node.
        let current_time = g_cost[&current.position] + 1;
        let tentative_g_cost = current.g_cost + 1;

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            if !map.is_passable(neighbor.0, neighbor.1) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: current_time
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            let h_open_cost = heuristic(*neighbor, goal);
            let h_focal_cost = current.h_focal_cost + heuristic_focal(current_agent, *neighbor, current_time, paths);
            let f_cost = tentative_g_cost + h_open_cost;

            if tentative_g_cost < *g_cost.get(&neighbor).unwrap_or(&usize::MAX) {
                trace.insert(*neighbor, current.position);
                g_cost.insert(*neighbor, tentative_g_cost);
                let mut neighbor_node = LowLevelNode { position: *neighbor, f_cost, g_cost: tentative_g_cost, h_open_cost, h_focal_cost};
                open_list.push(neighbor_node.clone());
                
                if f_cost <= (open_list.peek().unwrap().f_cost as f64 * subopt_factor) as usize {
                    neighbor_node.f_focal();
                    focal_list.push(neighbor_node);
                }
            }
        }

        // Maintain the focal list
        if focal_list.is_empty() {
            update_focal_list(&mut focal_list, &open_list, subopt_factor);
        }
    }

    None
}


fn heuristic(start: (usize, usize), goal: (usize, usize)) -> usize {
    // Using Manhattan distance as heuristic
    (start.0 as isize - goal.0 as isize).unsigned_abs() + (start.1 as isize - goal.1 as isize).unsigned_abs()
}

fn heuristic_focal(current_agent: usize, current_position: (usize, usize), current_time: usize, paths: &HashMap<usize, Vec<(usize, usize)>>) -> usize {
    let mut conflict_count = 0;
    for (&agent_id, path) in paths.iter() {
        if agent_id == current_agent {
            continue;
        }
        if let Some(&position) = path.get(current_time) {
            if position == current_position {
                conflict_count +=1;
            }
        }
    }
    conflict_count
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

fn update_focal_list(focal_list: &mut BinaryHeap<LowLevelNode>, open_list: &BinaryHeap<LowLevelNode>, subopt_factor: f64) {
    let min_f_cost = open_list.peek().map_or(usize::MAX, |n| n.f_cost);
    open_list.iter().for_each(|node| {
        if node.f_cost <= (min_f_cost as f64 * subopt_factor) as usize {
            let mut focal_node = node.clone();
            focal_node.f_focal();
            focal_list.push(focal_node);
        }
    });
}