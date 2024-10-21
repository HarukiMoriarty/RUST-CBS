use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "Rust MAPF", about = "Kinds of MAPF algorithm implemented in Rust.", author = "Moriarty Yu", version = "1.0")]
pub struct Cli {
    #[arg(long, short, help = "Path to the YAML config file")]
    pub config: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub test_yaml_path: String,
    pub test_map_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let config = Self {
            test_yaml_path: "map_file/test/test.yaml".to_string(),
            test_map_path: "map_file/test/test.map".to_string(),
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
        self.validate()
    }

    pub fn validate(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}