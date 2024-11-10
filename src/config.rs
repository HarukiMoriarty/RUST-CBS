use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(
    name = "Rust MAPF",
    about = "Kinds of MAPF algorithm implemented in Rust.",
    author = "Moriarty Yu",
    version = "1.0"
)]
pub struct Cli {
    #[arg(long, short, help = "Path to the YAML config file")]
    pub config: Option<String>,

    #[arg(
        long,
        help = "Seed for the random number generator",
        default_value_t = 0
    )]
    pub seed: usize,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub test_yaml_path: String,
    pub test_map_path: String,
    pub num_agents: usize,
    pub agents_dist: Vec<usize>,
    pub seed: usize,
    pub sub_optimal: (Option<f64>, Option<f64>),
    pub solver: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let config = Self {
            test_yaml_path: "map_file/maze-32-32-2-scen-even/maze-32-32-2-even-1.scen".to_string(),
            test_map_path: "map_file/maze-32-32-2-scen-even/maze-32-32-2.map".to_string(),
            num_agents: 10,
            agents_dist: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].to_vec(),
            seed: 0,
            sub_optimal: (None, None),
            solver: vec!["cbs".to_string()],
        };

        config
    }
}

impl Config {
    pub fn from_yaml_str(config_str: &str) -> anyhow::Result<Self> {
        let config: Self = serde_yaml::from_str(config_str)?;
        config.validate()
    }

    pub fn override_from_command_line(mut self, cli: &Cli) -> anyhow::Result<Self> {
        self.seed = cli.seed;
        self.validate()
    }

    pub fn validate(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}
