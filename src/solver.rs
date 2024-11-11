mod algorithm;
mod bcbs;
mod cbs;
mod comm;
mod ecbs;
mod hbcbs;
mod lbcbs;

pub use bcbs::BCBS;
pub use cbs::CBS;
pub use ecbs::ECBS;
pub use hbcbs::HBCBS;
pub use lbcbs::LBCBS;

use crate::common::Solution;

pub trait Solver {
    fn solve(&mut self) -> Option<Solution>;
}
