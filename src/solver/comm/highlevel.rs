use crate::common::Agent;
use crate::map::Map;
use crate::solver::algorithm::{a_star_search, focal_a_star_search};
use crate::stat::Stats;

use std::cmp::Ordering;
use std::collections::{hash_map::DefaultHasher, HashSet};
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
pub(crate) struct HighLevelNode {
    pub(crate) agents: Vec<Agent>,
    pub(crate) constraints: Vec<HashSet<Constraint>>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) paths: Vec<Vec<(usize, usize)>>, // Maps agent IDs to their paths
    pub(crate) cost: usize, // Total cost for all paths under current constraints
    pub(crate) focal: Option<usize>, // Focal cost under current constraints
    pub(crate) conflicts_hash: Option<usize>, // Hash value of conflicts for Btree index
    pub(crate) low_level_f_min_agents: Vec<usize>, // Agent's f_min, used for ECBS
}

impl Hash for HighLevelNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.paths.hash(state);
        for constraint in &self.constraints {
            let mut sorted_constraints: Vec<_> = constraint.iter().collect();
            sorted_constraints.sort_unstable();
            sorted_constraints.hash(state);
        }
    }
}

impl Ord for HighLevelNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost).reverse()
    }
}

impl PartialOrd for HighLevelNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HighLevelNode {
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
                focal_a_star_search(map, &agent, factor, &HashSet::new(), &paths, stats)
            } else {
                // Otherwise, use the standard A* search
                a_star_search(map, &agent, &HashSet::new(), stats)
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
            let mut start = HighLevelNode {
                agents: agents.to_vec(),
                constraints: vec![HashSet::new(); agents.len()],
                conflicts: Vec::new(),
                paths,
                cost: total_cost,
                focal: None,
                conflicts_hash: None,
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
        if conflicts.is_empty() {
            self.focal = Some(0);
        } else {
            self.focal = Some(conflicts.len());
            self.conflicts = conflicts.clone();

            let mut hasher = DefaultHasher::new();
            conflicts.hash(&mut hasher);
            self.conflicts_hash = Some(hasher.finish() as usize);
        }
    }

    pub(crate) fn update_constraint(
        &self,
        conflict: &Conflict,
        resolve_first: bool,
        map: &Map,
        low_level_subopt_factor: Option<f64>,
        stats: &mut Stats,
    ) -> Option<HighLevelNode> {
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

            let mut new_node = HighLevelNode {
                agents: self.agents.clone(),
                constraints: new_constraints,
                conflicts: Vec::new(),
                paths: new_paths,
                cost: new_cost,
                focal: None,
                conflicts_hash: None,
                low_level_f_min_agents: new_low_level_f_min_agents,
            };
            new_node.detect_conflicts();
            Some(new_node)
        } else {
            None
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

        let mut node_1 = HighLevelNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            conflicts: Vec::new(),
            paths: Vec::new(),
            cost: 0,
            focal: None,
            conflicts_hash: None,
            low_level_f_min_agents: Vec::new(),
        };
        node_1.paths.insert(0, path_1.to_vec());
        node_1.constraints[0].insert(constraint_1.clone());
        node_1.constraints[0].insert(constraint_2.clone());

        let mut node_2 = HighLevelNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            conflicts: Vec::new(),
            paths: Vec::new(),
            cost: 0,
            focal: None,
            conflicts_hash: None,
            low_level_f_min_agents: Vec::new(),
        };
        node_2.paths.insert(0, path_1.to_vec());
        node_2.paths.insert(1, path_2.to_vec());
        node_2.constraints[0].insert(constraint_1.clone());
        node_2.constraints[0].insert(constraint_2);

        let mut node_3 = HighLevelNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            conflicts: Vec::new(),
            paths: Vec::new(),
            cost: 0,
            focal: None,
            conflicts_hash: None,
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
