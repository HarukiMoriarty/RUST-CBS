use mapf_rust::config::{Cli, Config};
use mapf_rust::map::Map;
use mapf_rust::solver::{Solver, BCBS, CBS};
use mapf_rust::yaml::Scenario;

use anyhow::Context;
use clap::Parser;
use rand::rngs::StdRng;
use rand::SeedableRng;
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

    let setting =
        Scenario::load_from_file(&config.test_yaml_path).expect("Error loading YAML config");
    let map = Map::from_file(&config.test_map_path).expect("Error loading map");
    let mut rng = StdRng::seed_from_u64(config.seed as u64);
    let agents = setting
        .generate_agents(config.num_agents, config.agents_dist.clone(), &mut rng)
        .unwrap();
    for agent in agents.clone() {
        assert!(agent.verify(&map));
    }

    let mut cbs_solver = CBS::new(agents.clone(), &map, None);
    if let Some(cbs_solution) = cbs_solver.solve() {
        // info!("cbs solution: {cbs_solution:?}");
        assert!(cbs_solution.verify(&map, &agents));
    } else {
        error!("cbs solve fails");
    }

    let mut bcbs_solver = BCBS::new(agents.clone(), &map, Some(1.8));
    if let Some(bcbs_solution) = bcbs_solver.solve() {
        // println!("bcbs solution: {bcbs_solution:#?}");
        assert!(bcbs_solution.verify(&map, &agents));
    } else {
        info!("bcbs solve fails");
    }

    Ok(())
}
