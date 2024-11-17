use tracing::{debug, instrument};

use super::{construct_path, heuristic_focal};
use crate::common::Agent;
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelFocalNode, LowLevelOpenNode};
use crate::stat::Stats;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    usize,
};

#[instrument(skip_all, name="low_level_a_focal_star", fields(agent = agent.id, subopt_factor = subopt_factor), level = "debug")]
pub(crate) fn focal_a_star_search(
    map: &Map,
    agent: &Agent,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    paths: &[Vec<(usize, usize)>],
    stats: &mut Stats,
) -> Option<(Vec<(usize, usize)>, Option<usize>)> {
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

    // Open list is indexed based on (f_open_cost, time, position)
    let mut open_list = BTreeSet::new();
    let mut focal_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let mut g_cost_map = HashMap::new();
    let mut f_focal_cost_map = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];

    open_list.insert(LowLevelOpenNode {
        position: agent.start,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
        time: 0,
    });
    focal_list.insert(LowLevelFocalNode {
        position: agent.start,
        f_focal_cost: 0,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
        time: 0,
    });
    g_cost_map.insert((agent.start, 0), 0);
    f_focal_cost_map.insert((agent.start, 0), 0);

    while let Some(current) = focal_list.pop_first() {
        debug!("expand node: {current:?}");

        // Update stats.
        stats.low_level_expand_nodes += 1;

        closed_list.insert((current.position, current.time));

        let f_min = current.f_open_cost;

        // Remove the same node from open list.
        open_list.remove(&LowLevelOpenNode {
            position: current.position,
            f_open_cost: f_min,
            g_cost: current.g_cost,
            time: current.time,
        });

        if current.position == agent.goal && current.time > max_time {
            return Some((
                construct_path(&trace, (current.position, current.time)),
                Some(f_min),
            ));
        }

        // Time step increases as we move to the next node.
        let next_time = current.time + 1;

        // Assuming uniform cost.
        let tentative_g_cost = current.g_cost + 1;

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            // If node (position at current time) has closed, ignore.
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

            let old_g_cost = *g_cost_map
                .get(&(*neighbor, next_time))
                .unwrap_or(&usize::MAX);
            let old_f_focal_cost = *f_focal_cost_map
                .get(&(*neighbor, next_time))
                .unwrap_or(&usize::MAX);
            let f_focal_cost =
                current.f_focal_cost + heuristic_focal(agent.id, *neighbor, next_time, paths);
            // Our update policy:
            // 1. To keep suboptimal bound, we should not keep any larger new g-cost value in list
            // 2. To keep insistency, we should keep node (position, time) in open list and focal list (if exist) always the same node
            // Our update algorithm:
            // 1) If we see a larger new g-cost, we do nothing
            // 2) If we see an equal g-cost, we update only if see a smaller focal value, both update in open and focal
            // 3) If we see a smaller new g-cost, directly update in open and focal
            if tentative_g_cost < old_g_cost {
                trace.insert((*neighbor, next_time), (current.position, current.time));
                // small g cost observed, update corresponding hash map
                g_cost_map.insert((*neighbor, next_time), tentative_g_cost);
                // update f cost (even if it might be higher)
                f_focal_cost_map.insert((*neighbor, next_time), f_focal_cost);

                let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];
                let f_open_cost = tentative_g_cost + h_open_cost;

                // Update old node in open list if it is already appear in open list
                // Samely, update the corresponding node in focal list
                // Question: is this really needed?
                if old_g_cost != usize::MAX {
                    debug!(
                        "find a small g cost {:?} for node {:?} at time {next_time:?}",
                        tentative_g_cost + h_open_cost,
                        *neighbor
                    );
                    // We should find such old node already in open list and remove it.
                    assert!(open_list.remove(&LowLevelOpenNode {
                        position: *neighbor,
                        f_open_cost: old_g_cost + h_open_cost,
                        g_cost: old_g_cost,
                        time: next_time,
                    }));

                    if old_f_focal_cost != usize::MAX {
                        // We might not find such old node already in focal list, blind remove it.
                        focal_list.remove(&LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost: old_f_focal_cost,
                            f_open_cost: old_g_cost + h_open_cost,
                            g_cost: old_g_cost,
                            time: next_time,
                        });
                    }
                }

                open_list.insert(LowLevelOpenNode {
                    position: *neighbor,
                    f_open_cost,
                    g_cost: tentative_g_cost,
                    time: next_time,
                });

                // If the node is in the suboptimal bound, push it into focal list.
                // If this is an update in focal list, we update only we see less g cost,
                // which means the new f open cost must fall in suboptimal bound, we definitely
                // will reinsert updated node into focal list.
                if f_open_cost as f64 <= (f_min as f64 * subopt_factor) {
                    focal_list.insert(LowLevelFocalNode {
                        position: *neighbor,
                        f_focal_cost,
                        f_open_cost,
                        g_cost: tentative_g_cost,
                        time: next_time,
                    });
                }
            } else if tentative_g_cost == old_g_cost && f_focal_cost < old_f_focal_cost {
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
                            *f_focal_cost_map.get(&(node.position, node.time)).unwrap();
                        focal_list.insert(LowLevelFocalNode {
                            position: node.position,
                            f_focal_cost,
                            f_open_cost: node.f_open_cost,
                            g_cost: node.g_cost,
                            time: node.time,
                        });
                    }
                });
            }
        }
    }

    None
}
