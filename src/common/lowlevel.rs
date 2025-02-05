use std::cmp::Ordering;

#[derive(Clone, Eq, Debug, PartialEq, Hash)]
pub(crate) struct LowLevelOpenNode {
    pub(crate) position: (usize, usize), // once we can determine a position, we can also determine the h_cost
    pub(crate) f_open_cost: usize,
    pub(crate) g_cost: usize,
    pub(crate) time_step: usize, // before reach constraint limit, time_step is exactly same as g_cost
}

impl Ord for LowLevelOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_open_cost
            .cmp(&other.f_open_cost)
            // higher g cost (time) has higher priority
            .then_with(|| self.g_cost.cmp(&other.g_cost).reverse())
            // Tricky thing: if g cost is the same, then time step must be same;
            // if time step is the same, g cost might be different.
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
