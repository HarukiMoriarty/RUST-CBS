use mapf_rust::map::Map;
use mapf_rust::yaml::Yaml;
use mapf_rust::solver::{Solver, CBS};

fn main() {
    let test_path = "test";
    let prefix = "map_file/".to_owned() + test_path + "/";
    let setting = Yaml::from_yaml(&(prefix.to_owned() + test_path + ".yaml")).expect("Error loading YAML config");


    let map_path = &setting.map;
    let map = Map::from_file(&(prefix.to_owned() + map_path)).expect("Error loading map");

    let solver = CBS::new(setting.to_agents(), map);
    let solution = solver.solve();

    println!("solution: {solution:?}");
}
