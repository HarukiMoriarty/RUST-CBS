use std::collections::HashSet;

use crate::map::Map;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Constraint {
    agent_id: usize,
    position: (usize, usize),
    time_step: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
pub struct Path {
    pub steps: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub paths: Vec<Path>
}

impl Solution {
    pub fn verify(&self, map: &Map) -> bool {
        if self.paths.is_empty() {
            return true;  
        }

        let max_path_length = self.paths.iter()
            .map(|p| p.steps.len())
            .max()
            .unwrap_or(0);

        for time_step in 0..max_path_length {
            let mut seen_positions = HashSet::new();

            for path in &self.paths {
                // Check if the current path has a position at this time step
                if let Some(pos) = path.steps.get(time_step) {
                    // Check for passability and if the position is already taken
                    if !map.is_passable(pos.0, pos.1) || !seen_positions.insert(pos) {
                        return false;  
                    }
                }
            }
        }

        true 
    }
}