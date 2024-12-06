import pandas as pd
import numpy as np
import argparse
import logging

logging.basicConfig(level=logging.INFO)
LOG = logging.getLogger(__name__)

MAX_INT = np.iinfo(np.int64).max

def load_data(file_path):
    data = pd.read_csv(file_path)
    timeout_condition = data['costs'].astype(str).str.contains('timeout', na=False)
    data.loc[timeout_condition, ['time(us)', 'costs', 'high_level_expanded', 'low_level_open_expanded', 'low_level_focal_expanded', 'total_low_level_expanded']] = MAX_INT
    return data

def compute_stats(df, column):
    return np.percentile(df[column].dropna(), [0, 50, 99], method="nearest")

def analyze_experiments(file_path, output_file_path):
    data = load_data(file_path)
    results = []

    for num_agents in data['num_agents'].unique():
        for seed in data['seed'].unique():
            for op_PC in data['op_PC'].unique():
                group_data = data[(data['num_agents'] == num_agents) & (data['seed'] == seed) & (data['op_PC'] == op_PC)]
                cbs_data = group_data[group_data['solver'] == 'cbs']
                cbs_costs = cbs_data['costs']

                if not cbs_costs.empty:
                    cbs_cost_min = cbs_costs.min()
                    if cbs_cost_min != MAX_INT:
                        for solver in group_data['solver'].unique():
                            solver_data = group_data[group_data['solver'] == solver]
                            solver_costs = solver_data['costs']
                            if not solver_costs.empty:
                                if (solver_costs < cbs_cost_min).any():
                                    print(f"Discrepancy found for num_agents={num_agents}, seed={seed}, solver={solver}")

    for solver in data['solver'].unique():
        for num_agents in data['num_agents'].unique():
            for op_PC in data['op_PC'].unique():
                solver_agent_data = data[(data['solver'] == solver) & (data['num_agents'] == num_agents) & (data['op_PC'] == op_PC)]

                timeouts = solver_agent_data['time(us)'] == MAX_INT
                timeout_count = timeouts.sum()
                success_data = solver_agent_data[~timeouts]
                total_count = len(solver_agent_data)
                timeout_rate = timeout_count / total_count if total_count > 0 else 0

                if not success_data.empty:
                    # Time statistics
                    time_stats = compute_stats(success_data, 'time(us)')
                    # High level expanded nodes statistics
                    high_level_stats = compute_stats(success_data, 'high_level_expanded')
                    # Low level open expanded nodes statistics
                    open_low_level_stats = compute_stats(success_data, 'low_level_open_expanded')
                    # Low level focal expanded nodes statistics
                    focal_low_level_stats = compute_stats(success_data, 'low_level_focal_expanded')
                    # Total Low level expanded nodes statistics
                    total_low_level_stats = compute_stats(success_data, 'total_low_level_expanded')
                else:
                    time_stats = high_level_stats = open_low_level_stats = focal_low_level_stats = total_low_level_stats = [np.nan, np.nan, np.nan]

                result = {
                    "solver": solver,
                    "op_PC": op_PC,
                    "fail_rate": timeout_rate,
                    "num_agents": num_agents,
                    "P0time": time_stats[0],
                    "P50time": time_stats[1],
                    "P99time": time_stats[2],
                    "P0high": high_level_stats[0],
                    "P50high": high_level_stats[1],
                    "P99high": high_level_stats[2],
                    "P0lowOpen": open_low_level_stats[0],
                    "P50lowOpen": open_low_level_stats[1],
                    "P99lowOpen": open_low_level_stats[2],
                    "P0lowFocal": focal_low_level_stats[0],
                    "P50lowFocal": focal_low_level_stats[1],
                    "P99lowFocal": focal_low_level_stats[2],
                    "P0lowTotal": total_low_level_stats[0],
                    "P50lowTotal": total_low_level_stats[1],
                    "P99lowTotal": total_low_level_stats[2]
                }
                results.append(result)

    results_df = pd.DataFrame(results)
    results_df.to_csv(output_file_path, index=False)

def main():
    parser = argparse.ArgumentParser(description="Analyze experiment data from a CSV file and output results.")
    parser.add_argument("file_path", help="Path to the CSV file containing the experiment data.")
    parser.add_argument("output_file_path", help="Path to the CSV file to save the analysis results.")
    
    args = parser.parse_args()
    analyze_experiments(args.file_path, args.output_file_path)

if __name__ == "__main__":
    main()
