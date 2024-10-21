mod cbs;
mod common;
mod algorithm;

pub use cbs::CBS;

use crate::common::{Agent, Solution};
use crate::map::Map;

pub trait Solver {
    fn new(agents: Vec<Agent>, map: &Map) -> Self;
    fn solve(&self) -> Solution;
}
