use super::{
    construct_mdd, construct_path, heuristic_focal, standard_a_star_search_focal_cost,
    standard_a_star_search_open_cost,
};
use crate::common::{
    create_open_focal_node, Agent, Constraint, FocalOrderWrapper, OpenOrderWrapper, Path,
    SearchResult,
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
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    paths: &[Path],
    build_mdd: bool,
    solver: &str,
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

    let (sub_optimal_result, f_min) = match solver {
        "decbs" => {
            // If there is no constraints, we just keep use normal focal search
            if !constraints.is_empty() {
                match standard_focal_double_search(
                    map,
                    agent,
                    subopt_factor,
                    constraints,
                    path_length_constraint,
                    constraint_limit_time_step,
                    paths,
                    stats,
                ) {
                    Some((sub_optimal_result, f_min)) => (sub_optimal_result, f_min),
                    None => {
                        return if build_mdd {
                            SearchResult::WithMDD(None)
                        } else {
                            SearchResult::Standard(None)
                        }
                    }
                }
            } else {
                match standard_focal_a_star_search(
                    map,
                    agent,
                    subopt_factor,
                    constraints,
                    path_length_constraint,
                    constraint_limit_time_step,
                    paths,
                    stats,
                ) {
                    Some((sub_optimal_result, f_min)) => (sub_optimal_result, f_min),
                    None => {
                        return if build_mdd {
                            SearchResult::WithMDD(None)
                        } else {
                            SearchResult::Standard(None)
                        }
                    }
                }
            }
        }
        "lbcbs" | "bcbs" | "ecbs" => match standard_focal_a_star_search(
            map,
            agent,
            subopt_factor,
            constraints,
            path_length_constraint,
            constraint_limit_time_step,
            paths,
            stats,
        ) {
            Some((sub_optimal_result, f_min)) => (sub_optimal_result, f_min),
            None => {
                return if build_mdd {
                    SearchResult::WithMDD(None)
                } else {
                    SearchResult::Standard(None)
                }
            }
        },
        "acbs" => match alternating_focal_a_star_search(
            map,
            agent,
            subopt_factor,
            constraints,
            path_length_constraint,
            constraint_limit_time_step,
            paths,
            stats,
        ) {
            Some((sub_optimal_result, f_min)) => (sub_optimal_result, f_min),
            None => {
                return if build_mdd {
                    SearchResult::WithMDD(None)
                } else {
                    SearchResult::Standard(None)
                }
            }
        },
        _ => unreachable!(),
    };

    if !build_mdd {
        return SearchResult::Standard(Some((sub_optimal_result, f_min)));
    }

    // Assert the solution cost is sub-optimal bounded.
    assert!((sub_optimal_result.len() - 1) as f64 <= f_min as f64 * subopt_factor);

    // Build MDD using optimal f min.
    if sub_optimal_result.len() - 1 == f_min {
        debug!("building MDD for agent {:?}", agent.id);
        SearchResult::WithMDD(Some((
            sub_optimal_result,
            f_min,
            construct_mdd(map, agent, constraints, f_min),
        )))
    } else {
        debug!("no building mdd for agent {:?}", agent.id);
        SearchResult::Standard(Some((sub_optimal_result, f_min)))
    }
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, name="standard_focal_a_star_search", fields(agent = agent.id, subopt_factor = subopt_factor, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn standard_focal_a_star_search(
    map: &Map,
    agent: &Agent,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    constraint_limit_time_step: usize,
    paths: &[Path],
    stats: &mut Stats,
) -> Option<(Path, usize)> {
    debug!("constraints: {constraints:?}, limit time step: {constraint_limit_time_step:?}");

    let mut f_min = 0;

    // Open list is indexed based on (f_open_cost, g_cost(time), position)
    #[allow(clippy::mutable_key_type)]
    let mut open_list = BTreeSet::new();
    #[allow(clippy::mutable_key_type)]
    let mut focal_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let mut f_focal_cost_map = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];

    let (start_open_node, start_focal_node) =
        create_open_focal_node(agent.start, start_h_open_cost, 0, 0, 0);
    open_list.insert(start_open_node);
    focal_list.insert(start_focal_node);

    f_focal_cost_map.insert((agent.start, 0), 0);

    while let Some(current_wrapper) = focal_list.pop_first() {
        let current_ref = &current_wrapper.0;
        let current = current_ref.borrow();
        trace!("expand node: {current:?}");
        let exceed_constraints_limit_time_step = current.time_step > constraint_limit_time_step;

        // Update stats.
        stats.low_level_expand_focal_nodes += 1;

        closed_list.insert((current.position, current.time_step));

        f_min = max(open_list.first().unwrap().f_open_cost(), f_min);

        // Remove the same node from open list.
        assert!(open_list.remove(&OpenOrderWrapper::from_node(current_ref)));

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

            let (open_node_wrapper, focal_node_wrapper) = create_open_focal_node(
                *neighbor,
                f_open_cost,
                f_focal_cost,
                tentative_g_cost,
                tentative_g_cost,
            );

            // Check if we seen this node before
            if let Some(&old_f_focal_cost) = f_focal_cost_map.get(&(*neighbor, tentative_g_cost)) {
                // This node is appeared before, we need to check whether it has smaller focal cost
                if f_focal_cost < old_f_focal_cost {
                    // Update its corresponding focal cost,
                    // update focal cost map and focal list if it is in there.
                    f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);

                    // We need to check if this node still remain unexpanded
                    let (old_open_node_wrapper, old_focal_node_wrapper) = create_open_focal_node(
                        *neighbor,
                        f_open_cost,
                        old_f_focal_cost,
                        tentative_g_cost,
                        tentative_time_step,
                    );

                    if focal_list.remove(&old_focal_node_wrapper) {
                        // If focal list has this node, then open list must also have
                        assert!(open_list.remove(&old_open_node_wrapper));
                        open_list.insert(open_node_wrapper);

                        // Update old node in focal list
                        focal_list.insert(focal_node_wrapper);
                    } else if open_list.remove(&old_open_node_wrapper) {
                        // There still has possible only open list contain this old node
                        open_list.insert(open_node_wrapper);
                    }
                }
            } else {
                // This node is never appeared before, update open list and trace
                // Also means this node is new to focal history, update focal cost hashmap
                assert!(open_list.insert(open_node_wrapper));

                trace.insert(
                    (*neighbor, tentative_g_cost),
                    (current.position, current.g_cost),
                );
                f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);

                // If the node is in the suboptimal bound, push it into focal list.
                if f_open_cost as f64 <= (f_min as f64 * subopt_factor) {
                    focal_list.insert(focal_node_wrapper);
                }
            }
        }

        if !open_list.is_empty() {
            // Maintain the focal list, since we have changed the f min.
            let new_f_min = open_list.first().unwrap().0.borrow().f_open_cost;
            if f_min < new_f_min {
                open_list.iter().for_each(|open_wrapper| {
                    let node_ref = &open_wrapper.0;
                    let node = node_ref.borrow();
                    if node.f_open_cost as f64 > f_min as f64 * subopt_factor
                        && node.f_open_cost as f64 <= new_f_min as f64 * subopt_factor
                    {
                        focal_list.insert(FocalOrderWrapper::from_node(node_ref));
                    }
                });
            }
        }

        trace!("{:?}", open_list);
    }

    debug!("cannot find solution");
    None
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, name="double_search", fields(agent = agent.id, subopt_factor = subopt_factor, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn standard_focal_double_search(
    map: &Map,
    agent: &Agent,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    constraint_limit_time_step: usize,
    paths: &[Path],
    stats: &mut Stats,
) -> Option<(Path, usize)> {
    debug!("constraints: {constraints:?}, limit time step: {constraint_limit_time_step:?}");

    if let Some((_, f_min)) = standard_a_star_search_open_cost(
        map,
        agent,
        constraints,
        path_length_constraint,
        constraint_limit_time_step,
        stats,
    ) {
        standard_a_star_search_focal_cost(
            map,
            agent,
            constraints,
            path_length_constraint,
            paths,
            constraint_limit_time_step,
            f_min as f64 * subopt_factor,
            stats,
        )
    } else {
        debug!("cannot find solution");
        None
    }
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all, name="alternating_focal_a_star_search", fields(agent = agent.id, subopt_factor = subopt_factor, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn alternating_focal_a_star_search(
    map: &Map,
    agent: &Agent,
    subopt_factor: f64,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    constraint_limit_time_step: usize,
    paths: &[Path],
    stats: &mut Stats,
) -> Option<(Path, usize)> {
    debug!("constraints: {constraints:?}, limit time step: {constraint_limit_time_step:?}");

    let mut f_min = 0;

    // Open list is indexed based on (f_open_cost, g_cost(time), position)
    #[allow(clippy::mutable_key_type)]
    let mut open_list = BTreeSet::new();
    #[allow(clippy::mutable_key_type)]
    let mut focal_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let mut f_focal_cost_map = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];

    let (start_open_node, start_focal_node) =
        create_open_focal_node(agent.start, start_h_open_cost, 0, 0, 0);
    open_list.insert(start_open_node);
    focal_list.insert(start_focal_node);

    f_focal_cost_map.insert((agent.start, 0), 0);

    // Flag to alternate between focal and open list
    let mut use_focal = true;

    // If open is empty, then focal must be empty, if focal is empty, open might not be empty
    // We just check open list here
    while !open_list.is_empty() {
        // Choose which queue to expand from and get the node
        let current_ref;
        let current;

        if use_focal && !focal_list.is_empty() {
            // Extract from focal list
            let focal_wrapper = focal_list.pop_first().unwrap();
            current_ref = focal_wrapper.0;
            current = current_ref.borrow();
            trace!("expand node from focal: {current:?}");

            // Update f_min based on the first node in open list
            f_min = max(open_list.first().unwrap().f_open_cost(), f_min);

            // Remove the corresponding node from open list
            assert!(open_list.remove(&OpenOrderWrapper::from_node(&current_ref)));

            // Update stats for focal expansion
            stats.low_level_expand_focal_nodes += 1;

            // Toggle flag for next iteration
            use_focal = false;
        } else {
            // Extract from open list
            let open_wrapper = open_list.pop_first().unwrap();
            current_ref = open_wrapper.0;
            current = current_ref.borrow();
            trace!("expand node from open: {current:?}");

            // Update f_min based on the first node in open list (which is current)
            f_min = max(current.f_open_cost, f_min);

            // Remove the corresponding node from focal list if it's there
            focal_list.remove(&FocalOrderWrapper::from_node(&current_ref));

            // Update stats for open list expansion
            stats.low_level_expand_open_nodes += 1;

            // Toggle flag for next iteration
            use_focal = true;
        }

        let exceed_constraints_limit_time_step = current.time_step > constraint_limit_time_step;

        // Update stats
        if use_focal {
            stats.low_level_expand_focal_nodes += 1;
        } else {
            stats.low_level_expand_open_nodes += 1;
        }

        closed_list.insert((current.position, current.time_step));

        if current.position == agent.goal && current.g_cost > path_length_constraint {
            debug!("find solution with f min {f_min:?}");
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                f_min,
            ));
        }

        // Assuming uniform cost
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
            // If node (position at current time) has closed, ignore
            if closed_list.contains(&(*neighbor, tentative_time_step)) {
                continue;
            }

            // Check for constraints before exploring the neighbor
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

            let (open_node_wrapper, focal_node_wrapper) = create_open_focal_node(
                *neighbor,
                f_open_cost,
                f_focal_cost,
                tentative_g_cost,
                tentative_time_step,
            );

            // Check if we see this node before
            if let Some(&old_f_focal_cost) = f_focal_cost_map.get(&(*neighbor, tentative_g_cost)) {
                // This node is appeared before, we need to check whether it has smaller focal cost
                if f_focal_cost < old_f_focal_cost {
                    // Update its corresponding focal cost,
                    // update focal cost map and focal list if it is in there.
                    f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);

                    // We need to check if this node still remain unexpanded
                    let (old_open_node_wrapper, old_focal_node_wrapper) = create_open_focal_node(
                        *neighbor,
                        f_open_cost,
                        old_f_focal_cost,
                        tentative_g_cost,
                        tentative_time_step,
                    );

                    if focal_list.remove(&old_focal_node_wrapper) {
                        // If focal list has this node, then open list must also have
                        assert!(open_list.remove(&old_open_node_wrapper));
                        open_list.insert(open_node_wrapper);

                        // Update old node in focal list
                        focal_list.insert(focal_node_wrapper);
                    } else if open_list.remove(&old_open_node_wrapper) {
                        // There still has possible only open list contain this old node
                        open_list.insert(open_node_wrapper);
                    }
                }
            } else {
                // This node is never appeared before, update open list and trace
                // Also means this node is new to focal history, update focal cost hashmap
                assert!(open_list.insert(open_node_wrapper));

                trace.insert(
                    (*neighbor, tentative_g_cost),
                    (current.position, current.g_cost),
                );
                f_focal_cost_map.insert((*neighbor, tentative_g_cost), f_focal_cost);

                // If the node is in the suboptimal bound, push it into focal list.
                if f_open_cost as f64 <= (f_min as f64 * subopt_factor) {
                    focal_list.insert(focal_node_wrapper);
                }
            }
        }

        if !open_list.is_empty() {
            // Maintain the focal list, since we have changed the f min
            if let Some(first) = open_list.first() {
                let new_f_min = first.0.borrow().f_open_cost;
                if f_min < new_f_min {
                    open_list.iter().for_each(|open_wrapper| {
                        let node_ref = &open_wrapper.0;
                        let node = node_ref.borrow();
                        if node.f_open_cost as f64 > f_min as f64 * subopt_factor
                            && node.f_open_cost as f64 <= new_f_min as f64 * subopt_factor
                        {
                            focal_list.insert(FocalOrderWrapper::from_node(node_ref));
                        }
                    });
                }
                f_min = new_f_min;
            }
        }

        trace!("open list {:?}", open_list);
        trace!("focal list {:?}", focal_list);
    }

    debug!("cannot find solution");
    None
}
