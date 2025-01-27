use super::{sub_optimal_bypass_comparation, Solver};
use crate::common::{Agent, CardinalType, HighLevelOpenNode, Solution};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::collections::BTreeSet;
use std::time::Instant;
use tracing::debug;

pub struct DECBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
}

impl DECBS {
    pub fn new(agents: Vec<Agent>, map: &Map) -> Self {
        DECBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
        }
    }
}

impl Solver for DECBS {
    fn solve(&mut self, config: &Config) -> Option<Solution> {
        let total_solve_start_time = Instant::now();
        let subopt_factor = config.sub_optimal.1.unwrap();

        let mut open = BTreeSet::new();
        let mut focal = BTreeSet::new();

        if let Some(root) =
            HighLevelOpenNode::new(&self.agents, &self.map, config, "decbs", &mut self.stats)
        {
            open.insert(root.clone());
            focal.insert(root.to_focal_node());
            while let Some(current_focal_node) = focal.pop_first() {
                let current_open_node = current_focal_node.to_open_node();
                debug!("current node : {current_open_node:?}");
                let old_f_min: usize = open.first().unwrap().low_level_f_min_agents.iter().sum();

                open.remove(&current_open_node);

                let conflict = if config.op_prioritize_conflicts {
                    current_open_node
                        .conflicts
                        .iter()
                        .find(|c| c.cardinal_type == CardinalType::Cardinal)
                        .or_else(|| {
                            current_open_node
                                .conflicts
                                .iter()
                                .find(|c| c.cardinal_type == CardinalType::SemiCardinal)
                        })
                        .or_else(|| {
                            current_open_node
                                .conflicts
                                .iter()
                                .find(|c| c.cardinal_type == CardinalType::NonCardinal)
                        })
                        .or_else(|| {
                            current_open_node
                                .conflicts
                                .iter()
                                .find(|c| c.cardinal_type == CardinalType::Unknown)
                        })
                } else {
                    current_open_node.conflicts.first()
                };

                if let Some(conflict) = conflict {
                    debug!("conflict: {conflict:?}");
                    let mut bypass = false;

                    let child_1 = current_open_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        config,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_1 {
                            if sub_optimal_bypass_comparation(
                                &current_open_node,
                                child,
                                subopt_factor,
                            ) {
                                open.insert(
                                    current_open_node.update_bypass_node(child, conflict.agent_1),
                                );
                                focal.insert(child.to_focal_node());
                                self.stats.high_level_expand_nodes += 1;
                                bypass = true;
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
                            if sub_optimal_bypass_comparation(
                                &current_open_node,
                                child,
                                subopt_factor,
                            ) {
                                open.insert(
                                    current_open_node.update_bypass_node(child, conflict.agent_2),
                                );
                                focal.insert(child.to_focal_node());
                                self.stats.high_level_expand_nodes += 1;
                                bypass = true;
                            }
                        }
                    }

                    if bypass {
                        continue;
                    }

                    if let Some(child) = child_1 {
                        if child.cost as f64 <= (old_f_min as f64 * subopt_factor) {
                            focal.insert(child.to_focal_node());
                        }
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }

                    if let Some(child) = child_2 {
                        if child.cost as f64 <= (old_f_min as f64 * subopt_factor) {
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
                    self.stats.costs = current_open_node.cost;

                    self.stats.print(config);
                    return Some(Solution {
                        paths: current_open_node.paths,
                    });
                }

                // Maintain the focal list
                if !open.is_empty() {
                    let new_f_min = open.first().unwrap().low_level_f_min_agents.iter().sum();
                    if old_f_min < new_f_min {
                        open.iter().for_each(|node| {
                            if node.cost as f64 > subopt_factor * old_f_min as f64
                                && node.cost as f64 <= subopt_factor * new_f_min as f64
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
