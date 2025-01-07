use super::Solver;
use crate::common::{Agent, HighLevelOpenNode, Solution};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::collections::BTreeSet;
use std::time::Instant;
use tracing::debug;

pub struct HBCBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
}

impl HBCBS {
    pub fn new(agents: Vec<Agent>, map: &Map) -> Self {
        HBCBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
        }
    }
}

impl Solver for HBCBS {
    fn solve(&mut self, config: &Config) -> Option<Solution> {
        let total_solve_start_time = Instant::now();
        let high_level_subopt_factor = config.sub_optimal.0.unwrap();

        let mut open = BTreeSet::new();
        let mut focal = BTreeSet::new();

        if let Some(root) =
            HighLevelOpenNode::new(&self.agents, &self.map, config, "hbcbs", &mut self.stats)
        {
            open.insert(root.clone());
            focal.insert(root.to_focal_node());

            while let Some(current_focal_node) = focal.pop_first() {
                let current_open_node = current_focal_node.to_open_node();
                let old_f_min = open.first().unwrap().cost;

                open.remove(&current_open_node);

                if let Some(conflict) = current_open_node.conflicts.first() {
                    debug!("conflict: {conflict:?}");

                    let child_1 = current_open_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        config,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_1 {
                            if child.cost == current_open_node.cost
                                && child.conflicts.len() < current_open_node.conflicts.len()
                            {
                                open.insert(current_open_node.update_bypass_path(
                                    child.paths[conflict.agent_1].clone(),
                                    child.conflicts.clone(),
                                    conflict.agent_1,
                                ));
                                focal.insert(child.to_focal_node());
                                self.stats.high_level_expand_nodes += 1;
                                continue;
                            }
                        }
                    }

                    let child_2 = current_open_node.update_constraint(
                        conflict,
                        false,
                        &self.map,
                        config,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_2 {
                            if child.cost <= current_open_node.cost
                                && child.conflicts.len() < current_open_node.conflicts.len()
                            {
                                open.insert(current_open_node.update_bypass_path(
                                    child.paths[conflict.agent_2].clone(),
                                    child.conflicts.clone(),
                                    conflict.agent_2,
                                ));
                                focal.insert(child.to_focal_node());
                                self.stats.high_level_expand_nodes += 1;
                                continue;
                            }
                        }
                    }

                    if let Some(child) = child_1 {
                        if child.cost as f64 <= (old_f_min as f64 * high_level_subopt_factor) {
                            focal.insert(child.to_focal_node());
                        }
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }

                    if let Some(child) = child_2 {
                        if child.cost as f64 <= (old_f_min as f64 * high_level_subopt_factor) {
                            focal.insert(child.to_focal_node());
                        }
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }
                } else {
                    // No conflicts, return solution
                    debug!("Find solution");
                    let total_solve_time = total_solve_start_time.elapsed();
                    self.stats.time_ms = total_solve_time.as_micros() as usize;
                    self.stats.costs = current_focal_node.cost;

                    self.stats.print(config);
                    return Some(Solution {
                        paths: current_focal_node.paths,
                    });
                }

                // Maintain the focal list
                if !open.is_empty() {
                    let new_f_min = open.first().unwrap().cost;
                    if old_f_min < new_f_min {
                        open.iter().for_each(|node| {
                            if node.cost as f64 > high_level_subopt_factor * old_f_min as f64
                                && node.cost as f64 <= high_level_subopt_factor * new_f_min as f64
                            {
                                focal.insert(node.to_focal_node());
                            }
                        });
                    }
                }
            }

            None
        } else {
            None
        }
    }
}
