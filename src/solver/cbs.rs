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
                    let mut find_cardinal = false;
                    let mut find_semi_cardinal = false;
                    let mut find_adopting = false;

                    let mut child_left = None;
                    let mut child_right = None;
                    let mut adopting_node = None;

                    for conflict in &current_node.conflicts {
                        // (false, false): non-cardinal, (true, false) | (false, true): semi-cardinal, (true, true): cardinal
                        let mut cardinal = (true, true);

                        let child_candidate_left = current_node.update_constraint(
                            conflict,
                            true,
                            &self.map,
                            None,
                            "cbs",
                            &mut self.stats,
                        );
                        if let Some(ref child) = child_candidate_left {
                            self.stats.high_level_expand_nodes += 1;
                            if child.cost <= current_node.cost {
                                cardinal.0 = false;
                            }
                        } else {
                            // if solve failed, move on to next conflict.
                            continue;
                        }

                        let child_candidate_right = current_node.update_constraint(
                            conflict,
                            false,
                            &self.map,
                            None,
                            "cbs",
                            &mut self.stats,
                        );
                        if let Some(ref child) = child_candidate_right {
                            self.stats.high_level_expand_nodes += 1;
                            if child.cost <= current_node.cost {
                                cardinal.1 = false;
                            }
                        } else {
                            // if solve failed, move on to next conflict.
                            continue;
                        }

                        match cardinal {
                            // Cardinal node: break the loop immediately.
                            (true, true) => {
                                find_cardinal = true;
                                child_left = child_candidate_left;
                                child_right = child_candidate_right;
                                break;
                            }
                            // Semi Cardinal node: always record it since cardinal will directly break the loop.
                            (true, false) | (false, true) => {
                                find_semi_cardinal = true;

                                child_left = child_candidate_left;
                                child_right = child_candidate_right;

                                // If no optimization, we just pick the first solved conflict.
                                if !config.op_prioritize_conflicts && !config.op_bypass_conflicts {
                                    break;
                                }

                                if config.op_bypass_conflicts && !find_adopting {
                                    if cardinal.0
                                        && child_right.as_ref().unwrap().conflicts.len()
                                            < current_node.conflicts.len()
                                    {
                                        find_adopting = true;
                                        adopting_node = child_right.clone();
                                        if !config.op_prioritize_conflicts {
                                            break;
                                        }
                                    } else if cardinal.1
                                        && child_left.as_ref().unwrap().conflicts.len()
                                            < current_node.conflicts.len()
                                    {
                                        find_adopting = true;
                                        adopting_node = child_left.clone();
                                        if !config.op_prioritize_conflicts {
                                            break;
                                        }
                                    }
                                }
                            }
                            // Non Cardinal node: record only if there is no cardinal and semi cardinal node.
                            (false, false) => {
                                if find_semi_cardinal {
                                    continue;
                                } else {
                                    child_left = child_candidate_left;
                                    child_right = child_candidate_right;

                                    // If no optimization, we just pick the first solved conflict.
                                    if !config.op_prioritize_conflicts
                                        && !config.op_bypass_conflicts
                                    {
                                        break;
                                    }

                                    if config.op_bypass_conflicts && !find_adopting {
                                        if child_right.as_ref().unwrap().conflicts.len()
                                            < current_node.conflicts.len()
                                        {
                                            find_adopting = true;
                                            adopting_node = child_right.clone();
                                            if !config.op_prioritize_conflicts {
                                                break;
                                            }
                                        } else if child_left.as_ref().unwrap().conflicts.len()
                                            < current_node.conflicts.len()
                                        {
                                            find_adopting = true;
                                            adopting_node = child_left.clone();
                                            if !config.op_prioritize_conflicts {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    debug!("expand: cardinal {find_cardinal:?}, semi-cardinal {find_semi_cardinal:?} CT node");

                    if find_cardinal {
                        open.insert(child_left.unwrap());
                        open.insert(child_right.unwrap());
                    } else if config.op_bypass_conflicts && find_adopting {
                        open.insert(adopting_node.unwrap());
                    } else {
                        open.insert(child_left.unwrap());
                        open.insert(child_right.unwrap());
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
