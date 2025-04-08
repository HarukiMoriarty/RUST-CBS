use std::process::exit;

use mapf_rust::config::{Cli, Config};
use mapf_rust::map::Map;
use mapf_rust::scenario::Scenario;
use mapf_rust::solver::{Solver, ACBS, BCBS, CBS, DECBS, ECBS, HBCBS, LBCBS};

use clap::Parser;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use tokio::time::{timeout, Duration};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        if config.deterministic_scen {
            setting.generate_agents_in_order(config.num_agents).unwrap()
        } else {
            let mut rng = SmallRng::seed_from_u64(config.seed as u64);
            setting
                .generate_agents_randomly(config.num_agents, &mut rng)
                .unwrap()
        }
    };

    let map = Map::from_file(&config.map_path, &agents).expect("Error loading map");
    for agent in agents.clone() {
        assert!(agent.verify(&map));
    }

    let mut solver = match cli.solver.as_str() {
        "cbs" => Box::new(CBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "lbcbs" => Box::new(LBCBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "hbcbs" => Box::new(HBCBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "bcbs" => Box::new(BCBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "ecbs" => Box::new(ECBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "decbs" => Box::new(DECBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        "acbs" => Box::new(ACBS::new(agents.clone(), &map)) as Box<dyn Solver>,
        _ => unreachable!(),
    };

    let map_clone = map.clone();
    let agents_clone = agents.clone();
    let config_clone = config.clone();

    let solve_future = tokio::task::spawn_blocking(move || solver.solve(&config_clone));
    let result = timeout(Duration::from_secs(config.timeout_secs), solve_future).await;

    match result {
        Ok(Ok(Some(solution))) => {
            assert!(solution.verify(&map_clone, &agents_clone));
            solution.log_solution(&config);
        }
        Ok(Ok(None)) => {
            error!("{} solve failured with no solution", cli.solver);
            exit(1);
        }
        Ok(Err(e)) => {
            error!("Solver thread panicked: {e:?}");
            exit(1);
        }
        Err(_) => {
            error!(
                "{} solve timed out after {} seconds",
                cli.solver, config.timeout_secs
            );
            exit(1);
        }
    }

    Ok(())
}
