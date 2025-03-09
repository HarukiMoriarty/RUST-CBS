use anyhow::Result;
use rand::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use tracing::info;

use crate::common::Agent;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Route {
    pub start_x: usize,
    pub start_y: usize,
    pub goal_x: usize,
    pub goal_y: usize,
}

type Bucket = Vec<Route>;

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub map: String,
    pub map_width: usize,
    pub map_height: usize,
    pub buckets: Option<HashMap<usize, Bucket>>,
}

impl Scenario {
    pub fn load_from_scen(path: &str) -> io::Result<Scenario> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines().map(|line| line.unwrap());

        // First line is "version x.x" which we can skip
        let _version = lines.next().unwrap();

        // Initialize the scenario with an empty HashMap
        let mut scenario: Scenario = Scenario {
            map: String::new(),
            map_width: 0,
            map_height: 0,
            buckets: Some(HashMap::new()),
        };

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let bucket_index: usize = parts[0].parse().unwrap();

            let route = Route {
                start_x: parts[5].parse().unwrap(),
                start_y: parts[4].parse().unwrap(),
                goal_x: parts[7].parse().unwrap(),
                goal_y: parts[6].parse().unwrap(),
            };

            if scenario.map.is_empty() {
                // Initialize map details from the first route entry
                scenario.map = parts[1].to_string();
                scenario.map_width = parts[2].parse().unwrap();
                scenario.map_height = parts[3].parse().unwrap();
            }

            // Access the bucket, or initialize it if it does not exist
            scenario
                .buckets
                .as_mut()
                .unwrap()
                .entry(bucket_index)
                .or_default()
                .push(route);
        }

        Ok(scenario)
    }

    pub fn generate_agents_by_buckets<R: Rng + ?Sized>(
        &self,
        num_agents: usize,
        agent_buckets: Vec<usize>,
        rng: &mut R,
    ) -> Result<Vec<Agent>, String> {
        if agent_buckets.len() != num_agents {
            return Err("Number of agents does not match the length of agent_buckets".to_string());
        }

        let mut agents: Vec<Agent> = Vec::new();
        let mut used_routes: HashMap<usize, HashSet<usize>> = HashMap::new();

        for (agent_id, &bucket_index) in agent_buckets.iter().enumerate() {
            let bucket = self
                .buckets
                .as_ref()
                .unwrap()
                .get(&bucket_index)
                .ok_or_else(|| format!("Bucket {} not found", bucket_index))?;

            // Find unused routes
            let available_routes: Vec<usize> = (0..bucket.len())
                .filter(|idx| {
                    used_routes
                        .get(&bucket_index)
                        .is_none_or(|used| !used.contains(idx))
                })
                .collect();

            if available_routes.is_empty() {
                return Err(format!(
                    "No available routes left in bucket {}",
                    bucket_index
                ));
            }

            // Select a random route from available ones
            let route_index = available_routes
                .choose(rng)
                .ok_or_else(|| "Failed to choose a random route".to_string())?;

            let route = &bucket[*route_index];
            agents.push(Agent {
                id: agent_id,
                start: (route.start_x, route.start_y),
                goal: (route.goal_x, route.goal_y),
            });

            // Mark this route as used
            used_routes
                .entry(bucket_index)
                .or_default()
                .insert(*route_index);
        }

        info!("Generate scen: {agents:?}");
        Ok(agents)
    }

    pub fn generate_agents_randomly<R: Rng + ?Sized>(
        &self,
        num_agents: usize,
        rng: &mut R,
    ) -> Result<Vec<Agent>, String> {
        let mut agents: Vec<Agent> = Vec::new();
        let mut used_routes: HashSet<Route> = HashSet::new();

        let mut available_routes: Vec<Route> = self
            .buckets
            .as_ref()
            .unwrap()
            .clone()
            .into_iter()
            .flat_map(|(_, bucket)| bucket)
            .collect();
        available_routes.sort();

        if available_routes.len() < num_agents {
            return Err(
                "Not enough unique routes available to match the number of agents".to_string(),
            );
        }

        // Shuffle the available routes to randomize the route selection
        available_routes.shuffle(rng);

        for agent_id in 0..num_agents {
            let route = available_routes
                .pop()
                .ok_or("Ran out of routes unexpectedly")?;

            agents.push(Agent {
                id: agent_id,
                start: (route.start_x, route.start_y),
                goal: (route.goal_x, route.goal_y),
            });

            // Mark this route as used
            used_routes.insert(route);
        }

        info!("Generate scen: {agents:?}");
        Self::write_agents_to_yaml("debug.yaml", &agents).unwrap();
        Ok(agents)
    }

    pub fn load_agents_from_yaml(path: &str) -> Result<Vec<Agent>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let agents = serde_yaml::from_reader(reader)?;
        Ok(agents)
    }

    pub(crate) fn write_agents_to_yaml(path: &str, agents: &[Agent]) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = io::BufWriter::new(file);
        let yaml_data = serde_yaml::to_string(&agents)?;
        writer.write_all(yaml_data.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_read_scenario() {
        let scen = Scenario::load_from_scen("map_file/maze-32-32-2/maze-32-32-2-random-1.scen")
            .expect("Error loading YAML config");

        let seed = [0u8; 32];
        let mut rng = StdRng::from_seed(seed);

        let num_agents = 2;
        let agent_buckets = vec![0, 1];

        let agents = scen
            .generate_agents_by_buckets(num_agents, agent_buckets, &mut rng)
            .unwrap();
        let answer = [
            Agent {
                id: 0,
                start: (9, 25),
                goal: (8, 28),
            },
            Agent {
                id: 1,
                start: (8, 19),
                goal: (10, 17),
            },
        ];
        assert_eq!(agents, answer);
    }
}
