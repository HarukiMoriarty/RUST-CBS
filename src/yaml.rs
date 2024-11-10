use anyhow::Result;
use rand::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader};
use tracing::info;

use crate::common::Agent;

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub start_x: usize,
    pub start_y: usize,
    pub goal_x: usize,
    pub goal_y: usize,
    pub optimal_length: f64,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.start_x == other.start_x
            && self.start_y == other.start_y
            && self.goal_x == other.goal_x
            && self.goal_y == other.goal_y
    }
}

impl Eq for Route {}

impl Hash for Route {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start_x.hash(state);
        self.start_y.hash(state);
        self.goal_x.hash(state);
        self.goal_y.hash(state);
    }
}

type Bucket = Vec<Route>;

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub map: String,
    pub map_width: usize,
    pub map_height: usize,
    pub buckets: HashMap<usize, Bucket>,
}

impl Scenario {
    pub fn load_from_file(path: &str) -> io::Result<Scenario> {
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
            buckets: HashMap::new(),
        };

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let bucket_index: usize = parts[0].parse().unwrap();

            let route = Route {
                start_x: parts[5].parse().unwrap(),
                start_y: parts[4].parse().unwrap(),
                goal_x: parts[7].parse().unwrap(),
                goal_y: parts[6].parse().unwrap(),
                optimal_length: parts[8].parse().unwrap(),
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
                .entry(bucket_index)
                .or_insert_with(Vec::new)
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
                .get(&bucket_index)
                .ok_or_else(|| format!("Bucket {} not found", bucket_index))?;

            // Find unused routes
            let available_routes: Vec<usize> = (0..bucket.len())
                .filter(|idx| {
                    used_routes
                        .get(&bucket_index)
                        .map_or(true, |used| !used.contains(idx))
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
                .or_insert_with(HashSet::new)
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
            .clone()
            .into_iter()
            .flat_map(|(_, bucket)| bucket)
            .collect();

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
        Ok(agents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_read_yaml() {
        let scen =
            Scenario::load_from_file("map_file/maze-32-32-2-scen-even/maze-32-32-2-even-1.scen")
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
                start: (30, 23),
                goal: (29, 20),
            },
            Agent {
                id: 1,
                start: (13, 26),
                goal: (11, 22),
            },
        ];
        assert_eq!(agents, answer);
    }
}
