use mapf_rust::config::{Cli, Config};
use mapf_rust::map::Map;
use mapf_rust::scenario::Scenario;
use mapf_rust::solver::{Solver, BCBS, CBS, ECBS, HBCBS, LBCBS};

use clap::Parser;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let cli = Cli::parse();

    let config = Config::new(&cli);
    config.validate()?;
    info!("Config: {config:#?}");

    let agents = if cli.debug_yaml {
        Scenario::load_agents_from_yaml("debug.yaml").unwrap()
    } else {
        let setting =
            Scenario::load_from_scen(&config.yaml_path).expect("Error loading YAML config");
        let mut rng = SmallRng::seed_from_u64(config.seed as u64);
        setting
            .generate_agents_randomly(config.num_agents, &mut rng)
            .unwrap()
    };

    let map = Map::from_file(&config.map_path, &agents).expect("Error loading map");
    for agent in agents.clone() {
        assert!(agent.verify(&map));
    }

    let mut solver = match cli.solver.as_str() {
        "cbs" => Box::new(CBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "lbcbs" => {
            Box::new(LBCBS::new(agents.clone(), &map, config.sub_optimal)) as Box<dyn Solver>
        }
        "hbcbs" => {
            Box::new(HBCBS::new(agents.clone(), &map, config.sub_optimal)) as Box<dyn Solver>
        }
        "bcbs" => Box::new(BCBS::new(agents.clone(), &map, config.sub_optimal)) as Box<dyn Solver>,
        "ecbs" => Box::new(ECBS::new(agents.clone(), &map, config.sub_optimal)) as Box<dyn Solver>,
        _ => unreachable!(),
    };

    if let Some(solution) = solver.solve() {
        debug!("{:?} solution: {solution:?}", cli.solver);
        assert!(solution.verify(&map, &agents));
    } else {
        error!("{:?} solve fails", cli.solver);
    }

    Ok(())
}
