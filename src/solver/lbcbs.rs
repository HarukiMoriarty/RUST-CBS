use super::Solver;
use crate::common::{Agent, HighLevelOpenNode, Solution};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::collections::BTreeSet;
use std::time::Instant;
use tracing::debug;

pub struct LBCBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
}

impl LBCBS {
    pub fn new(agents: Vec<Agent>, map: &Map) -> Self {
        LBCBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
        }
    }
}

impl Solver for LBCBS {
    fn solve(&mut self, config: &Config) -> Option<Solution> {
        let total_solve_start_time = Instant::now();
        let mut open = BTreeSet::new();

        if let Some(root) =
            HighLevelOpenNode::new(&self.agents, &self.map, config, "lbcbs", &mut self.stats)
        {
            open.insert(root);
            while let Some(current_node) = open.pop_first() {
                if let Some(conflict) = current_node.conflicts.first() {
                    debug!("conflict: {conflict:?}");

                    let child_1 = current_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        config,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_1 {
                            if child.cost == current_node.cost
                                && child.conflicts.len() < current_node.conflicts.len()
                            {
                                open.insert(
                                    current_node.update_bypass_node(child, conflict.agent_1),
                                );
                                self.stats.high_level_expand_nodes += 1;
                                continue;
                            }
                        }
                    }

                    let child_2 = current_node.update_constraint(
                        conflict,
                        false,
                        &self.map,
                        config,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_2 {
                            if child.cost == current_node.cost
                                && child.conflicts.len() < current_node.conflicts.len()
                            {
                                open.insert(
                                    current_node.update_bypass_node(child, conflict.agent_2),
                                );
                                self.stats.high_level_expand_nodes += 1;
                                continue;
                            }
                        }
                    }

                    if let Some(child) = child_1 {
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }

                    if let Some(child) = child_2 {
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }
                } else {
                    // No conflicts, return solution.
                    debug!("Find solution");
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
