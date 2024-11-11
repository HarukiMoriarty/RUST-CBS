use mapf_rust::config::{Cli, Config};
use mapf_rust::map::Map;
use mapf_rust::solver::{Solver, BCBS, CBS, ECBS, HBCBS, LBCBS};
use mapf_rust::yaml::Scenario;

use anyhow::Context;
use clap::Parser;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tracing::{error, info, Level};
use tracing_subscriber;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
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
    info!("Config: {config:#?}");

    let setting =
        Scenario::load_from_file(&config.test_yaml_path).expect("Error loading YAML config");
    let map = Map::from_file(&config.test_map_path).expect("Error loading map");
    let mut rng = StdRng::seed_from_u64(config.seed as u64);
    let agents = setting
        .generate_agents_by_buckets(config.num_agents, config.agents_dist.clone(), &mut rng)
        .unwrap();
    for agent in agents.clone() {
        assert!(agent.verify(&map));
    }

    for solver_string in &config.solver {
        let mut solver =
            match solver_string.as_str() {
                "cbs" => Box::new(CBS::new(agents.clone(), &map)) as Box<dyn Solver>,
                "lbcbs" => Box::new(LBCBS::new(agents.clone(), &map, config.sub_optimal))
                    as Box<dyn Solver>,
                "hbcbs" => Box::new(HBCBS::new(agents.clone(), &map, config.sub_optimal))
                    as Box<dyn Solver>,
                "bcbs" => {
                    Box::new(BCBS::new(agents.clone(), &map, config.sub_optimal)) as Box<dyn Solver>
                }
                "ecbs" => {
                    Box::new(ECBS::new(agents.clone(), &map, config.sub_optimal)) as Box<dyn Solver>
                }
                _ => unreachable!(),
            };

        if let Some(solution) = solver.solve() {
            // info!("{solver_string:?} solution: {solution:?}");
            assert!(solution.verify(&map, &agents));
        } else {
            error!("{solver_string:?} solve fails");
        }
    }

    Ok(())
}
