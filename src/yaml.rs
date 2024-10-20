use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

use crate::common::Agent;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentYaml {
    pub name: String,
    #[serde(rename = "potentialGoals")]
    pub potential_goals: Vec<[usize; 2]>,
    pub start: [usize; 2],
}

impl AgentYaml {
    pub fn to_agent(&self, id:usize) -> Agent {
        let goal = (self.potential_goals[0][0], self.potential_goals[0][1]);

        Agent {
            id,
            start: (self.start[0], self.start[1]),
            goal
        }
    }
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

    pub fn to_agents(&self) -> Vec<Agent> {
        self.agent.iter().enumerate().map(|(index, agent_yaml)| {
            agent_yaml.to_agent(index)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_yaml() {
        let yaml = Yaml::from_yaml("map_file/test/test.yaml").unwrap();
        let agents = yaml.to_agents();

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, 0);
        assert_eq!(agents[0].start, (16, 16));
        assert_eq!(agents[0].goal, (2, 2));
    }
}
