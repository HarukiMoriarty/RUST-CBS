use super::{construct_path, heuristic_focal, standard_a_star_search};
use crate::common::{
    Agent, Constraint, LowLevelFocalNode, LowLevelOpenNode, Mdd, Path, SearchResult,
};
use crate::map::Map;
use crate::stat::Stats;

use std::cmp::max;
use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::{debug, instrument, trace};

#[allow(clippy::too_many_arguments)]
pub(crate) fn focal_a_star_search(
    map: &Map,
    agent: &Agent,
    last_search_f_min: Option<usize>,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    paths: &[Path],
    build_mdd: bool,
    stats: &mut Stats,
) -> SearchResult {
    let constraint_limit_time_step = constraints
        .iter()
        .map(|constraint| constraint.time_step)
        .max()
        .unwrap_or(0);

    if !build_mdd {
        return SearchResult::Standard(standard_focal_a_star_search(
            map,
            agent,
            last_search_f_min,
            subopt_factor,
            constraints,
            path_length_constraint,
            constraint_limit_time_step,
            paths,
            stats,
        ));
    }

    let (sub_optimal_result, f_min) = match standard_focal_a_star_search(
        map,
        agent,
        last_search_f_min,
        subopt_factor,
        constraints,
        path_length_constraint,
        constraint_limit_time_step,
        paths,
        stats,
    ) {
        Some((sub_optimal_result, f_min)) => (sub_optimal_result, f_min),
        None => return SearchResult::WithMDD(None),
    };

    // Build MDD using sub optimal f min.
    debug!("building mdd.");
    if sub_optimal_result.len() - 1 == f_min {
        let mut mdd_layers = vec![HashSet::new()];
        mdd_layers[0].insert(agent.start);

        let mut open_list = BTreeSet::new();
        let mut closed_list = BTreeSet::new();

        let start_node = LowLevelOpenNode {
            position: agent.start,
            f_open_cost: map.heuristic[agent.id][agent.start.0][agent.start.1],
            g_cost: 0,
            time_step: 0,
        };
        open_list.insert(start_node);

        while let Some(current) = open_list.pop_first() {
            trace!("expand node: {current:?}");

            // Update stats.
            stats.low_level_mdd_expand_open_nodes += 1;

            closed_list.insert((current.position, current.g_cost));

            // Assuming uniform cost, which also indicate the current time.
            let tentative_g_cost = current.g_cost + 1;

            while mdd_layers.len() <= tentative_g_cost && tentative_g_cost <= f_min {
                mdd_layers.push(HashSet::new());
            }

            // Expand nodes from the current position.
            for neighbor in &map.get_neighbors(current.position.0, current.position.1, true) {
                // Checck node (position at current time) has closed.
                if closed_list.contains(&(*neighbor, tentative_g_cost)) {
                    continue;
                }

                // Check for constraints before exploring the neighbor.
                if constraints
                    .iter()
                    .any(|c| c.is_violated(*neighbor, tentative_g_cost))
                {
                    continue;
                }

                let f_cost = tentative_g_cost + map.heuristic[agent.id][neighbor.0][neighbor.1];

                // Check for optimal solution
                if f_cost > f_min {
                    continue;
                }

                let new_node = LowLevelOpenNode {
                    position: *neighbor,
                    f_open_cost: f_cost,
                    g_cost: tentative_g_cost,
                    time_step: tentative_g_cost,
                };

                if open_list.insert(new_node) {
                    mdd_layers[tentative_g_cost].insert(*neighbor);
                }
            }
        }

        SearchResult::WithMDD(Some((
            sub_optimal_result,
            f_min,
            Mdd { layers: mdd_layers },
        )))
    } else {
        SearchResult::Standard(Some((sub_optimal_result, f_min)))
    }
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, name="standard_focal_a_star_search", fields(agent = agent.id, subopt_factor = subopt_factor, last_search_f_min = last_search_f_min, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn standard_focal_a_star_search(
    map: &Map,
    agent: &Agent,
    last_search_f_min: Option<usize>,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    constraint_limit_time_step: usize,
    paths: &[Path],
    stats: &mut Stats,
) -> Option<(Path, usize)> {
    debug!("constraints: {constraints:?}, limit time step: {constraint_limit_time_step:?}");

    let mut f_min = if let Some(last_search_f_min) = last_search_f_min {
        last_search_f_min
    } else {
        debug!("double search.");
        match standard_a_star_search(
            map,
            agent,
            constraints,
            path_length_constraint,
            constraint_limit_time_step,
            stats,
        ) {
            Some((_, f_min)) => f_min,
            None => return None,
        }
    };

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
        time_step: 0,
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

        // Use last search f min to speed search.
        f_min = max(open_list.first().unwrap().f_open_cost, f_min);

        // Remove the same node from open list.
        assert!(open_list.remove(&LowLevelOpenNode {
            position: current.position,
            f_open_cost: current.f_open_cost,
            g_cost: current.g_cost,
            time_step: current.g_cost,
        }));

        if current.position == agent.goal && current.g_cost > path_length_constraint {
            debug!("find solution with f min {f_min:?}");
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                f_min,
            ));
        }

        // Assuming uniform cost.
        let tentative_g_cost = current.g_cost + 1;

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(current.position.0, current.position.1, true) {
            // If node (position at current time) has closed, ignore.
            if closed_list.contains(&(*neighbor, tentative_g_cost)) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints
                .iter()
                .any(|constraint| constraint.is_violated(*neighbor, tentative_g_cost))
            {
                continue; // This move is prohibited due to a constraint
            }

            let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];
            let f_open_cost = tentative_g_cost + h_open_cost;
            let f_focal_cost = current.f_focal_cost
                + heuristic_focal(
                    agent.id,
                    *neighbor,
                    current.position,
                    tentative_g_cost,
                    paths,
                );

            // If this node is never appeared before, update open list and trace
            // Also means this node is new to focal history, update focal cost hashmap
            if open_list.insert(LowLevelOpenNode {
                position: *neighbor,
                f_open_cost,
                g_cost: tentative_g_cost,
                time_step: tentative_g_cost,
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
    }

    debug!("cannot find solution");
    None
}
