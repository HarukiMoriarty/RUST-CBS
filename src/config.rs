use anyhow::anyhow;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Rust MAPF",
    about = "Kinds of MAPF algorithm implemented in Rust.",
    author = "Moriarty Yu",
    version = "1.0"
)]
pub struct Cli {
    #[arg(
        long,
        help = "Path to the YAML scenario file",
        default_value = "map_file/Boston-0-256-scen-even/Boston_0_256-even-1.scen"
    )]
    pub yaml_path: String,

    #[arg(
        long,
        help = "Path to the map file",
        default_value = "map_file/Boston-0-256-scen-even/Boston_0_256.map"
    )]
    pub map_path: String,

    #[arg(
        long,
        help = "Path to the output file",
        default_value = "result/result.csv"
    )]
    pub output_path: String,

    #[arg(long, help = "Number of agents", default_value_t = 10)]
    pub num_agents: usize,

    #[arg(long, help = "Distribution of agents", use_value_delimiter = true)]
    pub agents_dist: Vec<usize>,

    #[arg(
        long,
        help = "Seed for the random number generator",
        default_value_t = 0
    )]
    pub seed: usize,

    #[arg(long, help = "Suboptimal limit for low-level operations")]
    pub low_level_sub_optimal: Option<f64>,

    #[arg(long, help = "Suboptimal limit for high-level operations")]
    pub high_level_sub_optimal: Option<f64>,

    #[arg(long, help = "Solver to use", default_value = "cbs")]
    pub solver: String,

    #[arg(
        long,
        help = "Enable debugging for YAML scenarios",
        default_value_t = false
    )]
    pub debug_yaml: bool,

    #[arg(
        long,
        help = "Optimization: Prioritize Conflicts",
        default_value_t = false
    )]
    pub op_prioritize_conflicts: bool,

    #[arg(long, help = "Optimization: Bypass Conflicts", default_value_t = false)]
    pub op_bypass_conflicts: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub yaml_path: String,
    pub map_path: String,
    pub output_path: String,
    pub num_agents: usize,
    pub agents_dist: Vec<usize>,
    pub seed: usize,
    pub sub_optimal: (Option<f64>, Option<f64>),
    pub solver: String,
    pub debug_yaml: bool,
    pub op_prioritize_conflicts: bool,
    pub op_bypass_conflicts: bool,
}

impl Config {
    pub fn new(cli: &Cli) -> Self {
        Self {
            yaml_path: cli.yaml_path.clone(),
            map_path: cli.map_path.clone(),
            output_path: cli.output_path.clone(),
            num_agents: cli.num_agents,
            agents_dist: cli.agents_dist.clone(),
            seed: cli.seed,
            sub_optimal: (cli.high_level_sub_optimal, cli.low_level_sub_optimal),
            solver: cli.solver.clone(),
            debug_yaml: cli.debug_yaml,
            op_prioritize_conflicts: cli.op_prioritize_conflicts,
            op_bypass_conflicts: cli.op_bypass_conflicts,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let mut check_low_level_sub_optimal = false;
        let mut check_high_level_sub_optimal = false;

        match self.solver.as_str() {
            "cbs" => {}
            "lbcbs" | "ecbs" | "decbs" => check_low_level_sub_optimal = true,
            "hbcbs" => check_high_level_sub_optimal = true,
            "bcbs" => {
                check_high_level_sub_optimal = true;
                check_low_level_sub_optimal = true;
            }
            _ => unreachable!(),
        }

        if check_low_level_sub_optimal {
            let low_level_sub_optimal = self.sub_optimal.1.unwrap();
            if low_level_sub_optimal < 1.0 {
                return Err(anyhow!(
                    "Low-level sub-optimal value must be greater than 1.0, got {}",
                    low_level_sub_optimal
                ));
            }
        }

        if check_high_level_sub_optimal {
            let high_level_sub_optimal = self.sub_optimal.0.unwrap();
            if high_level_sub_optimal < 1.0 {
                return Err(anyhow!(
                    "High-level sub-optimal value must be greater than 1.0, got {}",
                    high_level_sub_optimal
                ));
            }
        }
        Ok(())
    }
}
