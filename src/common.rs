mod highlevel;
mod lowlevel;

pub(crate) use highlevel::{CardinalType, Constraint, HighLevelOpenNode};
pub(crate) use lowlevel::{
    create_focal_node, create_open_focal_node, create_open_node, FocalOrderWrapper,
    OpenOrderWrapper,
};

use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::fs::write;
use std::hash::Hash;
use tracing::error;

use crate::config::Config;
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
            if path.first().is_none_or(|&s| s != agent.start)
                || path.last().is_none_or(|&g| g != agent.goal)
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

                if time_step >= 1 && time_step < path.len() {
                    let prev_pos = path.get(time_step - 1).unwrap();
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

    pub fn log_solution(&self, config: &Config) {
        let agents = self.paths.len();
        let mut soc = 0;
        let mut makespan = 0;
        let mut starts = Vec::new();
        let mut goals = Vec::new();

        // Determine soc and makespan
        for path in &self.paths {
            soc += path.len();
            makespan = makespan.max(path.len());
        }

        // Pad agent paths with final position to match makespan
        let mut padded_paths = Vec::with_capacity(agents);
        for path in &self.paths {
            let mut padded = path.clone();
            if let Some(&last) = path.last() {
                while padded.len() < makespan {
                    padded.push(last);
                }
            }
            padded_paths.push(padded);

            // record starts & goals
            starts.push(format!(
                "({},{})",
                path.first().unwrap().1,
                path.first().unwrap().0
            ));
            goals.push(format!(
                "({},{})",
                path.last().unwrap().1,
                path.last().unwrap().0
            ));
        }

        let mut formatted = String::new();
        formatted.push_str(&format!("agents={}\n", agents));
        formatted.push_str(&format!("map_file={}\n", config.map_path));
        formatted.push_str(&format!("solver={}\n", config.solver));
        formatted.push_str("solved=1\n");
        formatted.push_str(&format!("soc={}\n", soc));
        formatted.push_str("soc_lb=\n");
        formatted.push_str(&format!("makespan={}\n", makespan));
        formatted.push_str("makespan_lb=\n");
        formatted.push_str("sum_of_loss=\n");
        formatted.push_str("sum_of_loss_lb=\n");
        formatted.push_str("comp_time=\n");
        formatted.push_str("seed=\n");
        formatted.push_str("checkpoints=-1\n");
        formatted.push_str("comp_time_initial_solution=\n");
        formatted.push_str("cost_initial_solution=\n");
        formatted.push_str("search_iteration=\n");
        formatted.push_str("num_high_level_node=\n");
        formatted.push_str("num_low_level_node=\n");

        formatted.push_str(&format!("starts={}\n", starts.join(",")));
        formatted.push_str(&format!("goals={}\n", goals.join(",")));
        formatted.push_str("solution=\n");

        // Print each timestep
        for t in 0..makespan {
            let timestep_line: Vec<String> = padded_paths
                .iter()
                .map(|path| {
                    let (x, y) = path[t];
                    format!("({},{})", y, x)
                })
                .collect();

            formatted.push_str(&format!("{}:{}\n", t, timestep_line.join(",")));
        }

        write(config.solution_path.clone(), formatted).unwrap();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MddNode {
    pub(crate) parents: HashSet<(usize, usize)>,
    pub(crate) children: HashSet<(usize, usize)>,
}

pub(crate) type Mdd = Vec<HashMap<(usize, usize), MddNode>>;

pub(crate) fn is_singleton_at_position(
    mdd: &Mdd,
    time_step: usize,
    position: (usize, usize),
) -> bool {
    if time_step >= mdd.len() {
        // Only vertex and target conflicts will inqury extended time step,
        // when we see an extended time step, then it must be singleton (cost will increase).
        return true;
    }
    let layer = &mdd[time_step];
    layer.len() == 1 && layer.contains_key(&position)
}

pub(crate) enum SearchResult {
    Standard(Option<(Path, usize)>),
    WithMDD(Option<(Path, usize, Mdd)>),
}
