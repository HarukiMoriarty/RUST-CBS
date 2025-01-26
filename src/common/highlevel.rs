use super::{Agent, Mdd, Path, SearchResult};
use crate::algorithm::{a_star_search, focal_a_star_search};
use crate::config::Config;
use crate::map::Map;
use crate::stat::Stats;

use std::cmp::{max, Ordering};
use std::collections::HashSet;
use std::hash::Hash;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ConflictType {
    Vertex {
        position: (usize, usize),
        time_step: usize,
    },
    Edge {
        u: (usize, usize),
        v: (usize, usize),
        time_step: usize,
    },
    Target {
        position: (usize, usize),
        time_step: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum CardinalType {
    Cardinal,
    SemiCardinal,
    NonCardinal,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Conflict {
    pub(crate) agent_1: usize,
    pub(crate) agent_2: usize,
    pub(crate) conflict_type: ConflictType, // Symmetry Reasoning
    pub(crate) cardinal_type: CardinalType, // Prioritize Conflicts
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub(crate) struct Constraint {
    pub(crate) position: (usize, usize),
    pub(crate) time_step: usize,
    pub(crate) is_permanent: bool,
}

impl Constraint {
    pub fn is_violated(&self, position: (usize, usize), time: usize) -> bool {
        if position != self.position {
            return false;
        }

        if self.is_permanent {
            time >= self.time_step
        } else {
            time == self.time_step
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct HighLevelOpenNode {
    pub(crate) agents: Vec<Agent>,
    pub(crate) constraints: Vec<HashSet<Constraint>>,
    pub(crate) path_length_constraints: Vec<usize>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) paths: Vec<Path>, // Maps agent IDs to their paths
    pub(crate) cost: usize,      // Total cost for all paths under current constraints
    pub(crate) low_level_f_min_agents: Vec<usize>, // Agent's f_min, used for ECBS
    pub(crate) mdds: Vec<Option<Mdd>>,
}

impl Ord for HighLevelOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost
            .cmp(&other.cost)
            .then_with(|| self.conflicts.cmp(&other.conflicts))
            // We still need to compare the actual paths, since it will indeed
            // influence the optimal solution
            .then_with(|| self.paths.cmp(&other.paths))
    }
}

impl PartialOrd for HighLevelOpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HighLevelOpenNode {
    pub(crate) fn new(
        agents: &Vec<Agent>,
        map: &Map,
        config: &Config,
        solver: &str,
        stats: &mut Stats,
    ) -> Option<Self> {
        let mut paths = Vec::new();
        let mut low_level_f_min_agents = Vec::new();
        let mut mdds = Vec::new();
        let mut total_cost = 0;

        for agent in agents {
            let (path, low_level_f_min, mdd) = match solver {
                "cbs" | "hbcbs" => match a_star_search(
                    map,
                    agent,
                    &HashSet::new(),
                    0,
                    config.op_prioritize_conflicts,
                    stats,
                ) {
                    SearchResult::Standard(Some((path, low_level_f_min))) => {
                        (path, low_level_f_min, None)
                    }
                    SearchResult::WithMDD(Some((path, low_level_f_min, mdd))) => {
                        (path, low_level_f_min, Some(mdd))
                    }
                    _ => return None,
                },
                "lbcbs" | "bcbs" | "ecbs" | "decbs" => match focal_a_star_search(
                    map,
                    agent,
                    Some(0),
                    config.sub_optimal.1.unwrap(),
                    &HashSet::new(),
                    0,
                    &paths,
                    config.op_prioritize_conflicts,
                    stats,
                ) {
                    SearchResult::Standard(Some((path, low_level_f_min))) => {
                        (path, low_level_f_min, None)
                    }
                    SearchResult::WithMDD(Some((path, low_level_f_min, mdd))) => {
                        (path, low_level_f_min, Some(mdd))
                    }
                    _ => return None,
                },
                _ => unreachable!(),
            };

            // Notice: path include start node.
            total_cost += path.len() - 1;
            paths.insert(agent.id, path);
            low_level_f_min_agents.push(low_level_f_min);
            mdds.push(mdd);
        }

        let mut start = HighLevelOpenNode {
            agents: agents.to_vec(),
            constraints: vec![HashSet::new(); agents.len()],
            path_length_constraints: vec![0; agents.len()],
            conflicts: Vec::new(),
            paths,
            cost: total_cost,
            low_level_f_min_agents,
            mdds,
        };
        start.detect_conflicts();
        Some(start)
    }

    pub(crate) fn detect_conflicts(&mut self) {
        let mut conflicts = Vec::new();

        // Compare paths of each pair of agents to find conflicts
        for i in 0..self.agents.len() {
            for j in (i + 1)..self.agents.len() {
                let path1 = &self.paths[i];
                let path2 = &self.paths[j];
                let max_length = path1.len().max(path2.len());

                let mdd1 = &self.mdds[i];
                let mdd2 = &self.mdds[j];

                // Start from 1 since:
                // 1. Initial positions (step 0) can't have vertex conflicts (agents start at different positions).
                // 2. Edge conflicts need previous step, so can only start from step 1.
                for step in 1..max_length {
                    let pos1 = if step < path1.len() {
                        path1[step]
                    } else {
                        *path1.last().unwrap()
                    };
                    let pos2 = if step < path2.len() {
                        path2[step]
                    } else {
                        *path2.last().unwrap()
                    };

                    // Check for Vertex Conflict
                    if pos1 == pos2 {
                        // Check for cardinal type
                        let cardinal_type = match (&mdd1, &mdd2) {
                            (Some(mdd1), Some(mdd2)) => {
                                let singlenton1 = mdd1.is_singleton_at_position(step, pos1);
                                let singlenton2 = mdd2.is_singleton_at_position(step, pos2);
                                if singlenton1 && singlenton2 {
                                    CardinalType::Cardinal
                                } else if singlenton1 || singlenton2 {
                                    CardinalType::SemiCardinal
                                } else {
                                    CardinalType::NonCardinal
                                }
                            }
                            (Some(mdd), None) | (None, Some(mdd)) => {
                                let singlenton = mdd.is_singleton_at_position(step, pos1);
                                if singlenton {
                                    CardinalType::SemiCardinal
                                } else {
                                    CardinalType::NonCardinal
                                }
                            }
                            _ => CardinalType::Unknown,
                        };

                        // Check for target conflicts first
                        if step >= path1.len() - 1 && pos1 == self.agents[i].goal {
                            // Agent i is at its target and agent j is interfering
                            conflicts.push(Conflict {
                                agent_1: i,
                                agent_2: j,
                                conflict_type: ConflictType::Target {
                                    position: pos1,
                                    time_step: step,
                                },
                                cardinal_type,
                            });
                        } else if step >= path2.len() - 1 && pos2 == self.agents[j].goal {
                            // Agent j is at its target and agent i is interfering
                            conflicts.push(Conflict {
                                agent_1: j,
                                agent_2: i,
                                conflict_type: ConflictType::Target {
                                    position: pos2,
                                    time_step: step,
                                },
                                cardinal_type,
                            });
                        } else {
                            // Regular vertex conflict
                            conflicts.push(Conflict {
                                agent_1: i,
                                agent_2: j,
                                conflict_type: ConflictType::Vertex {
                                    position: pos1,
                                    time_step: step,
                                },
                                cardinal_type,
                            });
                        }
                    }

                    // Check for Edge Conflict.
                    if step >= path1.len() || step >= path2.len() {
                        continue;
                    }

                    let prev_pos1 = path1[step - 1];
                    let prev_pos2 = path2[step - 1];

                    if prev_pos1 == pos2 && prev_pos2 == pos1 {
                        let cardinal_type = match (&mdd1, &mdd2) {
                            (Some(mdd1), Some(mdd2)) => {
                                // For edge conflicts, need singletons at both t-1 and t.
                                let agent1_singleton = mdd1
                                    .is_singleton_at_position(step - 1, prev_pos1)
                                    && mdd1.is_singleton_at_position(step, pos1);
                                let agent2_singleton = mdd2
                                    .is_singleton_at_position(step - 1, prev_pos2)
                                    && mdd2.is_singleton_at_position(step, pos2);

                                if agent1_singleton && agent2_singleton {
                                    CardinalType::Cardinal
                                } else if agent1_singleton || agent2_singleton {
                                    CardinalType::SemiCardinal
                                } else {
                                    CardinalType::NonCardinal
                                }
                            }
                            (Some(mdd), None) | (None, Some(mdd)) => {
                                let singlenton = mdd.is_singleton_at_position(step - 1, prev_pos1)
                                    && mdd.is_singleton_at_position(step, pos1);
                                if singlenton {
                                    CardinalType::SemiCardinal
                                } else {
                                    CardinalType::NonCardinal
                                }
                            }
                            _ => CardinalType::Unknown,
                        };

                        conflicts.push(Conflict {
                            agent_1: i,
                            agent_2: j,
                            conflict_type: ConflictType::Edge {
                                u: pos1,
                                v: prev_pos1,
                                time_step: step,
                            },
                            cardinal_type,
                        });
                    }
                }
            }
        }

        debug!("Detect conflicts: {:?}", conflicts);
        self.conflicts = conflicts;
    }

    pub(crate) fn update_constraint(
        &self,
        conflict: &Conflict,
        resolve_first: bool,
        map: &Map,
        config: &Config,
        stats: &mut Stats,
    ) -> Option<HighLevelOpenNode> {
        let mut new_constraints = self.constraints.clone();
        let mut new_paths = self.paths.clone();
        let mut new_low_level_f_min_agents = self.low_level_f_min_agents.clone();
        let mut new_path_length_constraints = self.path_length_constraints.clone();
        let mut new_mdds = self.mdds.clone();

        let agent_to_update = if resolve_first {
            conflict.agent_1
        } else {
            conflict.agent_2
        };

        match conflict.conflict_type {
            ConflictType::Vertex {
                position,
                time_step,
            } => {
                new_constraints[agent_to_update].insert(Constraint {
                    position,
                    time_step,
                    is_permanent: false,
                });
            }
            ConflictType::Edge { u, v, time_step } => {
                let position = if resolve_first { u } else { v };
                new_constraints[agent_to_update].insert(Constraint {
                    position,
                    time_step,
                    is_permanent: false,
                });
            }
            ConflictType::Target {
                position,
                time_step,
            } => {
                if config.op_target_reasoning && !resolve_first {
                    new_constraints
                        .iter_mut()
                        .enumerate()
                        .filter(|&(agent, _)| agent != agent_to_update)
                        .for_each(|(_, constraints)| {
                            constraints.insert(Constraint {
                                position,
                                time_step,
                                is_permanent: true,
                            });
                        });
                } else {
                    new_constraints[agent_to_update].insert(Constraint {
                        position,
                        time_step,
                        is_permanent: false,
                    });
                }

                // Update path constaints
                if resolve_first {
                    new_path_length_constraints[agent_to_update] =
                        max(new_path_length_constraints[agent_to_update], time_step);
                }
            }
        }

        let (new_path, new_low_level_f_min, new_mdd) = match config.solver.as_str() {
            "cbs" | "hbcbs" => match a_star_search(
                map,
                &self.agents[agent_to_update],
                &new_constraints[agent_to_update],
                new_path_length_constraints[agent_to_update],
                config.op_prioritize_conflicts,
                stats,
            ) {
                SearchResult::Standard(Some((new_path, new_low_level_f_min))) => {
                    (new_path, new_low_level_f_min, None)
                }
                SearchResult::WithMDD(Some((new_path, new_low_level_f_min, new_mdd))) => {
                    (new_path, new_low_level_f_min, Some(new_mdd))
                }
                _ => return None,
            },
            "lbcbs" | "bcbs" | "ecbs" => match focal_a_star_search(
                map,
                &self.agents[agent_to_update],
                Some(0),
                config.sub_optimal.1.unwrap(),
                &new_constraints[agent_to_update],
                new_path_length_constraints[agent_to_update],
                &self.paths,
                config.op_prioritize_conflicts,
                stats,
            ) {
                SearchResult::Standard(Some((new_path, new_low_level_f_min))) => {
                    (new_path, new_low_level_f_min, None)
                }
                SearchResult::WithMDD(Some((new_path, new_low_level_f_min, new_mdd))) => {
                    (new_path, new_low_level_f_min, Some(new_mdd))
                }
                _ => return None,
            },
            "decbs" => match focal_a_star_search(
                map,
                &self.agents[agent_to_update],
                None,
                config.sub_optimal.1.unwrap(),
                &new_constraints[agent_to_update],
                new_path_length_constraints[agent_to_update],
                &self.paths,
                config.op_prioritize_conflicts,
                stats,
            ) {
                SearchResult::Standard(Some((new_path, new_low_level_f_min))) => {
                    (new_path, new_low_level_f_min, None)
                }
                SearchResult::WithMDD(Some((new_path, new_low_level_f_min, new_mdd))) => {
                    (new_path, new_low_level_f_min, Some(new_mdd))
                }
                _ => return None,
            },
            _ => unreachable!(),
        };

        debug!(
                "Update agent {agent_to_update:?} with path {new_path:?} for conflict {conflict:?}, new f min {new_low_level_f_min:?}"
            );

        // Notice: actually path include start point, calculation here counterbalance each other.
        let new_cost = self.cost - new_paths[agent_to_update].len() + new_path.len();
        new_paths[agent_to_update] = new_path;
        new_low_level_f_min_agents[agent_to_update] = new_low_level_f_min;
        new_mdds[agent_to_update] = new_mdd;

        let mut new_node = HighLevelOpenNode {
            agents: self.agents.clone(),
            constraints: new_constraints,
            path_length_constraints: new_path_length_constraints,
            conflicts: Vec::new(),
            paths: new_paths,
            cost: new_cost,
            low_level_f_min_agents: new_low_level_f_min_agents,
            mdds: new_mdds,
        };
        new_node.detect_conflicts();

        Some(new_node)
    }

    pub(crate) fn update_bypass_node(
        &self,
        new_node: &HighLevelOpenNode,
        agent_id: usize,
    ) -> HighLevelOpenNode {
        let mut bypass_node = self.clone();
        bypass_node.paths[agent_id] = new_node.paths[agent_id].clone();
        bypass_node.conflicts = new_node.conflicts.clone();
        bypass_node.mdds[agent_id] = new_node.mdds[agent_id].clone();
        // Notice: for focal search, bypass node cost might not be equal.
        bypass_node.cost = new_node.cost;
        bypass_node.low_level_f_min_agents[agent_id] = new_node.low_level_f_min_agents[agent_id];
        bypass_node
    }

    pub(crate) fn to_focal_node(&self) -> HighLevelFocalNode {
        HighLevelFocalNode {
            agents: self.agents.clone(),
            constraints: self.constraints.clone(),
            path_length_constraints: self.path_length_constraints.clone(),
            conflicts: self.conflicts.clone(),
            paths: self.paths.clone(),
            focal: self.conflicts.len(),
            cost: self.cost,
            low_level_f_min_agents: self.low_level_f_min_agents.clone(),
            mdds: self.mdds.clone(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct HighLevelFocalNode {
    pub(crate) agents: Vec<Agent>,
    pub(crate) constraints: Vec<HashSet<Constraint>>,
    pub(crate) path_length_constraints: Vec<usize>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) paths: Vec<Path>, // Maps agent IDs to their paths
    pub(crate) focal: usize,     // Focal cost for all paths under current constraints
    pub(crate) cost: usize,      // Open cost for all paths under current constraints
    pub(crate) low_level_f_min_agents: Vec<usize>, // Agent's f_min, used for ECBS
    pub(crate) mdds: Vec<Option<Mdd>>,
}

impl Ord for HighLevelFocalNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.focal
            .cmp(&other.focal)
            .then_with(|| self.cost.cmp(&other.cost))
            .then_with(|| self.conflicts.cmp(&other.conflicts))
            .then_with(|| self.paths.cmp(&other.paths))
    }
}

impl PartialOrd for HighLevelFocalNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HighLevelFocalNode {
    pub(crate) fn to_open_node(&self) -> HighLevelOpenNode {
        HighLevelOpenNode {
            agents: self.agents.clone(),
            constraints: self.constraints.clone(),
            path_length_constraints: self.path_length_constraints.clone(),
            conflicts: self.conflicts.clone(),
            paths: self.paths.clone(),
            cost: self.cost,
            low_level_f_min_agents: self.low_level_f_min_agents.clone(),
            mdds: self.mdds.clone(),
        }
    }
}
