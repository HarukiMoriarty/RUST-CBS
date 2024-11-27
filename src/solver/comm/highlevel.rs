use crate::common::Agent;
use crate::map::Map;
use crate::solver::algorithm::{a_star_search, focal_a_star_double_search, focal_a_star_search};
use crate::stat::Stats;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Conflict {
    pub(crate) agent_1: usize,
    pub(crate) agent_2: usize,
    pub(crate) position: (usize, usize),
    pub(crate) time_step: usize,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub(crate) struct Constraint {
    pub(crate) position: (usize, usize),
    pub(crate) time_step: usize,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct HighLevelOpenNode {
    pub(crate) agents: Vec<Agent>,
    pub(crate) constraints: Vec<HashSet<Constraint>>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) paths: Vec<Vec<(usize, usize)>>, // Maps agent IDs to their paths
    pub(crate) cost: usize, // Total cost for all paths under current constraints
    pub(crate) low_level_f_min_agents: Vec<usize>, // Agent's f_min, used for ECBS
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
        low_level_subopt_factor: Option<f64>,
        solver: &str,
        stats: &mut Stats,
    ) -> Option<Self> {
        let mut paths: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut low_level_f_min_agents = Vec::new();
        let mut total_cost = 0;
        let mut solve = true;

        for agent in agents {
            let path_f = match solver {
                "cbs" | "hbcbs" => a_star_search(map, agent, &HashSet::new(), stats),
                "lbcbs" | "bcbs" | "ecbs" | "decbs" => focal_a_star_search(
                    map,
                    agent,
                    low_level_subopt_factor.unwrap(),
                    &HashSet::new(),
                    &paths,
                    stats,
                ),
                _ => unreachable!(),
            };

            if let Some((path, low_level_f_min)) = path_f {
                total_cost += path.len();
                paths.insert(agent.id, path);
                low_level_f_min_agents.push(low_level_f_min);
            } else {
                solve = false;
            }
        }

        if solve {
            let mut start = HighLevelOpenNode {
                agents: agents.to_vec(),
                constraints: vec![HashSet::new(); agents.len()],
                conflicts: Vec::new(),
                paths,
                cost: total_cost,
                low_level_f_min_agents,
            };
            start.detect_conflicts();

            debug!("High level start node {start:?}");
            Some(start)
        } else {
            None
        }
    }

    pub(crate) fn detect_conflicts(&mut self) {
        let mut conflicts = Vec::new();

        // Compare paths of each pair of agents to find conflicts
        for i in 0..self.agents.len() {
            for j in (i + 1)..self.agents.len() {
                let path1 = &self.paths[i];
                let path2 = &self.paths[j];
                let max_length = path1.len().max(path2.len());

                for step in 0..max_length {
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

                    if pos1 == pos2 {
                        // Found a conflict at the same position and time step
                        conflicts.push(Conflict {
                            agent_1: i,
                            agent_2: j,
                            position: pos1,
                            time_step: step,
                        });
                    }
                }
            }
        }

        debug!("Detect conflicts: {conflicts:?}");
        self.conflicts = conflicts;
    }

    pub(crate) fn update_constraint(
        &self,
        conflict: &Conflict,
        resolve_first: bool,
        map: &Map,
        low_level_subopt_factor: Option<f64>,
        solver: &str,
        stats: &mut Stats,
    ) -> Option<HighLevelOpenNode> {
        let mut new_constraints = self.constraints.clone();
        let mut new_paths = self.paths.clone();
        let mut new_low_level_f_min_agents = self.low_level_f_min_agents.clone();

        let agent_to_update = if resolve_first {
            conflict.agent_1
        } else {
            conflict.agent_2
        };

        let constraints_for_agent = &mut new_constraints[agent_to_update];
        constraints_for_agent.insert(Constraint {
            position: conflict.position,
            time_step: conflict.time_step,
        });

        let new_path = match solver {
            "cbs" | "hbcbs" => a_star_search(
                map,
                &self.agents[agent_to_update],
                constraints_for_agent,
                stats,
            ),
            "lbcbs" | "bcbs" | "ecbs" => focal_a_star_search(
                map,
                &self.agents[agent_to_update],
                low_level_subopt_factor.unwrap(),
                constraints_for_agent,
                &self.paths,
                stats,
            ),
            "decbs" => focal_a_star_double_search(
                map,
                &self.agents[agent_to_update],
                low_level_subopt_factor.unwrap(),
                constraints_for_agent,
                &self.paths,
                stats,
            ),
            _ => unreachable!(),
        };

        if let Some((new_path, new_low_level_f_min)) = new_path {
            debug!(
                "Update agent {agent_to_update:?} with path {new_path:?} for conflict {conflict:?}"
            );
            let old_path_length = new_paths[agent_to_update].len();
            let new_path_length = new_path.len();
            new_paths[agent_to_update] = new_path;
            let new_cost = self.cost - old_path_length + new_path_length;
            new_low_level_f_min_agents[agent_to_update] = new_low_level_f_min;

            let mut new_node = HighLevelOpenNode {
                agents: self.agents.clone(),
                constraints: new_constraints,
                conflicts: Vec::new(),
                paths: new_paths,
                cost: new_cost,
                low_level_f_min_agents: new_low_level_f_min_agents,
            };
            new_node.detect_conflicts();

            Some(new_node)
        } else {
            None
        }
    }

    pub(crate) fn to_focal_node(&self) -> HighLevelFocalNode {
        HighLevelFocalNode {
            agents: self.agents.clone(),
            constraints: self.constraints.clone(),
            conflicts: self.conflicts.clone(),
            paths: self.paths.clone(),
            focal: self.conflicts.len(),
            cost: self.cost,
            low_level_f_min_agents: self.low_level_f_min_agents.clone(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct HighLevelFocalNode {
    pub(crate) agents: Vec<Agent>,
    pub(crate) constraints: Vec<HashSet<Constraint>>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) paths: Vec<Vec<(usize, usize)>>, // Maps agent IDs to their paths
    pub(crate) focal: usize, // Focal cost for all paths under current constraints
    pub(crate) cost: usize,  // Open cost for all paths under current constraints
    pub(crate) low_level_f_min_agents: Vec<usize>, // Agent's f_min, used for ECBS
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
            conflicts: self.conflicts.clone(),
            paths: self.paths.clone(),
            cost: self.cost,
            low_level_f_min_agents: self.low_level_f_min_agents.clone(),
        }
    }
}
