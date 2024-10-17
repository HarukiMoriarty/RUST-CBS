use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentYaml {
    pub name: String,
    pub potentialGoals: Vec<[usize; 2]>,
    pub start: [usize; 2],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Yaml {
    pub agent: Vec<AgentYaml>,
    pub map: String,
}

impl Yaml {
    pub fn from_yaml(path: &str) -> Result<Self, serde_yaml::Error> {
        let file = File::open(path).unwrap_or_else(|err| {
            panic!("Failed to open file {:?}: {}", path, err);
        });
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader)
    }
}
