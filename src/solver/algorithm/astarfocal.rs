use super::{a_star_search, construct_path, heuristic_focal};
use crate::common::Agent;
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelFocalNode, LowLevelOpenNode};
use crate::stat::Stats;

use std::cmp::max;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    usize,
};
use tracing::{debug, instrument, trace};

#[instrument(skip_all, name="low_level_a_focal_star", fields(agent = agent.id, subopt_factor = subopt_factor, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn focal_a_star_search(
    map: &Map,
    agent: &Agent,
    last_search_f_min: usize,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    paths: &[Vec<(usize, usize)>],
    stats: &mut Stats,
) -> Option<(Vec<(usize, usize)>, usize)> {
    debug!("constraints: {constraints:?}");
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

    // Open list is indexed based on (f_open_cost, time, position)
    let mut open_list = BTreeSet::new();
    let mut focal_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let mut f_focal_cost_map = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];

    open_list.insert(LowLevelOpenNode {
        position: agent.start,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
    });
    focal_list.insert(LowLevelFocalNode {
        position: agent.start,
        f_focal_cost: 0,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
    });

    f_focal_cost_map.insert((agent.start, 0), 0);

    while let Some(current) = focal_list.pop_first() {
        trace!("expand node: {current:?}");

        // Update stats.
        stats.low_level_expand_focal_nodes += 1;

        closed_list.insert((current.position, current.g_cost));

        let f_min = max(open_list.first().unwrap().f_open_cost, last_search_f_min);

        // Remove the same node from open list.
        open_list.remove(&LowLevelOpenNode {
            position: current.position,
            f_open_cost: f_min,
            g_cost: current.g_cost,
        });

        if current.position == agent.goal && current.g_cost > max_time {
            debug!("find solution");
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                f_min,
            ));
        }

        // Assuming uniform cost.
        let tentative_g_cost = current.g_cost + 1;

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            // If node (position at current time) has closed, ignore.
            if closed_list.contains(&(*neighbor, tentative_g_cost)) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: tentative_g_cost,
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];
            let f_open_cost = tentative_g_cost + h_open_cost;
            let f_focal_cost = current.f_focal_cost
                + heuristic_focal(agent.id, *neighbor, tentative_g_cost, paths);

            // If this node is never appeared before, update open list and trace
            // Also means this node is new to focal history, update focal cost hashmap
            if open_list.insert(LowLevelOpenNode {
                position: *neighbor,
                f_open_cost,
                g_cost: tentative_g_cost,
            }) {
                trace.insert(
                    (*neighbor, tentative_g_cost),
                    (current.position, current.g_cost),
                );
                f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);

                // If the node is in the suboptimal bound, push it into focal list.
                if f_open_cost as f64 <= (f_min as f64 * subopt_factor) {
                    focal_list.insert(LowLevelFocalNode {
                        position: *neighbor,
                        f_focal_cost,
                        f_open_cost,
                        g_cost: tentative_g_cost,
                    });
                }
            }
            // If this node is appeared before, we check if we get a smaller focal cost.
            else {
                // If this node appeared before in open list, we must have its focal cost history
                let old_f_focal_cost = *f_focal_cost_map
                    .get(&(*neighbor, tentative_g_cost))
                    .unwrap();

                if f_focal_cost < old_f_focal_cost {
                    // Update its corresponding focal cost,
                    // update focal cost map and focal list if it is in there.
                    f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);
                    if focal_list.contains(&LowLevelFocalNode {
                        position: *neighbor,
                        f_focal_cost: old_f_focal_cost,
                        f_open_cost,
                        g_cost: tentative_g_cost,
                    }) {
                        focal_list.remove(&LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost: old_f_focal_cost,
                            f_open_cost,
                            g_cost: tentative_g_cost,
                        });
                        focal_list.insert(LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost,
                            f_open_cost,
                            g_cost: tentative_g_cost,
                        });
                    }
                }
            }
        }

        if !open_list.is_empty() {
            // Maintain the focal list, since we have changed the f min.
            let new_f_min = open_list.first().unwrap().f_open_cost;
            if f_min < new_f_min {
                open_list.iter().for_each(|node| {
                    if node.f_open_cost as f64 > f_min as f64 * subopt_factor
                        && node.f_open_cost as f64 <= new_f_min as f64 * subopt_factor
                    {
                        let f_focal_cost =
                            *f_focal_cost_map.get(&(node.position, node.g_cost)).unwrap();
                        focal_list.insert(LowLevelFocalNode {
                            position: node.position,
                            f_focal_cost,
                            f_open_cost: node.f_open_cost,
                            g_cost: node.g_cost,
                        });
                    }
                });
            }
        }

        trace!("focal list: {focal_list:?}");
    }

    None
}

#[instrument(skip_all, name="low_level_a_focal_star_double", fields(agent = agent.id, subopt_factor = subopt_factor, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn focal_a_star_double_search(
    map: &Map,
    agent: &Agent,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    paths: &[Vec<(usize, usize)>],
    stats: &mut Stats,
) -> Option<(Vec<(usize, usize)>, usize)> {
    debug!("constraints: {constraints:?}");
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

    // We calculate f min by first perform an a star search.
    let f_min = a_star_search(map, agent, constraints, stats).unwrap().1;

    // Open list is indexed based on (f_open_cost, time, position)
    let mut open_list = BTreeSet::new();
    let mut focal_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let mut f_focal_cost_map = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];

    open_list.insert(LowLevelOpenNode {
        position: agent.start,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
    });
    focal_list.insert(LowLevelFocalNode {
        position: agent.start,
        f_focal_cost: 0,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
    });

    f_focal_cost_map.insert((agent.start, 0), 0);

    while let Some(current) = focal_list.pop_first() {
        trace!("expand node: {current:?}");

        // Update stats.
        stats.low_level_expand_focal_nodes += 1;

        closed_list.insert((current.position, current.g_cost));

        // Remove the same node from open list.
        open_list.remove(&LowLevelOpenNode {
            position: current.position,
            f_open_cost: current.f_open_cost,
            g_cost: current.g_cost,
        });

        if current.position == agent.goal && current.g_cost > max_time {
            debug!("find solution");
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                f_min,
            ));
        }

        // Assuming uniform cost.
        let tentative_g_cost = current.g_cost + 1;

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            // If node (position at current time) has closed, ignore.
            if closed_list.contains(&(*neighbor, tentative_g_cost)) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: tentative_g_cost,
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];
            let f_open_cost = tentative_g_cost + h_open_cost;
            let f_focal_cost = current.f_focal_cost
                + heuristic_focal(agent.id, *neighbor, tentative_g_cost, paths);

            // If this node is never appeared before, update open list and trace
            // Also means this node is new to focal history, update focal cost hashmap
            if open_list.insert(LowLevelOpenNode {
                position: *neighbor,
                f_open_cost,
                g_cost: tentative_g_cost,
            }) {
                trace.insert(
                    (*neighbor, tentative_g_cost),
                    (current.position, current.g_cost),
                );
                f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);

                // If the node is in the suboptimal bound, push it into focal list.
                if f_open_cost as f64 <= (f_min as f64 * subopt_factor) {
                    focal_list.insert(LowLevelFocalNode {
                        position: *neighbor,
                        f_focal_cost,
                        f_open_cost,
                        g_cost: tentative_g_cost,
                    });
                }
            }
            // If this node is appeared before, we check if we get a smaller focal cost.
            else {
                // If this node appeared before in open list, we must have its focal cost history
                let old_f_focal_cost = *f_focal_cost_map
                    .get(&(*neighbor, tentative_g_cost))
                    .unwrap();

                if f_focal_cost < old_f_focal_cost {
                    // Update its corresponding focal cost,
                    // update focal cost map and focal list if it is in there.
                    f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);
                    if focal_list.contains(&LowLevelFocalNode {
                        position: *neighbor,
                        f_focal_cost: old_f_focal_cost,
                        f_open_cost,
                        g_cost: tentative_g_cost,
                    }) {
                        focal_list.remove(&LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost: old_f_focal_cost,
                            f_open_cost,
                            g_cost: tentative_g_cost,
                        });
                        focal_list.insert(LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost,
                            f_open_cost,
                            g_cost: tentative_g_cost,
                        });
                    }
                }
            }
        }

        if !open_list.is_empty() {
            // Maintain the focal list, since we have changed the f min.
            let new_f_min = open_list.first().unwrap().f_open_cost;
            if f_min < new_f_min {
                open_list.iter().for_each(|node| {
                    if node.f_open_cost as f64 > f_min as f64 * subopt_factor
                        && node.f_open_cost as f64 <= new_f_min as f64 * subopt_factor
                    {
                        let f_focal_cost =
                            *f_focal_cost_map.get(&(node.position, node.g_cost)).unwrap();
                        focal_list.insert(LowLevelFocalNode {
                            position: node.position,
                            f_focal_cost,
                            f_open_cost: node.f_open_cost,
                            g_cost: node.g_cost,
                        });
                    }
                });
            }
        }

        trace!("focal list: {focal_list:?}");
    }

    None
}
