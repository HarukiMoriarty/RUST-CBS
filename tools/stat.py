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
    'solver', 'high_level_suboptimal', 'low_level_suboptimal',
    'costs', 'time(us)', 'high_level_expanded',
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
        data = pd.read_csv(file_path, keep_default_na=False)
        
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
            'low_level_focal_expanded', 'low_level_mdd_open_expanded', 
            'low_level_mdd_focal_expanded', 'total_low_level_expanded'
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
    Check if:
    1. CBS produces consistent optimal costs across all configurations for same (num_agents, seed)
    2. No other solver produces lower costs than CBS
    
    Args:
        data: DataFrame containing experiment results
    """
    # First check CBS consistency across all configurations
    for _, group in data.groupby(['num_agents', 'seed']):
        # Get all CBS results for this num_agents and seed
        cbs_data = group[group['solver'] == 'cbs']
        
        # Skip if no CBS data or all CBS runs timed out
        if cbs_data.empty or (cbs_data['costs'] == MAX_INT).all():
            continue
            
        # Filter successful CBS runs
        cbs_success_data = cbs_data[cbs_data['costs'] != MAX_INT]
        
        if len(cbs_success_data['costs'].unique()) > 1:
            costs = cbs_success_data['costs'].unique()
            configs = cbs_success_data[['op_PC', 'op_BC', 'op_TR']].to_dict('records')
            LOG.warning(
                f"CBS cost inconsistency found for num_agents={group['num_agents'].iloc[0]}, "
                f"seed={group['seed'].iloc[0]}\n"
                f"Costs: {costs}\n"
                f"Configurations: {configs}"
            )
        
        # Get the true optimal cost for this problem instance
        optimal_cost = cbs_success_data['costs'].min()
        
        # Check if any other solver produces lower costs
        other_solvers = group[group['solver'] != 'cbs']
        for _, solver_data in other_solvers.iterrows():
            if solver_data['costs'] < optimal_cost:
                LOG.warning(
                    f"Cost discrepancy found:\n"
                    f"Solver: {solver_data['solver']}\n"
                    f"num_agents={solver_data['num_agents']}, seed={solver_data['seed']}\n"
                    f"Solver cost: {solver_data['costs']}, CBS optimal cost: {optimal_cost}\n"
                    f"Configuration: {solver_data[['op_PC', 'op_BC', 'op_TR', 'high_level_suboptimal', 'low_level_suboptimal']].to_dict()}"
        )

def calculate_solver_stats(data: pd.DataFrame) -> pd.DataFrame:
    """
    Calculate statistics for each solver configuration.
    
    Args:
        data: DataFrame containing experiment results
        
    Returns:
        DataFrame containing computed statistics
    """
    results = []
    group_cols = ['solver', 'num_agents', 'op_PC', 'op_BC', 'op_TR',
                  'high_level_suboptimal', 'low_level_suboptimal']
    
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