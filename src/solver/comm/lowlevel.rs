use std::cmp::Ordering;

#[derive(Clone, Eq, Debug, PartialEq, Hash)]
pub(crate) struct LowLevelNode {
    pub(crate) position: (usize, usize),
    pub(crate) sort_key: usize,
    pub(crate) g_cost: usize,
    pub(crate) h_open_cost: usize,
    pub(crate) h_focal_cost: usize,
    pub(crate) time: usize,
}

impl Ord for LowLevelNode {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_f_score = self.g_cost + self.h_open_cost;
        let other_f_score = other.g_cost + other.h_open_cost;

        self.sort_key
            .cmp(&other.sort_key)
            .reverse()
            .then_with(|| self_f_score.cmp(&other_f_score).reverse())
            .then_with(|| self.g_cost.cmp(&other.g_cost))
            .then_with(|| self.h_open_cost.cmp(&other.h_open_cost).reverse())
    }
}

impl PartialOrd for LowLevelNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
