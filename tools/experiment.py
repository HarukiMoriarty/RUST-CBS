import argparse
import itertools
import logging
import subprocess
import yaml
import math

from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import TypedDict, List

BASE_PATH = Path(__file__).absolute().parent

LOG = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO, format="%(message)s")

class ExperimentParameters(TypedDict):
    yaml_path: List[str]
    map_path: List[str]
    num_agents: List[str]
    agents_dist: List[str]
    seed_num: int
    sub_optimal: List[float]
    solver: List[str]
    time_out: str

def load_experiment(exp_name: str):
    exp_path = BASE_PATH / "experiment" / f"{exp_name}.yaml"
    if not exp_path.exists():
        LOG.error(f"Experiment file {exp_path} not found.")
        return None

    with open(exp_path) as f:
        return yaml.safe_load(f)

def generate_combinations(params: ExperimentParameters):
    keys = list(params.keys())
    values = []

    for key in keys:
        if key == "seed_num":
            values.append(list(range(params[key])))  # Creates a list [0, 1, ..., seed_num - 1]
        elif isinstance(params[key], list):
            values.append(params[key])
        else:
            values.append([params[key]])
    for combination in itertools.product(*values):
        yield dict(zip(keys, combination))

def check_and_create_csv(output_csv_path: str):
    # Convert string path to Path object for easier handling
    csv_path = Path(output_csv_path)
    if not csv_path.exists():
        # Ensure the directory exists
        csv_path.parent.mkdir(parents=True, exist_ok=True)
        # Create the file and write the header
        with open(csv_path, 'w') as csv_file:
            csv_file.write("map_path,yaml_path,num_agents,agents_dist,seed,solver,low_level_suboptimal,high_level_suboptimal,costs,time(us),high_level_expanded,low_level_expanded\n")

def write_error_csv(params: ExperimentParameters, error_msg: str):
    with open(params.get("output_csv_result", "./result/result.csv"), 'a+') as file:
        file.write(params["map_path"] + "," + params["yaml_path"] + "," + str(params["num_agents"]) + ",[]," + str(params["seed_num"]) + "," + str(params["solver"]) + "," + error_msg + "\n")

def run_experiment(params: ExperimentParameters):
    check_and_create_csv(params.get("output_csv_result", "./result/result.csv"))
    timeout = params["time_out"]

    cmd_base = [
        "cargo", "run", "--release", "--",
        "--yaml-path", params["yaml_path"],
        "--map-path", params["map_path"],
        "--num-agents", str(params["num_agents"]),
        "--seed", str(params["seed_num"]),
        "--solver", str(params["solver"]),
    ]

    solver = params.get("solver", "")  
    if solver in ["lbcbs", "ecbs"]: 
        cmd_base.extend(["--low-level-sub-optimal", str(params.get("sub_optimal", 1.0))])  
    if solver in ["hbcbs"]:  
        cmd_base.extend(["--high-level-sub-optimal", str(params.get("sub_optimal", 1.0))])  
    if solver in ["bcbs"]:
        cmd_base.extend(["--low-level-sub-optimal", str(math.sqrt(params.get("sub_optimal", 1.0)))])
        cmd_base.extend(["--high-level-sub-optimal", str(math.sqrt(params.get("sub_optimal", 1.0)))])  

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
    parser = argparse.ArgumentParser(description="Run CBS experiments with different parameters.")
    parser.add_argument("experiment", help="Experiment name to run.")
    parser.add_argument("--max-threads", type=int, default=4, help="Maximum number of parallel threads")
    args = parser.parse_args()

    exp_params = load_experiment(args.experiment)
    if exp_params is None:
        return

    with ThreadPoolExecutor(max_workers=args.max_threads) as executor:
        futures = {executor.submit(run_experiment, combination): combination for combination in generate_combinations(exp_params)}
        for future in as_completed(futures):
            try:
                future.result()  # This will raise any exceptions caught during the experiment run
            except Exception as e:
                combination = futures[future]
                LOG.error(f"An error occurred with experiment settings {combination}: {str(e)}")

if __name__ == "__main__":
    main()