mod highlevel;
mod lowlevel;

pub(crate) use highlevel::{CardinalType, Constraint, HighLevelOpenNode};
pub(crate) use lowlevel::{LowLevelFocalNode, LowLevelOpenNode};

use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::collections::HashSet;
use tracing::{debug, error};

use crate::map::Map;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

pub(crate) type Path = Vec<(usize, usize)>;

#[derive(Debug, Clone)]
pub struct Solution {
    pub paths: Vec<Path>,
}

impl Solution {
    pub fn verify(&self, map: &Map, agents: &[Agent]) -> bool {
        if self.paths.is_empty() {
            return true;
        }

        if self.paths.len() != agents.len() {
            error!("incomplete solution");
            return false;
        }

        for (path, agent) in self.paths.iter().zip(agents.iter()) {
            if path.first().map_or(true, |&s| s != agent.start)
                || path.last().map_or(true, |&g| g != agent.goal)
            {
                error!(
                    "start and goal failed: path start {:?} path end {:?}, but agent start {:?} agent end {:?}",
                    path.first(),
                    path.last(),
                    agent.start,
                    agent.goal
                );
                return false;
            }

            for window in path.windows(2) {
                if let [first, second] = window {
                    if !Self::are_neighbors(*first, *second) {
                        error!("move step failed");
                        return false;
                    }
                }
            }
        }

        let max_path_length = self.paths.iter().map(|p| p.len()).max().unwrap_or(0);

        for time_step in 0..max_path_length {
            let mut seen_positions = HashSet::new();
            let mut seen_edges = HashSet::new();

            for path in &self.paths {
                let pos = path.get(time_step).unwrap_or_else(|| path.last().unwrap());
                if !map.is_passable(pos.0, pos.1) {
                    error!("impossible move");
                    return false;
                }

                if !seen_positions.insert(pos) {
                    error!("vertex conlict at {pos:?}");
                    return false;
                }

                if time_step >= 1 {
                    let prev_pos = path
                        .get(time_step - 1)
                        .unwrap_or_else(|| path.last().unwrap());
                    if prev_pos != pos {
                        let edge = (prev_pos, pos);
                        let reverse_edge = (pos, prev_pos);

                        if !seen_edges.insert(edge) || seen_edges.contains(&reverse_edge) {
                            error!("edge conflict between {edge:?} and {reverse_edge:?}");
                            return false;
                        }
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

    pub fn log_solution(&self, solver: &str) {
        let mut formatted_solution = String::new();
        for (index, path) in self.paths.iter().enumerate() {
            formatted_solution.push_str(&format!(" agent{}:\n", index));
            for (t, &(x, y)) in path.iter().enumerate() {
                formatted_solution
                    .push_str(&format!("   - x: {}\n     y: {}\n     t: {}\n", x, y, t));
            }
        }
        debug!("{} solution:\n{}", solver, formatted_solution);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mdd {
    pub(crate) layers: Vec<HashSet<(usize, usize)>>,
}

impl Mdd {
    pub(crate) fn is_singleton_at_position(
        &self,
        time_step: usize,
        position: (usize, usize),
    ) -> bool {
        if time_step >= self.layers.len() {
            return false;
        }
        let layer = &self.layers[time_step];
        layer.len() == 1 && layer.contains(&position)
    }
}

pub(crate) enum SearchResult {
    Standard(Option<(Path, usize)>),
    WithMDD(Option<(Path, usize, Mdd)>),
}
