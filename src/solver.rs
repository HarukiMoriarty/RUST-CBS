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
