use std::collections::HashSet;

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

#[derive(Debug, Clone)]
pub struct Path {
    pub steps: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub paths: Vec<Path>
}

impl Solution {
    pub fn verify(&self) -> bool {
        if self.paths.is_empty() {
            return true;  
        }

        let path_length = self.paths[0].steps.len(); 

        for time_step in 0..path_length {
            let mut seen_positions = HashSet::new();

            for path in &self.paths {
                let pos = &path.steps[time_step];
                if !seen_positions.insert(pos) {
                    return false; 
                }
            }
        }

        true 
    }
}