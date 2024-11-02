use mapf_rust::config::{Cli, Config};
use mapf_rust::map::Map;
use mapf_rust::solver::{Solver, BCBS, CBS};
use mapf_rust::yaml::Yaml;

use anyhow::Context;
use clap::Parser;
use tracing::{error, info, Level};
use tracing_subscriber;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    let cli = Cli::parse();

    let config = Box::leak(Box::new(
        if let Some(config_file) = cli.config.as_ref() {
            let config_str = std::fs::read_to_string(config_file)?;
            Config::from_yaml_str(&config_str)
                .with_context(|| format!("error with config file: {config_file}"))?
        } else {
            info!("No config file specified, using default config");
            Config::default()
        }
        .override_from_command_line(&cli)?,
    ));

    let setting = Yaml::from_yaml(&config.test_yaml_path).expect("Error loading YAML config");
    let map = Map::from_file(&config.test_map_path).expect("Error loading map");
    let agent = setting.to_agents(&map).unwrap();

    let mut cbs_solver = CBS::new(agent.clone(), &map, None);
    if let Some(cbs_solution) = cbs_solver.solve() {
        // println!("cbs solution: {cbs_solution:#?}");
        assert!(cbs_solution.verify(&map, &agent));
    } else {
        error!("cbs solve fails");
    }

    let mut bcbs_solver = BCBS::new(agent.clone(), &map, Some(1.8));
    if let Some(bcbs_solution) = bcbs_solver.solve() {
        // println!("bcbs solution: {bcbs_solution:#?}");
        assert!(bcbs_solution.verify(&map, &agent));
    } else {
        println!("bcbs solve fails");
    }

    Ok(())
}
