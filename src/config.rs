use anyhow::anyhow;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Rust CBS-MAPF",
    about = "Kinds of Conflict-Based MAPF algorithm implemented in Rust.",
    author = "Moriarty Yu",
    version = "1.0"
)]
pub struct Cli {
    #[arg(
        long,
        help = "Path to the YAML scenario file",
        default_value = "map_file/random-32-32-20/random-32-32-20-random-1.scen"
    )]
    pub yaml_path: String,

    #[arg(
        long,
        help = "Path to the map file",
        default_value = "map_file/random-32-32-20/random-32-32-20.map"
    )]
    pub map_path: String,

    #[arg(long, help = "Path to the output file")]
    pub output_path: Option<String>,

    #[arg(long, help = "Output LACAM-style formatted solution to a file")]
    pub solution_path: String,

    #[arg(long, help = "Number of agents", default_value_t = 10)]
    pub num_agents: usize,

    #[arg(long, help = "Distribution of agents", use_value_delimiter = true)]
    pub agents_dist: Vec<usize>,

    #[arg(
        long,
        help = "Use deterministic agent selection from .scen (no randomness)",
        default_value_t = false
    )]
    pub deterministic_scen: bool,

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

    #[arg(long, help = "Optimization: Target Reasoning", default_value_t = false)]
    pub op_target_reasoning: bool,

    #[arg(long, help = "Timeout seconds", default_value = "60")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub yaml_path: String,
    pub map_path: String,
    pub output_path: Option<String>,
    pub solution_path: String,
    pub num_agents: usize,
    pub agents_dist: Vec<usize>,
    pub deterministic_scen: bool,
    pub seed: usize,
    pub sub_optimal: (Option<f64>, Option<f64>), // (high level sub optimal, low level sub optimal)
    pub solver: String,
    pub debug_yaml: bool,
    pub op_prioritize_conflicts: bool,
    pub op_bypass_conflicts: bool,
    pub op_target_reasoning: bool,
    pub timeout_secs: u64,
}

impl Config {
    pub fn new(cli: &Cli) -> Self {
        Self {
            yaml_path: cli.yaml_path.clone(),
            map_path: cli.map_path.clone(),
            solution_path: cli.solution_path.clone(),
            output_path: cli.output_path.clone(),
            num_agents: cli.num_agents,
            agents_dist: cli.agents_dist.clone(),
            deterministic_scen: cli.deterministic_scen,
            seed: cli.seed,
            sub_optimal: (cli.high_level_sub_optimal, cli.low_level_sub_optimal),
            solver: cli.solver.clone(),
            debug_yaml: cli.debug_yaml,
            op_prioritize_conflicts: cli.op_prioritize_conflicts,
            op_bypass_conflicts: cli.op_bypass_conflicts,
            op_target_reasoning: cli.op_target_reasoning,
            timeout_secs: cli.timeout_secs,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate suboptimality values are present/absent correctly per solver
        match self.solver.as_str() {
            "cbs" => {
                // Both should be None for CBS
                if self.sub_optimal.0.is_some() || self.sub_optimal.1.is_some() {
                    return Err(anyhow!(
                        "CBS should not have any suboptimality bounds, got high-level: {:?}, low-level: {:?}",
                        self.sub_optimal.0,
                        self.sub_optimal.1
                    ));
                }
            }
            "lbcbs" | "ecbs" | "decbs" | "acbs" => {
                // Only low-level sub optimal should be Some
                if self.sub_optimal.0.is_some() || self.sub_optimal.1.is_none() {
                    return Err(anyhow!(
                        "LBCBS/ECBS/DECBS should only have low-level suboptimality bound, got high-level: {:?}, low-level: {:?}",
                        self.sub_optimal.0,
                        self.sub_optimal.1
                    ));
                }
            }
            "hbcbs" => {
                // Only high-level sub optimal should be Some
                if self.sub_optimal.0.is_none() || self.sub_optimal.1.is_some() {
                    return Err(anyhow!(
                        "HBCBS should only have high-level suboptimality bound, got high-level: {:?}, low-level: {:?}",
                        self.sub_optimal.0,
                        self.sub_optimal.1
                    ));
                }
            }
            "bcbs" => {
                // Both should be Some for BCBS
                if self.sub_optimal.0.is_none() || self.sub_optimal.1.is_none() {
                    return Err(anyhow!(
                        "BCBS should have both suboptimality bounds, got high-level: {:?}, low-level: {:?}",
                        self.sub_optimal.0,
                        self.sub_optimal.1
                    ));
                }
            }
            _ => unreachable!(),
        }

        // Validate the values if they are present
        if let Some(low_level_sub_optimal) = self.sub_optimal.1 {
            if low_level_sub_optimal < 1.0 {
                return Err(anyhow!(
                    "Low-level sub-optimal value must be greater than 1.0, got {}",
                    low_level_sub_optimal
                ));
            }
        }

        if let Some(high_level_sub_optimal) = self.sub_optimal.0 {
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
