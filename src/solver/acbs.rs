use super::{sub_optimal_bypass_comparation, Solver};
use crate::common::{Agent, CardinalType, HighLevelOpenNode, Solution};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::collections::BTreeSet;
use std::time::Instant;
use tracing::debug;

pub struct ACBS {
    agents: Vec<Agent>,
    map: Map,
    stats: Stats,
}

impl ACBS {
    pub fn new(agents: Vec<Agent>, map: &Map) -> Self {
        ACBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
        }
    }
}

impl Solver for ACBS {
    fn solve(&mut self, config: &Config) -> Option<Solution> {
        let total_solve_start_time = Instant::now();
        let mut global_high_level_node_id = 0;
        let subopt_factor = config.sub_optimal.1.unwrap();

        let mut open = BTreeSet::new();
        let mut focal = BTreeSet::new();

        if let Some(root) =
            HighLevelOpenNode::new(&self.agents, &self.map, config, "acbs", &mut self.stats)
        {
            open.insert(root.clone());
            focal.insert(root.to_focal_node());

            while let Some(current_focal_node) = focal.pop_first() {
                debug!(
                    "Node Id: {:?}, conflicts: {:?}",
                    current_focal_node.node_id, current_focal_node.conflicts
                );
                let current_open_node = current_focal_node.to_open_node();
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
                        .or_else(|| current_open_node.conflicts.first())
                } else {
                    current_open_node.conflicts.first()
                };

                if let Some(conflict) = conflict {
                    debug!("conflict: {conflict:?}");
                    let mut bypass = false;

                    global_high_level_node_id += 1;
                    let child_1 = current_open_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        config,
                        global_high_level_node_id,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_1 {
                            if sub_optimal_bypass_comparation(
                                &current_open_node,
                                child,
                                conflict.agent_1,
                                subopt_factor,
                            ) {
                                debug!(
                                    "Bypass Node {:?} into Node {:?}",
                                    current_open_node.node_id, child.node_id
                                );
                                open.insert(
                                    current_open_node.update_bypass_node(child, conflict.agent_1),
                                );
                                focal.insert(child.to_focal_node());
                                self.stats.high_level_expand_nodes += 1;
                                bypass = true;
                            }
                        }
                    }

                    global_high_level_node_id += 1;
                    let child_2 = current_open_node.update_constraint(
                        conflict,
                        false,
                        &self.map,
                        config,
                        global_high_level_node_id,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts {
                        if let Some(ref child) = child_2 {
                            if sub_optimal_bypass_comparation(
                                &current_open_node,
                                child,
                                conflict.agent_2,
                                subopt_factor,
                            ) {
                                debug!(
                                    "Bypass Node {:?} into Node {:?}",
                                    current_open_node.node_id, child.node_id
                                );
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
                        debug!(
                            "Expand Node {:?} into Node {:?}",
                            current_open_node.node_id, child.node_id
                        );
                        if child.cost as f64 <= (old_f_min as f64 * subopt_factor) {
                            focal.insert(child.to_focal_node());
                        }
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }

                    if let Some(child) = child_2 {
                        debug!(
                            "Expand Node {:?} into Node {:?}",
                            current_open_node.node_id, child.node_id
                        );
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
