#!/usr/bin/env python3
"""
Experiment Runner for CBS (Conflict-Based Search) Algorithm Variants

This script manages and executes experiments for different variants of the CBS algorithm,
handling various parameters and configurations. It supports parallel execution and
result logging.

Features:
- Loads experiment configurations from YAML files
- Supports multiple CBS variants (CBS, ECBS, BCBS, etc.)
- Parallel execution with configurable thread count
- Automatic result logging to CSV
- Error handling and timeout management
"""

import argparse
import itertools
import logging
import subprocess
import yaml
import math
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import TypedDict, List

# Configure base path and logging
BASE_PATH = Path(__file__).absolute().parent
LOG = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO, format="%(message)s")

class ExperimentParameters(TypedDict):
    """Type definitions for experiment parameters.
    
    Attributes:
        yaml_path (List[str]): Paths to YAML configuration files
        map_path (List[str]): Paths to map files
        num_agents (List[str]): Number of agents to simulate
        agents_dist (List[str]): Agent distribution configurations
        seed_num: int: Random seed number
        sub_optimal (List[float]): Suboptimality bounds
        solver (List[str]): List of solvers to use
        time_out (str): Timeout duration
        optimization_level (List[str]): Optimization levels to test
    """
    yaml_path: List[str]
    map_path: List[str]
    num_agents: List[str]
    agents_dist: List[str]
    seed_num: int
    sub_optimal: List[float]
    solver: List[str]
    time_out: str
    optimization_level: List[str]
    output: str

def get_optimization_flags(opt_level: str) -> tuple[bool, bool, bool]:
    """Convert optimization level to specific flags.
    
    Args:
        opt_level (str): Optimization level identifier
        
    Returns:
        tuple[bool, bool, bool]: (prioritize_conflicts, bypass_conflicts, target_reasoning)
    """
    optimization_levels = {
        "none": (False, False, False),    # No optimizations
        "bc": (False, True, False),       # Only Bypass Conflicts
        "pc": (True, False, False),       # Only Prioritize Conflicts
        "tr": (False, False, True),       # Only Target Reasoning
        "pc_bc": (True, True, False),     # PC + Bypass Conflicts
        "bc_tr": (False, True, True),     # TR + Bypass Conflicts
        "all": (True, True, True)         # All optimizations
    }
    return optimization_levels.get(opt_level, (False, False, False))

def load_experiment(exp_name: str) -> dict:
    """Load experiment configuration from a YAML file.
    
    Args:
        exp_name (str): Name of the experiment configuration file
        
    Returns:
        dict: Loaded experiment configuration, or None if file not found
    """
    exp_path = BASE_PATH / "experiment" / f"{exp_name}.yaml"
    if not exp_path.exists():
        LOG.error(f"Experiment file {exp_path} not found.")
        return None

    with open(exp_path) as f:
        return yaml.safe_load(f)

def generate_combinations(params: ExperimentParameters):
    """Generate parameter combinations for the experiment with specific optimization levels.
    
    Args:
        params (ExperimentParameters): Base parameters for the experiment
        
    Yields:
        dict: Parameter combination for a single experiment run
    """
    # Create base combinations without optimization flags and suboptimality
    base_keys = [k for k in params.keys() if k not in ['optimization_level', 'sub_optimal']]
    base_values = []

    for key in base_keys:
        if key == "seed_num":
            base_values.append(list(range(params[key])))
        elif isinstance(params[key], list):
            base_values.append(params[key])
        else:
            base_values.append([params[key]])
            
    # Generate combinations
    for base_combination in itertools.product(*base_values):
        base_params = dict(zip(base_keys, base_combination))
        
        # Handle suboptimality based on solver
        solver = base_params['solver']
        subopt_values = [1.0] if solver == 'cbs' else params['sub_optimal']
        
        for subopt in subopt_values:
            base_params_with_subopt = base_params.copy()
            if solver != 'cbs':  # Only add suboptimality for non-CBS solvers
                base_params_with_subopt['sub_optimal'] = subopt
            
            # Add each optimization combination
            for opt_level in params['optimization_level']:
                pc, bc, tr = get_optimization_flags(opt_level)
                full_params = base_params_with_subopt.copy()
                full_params['op_prioritize_conflicts'] = pc
                full_params['op_bypass_conflicts'] = bc
                full_params['op_target_reasoning'] = tr
                yield full_params

def check_and_create_csv(output_csv_path: str):
    """Initialize CSV file with headers if it doesn't exist.
    
    Args:
        output_csv_path (str): Path to the output CSV file
    """
    csv_path = Path(output_csv_path)
    if not csv_path.exists():
        csv_path.parent.mkdir(parents=True, exist_ok=True)
        with open(csv_path, 'w') as csv_file:
            headers = [
                "map_path", "yaml_path", "num_agents", "agents_dist", "seed",
                "solver", "high_level_suboptimal", "low_level_suboptimal",
                "op_PC", "op_BC", "op_TR", "costs", "time(us)",
                "high_level_expanded", "low_level_open_expanded",
                "low_level_focal_expanded", "total_low_level_expanded"
            ]
            csv_file.write(",".join(headers) + "\n")

def write_error_csv(params: ExperimentParameters, error_msg: str):
    """Write error information to CSV file when experiment fails.
    
    Args:
        params (ExperimentParameters): Parameters of the failed experiment
        error_msg (str): Error message to log
    """
    output_path = params.get("output", "./result/result.csv")
    with open(output_path, 'a+') as file:
        # Build CSV row components
        base_info = [
            params["map_path"],
            params["yaml_path"],
            str(params["num_agents"]),
            "[]",
            str(params["seed_num"]),
            str(params["solver"])
        ]
        
        # Handle suboptimality values based on solver type
        if params["solver"] == "cbs":
            subopt_values = ["NaN", "NaN"]
        elif params["solver"] in ["lbcbs", "ecbs", "decbs"]:
            subopt_values = ["NaN", str(params["sub_optimal"])]
        elif params["solver"] == "hbcbs":
            subopt_values = [str(params["sub_optimal"]), "NaN"]
        elif params["solver"] == "bcbs":
            sqrt_subopt = str(math.sqrt(params["sub_optimal"]))
            subopt_values = [sqrt_subopt, sqrt_subopt]
            
        # Add operation flags
        op_flags = [
            str(params["op_prioritize_conflicts"]).lower(),
            str(params["op_bypass_conflicts"]).lower(),
            str(params["op_target_reasoning"]).lower()
        ]
        
        # Combine all components and write
        row = ",".join(base_info + subopt_values + op_flags + [error_msg])
        file.write(row + "\n")

def run_experiment(params: ExperimentParameters):
    """Execute a single experiment with given parameters.
    
    Args:
        params (ExperimentParameters): Parameters for this experiment run
    """
    # Ensure output CSV exists
    check_and_create_csv(params.get("output", "./result/result.csv"))
    timeout = params["time_out"]

    # Build base command
    cmd_base = [
        "cargo", "run", "--release", "--",
        "--yaml-path", params["yaml_path"],
        "--map-path", params["map_path"],
        "--num-agents", str(params["num_agents"]),
        "--seed", str(params["seed_num"]),
        "--solver", str(params["solver"]),
        "--output-path", str(params["output"])
    ]

    # Add solver-specific parameters
    solver = params["solver"]
    if solver in ["lbcbs", "ecbs", "decbs"]:
        cmd_base.extend(["--low-level-sub-optimal", str(params["sub_optimal"])])
    elif solver == "hbcbs":
        cmd_base.extend(["--high-level-sub-optimal", str(params["sub_optimal"])])
    elif solver == "bcbs":
        sqrt_subopt = str(math.sqrt(params["sub_optimal"]))
        cmd_base.extend([
            "--low-level-sub-optimal", sqrt_subopt,
            "--high-level-sub-optimal", sqrt_subopt
        ])

    # Add optional parameters
    if params.get("op_prioritize_conflicts", False):
        cmd_base.append("--op-prioritize-conflicts")
    if params.get("op_bypass_conflicts", False):
        cmd_base.append("--op-bypass-conflicts")
    if params.get("op_target_reasoning", False):
        cmd_base.append("--op-target-reasoning")

    # Execute experiment
    LOG.info(f"Executing: {' '.join(cmd_base)}")
    try:
        subprocess.run(cmd_base, check=True, timeout=timeout)
        LOG.info("Experiment completed successfully.")
    except subprocess.TimeoutExpired:
        LOG.error(f"Experiment timed out after {timeout} seconds.")
        write_error_csv(params, "timeout")
    except subprocess.CalledProcessError:
        LOG.error("Experiment failed to run successfully.")
        write_error_csv(params, "solvefailure")
    except Exception as e:
        LOG.error(f"An error occurred: {str(e)}")

def main():
    """Main entry point for the experiment runner."""
    parser = argparse.ArgumentParser(description="Run CBS experiments with different parameters.")
    parser.add_argument("experiment", help="Experiment name to run.")
    parser.add_argument("--max-threads", type=int, default=4,
                      help="Maximum number of parallel threads")
    args = parser.parse_args()

    # Load experiment configuration
    exp_params = load_experiment(args.experiment)
    if exp_params is None:
        return

    # Execute experiments in parallel
    with ThreadPoolExecutor(max_workers=args.max_threads) as executor:
        futures = {
            executor.submit(run_experiment, combination): combination 
            for combination in generate_combinations(exp_params)
        }
        
        for future in as_completed(futures):
            try:
                future.result()
            except Exception as e:
                combination = futures[future]
                LOG.error(f"An error occurred with experiment settings {combination}: {str(e)}")

if __name__ == "__main__":
    main()