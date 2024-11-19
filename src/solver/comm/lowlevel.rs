use std::cmp::Ordering;

#[derive(Clone, Eq, Debug, PartialEq, Hash)]
pub(crate) struct LowLevelOpenNode {
    pub(crate) position: (usize, usize),
    pub(crate) f_open_cost: usize,
    pub(crate) g_cost: usize, // (time) we assume uniform cost
}

impl Ord for LowLevelOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_open_cost
            .cmp(&other.f_open_cost)
            .then_with(|| self.g_cost.cmp(&other.g_cost).reverse())
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for LowLevelOpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Eq, Debug, PartialEq, Hash)]
pub(crate) struct LowLevelFocalNode {
    pub(crate) position: (usize, usize),
    pub(crate) f_focal_cost: usize,
    pub(crate) f_open_cost: usize,
    pub(crate) g_cost: usize,
}

impl Ord for LowLevelFocalNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_focal_cost
            .cmp(&other.f_focal_cost)
            .then_with(|| self.f_open_cost.cmp(&other.f_open_cost))
            .then_with(|| self.g_cost.cmp(&other.g_cost).reverse())
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for LowLevelFocalNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
