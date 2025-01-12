#!/usr/bin/env python3
"""
Analysis script for CBS (Conflict-Based Search) experiment results.
Processes experiment data and generates statistical analysis.
"""

import pandas as pd
import numpy as np
import argparse
import logging
from typing import Tuple, List

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(message)s')
LOG = logging.getLogger(__name__)

# Constants
MAX_INT = np.iinfo(np.int64).max
TIMEOUT_VALUE = 'timeout'
REQUIRED_COLUMNS = [
    'num_agents', 'seed', 'op_PC', 'op_BC', 'op_TR',
    'solver', 'costs', 'time(us)', 'high_level_expanded',
    'low_level_open_expanded', 'low_level_focal_expanded',
    'low_level_mdd_open_expanded', 'low_level_mdd_focal_expanded',
    'total_low_level_expanded'
]

def load_and_clean_data(file_path: str) -> pd.DataFrame:
    """
    Load CSV data and clean timeout entries.
    
    Args:
        file_path: Path to the input CSV file
    
    Returns:
        Cleaned DataFrame with proper timeout handling
    """
    try:
        data = pd.read_csv(file_path)
        
        # Verify required columns exist
        missing_cols = [col for col in REQUIRED_COLUMNS if col not in data.columns]
        if missing_cols:
            raise ValueError(f"Missing required columns: {missing_cols}")
        
        # Create a mask for timeout entries
        timeout_mask = data['costs'].astype(str).str.contains(TIMEOUT_VALUE, na=False)
        
        # Convert costs column to numeric, forcing timeout entries to MAX_INT
        data['costs'] = pd.to_numeric(data['costs'], errors='coerce')
        data.loc[timeout_mask, 'costs'] = MAX_INT
        
        # Set other metrics to MAX_INT for timeout cases
        timeout_columns = [
            'time(us)', 'high_level_expanded', 'low_level_open_expanded',
            'low_level_focal_expanded', 'low_level_mdd_open_expanded', 'low_level_mdd_focal_expanded', 'total_low_level_expanded'
        ]
        data.loc[timeout_mask, timeout_columns] = MAX_INT
        
        return data
        
    except Exception as e:
        LOG.error(f"Error loading data: {str(e)}")
        raise

def compute_stats(data: pd.Series) -> Tuple[float, float, float]:
    """
    Compute percentile statistics for a series of data.
    
    Args:
        data: Series of numerical data
        
    Returns:
        Tuple of (0th, 50th, 99th) percentiles
    """
    if data.empty:
        return np.nan, np.nan, np.nan
    return tuple(np.percentile(data.dropna(), [0, 50, 99], method="nearest"))

def check_solver_costs(data: pd.DataFrame) -> None:
    """
    Check if any solver produces lower costs than CBS for the same configuration.
    
    Args:
        data: DataFrame containing experiment results
    """
    # Group by all parameters except solver
    group_params = ['num_agents', 'seed', 'op_PC', 'op_BC', 'op_TR']
    
    for _, group in data.groupby(group_params):
        # Get CBS data for this configuration
        cbs_data = group[group['solver'] == 'cbs']
        
        # Skip if no CBS data or CBS timed out
        if cbs_data.empty or (cbs_data['costs'] == MAX_INT).all():
            continue

        # Skip timeouts
        success_runs = cbs_data['costs'] != MAX_INT

        # Check if CBS produces consistent costs across runs with same configuration
        if not (success_runs['costs'] == success_runs['costs'].iloc[0]).all():
            config = dict(zip(group_params, group[group_params].iloc[0]))
            costs = success_runs['costs'].unique()
            LOG.warning(f"CBS cost inconsistency found - Configuration: {config}, Costs: {costs}")
            
        cbs_min_cost = cbs_data['costs'].min()
        
        # Check other solvers
        for solver in group['solver'].unique():
            if solver == 'cbs':
                continue
                
            solver_data = group[group['solver'] == solver]
            if solver_data.empty:
                continue
                
            # Check if any solution has lower cost than CBS
            if (solver_data['costs'] < cbs_min_cost).any():
                config = dict(zip(group_params, group[group_params].iloc[0]))
                LOG.warning(f"Cost discrepancy found - Solver: {solver}, Configuration: {config}")

def calculate_solver_stats(data: pd.DataFrame) -> pd.DataFrame:
    """
    Calculate statistics for each solver configuration.
    
    Args:
        data: DataFrame containing experiment results
        
    Returns:
        DataFrame containing computed statistics
    """
    results = []
    group_cols = ['solver', 'num_agents', 'op_PC', 'op_BC', 'op_TR']
    
    for group_key, group_data in data.groupby(group_cols):
        # Calculate timeout rate
        timeouts = group_data['time(us)'] == MAX_INT
        timeout_rate = timeouts.sum() / len(group_data)
        
        # Get successful runs
        success_data = group_data[~timeouts]
        
        # Calculate statistics for successful runs
        metric_stats = {
            'time': compute_stats(success_data['time(us)']),
            'high': compute_stats(success_data['high_level_expanded']),
            'lowOpen': compute_stats(success_data['low_level_open_expanded']),
            'lowFocal': compute_stats(success_data['low_level_focal_expanded']),
            'lowOpenMdd': compute_stats(success_data['low_level_mdd_open_expanded']),
            'lowFocalMdd': compute_stats(success_data['low_level_mdd_focal_expanded']),
            'lowTotal': compute_stats(success_data['total_low_level_expanded'])
        }
        
        # Create result dictionary
        result = dict(zip(group_cols, group_key))
        result['success_rate'] = (1 - timeout_rate) * 100
        
        # Add statistics to result
        for metric, (p0, p50, p99) in metric_stats.items():
            result.update({
                f'P0{metric}': p0,
                f'P50{metric}': p50,
                f'P99{metric}': p99
            })
            
        results.append(result)
    
    return pd.DataFrame(results)

def analyze_experiments(input_path: str, output_path: str) -> None:
    """
    Main analysis function that processes experiment data and saves results.
    
    Args:
        input_path: Path to input CSV file
        output_path: Path to save output CSV file
    """
    try:
        LOG.info(f"Loading data from {input_path}")
        data = load_and_clean_data(input_path)
        
        LOG.info("Checking solver costs against CBS")
        check_solver_costs(data)
        
        LOG.info("Calculating solver statistics")
        results_df = calculate_solver_stats(data)
        
        LOG.info(f"Saving results to {output_path}")
        results_df.to_csv(output_path, index=False)
        
        LOG.info("Analysis completed successfully")
        
    except Exception as e:
        LOG.error(f"Analysis failed: {str(e)}")
        raise

def main():
    """Main entry point for the analysis script."""
    parser = argparse.ArgumentParser(
        description="Analyze CBS experiment results and generate statistics."
    )
    parser.add_argument(
        "file_path",
        help="Path to the input CSV file containing experiment data"
    )
    parser.add_argument(
        "output_file_path",
        help="Path to save the analysis results CSV file"
    )
    
    args = parser.parse_args()
    analyze_experiments(args.file_path, args.output_file_path)

if __name__ == "__main__":
    main()