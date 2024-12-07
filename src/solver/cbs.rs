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
                    // 1: No Optimization.
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
                    // 2: Prioritize Conflicts Optimization.
                    else {
                        let mut find_cardinal = false;
                        let mut find_semi_cardinal = false;
                        let mut child_candidate_1 = None;
                        let mut child_candidate_2 = None;

                        for conflict in &current_node.conflicts {
                            // 0: non-cardinal, 1: semi-cardinal, 2: cardinal
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
                                // if solve failed, move on to next conflict.
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
                                // if solve failed, move on to next conflict.
                                continue;
                            }

                            match cardinal {
                                // Cardinal node: break the loop immediately.
                                2 => {
                                    find_cardinal = true;
                                    child_candidate_1 = child_1;
                                    child_candidate_2 = child_2;
                                    break;
                                }
                                // Semi Cardinal node: always record it since cardinal will directly break the loop.
                                1 => {
                                    find_semi_cardinal = true;
                                    child_candidate_1 = child_1;
                                    child_candidate_2 = child_2;
                                    continue;
                                }
                                // Non Cardinal node: record only if there is no cardinal and semi cardinal node.
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

                        debug!("expand: cardinal {find_cardinal:?}, semi-cardinal {find_semi_cardinal:?} CT node");
                        open.insert(child_candidate_1.unwrap());
                        open.insert(child_candidate_2.unwrap());
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
