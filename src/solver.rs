mod bcbs;
mod cbs;
mod decbs;
mod ecbs;
mod hbcbs;
mod lbcbs;

pub use bcbs::BCBS;
pub use cbs::CBS;
pub use decbs::DECBS;
pub use ecbs::ECBS;
pub use hbcbs::HBCBS;
pub use lbcbs::LBCBS;

use crate::common::{HighLevelOpenNode, Solution};
use crate::config::Config;

pub trait Solver {
    fn solve(&mut self, config: &Config) -> Option<Solution>;
}

pub(crate) fn sub_optimal_bypass_comparation(
    parent: &HighLevelOpenNode,
    child: &HighLevelOpenNode,
    agent_to_update: usize,
    suboptimal_factor: f64,
) -> bool {
    let lb: usize = parent.low_level_f_min_agents.iter().sum();
    if child.conflicts.len() < parent.conflicts.len()
        && child.cost as f64 <= lb as f64 * suboptimal_factor
        && (child.paths[agent_to_update].len() - 1) as f64
            <= parent.low_level_f_min_agents[agent_to_update] as f64 * suboptimal_factor
    {
        return true;
    }
    false
}
