use std::cmp::Ordering;

#[derive(Clone, Eq, Debug, PartialEq)]
pub(crate) struct LowLevelNode {
    pub(crate) position: (usize, usize),
    pub(crate) f_cost: usize,
    pub(crate) g_cost: usize,
    pub(crate) h_open_cost: usize,
    pub(crate) h_focal_cost: Option<usize>,
    pub(crate) time: usize,
}

impl Ord for LowLevelNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_cost.cmp(&other.f_cost).reverse()
    }
}

impl PartialOrd for LowLevelNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
