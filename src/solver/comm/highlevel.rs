use crate::common::Agent;
use crate::map::Map;
use crate::solver::algorithm::{a_star_search, focal_a_star_search};
use crate::stat::Stats;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl Hash for HighLevelOpenNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Agents should be always equal during the experiment.
        // Check paths.
        // If paths are equal, then cost and conflicts should be equal too.
        self.paths.hash(state);
        // Check constraints.
        for constraint in &self.constraints {
            let mut sorted_constraints: Vec<_> = constraint.iter().collect();
            sorted_constraints.sort_unstable();
            sorted_constraints.hash(state);
        }
    }
}

impl Ord for HighLevelOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
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
        stats: &mut Stats,
    ) -> Option<Self> {
        let mut paths: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut low_level_f_min_agents = Vec::new();
        let mut total_cost = 0;
        let mut solve = true;

        for agent in agents {
            let path_f = if let Some(factor) = low_level_subopt_factor {
                // If a suboptimal factor is provided, use the focal A* search
                focal_a_star_search(map, agent, factor, &HashSet::new(), &paths, stats)
            } else {
                // Otherwise, use the standard A* search
                a_star_search(map, agent, &HashSet::new(), stats)
            };

            if let Some((path, low_level_f_min)) = path_f {
                total_cost += path.len();
                paths.insert(agent.id, path);
                if let Some(low_level_f_min) = low_level_f_min {
                    low_level_f_min_agents.push(low_level_f_min);
                }
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

        let new_path = if let Some(factor) = low_level_subopt_factor {
            focal_a_star_search(
                map,
                &self.agents[agent_to_update],
                factor,
                constraints_for_agent,
                &self.paths,
                stats,
            )
        } else {
            a_star_search(
                map,
                &self.agents[agent_to_update],
                constraints_for_agent,
                stats,
            )
        };

        if let Some((new_path, new_low_level_f_min)) = new_path {
            debug!(
                "Update agent {agent_to_update:?} with path {new_path:?} for conflict {conflict:?}"
            );
            let old_path_length = new_paths[agent_to_update].len();
            let new_path_length = new_path.len();
            new_paths[agent_to_update] = new_path;
            let new_cost = self.cost - old_path_length + new_path_length;
            if let Some(new_low_level_f_min) = new_low_level_f_min {
                new_low_level_f_min_agents[agent_to_update] = new_low_level_f_min;
            }

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

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    // Ideal Path
    // [(25, 14), (24, 14), (23, 14), (22, 14), (21, 14),
    //  (20, 14), (19, 14), (18, 14), (17, 14), (17, 15),
    //  (17, 16), (17, 17), (16, 17), (15, 17), (14, 17),
    //  (14, 18), (14, 19), (15, 19), (16, 19), (17, 19)]
    #[test]
    fn test_high_level_hash() {
        let path_1 = [(25, 14)];
        let path_2 = [(25, 14), (24, 14)];

        let constraint_1 = Constraint {
            position: (25, 14),
            time_step: 0,
        };
        let constraint_2 = Constraint {
            position: (25, 14),
            time_step: 1,
        };
        let constraint_3 = Constraint {
            position: (25, 14),
            time_step: 3,
        };

        let mut node_1 = HighLevelOpenNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            conflicts: Vec::new(),
            paths: Vec::new(),
            cost: 0,
            low_level_f_min_agents: Vec::new(),
        };
        node_1.paths.insert(0, path_1.to_vec());
        node_1.constraints[0].insert(constraint_1.clone());
        node_1.constraints[0].insert(constraint_2.clone());

        let mut node_2 = HighLevelOpenNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            conflicts: Vec::new(),
            paths: Vec::new(),
            cost: 0,
            low_level_f_min_agents: Vec::new(),
        };
        node_2.paths.insert(0, path_1.to_vec());
        node_2.paths.insert(1, path_2.to_vec());
        node_2.constraints[0].insert(constraint_1.clone());
        node_2.constraints[0].insert(constraint_2);

        let mut node_3 = HighLevelOpenNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            conflicts: Vec::new(),
            paths: Vec::new(),
            cost: 0,
            low_level_f_min_agents: Vec::new(),
        };
        node_3.paths.insert(0, path_1.to_vec());
        node_3.paths.insert(1, path_2.to_vec());
        node_3.constraints[0].insert(constraint_1);
        node_3.constraints[0].insert(constraint_3);

        let mut closed = HashSet::new();
        closed.insert(node_1);

        assert!(!closed.contains(&node_2));
        closed.insert(node_2);

        assert!(!closed.contains(&node_3));
    }
}
