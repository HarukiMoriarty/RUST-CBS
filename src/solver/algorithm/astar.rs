use super::construct_path;
use crate::common::Agent;
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelNode};
use crate::stat::Stats;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    usize,
};

pub(crate) fn a_star_search(
    map: &Map,
    agent: &Agent,
    constraints: &HashSet<Constraint>,
    stats: &mut Stats,
) -> Option<(Vec<(usize, usize)>, Option<usize>)> {
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

    let mut open = BinaryHeap::new();
    let mut close = HashSet::new();
    let mut trace: HashMap<((usize, usize), usize), ((usize, usize), usize)> = HashMap::new();
    let mut g_cost = HashMap::new();

    g_cost.insert((agent.start, 0), 0);

    let start_heuristic = map.heuristic[agent.id][agent.start.0][agent.start.1];
    let start_node = LowLevelNode {
        position: agent.start,
        sort_key: start_heuristic,
        g_cost: 0,
        h_open_cost: start_heuristic,
        h_focal_cost: None,
        time: 0,
    };

    open.push(start_node);

    while let Some(current) = open.pop() {
        // Update stats
        stats.low_level_expand_nodes += 1;
        close.insert((current.position, current.time));
        let current_time = current.time;

        if current.position == agent.goal && current_time > max_time {
            return Some((
                construct_path(&trace, (current.position, current_time)),
                None,
            ));
        }

        // Time step increases as we move to the next node.
        let next_time = current_time + 1;

        // Assuming uniform cost, typically 1 in a grid.
        let tentative_g_cost = current.g_cost + 1;

        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
            if close.contains(&(*neighbor, next_time)) {
                continue;
            }

            // Check for constraints before exploring the neighbor.
            if constraints.contains(&Constraint {
                position: *neighbor,
                time_step: next_time,
            }) {
                continue; // This move is prohibited due to a constraint.
            }

            if tentative_g_cost < *g_cost.get(&(*neighbor, next_time)).unwrap_or(&usize::MAX) {
                trace.insert((*neighbor, next_time), (current.position, current_time));
                g_cost.insert((*neighbor, next_time), tentative_g_cost);
                let h_open_cost = map.heuristic[agent.id][neighbor.0][neighbor.1];
                open.push(LowLevelNode {
                    position: *neighbor,
                    sort_key: tentative_g_cost + h_open_cost,
                    g_cost: tentative_g_cost,
                    h_open_cost,
                    h_focal_cost: None,
                    time: next_time,
                });
            }
        }
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
