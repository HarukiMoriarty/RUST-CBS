use std::fs::OpenOptions;
use std::io::Write;
use tracing::error;

use crate::config::Config;

#[derive(Debug, Clone, Default)]
pub(super) struct Stats {
    pub(super) costs: usize,
    pub(super) time_ms: usize,
    pub(super) low_level_expand_nodes: usize,
    pub(super) high_level_expand_nodes: usize,
}

impl Stats {
    pub(crate) fn print(&self, config: &Config) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&config.output_path)
            .unwrap();

        let file_content = format!(
            "{},{},{},{:?},{},{},{:?},{:?},{},{},{},{}\n",
            config.map_path,
            config.yaml_path,
            config.num_agents,
            config.agents_dist,
            config.seed,
            config.solver,
            config.sub_optimal.0,
            config.sub_optimal.1,
            self.costs,
            self.time_ms,
            self.high_level_expand_nodes,
            self.low_level_expand_nodes
        );

        if let Err(e) = file.write_all(file_content.as_bytes()) {
            error!("Failed to write to file '{}': {}", config.output_path, e);
        }
    }
}
