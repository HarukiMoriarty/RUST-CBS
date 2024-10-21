use mapf_rust::map::Map;
use mapf_rust::yaml::Yaml;
use mapf_rust::solver::{Solver, CBS};
use mapf_rust::config::{Cli, Config};

use clap::Parser;
use anyhow::Context;

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    let config = Box::leak(Box::new(
        if let Some(config_file) = cli.config.as_ref() {
            let config_str = std::fs::read_to_string(config_file)?;
            Config::from_yaml_str(&config_str)
                .with_context(|| format!("error with config file: {config_file}"))?
        } else {
            println!("No config file specified, using default config");
            Config::default()
        }
        .override_from_command_line(&cli)?,
    ));

    let setting = Yaml::from_yaml(&config.test_yaml_path).expect("Error loading YAML config");
    let map = Map::from_file(&config.test_map_path).expect("Error loading map");

    let solver = CBS::new(setting.to_agents(&map).unwrap(), &map);
    let solution = solver.solve();

    // Verify solution.
    assert!(solution.verify(&map));

    println!("solution: {solution:?}");

    Ok(())
}
