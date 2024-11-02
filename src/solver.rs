mod algorithm;
mod bcbs;
mod cbs;
mod comm;

pub use bcbs::BCBS;
pub use cbs::CBS;

use crate::common::{Agent, Solution};
use crate::map::Map;

pub trait Solver {
    fn new(agents: Vec<Agent>, map: &Map, subopt_factor: Option<f64>) -> Self;
    fn solve(&mut self) -> Option<Solution>;
}

#[derive(Debug, Clone)]
pub(super) struct Stats {
    pub(super) costs: usize,
    pub(super) low_level_expand_nodes: usize,
    pub(super) high_level_expand_nodes: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            costs: 0,
            low_level_expand_nodes: 0,
            high_level_expand_nodes: 0,
        }
    }
}
