use super::{construct_mdd, construct_path};
use crate::common::{Agent, Constraint, LowLevelOpenNode, Path, SearchResult};
use crate::map::Map;
use crate::stat::Stats;

use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::{debug, instrument, trace};

#[instrument(skip_all, name="standard_a_star", fields(agent = agent.id, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn standard_a_star_search(
    map: &Map,
    agent: &Agent,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    constraint_limit_time_step: usize,
    stats: &mut Stats,
) -> Option<(Path, usize)> {
    debug!("constraints: {constraints:?}, limit time step: {constraint_limit_time_step:?}");

    let mut open_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];
    let start_node = LowLevelOpenNode {
        position: agent.start,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
        time_step: 0,
    };
    open_list.insert(start_node.clone());

    while let Some(current) = open_list.pop_first() {
        trace!("expand node: {current:?}");
        let exceed_constraints_limit_time_step = current.time_step > constraint_limit_time_step;

        // Update stats.
        stats.low_level_expand_open_nodes += 1;

        closed_list.insert((current.position, current.time_step));

        if current.position == agent.goal && current.g_cost > path_length_constraint {
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                current.f_open_cost,
            ));
        }

        // Assuming uniform cost, which also indicate the current time.
        let tentative_g_cost = current.g_cost + 1;

        // time step only increase if we haven't passed constraint limit
        // Tricky: after constraint limit, we fixed time step as T + 1, algorithm
        // demote to 2-D a star, enable branch pruning
        let tentative_time_step = if exceed_constraints_limit_time_step {
            current.time_step
        } else {
            current.time_step + 1
        };

        // Expand nodes from the current position.
        for neighbor in &map.get_neighbors(
            current.position.0,
            current.position.1,
            !exceed_constraints_limit_time_step,
        ) {
            // Check node (position at current time) has closed.
            if closed_list.contains(&(*neighbor, tentative_time_step)) {
                continue;
            }

            // Check for constraints before exploring the neighbor
            if constraints.iter().any(|constraint| {
                constraint.is_violated(current.position, *neighbor, tentative_g_cost)
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];

            // If this node has already in the open list, we ignore this update.
            if open_list.insert(LowLevelOpenNode {
                position: *neighbor,
                f_open_cost: tentative_g_cost + h_open_cost,
                g_cost: tentative_g_cost,
                time_step: tentative_time_step,
            }) {
                trace.insert(
                    (*neighbor, tentative_g_cost),
                    (current.position, current.g_cost),
                );
            }
        }
        trace!("open list {open_list:?}");
    }

    debug!("cannot find solution");
    None
}

#[instrument(skip_all, name="a_star", fields(agent = agent.id, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn a_star_search(
    map: &Map,
    agent: &Agent,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
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
        return SearchResult::Standard(standard_a_star_search(
            map,
            agent,
            constraints,
            path_length_constraint,
            constraint_limit_time_step,
            stats,
        ));
    }

    let (path, f_min) = match standard_a_star_search(
        map,
        agent,
        constraints,
        path_length_constraint,
        constraint_limit_time_step,
        stats,
    ) {
        Some((path, f_min)) => (path, f_min),
        None => return SearchResult::WithMDD(None),
    };

    // f min should equal to cost.
    assert_eq!(path.len() - 1, f_min);

    // Build MDD using optimal cost.
    SearchResult::WithMDD(Some((
        path,
        f_min,
        construct_mdd(map, agent, constraints, f_min),
    )))
}

#[cfg(test)]
mod tests {
    use std::vec;
    use tracing_subscriber;

    use super::*;
    use crate::common::{is_singleton_at_position, Agent, Mdd};

    // Helper function to setup tracing
    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("trace")
            .try_init();
    }

    // Helper function to extract path from SearchResult.
    fn get_path_from_result(result: SearchResult) -> Option<(Path, usize)> {
        match result {
            SearchResult::Standard(result) => result,
            SearchResult::WithMDD(result) => result.map(|(path, cost, _)| (path, cost)),
        }
    }

    // Helper function to examine Mdd.
    fn check_mdd_layer_positions(
        mdd: &Mdd,
        layer: usize,
        expected_positions: HashSet<(usize, usize)>,
    ) {
        let actual_positions: HashSet<_> = mdd[layer].keys().cloned().collect();
        assert_eq!(actual_positions, expected_positions);
    }

    // Ideal Path
    // [(2, 2), (1, 2), (0, 2), (0, 1), (0, 0)]
    // or
    // [(2, 2), (2, 1), (2, 0), (1, 0), (0, 0)]
    #[test]
    fn test_a_star_no_constraint_without_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let constraints = HashSet::new();
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        debug!("{path:?}");
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn test_a_star_in_path_vertex_constraint_alternative_path_without_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Vertex {
            position: (0, 2),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        debug!("{path:?}");
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn test_a_star_in_path_vertex_constraint_without_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Vertex {
            position: (0, 2),
            time_step: 2,
            is_permanent: false,
        });
        constraints.insert(Constraint::Vertex {
            position: (2, 0),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        debug!("{path:?}");
        assert_eq!(path.len(), 6);
    }

    #[test]
    fn test_a_star_path_length_constraint_without_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Vertex {
            position: (0, 0),
            time_step: 4,
            is_permanent: false,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 4, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        debug!("{path:?}");
        assert_eq!(path.len(), 6);
    }

    #[test]
    fn test_a_star_no_constraint_with_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let constraints = HashSet::new();
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 0, true, stats)
        {
            assert_eq!(path.len(), 5);
            debug!("{mdd:?}");

            // Start position should be singleton.
            assert!(is_singleton_at_position(&mdd, 0, (2, 2)));

            check_mdd_layer_positions(&mdd, 1, HashSet::from([(1, 2), (2, 1)]));
            check_mdd_layer_positions(&mdd, 2, HashSet::from([(0, 2), (2, 0)]));
            check_mdd_layer_positions(&mdd, 3, HashSet::from([(0, 1), (1, 0)]));

            // End position should be singleton.
            assert!(is_singleton_at_position(&mdd, 4, (0, 0)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }

    #[test]
    fn test_a_star_in_path_vertex_constraint_alternative_path_with_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Vertex {
            position: (0, 2),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 0, true, stats)
        {
            assert_eq!(path.len(), 5);
            debug!("{mdd:?}");

            // Start position should be singleton.
            assert!(is_singleton_at_position(&mdd, 0, (2, 2)));

            assert!(is_singleton_at_position(&mdd, 1, (2, 1)));
            assert!(is_singleton_at_position(&mdd, 2, (2, 0)));
            assert!(is_singleton_at_position(&mdd, 3, (1, 0)));

            // End position should be singleton.
            assert!(is_singleton_at_position(&mdd, 4, (0, 0)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }

    #[test]
    fn test_a_star_in_path_vertex_constraint_with_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Vertex {
            position: (0, 2),
            time_step: 2,
            is_permanent: false,
        });
        constraints.insert(Constraint::Vertex {
            position: (2, 0),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 0, true, stats)
        {
            assert_eq!(path.len(), 6);
            debug!("{mdd:?}");

            // Start position should be singleton.
            assert!(is_singleton_at_position(&mdd, 0, (2, 2)));

            check_mdd_layer_positions(&mdd, 1, HashSet::from([(2, 2), (1, 2), (2, 1)]));
            check_mdd_layer_positions(&mdd, 2, HashSet::from([(1, 2), (2, 1)]));
            check_mdd_layer_positions(&mdd, 3, HashSet::from([(0, 2), (2, 0)]));
            check_mdd_layer_positions(&mdd, 4, HashSet::from([(0, 1), (1, 0)]));

            // End position should be singleton.
            assert!(is_singleton_at_position(&mdd, 5, (0, 0)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }

    #[test]
    fn test_a_star_path_length_constraint_with_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Vertex {
            position: (0, 0),
            time_step: 4,
            is_permanent: false,
        });
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 4, true, stats)
        {
            assert_eq!(path.len(), 6);
            debug!("{mdd:?}");

            // Start position should be singleton.
            assert!(is_singleton_at_position(&mdd, 0, (2, 2)));

            check_mdd_layer_positions(&mdd, 1, HashSet::from([(2, 2), (1, 2), (2, 1)]));
            check_mdd_layer_positions(&mdd, 2, HashSet::from([(1, 2), (2, 1), (2, 0), (0, 2)]));
            check_mdd_layer_positions(&mdd, 3, HashSet::from([(0, 2), (2, 0), (1, 0), (0, 1)]));
            check_mdd_layer_positions(&mdd, 4, HashSet::from([(0, 1), (1, 0)]));

            // Goal position in final layer
            assert!(is_singleton_at_position(&mdd, 5, (0, 0)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }

    #[test]
    fn test_a_star_edge_constraint_alternative_path_without_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Edge {
            from_position: (0, 2),
            to_position: (1, 2),
            to_time_step: 2,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        debug!("{path:?}");
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn test_a_star_edge_constraint_without_mdd() {
        init_tracing();
        let agent = Agent {
            id: 0,
            start: (2, 2),
            goal: (0, 0),
        };
        let map = Map::from_file("map_file/test/test.map", &vec![agent.clone()]).unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint::Edge {
            from_position: (1, 2),
            to_position: (0, 2),
            to_time_step: 2,
        });
        constraints.insert(Constraint::Edge {
            from_position: (2, 0),
            to_position: (1, 0),
            to_time_step: 3,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        debug!("{path:?}");
        assert_eq!(path.len(), 6);
    }
}
