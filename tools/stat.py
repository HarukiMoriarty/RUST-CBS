import pandas as pd
import numpy as np
import argparse
import logging

logging.basicConfig(level=logging.INFO)
LOG = logging.getLogger(__name__)

def load_data(file_path):
    return pd.read_csv(file_path)

def compute_stats(df, column):
    return np.percentile(df[column].dropna(), [0, 50, 99])

def analyze_experiments(file_path):
    data = load_data(file_path)
    
    solvers = data['solver'].unique()
    for solver in solvers:
        solver_data = data[data['solver'] == solver]

        timeouts = solver_data['low_level_suboptimal'].astype(str).str.contains('timeout', na=False)
        timeout_count = timeouts.sum()
        success_data = solver_data[~timeouts]
        total_count = len(solver_data)
        timeout_rate = timeout_count / total_count if total_count > 0 else 0

        print(f"Stats for solver: {solver}")
        print(f"Total entries for {solver}: {total_count}")
        print(f"Timeout entries for {solver}: {timeout_count}")
        print(f"Timeout rate for {solver}: {timeout_rate:.2%}")

        if not success_data.empty:
            # Time statistics
            time_stats = compute_stats(success_data, 'time(us)')
            print(f"Time P0: {time_stats[0]} us, P50: {time_stats[1]} us, P99: {time_stats[2]} us")
            
            # High level expanded nodes statistics
            high_level_stats = compute_stats(success_data, 'high_level_expanded')
            print(f"High level expanded nodes P0: {high_level_stats[0]}, P50: {high_level_stats[1]}, P99: {high_level_stats[2]}")
            
            # Low level expanded nodes statistics
            low_level_stats = compute_stats(success_data, 'low_level_expanded')
            print(f"Low level expanded nodes P0: {low_level_stats[0]}, P50: {low_level_stats[1]}, P99: {low_level_stats[2]}")
        else:
            print("No successful runs for this solver to analyze.")

def main():
    parser = argparse.ArgumentParser(description="Analyze experiment data from a CSV file.")
    parser.add_argument("file_path", help="Path to the CSV file containing the experiment data.")
    
    args = parser.parse_args()
    analyze_experiments(args.file_path)

if __name__ == "__main__":
    main()
