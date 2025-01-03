mod highlevel;
mod lowlevel;

pub(crate) use highlevel::{Constraint, HighLevelOpenNode};
pub(crate) use lowlevel::{LowLevelFocalNode, LowLevelOpenNode};

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Mdd {
    pub(crate) layers: Vec<HashSet<(usize, usize)>>,
}

impl Mdd {
    pub(crate) fn is_singleton_at_position(
        &self,
        time_step: usize,
        position: (usize, usize),
    ) -> bool {
        if time_step >= self.layers.len() {
            return false;
        }
        let layer = &self.layers[time_step];
        layer.len() == 1 && layer.contains(&position)
    }
}

pub(crate) enum SearchResult {
    Standard(Option<(Vec<(usize, usize)>, usize)>),
    WithMDD(Option<(Vec<(usize, usize)>, usize, Mdd)>),
}
