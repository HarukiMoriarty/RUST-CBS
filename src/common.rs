use std::cmp::{max, min};
use std::collections::HashSet;

use crate::map::Map;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Constraint {
    agent_id: usize,
    position: (usize, usize),
    time_step: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Agent {
    pub id: usize,
    pub start: (usize, usize),
    pub goal: (usize, usize),
}

impl Agent {
    pub fn verify(&self, map: &Map) -> bool {
        map.is_passable(self.start.0, self.start.1) && map.is_passable(self.goal.0, self.goal.1)
    }
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub paths: Vec<Vec<(usize, usize)>>,
}

impl Solution {
    pub fn verify(&self, map: &Map, agents: &Vec<Agent>) -> bool {
        if self.paths.is_empty() {
            return true;
        }

        if self.paths.len() != agents.len() {
            println!("incomplete solution");
            return false;
        }

        for (path, agent) in self.paths.iter().zip(agents.iter()) {
            if path.first().map_or(true, |&s| s != agent.start)
                || path.last().map_or(true, |&g| g != agent.goal)
            {
                print!(
                    "path start {:?} path end {:?} agent start {:?} agent end {:?}",
                    path.first(),
                    path.last(),
                    agent.start,
                    agent.goal
                );
                println!("start and goal failed");
                return false;
            }

            for window in path.windows(2) {
                if let [first, second] = window {
                    if !Self::are_neighbors(*first, *second) {
                        println!("move step failed");
                        return false;
                    }
                }
            }
        }

        let max_path_length = self.paths.iter().map(|p| p.len()).max().unwrap_or(0);

        for time_step in 0..max_path_length {
            let mut seen_positions = HashSet::new();

            for path in &self.paths {
                // Check if the current path has a position at this time step
                if let Some(pos) = path.get(time_step) {
                    // Check for passability and if the position is already taken
                    if !map.is_passable(pos.0, pos.1) || !seen_positions.insert(pos) {
                        println!("conflict or impossible move");
                        return false;
                    }
                }
            }
        }

        true
    }

    fn are_neighbors(pos1: (usize, usize), pos2: (usize, usize)) -> bool {
        (pos1.0 == pos2.0 && (max(pos1.1, pos2.1) - min(pos1.1, pos2.1)) == 1)
            || (pos1.1 == pos2.1 && (max(pos1.0, pos2.0) - min(pos1.0, pos2.0)) == 1)
            || (pos1.0 == pos2.0 && pos1.1 == pos2.1)
    }
}
