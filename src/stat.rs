use tracing::info;

#[derive(Debug, Clone, Default)]
pub(super) struct Stats {
    pub(super) costs: usize,
    pub(super) time_ms: usize,
    pub(super) low_level_expand_nodes: usize,
    pub(super) high_level_expand_nodes: usize,
}

impl Stats {
    pub(crate) fn print(&self, solver_name: String) {
        info!(
            "{solver_name:?} Cost {:?} Time {:?}(us) High level expand nodes number: {:?} Low level expand nodes number {:?}",
            self.costs, self.time_ms, self.high_level_expand_nodes, self.low_level_expand_nodes
        );
    }
}
