use super::{construct_path, heuristic_focal};
use crate::common::Agent;
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelNode};
use crate::stat::Stats;
use std::{
    collections::{BTreeMap, BinaryHeap, HashMap, HashSet},
    usize,
};

pub(crate) fn focal_a_star_search(
    map: &Map,
    agent: &Agent,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    paths: &Vec<Vec<(usize, usize)>>,
    stats: &mut Stats,
) -> Option<(Vec<(usize, usize)>, Option<usize>)> {
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

    // Open list is indexed based on (f_open_cost, time, position)
    let mut open_list = BTreeMap::new();
    let mut focal_list = BinaryHeap::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();
    let mut g_cost = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];
    // Calculate as f open cost
    let start_node = LowLevelNode {
        position: agent.start,
        sort_key: 0,
        g_cost: 0,
        h_open_cost: start_h_open_cost,
        h_focal_cost: Some(0),
        time: 0,
    };

    open_list.insert((0 + start_h_open_cost, 0, agent.start), start_node.clone());
    focal_list.push(start_node);
    g_cost.insert((agent.start, 0), 0);

    while let Some(current) = focal_list.pop() {
        // Update stats
        stats.low_level_expand_nodes += 1;
        closed_list.insert((current.position, current.time));

        let f_min = current.g_cost + current.h_open_cost;
        open_list.remove(&(
            current.h_open_cost + current.g_cost,
            current.time,
            current.position,
        ));

        let current_time = current.time;
        if current.position == agent.goal && current_time > max_time {
            return Some((
                construct_path(&trace, (current.position, current_time)),
                Some(f_min),
            ));
        }

        // Time step increases as we move to the next node.
        let next_time = current_time + 1;

        // Assuming uniform cost.
        let tentative_g_cost = current.g_cost + 1;

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            if closed_list.contains(&(*neighbor, next_time)) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: next_time,
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];
            let h_focal_cost = current.h_focal_cost.unwrap()
                + heuristic_focal(agent.id, *neighbor, next_time, paths);
            let f_open_cost = tentative_g_cost + h_open_cost;

            if tentative_g_cost < *g_cost.get(&(*neighbor, next_time)).unwrap_or(&usize::MAX) {
                trace.insert((*neighbor, next_time), (current.position, current_time));
                g_cost.insert((*neighbor, next_time), tentative_g_cost);
                let neighbor_node = LowLevelNode {
                    position: *neighbor,
                    sort_key: h_focal_cost,
                    g_cost: tentative_g_cost,
                    h_open_cost,
                    h_focal_cost: Some(h_focal_cost),
                    time: next_time,
                };

                open_list.insert((f_open_cost, next_time, *neighbor), neighbor_node.clone());

                if f_open_cost as f64 <= (f_min as f64 * subopt_factor) {
                    focal_list.push(neighbor_node);
                }
            }
        }

        // Maintain the focal list
        let new_f_min = open_list
            .iter()
            .next()
            .map_or(usize::MAX, |((f_open_cost, _, _), _)| *f_open_cost);
        if !open_list.is_empty() && f_min < new_f_min {
            update_lower_bound(
                &mut focal_list,
                &mut open_list,
                subopt_factor * f_min as f64,
                subopt_factor * new_f_min as f64,
            );
        }
    }

    None
}

fn update_lower_bound(
    focal_list: &mut BinaryHeap<LowLevelNode>,
    open_list: &BTreeMap<(usize, usize, (usize, usize)), LowLevelNode>,
    old_bound: f64,
    new_bound: f64,
) {
    open_list.iter().for_each(|((f_open_cost, _, _), node)| {
        if *f_open_cost as f64 > old_bound && *f_open_cost as f64 <= new_bound {
            focal_list.push(node.clone());
        }
    });
}
