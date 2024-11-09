use tracing::info;

#[derive(Debug, Clone)]
pub(super) struct Stats {
    pub(super) costs: usize,
    pub(super) time_ms: usize,
    pub(super) low_level_expand_nodes: usize,
    pub(super) high_level_expand_nodes: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            costs: 0,
            time_ms: 0,
            low_level_expand_nodes: 0,
            high_level_expand_nodes: 0,
        }
    }
}

impl Stats {
    pub(crate) fn print(&self) {
        info!(
            "Cost {:?} Time(microseconds) {:?} High level expand nodes number: {:?} Low level expand nodes number {:?}",
            self.costs, self.time_ms, self.high_level_expand_nodes, self.low_level_expand_nodes
        );
    }
}
