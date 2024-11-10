use super::comm::HighLevelNode;
use super::Solver;
use crate::common::{Agent, Solution};
use crate::map::Map;
use crate::stat::Stats;
use std::time::Instant;

use std::collections::{BinaryHeap, HashSet};
pub struct LBCBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
    subopt_factor: (Option<f64>, Option<f64>),
}

impl LBCBS {
    pub fn new(agents: Vec<Agent>, map: &Map, subopt_factor: (Option<f64>, Option<f64>)) -> Self {
        LBCBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
            subopt_factor,
        }
    }
}

impl Solver for LBCBS {
    fn solve(&mut self) -> Option<Solution> {
        let total_solve_start_time = Instant::now();
        let mut open = BinaryHeap::new();
        let mut closed = HashSet::new();

        if let Some(root) = HighLevelNode::new(
            &self.agents,
            &self.map,
            self.subopt_factor.1,
            &mut self.stats,
        ) {
            open.push(root);
            while let Some(current_node) = open.pop() {
                closed.insert(current_node.clone());
                if !current_node.conflicts.is_empty() {
                    for conflict in &current_node.conflicts {
                        if let Some(child_1) = current_node.update_constraint(
                            &conflict,
                            true,
                            &self.map,
                            self.subopt_factor.1,
                            &mut self.stats,
                        ) {
                            if !closed.contains(&child_1) {
                                open.push(child_1);
                            }
                        }

                        if let Some(child_2) = current_node.update_constraint(
                            &conflict,
                            false,
                            &self.map,
                            self.subopt_factor.1,
                            &mut self.stats,
                        ) {
                            if !closed.contains(&child_2) {
                                open.push(child_2);
                            }
                        }

                        // Updates stats
                        self.stats.high_level_expand_nodes += 2;
                    }
                } else {
                    // No conflicts, return solution
                    let total_solve_time = total_solve_start_time.elapsed();
                    self.stats.time_ms = total_solve_time.as_micros() as usize;
                    self.stats.costs = current_node.cost;

                    self.stats.print("LBCBS".to_string());
                    return Some(Solution {
                        paths: current_node.paths,
                    });
                }
            }

            None
        } else {
            None
        }
    }
}
