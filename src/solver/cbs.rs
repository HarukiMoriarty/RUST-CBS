use super::Solver;
use crate::common::{Agent, CardinalType, HighLevelOpenNode, Solution};
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
        let mut global_high_level_node_id = 0;
        let mut open = BTreeSet::new();

        if let Some(root) =
            HighLevelOpenNode::new(&self.agents, &self.map, config, "cbs", &mut self.stats)
        {
            open.insert(root);
            while let Some(current_node) = open.pop_first() {
                debug!(
                    "Node Id: {:?}, conflicts: {:?}",
                    current_node.node_id, current_node.conflicts
                );
                let conflict = if config.op_prioritize_conflicts {
                    current_node
                        .conflicts
                        .iter()
                        .find(|c| c.cardinal_type == CardinalType::Cardinal)
                        .or_else(|| {
                            current_node
                                .conflicts
                                .iter()
                                .find(|c| c.cardinal_type == CardinalType::SemiCardinal)
                        })
                        .or_else(|| {
                            current_node
                                .conflicts
                                .iter()
                                .find(|c| c.cardinal_type == CardinalType::NonCardinal)
                        })
                        .or_else(|| current_node.conflicts.first())
                } else {
                    current_node.conflicts.first()
                };

                if let Some(conflict) = conflict {
                    debug!("conflict: {conflict:?}");
                    let mut bypass = false;

                    global_high_level_node_id += 1;
                    let child_1 = current_node.update_constraint(
                        conflict,
                        true,
                        &self.map,
                        config,
                        global_high_level_node_id,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts
                        && conflict.cardinal_type != CardinalType::Cardinal
                    {
                        if let Some(ref child) = child_1 {
                            if child.cost == current_node.cost
                                && child.conflicts.len() < current_node.conflicts.len()
                            {
                                debug!(
                                    "Bypass Node {:?} into Node {:?}",
                                    current_node.node_id, child.node_id
                                );
                                open.insert(
                                    current_node.update_bypass_node(child, conflict.agent_1),
                                );
                                self.stats.high_level_expand_nodes += 1;
                                bypass = true;
                            }
                        }
                    }

                    global_high_level_node_id += 1;
                    let child_2 = current_node.update_constraint(
                        conflict,
                        false,
                        &self.map,
                        config,
                        global_high_level_node_id,
                        &mut self.stats,
                    );

                    if config.op_bypass_conflicts
                        && conflict.cardinal_type != CardinalType::Cardinal
                    {
                        if let Some(ref child) = child_2 {
                            if child.cost == current_node.cost
                                && child.conflicts.len() < current_node.conflicts.len()
                            {
                                debug!(
                                    "Bypass Node {:?} into Node {:?}",
                                    current_node.node_id, child.node_id
                                );
                                open.insert(
                                    current_node.update_bypass_node(child, conflict.agent_2),
                                );
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
                            current_node.node_id, child.node_id
                        );
                        open.insert(child);
                        self.stats.high_level_expand_nodes += 1;
                    }

                    if let Some(child) = child_2 {
                        debug!(
                            "Expand Node {:?} into Node {:?}",
                            current_node.node_id, child.node_id
                        );
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
