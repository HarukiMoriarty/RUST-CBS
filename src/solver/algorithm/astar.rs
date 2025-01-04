use super::{construct_path, Path};
use crate::common::Agent;
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelOpenNode, Mdd, SearchResult};
use crate::stat::Stats;
use std::collections::{BTreeSet, HashMap, HashSet};
use tracing::{debug, instrument, trace};

#[instrument(skip_all, name="low_level_standard_a_star", fields(agent = agent.id, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn standard_a_star_search(
    map: &Map,
    agent: &Agent,
    constraints: &HashSet<Constraint>,
    path_length_constraint: usize,
    stats: &mut Stats,
) -> Option<(Path, usize)> {
    debug!("constraints: {constraints:?}");

    let mut open_list = BTreeSet::new();
    let mut closed_list = HashSet::new();
    let mut trace = HashMap::new();

    let start_h_open_cost = map.heuristic[agent.id][agent.start.0][agent.start.1];
    let start_node = LowLevelOpenNode {
        position: agent.start,
        f_open_cost: start_h_open_cost,
        g_cost: 0,
    };
    open_list.insert(start_node.clone());

    while let Some(current) = open_list.pop_first() {
        trace!("expand node: {current:?}");

        // Update stats.
        stats.low_level_expand_open_nodes += 1;

        closed_list.insert((current.position, current.g_cost));

        if current.position == agent.goal && current.g_cost > path_length_constraint {
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                current.f_open_cost,
            ));
        }

        // Assuming uniform cost, which also indicate the current time.
        let tentative_g_cost = current.g_cost + 1;

        // Expand nodes from the current position.
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            // Checck node (position at current time) has closed.
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

            // If this node has already in the open list, we ignore this update.
            if open_list.insert(LowLevelOpenNode {
                position: *neighbor,
                f_open_cost: tentative_g_cost + h_open_cost,
                g_cost: tentative_g_cost,
            }) {
                trace.insert(
                    (*neighbor, tentative_g_cost),
                    (current.position, current.g_cost),
                );
            }
        }
        trace!("open list {open_list:?}");
    }

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
    if !build_mdd {
        return SearchResult::Standard(standard_a_star_search(
            map,
            agent,
            constraints,
            path_length_constraint,
            stats,
        ));
    }

    let optimal_result =
        match standard_a_star_search(map, agent, constraints, path_length_constraint, stats) {
            Some((path, cost)) => (path, cost),
            None => return SearchResult::WithMDD(None),
        };

    // Build MDD using optimal cost
    let mut mdd_layers = vec![HashSet::new()];
    mdd_layers[0].insert(agent.start);
    let mut open_list = BTreeSet::new();
    let mut closed_list = BTreeSet::new();

    let start_node = LowLevelOpenNode {
        position: agent.start,
        f_open_cost: map.heuristic[agent.id][agent.start.0][agent.start.1],
        g_cost: 0,
    };
    open_list.insert(start_node);

    while let Some(current) = open_list.pop_first() {
        trace!("expand node: {current:?}");

        // Update stats.
        stats.low_level_mdd_expand_open_nodes += 1;

        closed_list.insert((current.position, current.g_cost));

        // Assuming uniform cost, which also indicate the current time.
        let tentative_g_cost = current.g_cost + 1;

        while mdd_layers.len() <= tentative_g_cost && tentative_g_cost <= optimal_result.1 {
            mdd_layers.push(HashSet::new());
        }

        // Expand nodes from the current position.
        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
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
            if f_cost > optimal_result.1 {
                continue;
            }

            let new_node = LowLevelOpenNode {
                position: *neighbor,
                f_open_cost: f_cost,
                g_cost: tentative_g_cost,
            };

            if open_list.insert(new_node) {
                mdd_layers[tentative_g_cost].insert(*neighbor);
            }
        }
    }

    SearchResult::WithMDD(Some((
        optimal_result.0,
        optimal_result.1,
        Mdd { layers: mdd_layers },
    )))
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::common::Agent;

    // Helper function to extract path from SearchResult
    fn get_path_from_result(result: SearchResult) -> Option<(Path, usize)> {
        match result {
            SearchResult::Standard(result) => result,
            SearchResult::WithMDD(result) => result.map(|(path, cost, _)| (path, cost)),
        }
    }

    // Ideal Path
    // [(25, 14), (24, 14), (23, 14), (22, 14), (21, 14),
    //  (20, 14), (19, 14), (18, 14), (17, 14), (17, 15),
    //  (17, 16), (17, 17), (16, 17), (15, 17), (14, 17),
    //  (14, 18), (14, 19), (15, 19), (16, 19), (17, 19)]
    #[test]
    fn test_a_star_normal() {
        let agent = Agent {
            id: 0,
            start: (25, 14),
            goal: (17, 19),
        };
        let map = Map::from_file(
            "map_file/maze-32-32-2-scen-even/maze-32-32-2.map",
            &vec![agent.clone()],
        )
        .unwrap();
        let constraints = HashSet::new();
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        assert_eq!(path.len(), 20);
    }

    #[test]
    fn test_a_star_in_path_conflict() {
        let agent = Agent {
            id: 0,
            start: (25, 14),
            goal: (17, 19),
        };
        let map = Map::from_file(
            "map_file/maze-32-32-2-scen-even/maze-32-32-2.map",
            &vec![agent.clone()],
        )
        .unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint {
            position: (23, 14),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 0, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        assert_eq!(path.len(), 21);
    }

    #[test]
    fn test_a_star_path_length_constraint() {
        let agent = Agent {
            id: 0,
            start: (25, 14),
            goal: (17, 19),
        };
        let map = Map::from_file(
            "map_file/maze-32-32-2-scen-even/maze-32-32-2.map",
            &vec![agent.clone()],
        )
        .unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint {
            position: (17, 19),
            time_step: 29,
            is_permanent: false,
        });
        let stats = &mut Stats::default();
        let result = a_star_search(&map, &agent, &constraints, 29, false, stats);
        let (path, _) = get_path_from_result(result).unwrap();
        println!("{path:?}");
        assert_eq!(path.len(), 31);
    }

    #[test]
    fn test_a_star_with_mdd() {
        let agent = Agent {
            id: 0,
            start: (25, 14),
            goal: (17, 19),
        };
        let map = Map::from_file(
            "map_file/maze-32-32-2-scen-even/maze-32-32-2.map",
            &vec![agent.clone()],
        )
        .unwrap();
        let constraints = HashSet::new();
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 0, true, stats)
        {
            assert_eq!(path.len(), 20);
            println!("{mdd:?}");

            // Start position should be singleton
            assert!(mdd.is_singleton_at_position(0, (25, 14)));

            // Critical points where path has no alternatives should be singletons
            assert!(mdd.is_singleton_at_position(8, (17, 14)));

            // Points with potential alternatives should not be singletons
            assert!(!mdd.is_singleton_at_position(15, (14, 19))); // Around (14, 19)

            // Check some MDD layer sizes for bottleneck points
            assert_eq!(mdd.layers[8].len(), 1); // Turning point
            assert!(mdd.layers[13].len() > 1); // Area with alternatives

            // Goal position in final layer
            assert!(mdd.layers[path.len() - 1].contains(&(17, 19)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }

    #[test]
    fn test_a_star_in_path_conflicts_with_mdd() {
        let agent = Agent {
            id: 0,
            start: (25, 14),
            goal: (17, 19),
        };
        let map = Map::from_file(
            "map_file/maze-32-32-2-scen-even/maze-32-32-2.map",
            &vec![agent.clone()],
        )
        .unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint {
            position: (23, 14),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 0, true, stats)
        {
            assert_eq!(path.len(), 21);
            println!("{mdd:?}");

            // Start position should be singleton
            assert!(mdd.is_singleton_at_position(0, (25, 14)));

            // Critical points where path has no alternatives should be singletons
            assert!(mdd.is_singleton_at_position(9, (17, 14)));

            // Points with potential alternatives should not be singletons
            assert!(!mdd.is_singleton_at_position(16, (14, 19))); // Around (14, 19)

            // Check some MDD layer sizes for bottleneck points
            assert_eq!(mdd.layers[9].len(), 1); // Turning point
            assert!(mdd.layers[14].len() > 1); // Area with alternatives

            // Goal position in final layer
            assert!(mdd.layers[path.len() - 1].contains(&(17, 19)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }

    #[test]
    fn test_a_star_path_length_constraints_with_mdd() {
        let agent = Agent {
            id: 0,
            start: (25, 14),
            goal: (17, 19),
        };
        let map = Map::from_file(
            "map_file/maze-32-32-2-scen-even/maze-32-32-2.map",
            &vec![agent.clone()],
        )
        .unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint {
            position: (23, 14),
            time_step: 2,
            is_permanent: false,
        });
        let stats = &mut Stats::default();

        if let SearchResult::WithMDD(Some((path, _, mdd))) =
            a_star_search(&map, &agent, &constraints, 29, true, stats)
        {
            assert_eq!(path.len(), 31);
            println!("{mdd:?}");

            // Start position should be singleton
            assert!(mdd.is_singleton_at_position(0, (25, 14)));

            // Critical points where path has no alternatives should be singletons
            assert!(!mdd.is_singleton_at_position(9, (17, 14)));

            // Points with potential alternatives should not be singletons
            assert!(!mdd.is_singleton_at_position(16, (14, 19))); // Around (14, 19)

            // Check some MDD layer sizes for bottleneck points
            assert!(mdd.layers[9].len() > 1); // Turning point
            assert!(mdd.layers[14].len() > 1); // Area with alternatives

            // Goal position in final layer
            assert!(mdd.layers[path.len() - 1].contains(&(17, 19)));
        } else {
            panic!("Expected WithMDD result with valid path and Mdd");
        }
    }
}
