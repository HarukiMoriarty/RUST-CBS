use super::{construct_mdd, construct_path, heuristic_focal, standard_a_star_search};
use crate::common::{Agent, Constraint, LowLevelFocalNode, LowLevelOpenNode, Path, SearchResult};
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
        .map(|constraint| match constraint {
            Constraint::Vertex { time_step, .. } => *time_step,
            Constraint::Edge { to_time_step, .. } => *to_time_step,
        })
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

    // Build MDD using optimal f min.
    debug!("building mdd.");
    if sub_optimal_result.len() - 1 == f_min {
        SearchResult::WithMDD(Some((
            sub_optimal_result,
            f_min,
            construct_mdd(map, agent, constraints, f_min),
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

    // Open list is indexed based on (f_open_cost, g_cost(time), position)
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
        time_step: 0,
    });
    f_focal_cost_map.insert((agent.start, 0), 0);

    while let Some(current) = focal_list.pop_first() {
        trace!("expand node: {current:?}");
        let exceed_constraints_limit_time_step = current.time_step > constraint_limit_time_step;

        // Update stats.
        stats.low_level_expand_focal_nodes += 1;

        closed_list.insert((current.position, current.time_step));

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

        // time step only increase if we haven't passed constraint limit
        // Tricky: after constraint limit, we fixed time step as T + 1, algorithm
        // demote to 2-D a star, enable branch pruning
        let tentative_time_step = if exceed_constraints_limit_time_step {
            current.time_step
        } else {
            current.time_step + 1
        };

        // Expanding nodes from the current position
        for neighbor in &map.get_neighbors(
            current.position.0,
            current.position.1,
            !exceed_constraints_limit_time_step,
        ) {
            // If node (position at current time) has closed, ignore.
            if closed_list.contains(&(*neighbor, tentative_time_step)) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints.iter().any(|constraint| {
                constraint.is_violated(current.position, *neighbor, tentative_g_cost)
            }) {
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
                time_step: tentative_time_step,
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
                        time_step: tentative_time_step,
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
                        time_step: tentative_time_step,
                    }) {
                        focal_list.remove(&LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost: old_f_focal_cost,
                            f_open_cost,
                            g_cost: tentative_g_cost,
                            time_step: tentative_time_step,
                        });
                        focal_list.insert(LowLevelFocalNode {
                            position: *neighbor,
                            f_focal_cost,
                            f_open_cost,
                            g_cost: tentative_g_cost,
                            time_step: tentative_time_step,
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
                            time_step: node.time_step,
                        });
                    }
                });
            }
        }
    }

    debug!("cannot find solution");
    None
}
