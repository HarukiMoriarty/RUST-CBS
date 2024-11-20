use super::construct_path;
use crate::common::Agent;
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelOpenNode};
use crate::stat::Stats;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    usize,
};
use tracing::{debug, instrument, trace};

#[instrument(skip_all, name="low_level_a_star", fields(agent = agent.id, start = format!("{:?}", agent.start), goal = format!("{:?}", agent.goal)), level = "debug")]
pub(crate) fn a_star_search(
    map: &Map,
    agent: &Agent,
    constraints: &HashSet<Constraint>,
    stats: &mut Stats,
) -> Option<(Vec<(usize, usize)>, Option<usize>)> {
    debug!("constraints: {constraints:?}");
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

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
        stats.low_level_expand_nodes += 1;

        closed_list.insert((current.position, current.g_cost));

        if current.position == agent.goal && current.g_cost > max_time {
            return Some((
                construct_path(&trace, (current.position, current.g_cost)),
                None,
            ));
        }

        // Assuming uniform cost, which also indicate the current time.
        let tentative_g_cost = current.g_cost + 1;

        // Expand nodes from the current position.
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

            // If this node has already in the open list, we ignore this update
            // since we have the same g cost (time) and h cost.
            // Actually here we apply a symmetric optimization, we ingore the paths
            // lead to current node that has the same cost.
            // A potential problem here we might ignore paths that might lead to less conflicts,
            // the optimal property should be kept since any conflicts will lead to a high level
            // constraints later, but might lead to potential inefficiency.
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

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::common::Agent;

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
        let (path, _) = a_star_search(&map, &agent, &constraints, stats).unwrap();
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
        });
        let stats = &mut Stats::default();
        let (path, _) = a_star_search(&map, &agent, &constraints, stats).unwrap();
        assert_eq!(path.len(), 21);
    }

    #[test]
    fn test_a_star_out_path_conflict() {
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
        });
        let stats = &mut Stats::default();
        let (path, _) = a_star_search(&map, &agent, &constraints, stats).unwrap();
        assert_eq!(path.len(), 31);
    }
}
