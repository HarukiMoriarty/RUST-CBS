use super::comm::HighLevelNode;
use super::Solver;
use crate::common::{Agent, Solution};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::collections::{BinaryHeap, HashSet};
use std::time::Instant;
pub struct CBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
}

impl CBS {
    pub fn new(agents: Vec<Agent>, map: &Map) -> Self {
        CBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
        }
    }
}

impl Solver for CBS {
    fn solve(&mut self, config: &Config) -> Option<Solution> {
        let total_solve_start_time = Instant::now();
        let mut open = BinaryHeap::new();
        let mut closed = HashSet::new();

        if let Some(root) = HighLevelNode::new(&self.agents, &self.map, None, &mut self.stats) {
            open.push(root);
            while let Some(current_node) = open.pop() {
                closed.insert(current_node.clone());
                if let Some(conflict) = &current_node.conflicts.first() {
                    if let Some(child_1) = current_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        None,
                        &mut self.stats,
                    ) {
                        if !closed.contains(&child_1) {
                            open.push(child_1);
                            self.stats.high_level_expand_nodes += 1;
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
                            open.push(child_2);
                            self.stats.high_level_expand_nodes += 1;
                        }
                    }
                } else {
                    // No conflicts, return solution.
                    let total_solve_time = total_solve_start_time.elapsed();
                    self.stats.time_ms = total_solve_time.as_micros() as usize;
                    self.stats.costs = current_node.cost;

                    self.stats.print(config);
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
