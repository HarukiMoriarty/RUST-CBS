mod map;
mod yaml;
mod astar;

use map::Map;
use yaml::Yaml;
use astar::a_star_search;

fn main() {
    let test_path = "test";
    let prefix = "map_file/".to_owned() + &test_path + "/";
    let setting = Yaml::from_yaml(&(prefix.to_owned() + &test_path + ".yaml")).expect("Error loading YAML config");


    let map_path = &setting.map;
    let map = Map::from_file(&(prefix.to_owned() + &map_path)).expect("Error loading map");

    for agent_idx in 0..setting.agent.len() {
        if let Some(path) = a_star_search(&map, setting.agent[agent_idx].start.into(), setting.agent[agent_idx].potentialGoals[0].into()) {
            println!("Path found: {:?}", path);
        } else {
            println!("No path found");
        }
        
    }
}
