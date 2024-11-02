use super::{construct_path, heuristic};
use crate::map::Map;
use crate::solver::comm::{Constraint, LowLevelNode};
use crate::solver::Stats;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    usize,
};

pub(crate) fn a_star_search(
    map: &Map,
    start: (usize, usize),
    goal: (usize, usize),
    constraints: &HashSet<Constraint>,
    stats: &mut Stats,
) -> Option<Vec<(usize, usize)>> {
    let max_time = constraints.iter().map(|c| c.time_step).max().unwrap_or(0);

    let mut open = BinaryHeap::new();
    let mut trace: HashMap<((usize, usize), usize), ((usize, usize), usize)> = HashMap::new();
    let mut g_cost = HashMap::new();

    g_cost.insert((start, 0), 0);

    let start_node = LowLevelNode {
        position: start,
        f_cost: heuristic(start, goal),
        g_cost: 0,
        h_open_cost: heuristic(start, goal),
        h_focal_cost: 0,
        time: 0,
    };

    open.push(start_node);

    while let Some(current) = open.pop() {
        let current_time = current.time;

        if current.position == goal && current_time > max_time {
            return Some(construct_path(&trace, (current.position, current_time)));
        }

        // Time step increases as we move to the next node.
        let next_time = current_time + 1;

        // Assuming uniform cost, typically 1 in a grid.
        let tentative_g_cost = current.g_cost + 1;

        for neighbor in &map.get_neighbors(current.position.0, current.position.1) {
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
                let h_open_cost = heuristic(*neighbor, goal);
                open.push(LowLevelNode {
                    position: *neighbor,
                    f_cost: tentative_g_cost + h_open_cost,
                    g_cost: tentative_g_cost,
                    h_open_cost,
                    h_focal_cost: 0,
                    time: next_time,
                });
            }
        }

        // Update stats
        stats.low_level_expand_nodes += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ideal Path
    // [(25, 14), (24, 14), (23, 14), (22, 14), (21, 14),
    //  (20, 14), (19, 14), (18, 14), (17, 14), (17, 15),
    //  (17, 16), (17, 17), (16, 17), (15, 17), (14, 17),
    //  (14, 18), (14, 19), (15, 19), (16, 19), (17, 19)]
    #[test]
    fn test_a_star_normal() {
        let map = Map::from_file("map_file/test/test.map").unwrap();
        let constraints = HashSet::new();
        let stats = &mut Stats::default();
        let path = a_star_search(&map, (25, 14), (17, 19), &constraints, stats).unwrap();
        assert_eq!(path.len(), 20);
    }

    #[test]
    fn test_a_star_in_path_conflict() {
        let map = Map::from_file("map_file/test/test.map").unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint {
            position: (23, 14),
            time_step: 2,
        });
        let stats = &mut Stats::default();
        let path = a_star_search(&map, (25, 14), (17, 19), &constraints, stats).unwrap();
        assert_eq!(path.len(), 21);
    }

    #[test]
    fn test_a_star_out_path_conflict() {
        let map = Map::from_file("map_file/test/test.map").unwrap();
        let mut constraints = HashSet::new();
        constraints.insert(Constraint {
            position: (17, 19),
            time_step: 29,
        });
        let stats = &mut Stats::default();
        let path = a_star_search(&map, (25, 14), (17, 19), &constraints, stats).unwrap();
        assert_eq!(path.len(), 31);
    }
}
