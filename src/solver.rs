mod algorithm;
mod bcbs;
mod cbs;
mod comm;
mod hbcbs;
mod lbcbs;

pub use bcbs::BCBS;
pub use cbs::CBS;
pub use hbcbs::HBCBS;
pub use lbcbs::LBCBS;

use crate::common::Solution;

pub trait Solver {
    fn solve(&mut self) -> Option<Solution>;
}
