use crate::common::Agent;
use crate::map::Map;
use crate::solver::algorithm::{a_star_search, focal_a_star_search};
use crate::solver::Stats;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use tracing::debug;

#[derive(Debug)]
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
    pub(crate) paths: Vec<Vec<(usize, usize)>>, // Maps agent IDs to their paths
    pub(crate) cost: usize, // Total cost for all paths under current constraints
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
        subopt_factor: Option<f64>,
        stats: &mut Stats,
    ) -> Option<Self> {
        let mut paths: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut total_cost = 0;
        let mut solve = true;

        for agent in agents {
            let path = if let Some(factor) = subopt_factor {
                // If a suboptimal factor is provided, use the focal A* search
                focal_a_star_search(
                    map,
                    agent.start,
                    agent.goal,
                    factor,
                    &HashSet::new(),
                    agent.id,
                    &paths,
                    stats,
                )
            } else {
                // Otherwise, use the standard A* search
                a_star_search(map, agent.start, agent.goal, &HashSet::new(), stats)
            };

            if let Some(path) = path {
                total_cost += path.len();
                paths.insert(agent.id, path);
            } else {
                solve = false;
            }
        }

        if solve {
            let start = HighLevelNode {
                agents: agents.to_vec(),
                constraints: vec![HashSet::new(); agents.len()],
                paths,
                cost: total_cost,
            };
            debug!("High level start node {start:?}");
            Some(start)
        } else {
            None
        }
    }

    pub(crate) fn detect_conflicts(&self) -> Option<Vec<Conflict>> {
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

        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }

    pub(crate) fn update_constraint(
        &self,
        conflict: &Conflict,
        resolve_first: bool,
        map: &Map,
        subopt_factor: Option<f64>,
        stats: &mut Stats,
    ) -> Option<HighLevelNode> {
        let mut new_constraints = self.constraints.clone();
        let mut new_paths = self.paths.clone();

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

        let new_path = if let Some(factor) = subopt_factor {
            focal_a_star_search(
                map,
                self.agents[agent_to_update].start,
                self.agents[agent_to_update].goal,
                factor,
                constraints_for_agent,
                agent_to_update,
                &self.paths,
                stats,
            )
        } else {
            a_star_search(
                map,
                self.agents[agent_to_update].start,
                self.agents[agent_to_update].goal,
                constraints_for_agent,
                stats,
            )
        };

        if let Some(new_path) = new_path {
            debug!(
                "Update agent {agent_to_update:?} with path {new_path:?} for conflict {conflict:?}"
            );
            let old_path_length = new_paths[agent_to_update].len();
            let new_path_length = new_path.len();
            new_paths[agent_to_update] = new_path;
            let new_cost = self.cost - old_path_length + new_path_length;

            Some(HighLevelNode {
                agents: self.agents.clone(),
                constraints: new_constraints,
                paths: new_paths,
                cost: new_cost,
            })
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
            paths: Vec::new(),
            cost: 0,
        };
        node_1.paths.insert(0, path_1.to_vec());
        node_1.constraints[0].insert(constraint_1.clone());
        node_1.constraints[0].insert(constraint_2.clone());

        let mut node_2 = HighLevelNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            paths: Vec::new(),
            cost: 0,
        };
        node_2.paths.insert(0, path_1.to_vec());
        node_2.paths.insert(1, path_2.to_vec());
        node_2.constraints[0].insert(constraint_1.clone());
        node_2.constraints[0].insert(constraint_2);

        let mut node_3 = HighLevelNode {
            agents: Vec::new(),
            constraints: vec![HashSet::new(); 2],
            paths: Vec::new(),
            cost: 0,
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
