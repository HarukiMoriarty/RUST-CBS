use crate::map::Map;
use crate::common::Agent;
use super::algorithm::a_star_search;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

// Low level Node
#[derive(Clone, Eq, PartialEq)]
pub(super) struct LowLevelNode {
    pub(super) position: (usize, usize),
    pub(super) f_cost: usize,
    pub(super) g_cost: usize,
}

impl Ord for LowLevelNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for LowLevelNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// High level node.
pub(super) struct Conflict {
    pub(super) agent_1: usize,
    pub(super) agent_2: usize,
    pub(super) position: (usize, usize),
    pub(super) time_step: usize,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(super) struct Constraint {
    pub(super) position: (usize, usize),
    pub(super) time_step: usize,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct HighLevelNode {
    pub(super) agents: Vec<Agent>,
    pub(super) constraints: HashMap<usize, HashSet<Constraint>>,
    pub(super) paths: HashMap<usize, Vec<(usize, usize)>>,  // Maps agent IDs to their paths
    pub(super) cost: usize,                                 // Total cost for all paths under current constraints
    pub(super) parent: Option<Box<HighLevelNode>>,          // Optional parent node for path reconstruction
}

impl Ord for HighLevelNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // Primary criterion: cost (inverted for min-heap behavior)
            .then_with(|| { // Second criterion: total number of constraints
                let self_constraints_count = self.constraints.values().map(|set| set.len()).sum::<usize>();
                let other_constraints_count = other.constraints.values().map(|set| set.len()).sum::<usize>();
                self_constraints_count.cmp(&other_constraints_count)
            })
    }
}

impl PartialOrd for HighLevelNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HighLevelNode {
    pub(super) fn new(agents: &Vec<Agent>, map: &Map) -> Self {
        let mut paths = HashMap::new();
        let mut total_cost = 0;
        for agent in agents {
            if let Some(path) = a_star_search(map, agent.start, agent.goal, &HashSet::new()) {
                total_cost += path.len();
                paths.insert(agent.id, path);
            }
        }
        HighLevelNode {
            agents: agents.to_vec(),
            constraints: HashMap::new(),
            paths,
            cost: total_cost,
            parent: None,
        }
    }

    pub(super) fn detect_conflicts(&self) -> Option<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        let agent_ids: Vec<usize> = self.paths.keys().cloned().collect();
    
        // Compare paths of each pair of agents to find conflicts
        for i in 0..agent_ids.len() {
            for j in (i + 1)..agent_ids.len() {
                let path1 = &self.paths[&agent_ids[i]];
                let path2 = &self.paths[&agent_ids[j]];
                let min_length = path1.len().min(path2.len());
    
                for step in 0..min_length {
                    if path1[step] == path2[step] {
                        // Found a conflict at the same position and time step
                        conflicts.push(Conflict {
                            agent_1: agent_ids[i],
                            agent_2: agent_ids[j],
                            position: path1[step],
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

    pub(super) fn update_constraint(&self, conflict: &Conflict, resolve_first: bool, map: &Map) -> HighLevelNode {
        let mut new_constraints = self.constraints.clone();
        let mut new_paths = self.paths.clone();
        let agent_to_update = if resolve_first {conflict.agent_1} else {conflict.agent_2};

        let constraints_for_agent = new_constraints.entry(agent_to_update).or_default();
        constraints_for_agent.insert(Constraint {
            position: conflict.position,
            time_step: conflict.time_step,
        });

        if let Some(new_path) = a_star_search(map, self.agents[agent_to_update].start, self.agents[agent_to_update].goal, constraints_for_agent) {
            let old_path_length = new_paths.get(&agent_to_update).map_or(0, |p| p.len());
            new_paths.insert(agent_to_update, new_path);

            let new_path_length = new_paths.get(&agent_to_update).unwrap().len();
            let new_cost = self.cost - old_path_length + new_path_length;

            HighLevelNode {
                agents: self.agents.clone(),
                constraints: new_constraints,
                paths: new_paths,
                cost: new_cost,
                parent: Some(Box::new(self.clone())),
            }
        } else {
            self.clone()
        }
    }
}