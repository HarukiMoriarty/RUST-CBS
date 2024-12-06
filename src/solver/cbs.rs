use super::comm::HighLevelOpenNode;
use super::Solver;
use crate::common::{Agent, Solution};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::collections::BTreeSet;
use std::time::Instant;
use tracing::debug;

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
        let mut open = BTreeSet::new();

        if let Some(root) =
            HighLevelOpenNode::new(&self.agents, &self.map, None, "cbs", &mut self.stats)
        {
            open.insert(root);
            while let Some(current_node) = open.pop_first() {
                if !current_node.conflicts.is_empty() {
                    // 1. No optimization
                    if !config.op_prioritize_conflicts {
                        let conflict = current_node.conflicts.first().unwrap();
                        debug!("conflict: {conflict:?}");

                        if let Some(child_1) = current_node.update_constraint(
                            conflict,
                            true,
                            &self.map,
                            None,
                            "cbs",
                            &mut self.stats,
                        ) {
                            open.insert(child_1);
                            self.stats.high_level_expand_nodes += 1;
                        }

                        if let Some(child_2) = current_node.update_constraint(
                            conflict,
                            false,
                            &self.map,
                            None,
                            "cbs",
                            &mut self.stats,
                        ) {
                            open.insert(child_2);
                            self.stats.high_level_expand_nodes += 1;
                        }
                    }
                    // 2. Prioritize Conflicts Optimization
                    else {
                        let mut find_cardinal = false;
                        let mut find_semi_cardinal = false;
                        let mut child_candidate_1 = None;
                        let mut child_candidate_2 = None;

                        for conflict in &current_node.conflicts {
                            let mut cardinal = 2;

                            let child_1 = current_node.update_constraint(
                                conflict,
                                true,
                                &self.map,
                                None,
                                "cbs",
                                &mut self.stats,
                            );
                            if let Some(ref child_1) = child_1 {
                                self.stats.high_level_expand_nodes += 1;
                                if child_1.cost <= current_node.cost {
                                    cardinal -= 1;
                                }
                            } else {
                                continue;
                            }

                            let child_2 = current_node.update_constraint(
                                conflict,
                                false,
                                &self.map,
                                None,
                                "cbs",
                                &mut self.stats,
                            );
                            if let Some(ref child_2) = child_2 {
                                self.stats.high_level_expand_nodes += 1;
                                if child_2.cost <= current_node.cost {
                                    cardinal -= 1;
                                }
                            } else {
                                continue;
                            }

                            match cardinal {
                                // Cardinal node, directly expanded
                                2 => {
                                    debug!("find a cardinal CT node");
                                    find_cardinal = true;
                                    open.insert(child_1.unwrap());
                                    open.insert(child_2.unwrap());
                                    break;
                                }
                                // Semi Cardinal node, only if there is no cardinal node
                                1 => {
                                    find_semi_cardinal = true;
                                    child_candidate_1 = child_1;
                                    child_candidate_2 = child_2;
                                    continue;
                                }
                                // Non Cardinal node, only if there is no cardinal and semi cardinal node.
                                0 => {
                                    if find_semi_cardinal {
                                        continue;
                                    } else {
                                        child_candidate_1 = child_1;
                                        child_candidate_2 = child_2;
                                        continue;
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }

                        // If there is no cardinal CT node, pick a semi cardinal node.
                        if !find_cardinal && find_semi_cardinal {
                            debug!("find a semi cardinal CT node");
                            open.insert(child_candidate_1.unwrap());
                            open.insert(child_candidate_2.unwrap());
                        }
                        // If there is neither have cardinal CT node nor semi cardinal CT node, random pick non cardinal CT node.
                        else if !find_cardinal && !find_semi_cardinal {
                            debug!("find a non cardinal CT node");
                            open.insert(child_candidate_1.unwrap());
                            open.insert(child_candidate_2.unwrap());
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
