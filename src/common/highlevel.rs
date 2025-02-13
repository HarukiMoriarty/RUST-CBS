use super::{is_singleton_at_position, Agent, Mdd, Path, SearchResult};
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
        from_position: (usize, usize),
        to_position: (usize, usize),
        to_time_step: usize,
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
pub(crate) enum Constraint {
    Vertex {
        position: (usize, usize),
        time_step: usize,
        is_permanent: bool,
    },
    Edge {
        from_position: (usize, usize),
        to_position: (usize, usize),
        to_time_step: usize,
    },
}

impl Constraint {
    pub(crate) fn is_violated(
        &self,
        from_pos: (usize, usize),
        to_pos: (usize, usize),
        to_tmstep: usize,
    ) -> bool {
        match self {
            Constraint::Vertex {
                position,
                time_step,
                is_permanent,
            } => {
                if to_pos != *position {
                    return false;
                }
                if *is_permanent {
                    to_tmstep >= *time_step
                } else {
                    to_tmstep == *time_step
                }
            }
            Constraint::Edge {
                from_position,
                to_position,
                to_time_step,
            } => from_pos == *from_position && to_pos == *to_position && to_tmstep == *to_time_step,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct HighLevelOpenNode {
    pub(crate) node_id: u64,
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

pub(crate) fn convert_conflict_to_constraint(
    conflict: &Conflict,
    resolve_first: bool,
    target_reasoning: bool,
    agent_to_update: usize,
    new_constraints: &mut [HashSet<Constraint>],
    new_path_length_constraints: &mut [usize],
) {
    match conflict.conflict_type {
        ConflictType::Vertex {
            position,
            time_step,
        } => {
            new_constraints[agent_to_update].insert(Constraint::Vertex {
                position,
                time_step,
                is_permanent: false,
            });
        }
        ConflictType::Edge {
            from_position,
            to_position,
            to_time_step,
        } => {
            new_constraints[agent_to_update].insert(if resolve_first {
                Constraint::Edge {
                    from_position,
                    to_position,
                    to_time_step,
                }
            } else {
                Constraint::Edge {
                    from_position: to_position,
                    to_position: from_position,
                    to_time_step,
                }
            });
        }
        ConflictType::Target {
            position,
            time_step,
        } => {
            if target_reasoning && !resolve_first {
                new_constraints
                    .iter_mut()
                    .enumerate()
                    .filter(|&(agent, _)| agent != conflict.agent_1)
                    .for_each(|(_, constraints)| {
                        constraints.insert(Constraint::Vertex {
                            position,
                            time_step,
                            is_permanent: true,
                        });
                    });
            } else {
                new_constraints[agent_to_update].insert(Constraint::Vertex {
                    position,
                    time_step,
                    is_permanent: false,
                });

                if resolve_first {
                    new_path_length_constraints[agent_to_update] =
                        max(new_path_length_constraints[agent_to_update], time_step);
                }
            }
        }
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
            node_id: 0,
            agents: agents.to_vec(),
            constraints: vec![HashSet::new(); agents.len()],
            path_length_constraints: vec![0; agents.len()],
            conflicts: Vec::new(),
            paths,
            cost: total_cost,
            low_level_f_min_agents,
            mdds,
        };
        start.detect_conflicts(config.op_target_reasoning);
        Some(start)
    }

    pub(crate) fn detect_conflicts(&mut self, op_target_reasoning: bool) {
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
                                let singlenton1 = is_singleton_at_position(mdd1, step, pos1);
                                let singlenton2 = is_singleton_at_position(mdd2, step, pos2);
                                if singlenton1 && singlenton2 {
                                    CardinalType::Cardinal
                                } else if singlenton1 || singlenton2 {
                                    CardinalType::SemiCardinal
                                } else {
                                    CardinalType::NonCardinal
                                }
                            }
                            (Some(mdd), None) | (None, Some(mdd)) => {
                                let singlenton = is_singleton_at_position(mdd, step, pos1);
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
                                cardinal_type: if op_target_reasoning {
                                    cardinal_type
                                } else {
                                    CardinalType::Unknown
                                },
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
                                cardinal_type: if op_target_reasoning {
                                    cardinal_type
                                } else {
                                    CardinalType::Unknown
                                },
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
                                let agent1_singleton =
                                    is_singleton_at_position(mdd1, step - 1, prev_pos1)
                                        && is_singleton_at_position(mdd1, step, pos1);
                                let agent2_singleton =
                                    is_singleton_at_position(mdd2, step - 1, prev_pos2)
                                        && is_singleton_at_position(mdd2, step, pos2);

                                if agent1_singleton && agent2_singleton {
                                    CardinalType::Cardinal
                                } else if agent1_singleton || agent2_singleton {
                                    CardinalType::SemiCardinal
                                } else {
                                    CardinalType::NonCardinal
                                }
                            }
                            (Some(mdd), None) | (None, Some(mdd)) => {
                                let singlenton = is_singleton_at_position(mdd, step - 1, prev_pos1)
                                    && is_singleton_at_position(mdd, step, pos1);
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
                                from_position: prev_pos1,
                                to_position: pos1,
                                to_time_step: step,
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
        new_node_id: u64,
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

        convert_conflict_to_constraint(
            conflict,
            resolve_first,
            config.op_target_reasoning,
            agent_to_update,
            &mut new_constraints,
            &mut new_path_length_constraints,
        );

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
            node_id: new_node_id,
            agents: self.agents.clone(),
            constraints: new_constraints,
            path_length_constraints: new_path_length_constraints,
            conflicts: Vec::new(),
            paths: new_paths,
            cost: new_cost,
            low_level_f_min_agents: new_low_level_f_min_agents,
            mdds: new_mdds,
        };
        new_node.detect_conflicts(config.op_target_reasoning);

        Some(new_node)
    }

    pub(crate) fn update_bypass_node(
        &self,
        new_node: &HighLevelOpenNode,
        agent_id: usize,
    ) -> HighLevelOpenNode {
        let mut bypass_node = self.clone();
        // Update node id
        bypass_node.node_id = new_node.node_id;
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
            node_id: self.node_id,
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
    pub(crate) node_id: u64,
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
            node_id: self.node_id,
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

#[cfg(test)]
mod tests {
    use tracing_subscriber;

    use super::*;

    // Helper function to setup tracing
    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("trace")
            .try_init();
    }

    #[test]
    fn test_constraints_violation() {
        init_tracing();
        // Test non perminant vertex constraint
        let non_perminant_vertex_constraint = Constraint::Vertex {
            position: (0, 0),
            time_step: 1,
            is_permanent: false,
        };

        assert!(!non_perminant_vertex_constraint.is_violated((0, 0), (0, 1), 1));
        assert!(non_perminant_vertex_constraint.is_violated((0, 1), (0, 0), 1));
        assert!(!non_perminant_vertex_constraint.is_violated((0, 1), (0, 0), 2));

        // Test perminant vertex constraint
        let perminant_vertex_constraint = Constraint::Vertex {
            position: (0, 0),
            time_step: 5,
            is_permanent: true,
        };

        assert!(!perminant_vertex_constraint.is_violated((0, 0), (0, 1), 5));
        assert!(perminant_vertex_constraint.is_violated((0, 1), (0, 0), 5));
        assert!(perminant_vertex_constraint.is_violated((0, 1), (0, 0), 6));
        assert!(!perminant_vertex_constraint.is_violated((0, 0), (0, 1), 4));

        // Test edge constraint
        let edge_constraint = Constraint::Edge {
            from_position: (0, 0),
            to_position: (0, 1),
            to_time_step: 2,
        };

        assert!(!edge_constraint.is_violated((0, 0), (0, 1), 1));
        assert!(!edge_constraint.is_violated((1, 1), (0, 1), 2));
        assert!(edge_constraint.is_violated((0, 0), (0, 1), 2));
    }

    use crate::common::MddNode;
    use std::collections::HashMap;

    // Helper function for test mdd construction
    fn create_mdd_from_layers(layers: Vec<Vec<(usize, usize)>>) -> Mdd {
        let mut mdd = vec![HashMap::new(); layers.len()];
        for (layer, positions) in layers.iter().enumerate() {
            for &pos in positions {
                mdd[layer].insert(
                    pos,
                    MddNode {
                        parents: HashSet::new(),
                        children: HashSet::new(),
                    },
                );
            }
        }
        mdd
    }

    #[test]
    fn test_detect_conflicts_cardinal_vertex() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (2, 2),
                goal: (0, 1),
            },
            Agent {
                id: 1,
                start: (0, 0),
                goal: (0, 3),
            },
        ];

        let paths = vec![
            vec![(2, 2), (1, 2), (0, 2), (0, 1)],
            vec![(0, 0), (0, 1), (0, 2), (0, 3)],
        ];

        let mdd1 =
            create_mdd_from_layers(vec![vec![(2, 2)], vec![(1, 2)], vec![(0, 2)], vec![(0, 1)]]);

        let mdd2 =
            create_mdd_from_layers(vec![vec![(0, 0)], vec![(0, 1)], vec![(0, 2)], vec![(0, 3)]]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Vertex {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::Cardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_semi_cardinal_vertex() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (2, 2),
                goal: (0, 0),
            },
            Agent {
                id: 1,
                start: (0, 0),
                goal: (0, 3),
            },
        ];

        let paths = vec![
            vec![(2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
            vec![(0, 0), (0, 1), (0, 2), (0, 3)],
        ];

        let mdd1 = create_mdd_from_layers(vec![
            vec![(2, 2)],         // layer 0
            vec![(1, 2), (2, 1)], // layer 1
            vec![(0, 2), (2, 0)], // layer 2
            vec![(0, 1), (1, 0)], // layer 3
            vec![(0, 0)],         // layer 4
        ]);

        let mdd2 =
            create_mdd_from_layers(vec![vec![(0, 0)], vec![(0, 1)], vec![(0, 2)], vec![(0, 3)]]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Vertex {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::SemiCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_non_cardinal_vertex() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (2, 2),
                goal: (0, 0),
            },
            Agent {
                id: 1,
                start: (0, 4),
                goal: (2, 2),
            },
        ];

        let paths = vec![
            vec![(2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
            vec![(0, 4), (0, 3), (0, 2), (1, 2), (2, 2)],
        ];

        let mdd1 = create_mdd_from_layers(vec![
            vec![(2, 2)],         // layer 0
            vec![(1, 2), (2, 1)], // layer 1
            vec![(0, 2), (2, 0)], // layer 2
            vec![(0, 1), (1, 0)], // layer 3
            vec![(0, 0)],         // layer 4
        ]);

        let mdd2 = create_mdd_from_layers(vec![
            vec![(0, 4)],
            vec![(0, 3), (1, 4)],
            vec![(0, 2), (2, 4)],
            vec![(1, 2), (2, 3)],
            vec![(2, 2)],
        ]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Vertex {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::NonCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_vertex_non_mdd_semicardinal() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (2, 2),
                goal: (0, 0),
            },
            Agent {
                id: 1,
                start: (0, 0),
                goal: (0, 3),
            },
        ];

        let paths = vec![
            vec![(2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
            vec![(0, 0), (0, 1), (0, 2), (0, 3)],
        ];

        let mdd2 =
            create_mdd_from_layers(vec![vec![(0, 0)], vec![(0, 1)], vec![(0, 2)], vec![(0, 3)]]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![None, Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Vertex {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::SemiCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_vertex_non_mdd_noncardinal() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (2, 2),
                goal: (0, 0),
            },
            Agent {
                id: 1,
                start: (0, 0),
                goal: (0, 3),
            },
        ];

        let paths = vec![
            vec![(2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
            vec![(0, 0), (0, 1), (0, 2), (0, 3)],
        ];

        let mdd1 = create_mdd_from_layers(vec![
            vec![(2, 2)],         // layer 0
            vec![(1, 2), (2, 1)], // layer 1
            vec![(0, 2), (2, 0)], // layer 2
            vec![(0, 1), (1, 0)], // layer 3
            vec![(0, 0)],         // layer 4
        ]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), None],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Vertex {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::NonCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_vertex_unknowncardinal() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (2, 2),
                goal: (0, 0),
            },
            Agent {
                id: 1,
                start: (0, 0),
                goal: (0, 3),
            },
        ];

        let paths = vec![
            vec![(2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
            vec![(0, 0), (0, 1), (0, 2), (0, 3)],
        ];

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![None, None],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Vertex {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::Unknown
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_cardinal_edge() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 1),
                goal: (2, 2),
            },
            Agent {
                id: 1,
                start: (2, 2),
                goal: (0, 1),
            },
        ];

        let paths = vec![
            vec![(0, 1), (0, 2), (1, 2), (2, 2)],
            vec![(2, 2), (1, 2), (0, 2), (0, 1)],
        ];

        let mdd1 =
            create_mdd_from_layers(vec![vec![(0, 1)], vec![(0, 2)], vec![(1, 2)], vec![(2, 2)]]);

        let mdd2 =
            create_mdd_from_layers(vec![vec![(2, 2)], vec![(1, 2)], vec![(0, 2)], vec![(0, 1)]]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 6,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Edge {
                    from_position: (0, 2),
                    to_position: (1, 2),
                    to_time_step: 2
                },
                cardinal_type: CardinalType::Cardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_semicardinal_edge() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 2),
                goal: (2, 2),
            },
            Agent {
                id: 1,
                start: (2, 3),
                goal: (0, 0),
            },
        ];

        let paths = vec![
            vec![(0, 2), (1, 2), (2, 2)],
            vec![(2, 3), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
        ];

        let mdd1 = create_mdd_from_layers(vec![vec![(0, 2)], vec![(1, 2)], vec![(2, 2)]]);

        let mdd2 = create_mdd_from_layers(vec![
            vec![(2, 3)],
            vec![(2, 2)],
            vec![(1, 2), (2, 1)],
            vec![(0, 2), (2, 0)],
            vec![(0, 1), (1, 0)],
            vec![(0, 0)],
        ]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Edge {
                    from_position: (1, 2),
                    to_position: (2, 2),
                    to_time_step: 2
                },
                cardinal_type: CardinalType::SemiCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_noncardinal_edge() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 0),
                goal: (2, 3),
            },
            Agent {
                id: 1,
                start: (2, 3),
                goal: (0, 0),
            },
        ];

        let paths = vec![
            vec![(0, 0), (0, 1), (0, 2), (1, 2), (2, 2), (2, 3)],
            vec![(2, 3), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
        ];

        let mdd1 = create_mdd_from_layers(vec![
            vec![(0, 0)],
            vec![(1, 0), (0, 1)],
            vec![(2, 0), (0, 2)],
            vec![(2, 1), (1, 2)],
            vec![(2, 2)],
            vec![(2, 3)],
        ]);

        let mdd2 = create_mdd_from_layers(vec![
            vec![(2, 3)],
            vec![(2, 2)],
            vec![(1, 2), (2, 1)],
            vec![(0, 2), (2, 0)],
            vec![(0, 1), (1, 0)],
            vec![(0, 0)],
        ]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 10,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Edge {
                    from_position: (0, 2),
                    to_position: (1, 2),
                    to_time_step: 3
                },
                cardinal_type: CardinalType::NonCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_none_mdd_semicardinal_edge() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 2),
                goal: (2, 2),
            },
            Agent {
                id: 1,
                start: (2, 3),
                goal: (0, 0),
            },
        ];

        let paths = vec![
            vec![(0, 2), (1, 2), (2, 2)],
            vec![(2, 3), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
        ];

        let mdd1 = create_mdd_from_layers(vec![vec![(0, 2)], vec![(1, 2)], vec![(2, 2)]]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), None],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Edge {
                    from_position: (1, 2),
                    to_position: (2, 2),
                    to_time_step: 2
                },
                cardinal_type: CardinalType::SemiCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_none_mdd_noncardinal_edge() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 2),
                goal: (2, 2),
            },
            Agent {
                id: 1,
                start: (2, 3),
                goal: (0, 0),
            },
        ];

        let paths = vec![
            vec![(0, 2), (1, 2), (2, 2)],
            vec![(2, 3), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)],
        ];

        let mdd2 = create_mdd_from_layers(vec![
            vec![(2, 3)],
            vec![(2, 2)],
            vec![(1, 2), (2, 1)],
            vec![(0, 2), (2, 0)],
            vec![(0, 1), (1, 0)],
            vec![(0, 0)],
        ]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 7,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![None, Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Edge {
                    from_position: (1, 2),
                    to_position: (2, 2),
                    to_time_step: 2
                },
                cardinal_type: CardinalType::NonCardinal
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_none_mdd_unknown_edge() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 1),
                goal: (2, 2),
            },
            Agent {
                id: 1,
                start: (2, 2),
                goal: (0, 1),
            },
        ];

        let paths = vec![
            vec![(0, 1), (0, 2), (1, 2), (2, 2)],
            vec![(2, 2), (1, 2), (0, 2), (0, 1)],
        ];

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 6,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![None, None],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 0,
                agent_2: 1,
                conflict_type: ConflictType::Edge {
                    from_position: (0, 2),
                    to_position: (1, 2),
                    to_time_step: 2
                },
                cardinal_type: CardinalType::Unknown
            }]
        );
    }

    #[test]
    fn test_detect_conflicts_cardinal_target() {
        init_tracing();
        let agents = vec![
            Agent {
                id: 0,
                start: (0, 0),
                goal: (0, 4),
            },
            Agent {
                id: 1,
                start: (2, 2),
                goal: (0, 2),
            },
        ];

        let paths = vec![
            vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
            vec![(2, 2), (1, 2), (0, 2)],
        ];

        let mdd1 = create_mdd_from_layers(vec![
            vec![(0, 0)],
            vec![(0, 1)],
            vec![(0, 2)],
            vec![(0, 3)],
            vec![(0, 4)],
        ]);

        let mdd2 = create_mdd_from_layers(vec![vec![(2, 2)], vec![(1, 2)], vec![(0, 2)]]);

        let mut node = HighLevelOpenNode {
            node_id: 0,
            agents,
            constraints: Vec::new(),
            path_length_constraints: Vec::new(),
            conflicts: Vec::new(),
            paths,
            cost: 6,
            low_level_f_min_agents: Vec::new(),
            mdds: vec![Some(mdd1), Some(mdd2)],
        };

        node.detect_conflicts(true);

        assert_eq!(
            node.conflicts,
            vec![Conflict {
                agent_1: 1,
                agent_2: 0,
                conflict_type: ConflictType::Target {
                    position: (0, 2),
                    time_step: 2
                },
                cardinal_type: CardinalType::Cardinal
            }]
        );
    }

    #[test]
    fn test_convert_vertex_conflict_to_constraint() {
        init_tracing();
        let conflict = Conflict {
            agent_1: 0,
            agent_2: 1,
            cardinal_type: CardinalType::Cardinal,
            conflict_type: ConflictType::Vertex {
                position: (0, 0),
                time_step: 1,
            },
        };
        let mut constraints = vec![HashSet::new(), HashSet::new()];
        let mut path_length_constraints: Vec<usize> = vec![0, 0];

        convert_conflict_to_constraint(
            &conflict,
            true,
            false,
            0,
            &mut constraints,
            &mut path_length_constraints,
        );

        assert_eq!(constraints[0].len(), 1);
        assert!(constraints[0].contains(&Constraint::Vertex {
            position: (0, 0),
            time_step: 1,
            is_permanent: false,
        }));
        assert!(constraints[1].is_empty());

        // Assert path length constraints remain unchanged
        assert_eq!(path_length_constraints, vec![0, 0]);

        convert_conflict_to_constraint(
            &conflict,
            false,
            false,
            1,
            &mut constraints,
            &mut path_length_constraints,
        );

        assert_eq!(constraints[1].len(), 1);
        assert!(constraints[1].contains(&Constraint::Vertex {
            position: (0, 0),
            time_step: 1,
            is_permanent: false,
        }));
        // Assert path length constraints remain unchanged
        assert_eq!(path_length_constraints, vec![0, 0]);
    }

    #[test]
    fn test_convert_edge_conflict_to_constraint() {
        init_tracing();
        let conflict = Conflict {
            agent_1: 0,
            agent_2: 1,
            cardinal_type: CardinalType::Cardinal,
            conflict_type: ConflictType::Edge {
                from_position: (0, 0),
                to_position: (0, 1),
                to_time_step: 2,
            },
        };
        let mut constraints = vec![HashSet::new(), HashSet::new()];
        let mut path_length_constraints: Vec<usize> = vec![0, 0];

        convert_conflict_to_constraint(
            &conflict,
            true,
            false,
            0,
            &mut constraints,
            &mut path_length_constraints,
        );

        assert_eq!(constraints[0].len(), 1);
        assert!(constraints[0].contains(&Constraint::Edge {
            from_position: (0, 0),
            to_position: (0, 1),
            to_time_step: 2,
        }));
        assert!(constraints[1].is_empty());

        // Assert path length constraints remain unchanged
        assert_eq!(path_length_constraints, vec![0, 0]);

        convert_conflict_to_constraint(
            &conflict,
            false,
            false,
            1,
            &mut constraints,
            &mut path_length_constraints,
        );

        assert_eq!(constraints[1].len(), 1);
        assert!(constraints[1].contains(&Constraint::Edge {
            from_position: (0, 1),
            to_position: (0, 0),
            to_time_step: 2,
        }));
        // Assert path length constraints remain unchanged
        assert_eq!(path_length_constraints, vec![0, 0]);
    }

    #[test]
    fn test_convert_target_conflict_to_constraint() {
        init_tracing();
        let conflict = Conflict {
            agent_1: 0,
            agent_2: 1,
            cardinal_type: CardinalType::Cardinal,
            conflict_type: ConflictType::Target {
                position: (0, 0),
                time_step: 5,
            },
        };
        let mut constraints = vec![HashSet::new(), HashSet::new()];
        let mut path_length_constraints: Vec<usize> = vec![0, 0];

        convert_conflict_to_constraint(
            &conflict,
            true,
            true,
            0,
            &mut constraints,
            &mut path_length_constraints,
        );

        assert_eq!(constraints[0].len(), 1);
        assert!(constraints[0].contains(&Constraint::Vertex {
            position: (0, 0),
            time_step: 5,
            is_permanent: false,
        }));
        assert!(constraints[1].is_empty());

        // Assert path length constraints remain unchanged
        assert_eq!(path_length_constraints, vec![5, 0]);

        convert_conflict_to_constraint(
            &conflict,
            false,
            true,
            1,
            &mut constraints,
            &mut path_length_constraints,
        );

        assert_eq!(constraints[1].len(), 1);
        assert!(constraints[1].contains(&Constraint::Vertex {
            position: (0, 0),
            time_step: 5,
            is_permanent: true,
        }));
        // Assert path length constraints remain unchanged
        assert_eq!(path_length_constraints, vec![5, 0]);
    }
}
