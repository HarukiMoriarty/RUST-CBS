use super::comm::HighLevelOpenNode;
use super::Solver;
use crate::common::{Agent, Solution};
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
    low_level_subopt_factor: Option<f64>, // The lattar one should be always none for HBCBS
}

impl DECBS {
    pub fn new(agents: Vec<Agent>, map: &Map, subopt_factor: (Option<f64>, Option<f64>)) -> Self {
        DECBS {
            agents,
            map: map.clone(),
            stats: Stats::default(),
            low_level_subopt_factor: subopt_factor.1,
        }
    }
}

impl Solver for DECBS {
    fn solve(&mut self, config: &Config) -> Option<Solution> {
        let total_solve_start_time = Instant::now();

        let mut open = BTreeSet::new();
        let mut focal = BTreeSet::new();

        if let Some(root) = HighLevelOpenNode::new(
            &self.agents,
            &self.map,
            self.low_level_subopt_factor,
            "decbs",
            &mut self.stats,
        ) {
            open.insert(root.clone());
            focal.insert(root.to_focal_node());
            while let Some(current_focal_node) = focal.pop_first() {
                let current_open_node = current_focal_node.to_open_node();
                let old_f_min: usize = open.first().unwrap().low_level_f_min_agents.iter().sum();

                open.remove(&current_open_node);
                if let Some(conflict) = current_open_node.conflicts.first() {
                    debug!("conflict: {conflict:?}");

                    if let Some(child_1) = current_open_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        self.low_level_subopt_factor,
                        "decbs",
                        config.op_target_reasoning,
                        &mut self.stats,
                    ) {
                        open.insert(child_1.clone());
                        self.stats.high_level_expand_nodes += 1;

                        if child_1.cost as f64
                            <= (old_f_min as f64 * self.low_level_subopt_factor.unwrap())
                        {
                            focal.insert(child_1.to_focal_node());
                        }
                    }

                    if let Some(child_2) = current_open_node.update_constraint(
                        conflict,
                        false,
                        &self.map,
                        self.low_level_subopt_factor,
                        "decbs",
                        config.op_target_reasoning,
                        &mut self.stats,
                    ) {
                        open.insert(child_2.clone());
                        self.stats.high_level_expand_nodes += 1;

                        if child_2.cost as f64
                            <= (old_f_min as f64 * self.low_level_subopt_factor.unwrap())
                        {
                            focal.insert(child_2.to_focal_node());
                        }
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
                            if node.cost as f64
                                > self.low_level_subopt_factor.unwrap() * old_f_min as f64
                                && node.cost as f64
                                    <= self.low_level_subopt_factor.unwrap() * new_f_min as f64
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
