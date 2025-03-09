import pandas as pd
import numpy as np
import glob
import os

def analyze_timeouts_total(data_paths, timeout_threshold=60):
    """
    Analyze timeouts in DECBS vs ECBS experiments and return only the total statistics.
    
    Parameters:
    - data_paths: Path to the CSV data file(s). Can be a single file path, 
                 a list of file paths, or a glob pattern
    - timeout_threshold: Threshold in seconds to consider a run as timeout (default: 60)
    
    Returns:
    - Dictionary with total timeout statistics
    """
    # Handle different input types for data_paths
    if isinstance(data_paths, str):
        # Check if it's a glob pattern
        if any(char in data_paths for char in ['*', '?', '[']):
            file_paths = glob.glob(data_paths)
        else:
            # Single file
            file_paths = [data_paths]
    else:
        # Assume it's a list/iterable of paths
        file_paths = data_paths
    
    # Initialize an empty DataFrame to store combined data
    combined_df = pd.DataFrame()
    
    # Process each file and concatenate
    for file_path in file_paths:
        print(f"Processing {file_path}...")
        try:
            df = pd.read_csv(file_path)
            combined_df = pd.concat([combined_df, df], ignore_index=True)
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
    
    # If no data was loaded, return empty results
    if combined_df.empty:
        print("No data was loaded. Please check your file paths.")
        return None
    
    # Continue with the combined DataFrame
    df = combined_df
    
    # Convert the 'time(us)' column to seconds
    df['time(us)'] = pd.to_numeric(df['time(us)'], errors='coerce') / 1_000_000
    
    # Columns that define a unique experiment setting
    merge_cols = ["map_path", "yaml_path", 
                  "num_agents", "seed",
                  "low_level_suboptimal",
                  "op_PC", "op_BC", "op_TR"]
    
    # Group by the unique experiment settings and 'solver', aggregating time with the first value
    df_grouped = df.groupby(merge_cols + ['solver'], as_index=False)['time(us)'].first()
    
    # Pivot the DataFrame to create separate columns for decbs and ecbs times
    df_pivot = df_grouped.pivot(index=merge_cols, columns='solver', values='time(us)').reset_index()
    
    # For any missing time, fill with the timeout value
    df_pivot = df_pivot.fillna(timeout_threshold)
    
    # Rename solver columns for consistency
    df_pivot = df_pivot.rename(columns={'decbs': 'time_decbs', 'ecbs': 'time_ecbs'})
    
    # Define the timeout conditions
    df_pivot['decbs_timeout'] = df_pivot['time_decbs'] >= timeout_threshold
    df_pivot['ecbs_timeout'] = df_pivot['time_ecbs'] >= timeout_threshold
    
    # Define the three cases based on different optimizations
    case_conditions = [
        # Case 1: No optimizations
        ((df_pivot['op_PC'] == False) & (df_pivot['op_BC'] == False) & (df_pivot['op_TR'] == False)),
        # Case 2: Only BC optimization
        ((df_pivot['op_PC'] == False) & (df_pivot['op_BC'] == True) & (df_pivot['op_TR'] == False)),
        # Case 3: BC and TR optimizations
        ((df_pivot['op_PC'] == False) & (df_pivot['op_BC'] == True) & (df_pivot['op_TR'] == True))
    ]
    case_names = ['No Optimizations', 'BC', 'BC+TR']
    
    # Initialize counters for total statistics
    total_cases = 0
    total_only_ecbs_timeout = 0
    total_only_decbs_timeout = 0
    total_both_timeout = 0
    total_both_no_timeout = 0
    total_missing = 0
    expected_total = 0
    
    # Initialize counters for performance comparison when both don't timeout
    decbs_faster_count = 0
    ecbs_faster_count = 0
    equal_performance_count = 0
    
    # Calculate statistics for each unique combination
    print("\n=== Calculating expected counts ===")
    
    # Get unique combinations that should exist
    unique_combinations = []
    for idx, condition in enumerate(case_conditions):
        case_df = df_pivot[condition]
        if len(case_df) == 0:
            continue
            
        # Get unique values for each parameter
        unique_maps = sorted(case_df['map_path'].unique())
        unique_yamls = sorted(case_df['yaml_path'].unique())
        unique_agents = sorted(case_df['num_agents'].unique())
        unique_suboptimal = sorted(case_df['low_level_suboptimal'].unique())
        
        print(f"Case: {case_names[idx]}")
        print(f"  Maps: {len(unique_maps)}")
        print(f"  YAMLs: {len(unique_yamls)}")
        print(f"  Agents: {len(unique_agents)}, {unique_agents}")
        print(f"  Suboptimal values: {len(unique_suboptimal)}, {unique_suboptimal}")
        
        # For each combination of map, yaml, agent, suboptimal
        for map_path in unique_maps:
            for yaml_path in unique_yamls:
                for agent_count in unique_agents:
                    for suboptimal in unique_suboptimal:
                        # Create a filter for this specific configuration
                        config_filter = (
                            (case_df['map_path'] == map_path) &
                            (case_df['yaml_path'] == yaml_path) &
                            (case_df['num_agents'] == agent_count) &
                            (case_df['low_level_suboptimal'] == suboptimal)
                        )
                        
                        # Get the records matching this configuration
                        config_df = case_df[config_filter]
                        
                        # If we found any records, this combination exists
                        if len(config_df) > 0:
                            # We expect 200 seeds for this combination
                            expected_count = 200
                            expected_total += expected_count
                            
                            # Actual test cases found
                            actual_count = len(config_df)
                            total_cases += actual_count
                            
                            # Missing cases (considered as both timeout)
                            missing_cases = max(0, expected_count - actual_count)
                            total_missing += missing_cases
                            
                            if missing_cases > 0:
                                print(f"  Missing {missing_cases} for Map: {map_path}, YAML: {yaml_path}, Agents: {agent_count}, Suboptimal: {suboptimal}")
                            
                            # Count timeouts
                            only_ecbs_to = sum(config_df['ecbs_timeout'] & ~config_df['decbs_timeout'])
                            only_decbs_to = sum(~config_df['ecbs_timeout'] & config_df['decbs_timeout'])
                            both_to = sum(config_df['ecbs_timeout'] & config_df['decbs_timeout'])
                            
                            # Count cases where both solvers don't timeout
                            both_no_to_df = config_df[~config_df['ecbs_timeout'] & ~config_df['decbs_timeout']]
                            both_no_to = len(both_no_to_df)
                            
                            # For cases where both don't timeout, compare performance
                            if both_no_to > 0:
                                decbs_faster = sum(both_no_to_df['time_decbs'] < both_no_to_df['time_ecbs'])
                                ecbs_faster = sum(both_no_to_df['time_ecbs'] < both_no_to_df['time_decbs'])
                                equal_perf = sum(both_no_to_df['time_decbs'] == both_no_to_df['time_ecbs'])
                                
                                decbs_faster_count += decbs_faster
                                ecbs_faster_count += ecbs_faster
                                equal_performance_count += equal_perf
                            
                            # Add to totals
                            total_only_ecbs_timeout += only_ecbs_to
                            total_only_decbs_timeout += only_decbs_to
                            total_both_timeout += both_to
                            total_both_no_timeout += both_no_to
                            
                            unique_combinations.append({
                                'case': case_names[idx],
                                'map_path': map_path,
                                'yaml_path': yaml_path,
                                'agent_count': agent_count,
                                'suboptimal': suboptimal,
                                'actual_count': actual_count,
                                'missing': missing_cases
                            })
    
    print(f"\nFound {len(unique_combinations)} unique test configurations")
    print(f"Expected total: {expected_total} test cases")
    print(f"Found total: {total_cases} test cases")
    print(f"Missing: {total_missing} test cases\n")
    
    # Calculate adjusted totals
    adjusted_both_timeout = total_both_timeout + total_missing
    adjusted_total = total_cases + total_missing
    
    # Create and return the total summary dictionary
    total_summary = {
        'total_test_cases': total_cases,
        'expected_total': expected_total,
        'only_ecbs_timeout': total_only_ecbs_timeout,
        'only_decbs_timeout': total_only_decbs_timeout,
        'both_timeout': total_both_timeout,
        'both_no_timeout': total_both_no_timeout,
        'missing_cases': total_missing,
        'adjusted_both_timeout': adjusted_both_timeout,
        'adjusted_total': adjusted_total,
        'decbs_faster_count': decbs_faster_count,
        'ecbs_faster_count': ecbs_faster_count,
        'equal_performance_count': equal_performance_count
    }
    
    return total_summary

def print_total_summary_only(data_paths, timeout_threshold=60):
    """
    Analyze and print only the total summary statistics.
    
    Parameters:
    - data_paths: Path to the CSV data file(s)
    - timeout_threshold: Threshold in seconds for timeout
    """
    total_summary = analyze_timeouts_total(data_paths, timeout_threshold)
    
    if total_summary is None:
        print("No statistics available.")
        return
    
    # Print the totals
    print("TOTAL SUMMARY ACROSS ALL CONFIGURATIONS:")
    print("=" * 50)
    print(f"Total test cases found: {total_summary['total_test_cases']}")
    print(f"Expected test cases: {total_summary['expected_total']}")
    print(f"Missing cases: {total_summary['missing_cases']}")
    print()
    print("TIMEOUT STATISTICS:")
    print(f"Only ECBS timeout: {total_summary['only_ecbs_timeout']}")
    print(f"Only DECBS timeout: {total_summary['only_decbs_timeout']}")
    print(f"Both timeout: {total_summary['both_timeout']}")
    print(f"Both no timeout: {total_summary['both_no_timeout']}")
    print(f"Adjusted both timeout (including missing): {total_summary['adjusted_both_timeout']}")
    print()
    
    # Print performance comparison for non-timeout cases
    print("PERFORMANCE COMPARISON (when both algorithms complete):")
    print(f"DECBS faster than ECBS: {total_summary['decbs_faster_count']} cases " + 
          f"({total_summary['decbs_faster_count']/total_summary['both_no_timeout']*100:.2f}%)")
    print(f"ECBS faster than DECBS: {total_summary['ecbs_faster_count']} cases " + 
          f"({total_summary['ecbs_faster_count']/total_summary['both_no_timeout']*100:.2f}%)")
    print(f"Equal performance: {total_summary['equal_performance_count']} cases " +
          f"({total_summary['equal_performance_count']/total_summary['both_no_timeout']*100:.2f}%)")
    print()
    
    # Print percentages
    adj_total = total_summary['adjusted_total']
    if adj_total > 0:
        print("PERCENTAGES (based on adjusted total):")
        print(f"Only ECBS timeout: {total_summary['only_ecbs_timeout']/adj_total*100:.2f}%")
        print(f"Only DECBS timeout: {total_summary['only_decbs_timeout']/adj_total*100:.2f}%")
        print(f"Both timeout (with missing): {total_summary['adjusted_both_timeout']/adj_total*100:.2f}%")
        print(f"Both no timeout: {total_summary['both_no_timeout']/adj_total*100:.2f}%")
    print("=" * 50)

# Example usage
if __name__ == "__main__":
    csv_files = [
        "result/decbs_den_312d_result.csv",
        "result/decbs_warehouse-10-20-10-2-1_result.csv",
    ]
    print_total_summary_only(csv_files)