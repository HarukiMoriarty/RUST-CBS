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