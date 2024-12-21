use std::fs::OpenOptions;
use std::io::Write;
use tracing::{debug, error};

use crate::config::Config;

#[derive(Debug, Clone, Default)]
pub(super) struct Stats {
    pub(super) costs: usize,
    pub(super) time_ms: usize,
    pub(super) low_level_expand_open_nodes: usize,
    pub(super) low_level_expand_focal_nodes: usize,
    pub(super) high_level_expand_nodes: usize,
}

impl Stats {
    pub(crate) fn print(&self, config: &Config) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&config.output_path)
            .unwrap();

        let file_content = format!(
            "{},{},{},{:?},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            config.map_path,
            config.yaml_path,
            config.num_agents,
            config.agents_dist,
            config.seed,
            config.solver,
            config.sub_optimal.0.unwrap_or(f64::NAN),
            config.sub_optimal.1.unwrap_or(f64::NAN),
            config.op_prioritize_conflicts,
            config.op_bypass_conflicts,
            config.op_target_reasoning,
            self.costs,
            self.time_ms,
            self.high_level_expand_nodes,
            self.low_level_expand_open_nodes,
            self.low_level_expand_focal_nodes,
            self.low_level_expand_focal_nodes + self.low_level_expand_open_nodes,
        );

        debug!(
            "{:?} Cost {:?} Time {:?}(microseconds) High level expand nodes number: {:?} Low level expand nodes number {:?}", config.solver,
            self.costs, self.time_ms, self.high_level_expand_nodes, self.low_level_expand_focal_nodes + self.low_level_expand_open_nodes
        );

        if let Err(e) = file.write_all(file_content.as_bytes()) {
            error!("Failed to write to file '{}': {}", config.output_path, e);
        }
    }
}
