use super::comm::HighLevelNode;
use super::Solver;
use crate::common::{Agent, Solution};
use crate::map::Map;
use crate::stat::Stats;
use std::time::Instant;
use tracing::debug;

use std::collections::{BTreeMap, BinaryHeap, HashSet};

pub struct HBCBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
    subopt_factor: (Option<f64>, Option<f64>), // The lattar one should be always none for HBCBS
}

impl HBCBS {
    pub fn new(agents: Vec<Agent>, map: &Map, subopt_factor: (Option<f64>, Option<f64>)) -> Self {
        HBCBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
            subopt_factor,
        }
    }
}

impl Solver for HBCBS {
    fn solve(&mut self) -> Option<Solution> {
        let total_solve_start_time = Instant::now();

        // Open lisst is indexed based on (cost, conflicts_hash)
        let mut open = BTreeMap::new();
        let mut focal = BinaryHeap::new();
        let mut closed = HashSet::new();

        if let Some(root) = HighLevelNode::new(&self.agents, &self.map, None, &mut self.stats) {
            open.insert((root.cost, root.conflicts_hash.unwrap_or(0)), root.clone());
            focal.push(root);
            while let Some(current_node) = focal.pop() {
                let f_min = current_node.cost;
                open.remove(&(current_node.cost, current_node.conflicts_hash.unwrap_or(0)));
                closed.insert(current_node.clone());
                if !current_node.conflicts.is_empty() {
                    for conflict in &current_node.conflicts {
                        if let Some(child_1) = current_node.update_constraint(
                            conflict,
                            true,
                            &self.map,
                            None,
                            &mut self.stats,
                        ) {
                            if !closed.contains(&child_1) {
                                open.insert(
                                    (child_1.cost, child_1.conflicts_hash.unwrap_or(0)),
                                    child_1.clone(),
                                );
                                self.stats.high_level_expand_nodes += 1;

                                if child_1.cost as f64
                                    <= (f_min as f64 * self.subopt_factor.0.unwrap())
                                {
                                    focal.push(child_1);
                                }
                            }
                        }

                        if let Some(child_2) = current_node.update_constraint(
                            conflict,
                            false,
                            &self.map,
                            None,
                            &mut self.stats,
                        ) {
                            if !closed.contains(&child_2) {
                                open.insert(
                                    (child_2.cost, child_2.conflicts_hash.unwrap_or(0)),
                                    child_2.clone(),
                                );
                                self.stats.high_level_expand_nodes += 1;

                                if child_2.cost as f64
                                    <= (f_min as f64 * self.subopt_factor.0.unwrap())
                                {
                                    focal.push(child_2);
                                }
                            }
                        }
                    }
                } else {
                    // No conflicts, return solution
                    debug!("Find solution");
                    let total_solve_time = total_solve_start_time.elapsed();
                    self.stats.time_ms = total_solve_time.as_micros() as usize;
                    self.stats.costs = current_node.cost;

                    self.stats.print("HBCBS".to_string());
                    return Some(Solution {
                        paths: current_node.paths,
                    });
                }

                // Maintain the focal list
                let new_f_min = open
                    .iter()
                    .next()
                    .map_or(usize::MAX, |((open_cost, _), _)| *open_cost);
                if !open.is_empty() && f_min < new_f_min {
                    open.iter().for_each(|((open_cost, _), node)| {
                        if *open_cost as f64 > self.subopt_factor.0.unwrap() * f_min as f64
                            && *open_cost as f64 <= self.subopt_factor.0.unwrap() * new_f_min as f64
                        {
                            focal.push(node.clone());
                        }
                    });
                }
            }

            None
        } else {
            None
        }
    }
}
