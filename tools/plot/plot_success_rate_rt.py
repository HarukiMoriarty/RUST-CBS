import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse
import seaborn as sns
import os
from matplotlib.lines import Line2D

def get_full_name(row):
    """
    Get the full solver name based on the solver type and optimization options.
    Returns a default value if no matching configuration is found.
    """
    result = "Unknown"
    
    if row['solver'] == 'ecbs':
        if not row['op_PC'] and not row['op_BC'] and not row['op_TR']:
            result = 'ECBS'
        elif not row['op_PC'] and row['op_BC'] and not row['op_TR']:
            result = 'ECBS+BC'
        elif not row['op_PC'] and row['op_BC'] and row['op_TR']:
            result = 'ECBS+BC+TR'
        # Add other combinations if needed
    elif row['solver'] == 'decbs':
        if not row['op_PC'] and not row['op_BC'] and not row['op_TR']:
            result = 'DECBS'
        elif not row['op_PC'] and row['op_BC'] and not row['op_TR']:
            result = 'DECBS+BC'
        elif not row['op_PC'] and row['op_BC'] and row['op_TR']:
            result = 'DECBS+BC+TR'
        # Add other combinations if needed
    
    return result

def plot_success_rate(ax, csv_path, subopt_factors, line_styles, store_legend=False):
    """
    Plot success rate data from a CSV file on the given axis.
    Returns legend lines and labels if store_legend is True.
    """
    sns.set_theme(style="whitegrid", font_scale=1.0)
    sns.set_palette("deep")
    
    # Print the CSV filename being processed for debugging
    print(f"Processing success rate file: {csv_path}")

    try:
        # Read the CSV file
        df = pd.read_csv(csv_path)
        
        # Check if required columns exist
        required_columns = ['solver', 'op_PC', 'op_BC', 'op_TR', 'success_rate', 'num_agents', 'low_level_suboptimal']
        missing_columns = [col for col in required_columns if col not in df.columns]
        if missing_columns:
            print(f"Warning: Missing columns in {csv_path}: {missing_columns}")
            return [], []
        
        # Normalize success_rate only if it's not already normalized
        if df['success_rate'].max() > 1.0:
            df['success_rate'] = df['success_rate'] / 100.0
        
        # Apply the get_full_name function and handle any errors
        df['full_name'] = df.apply(get_full_name, axis=1)
        
        # Check if any rows have "Unknown" as the full_name
        unknown_rows = df[df['full_name'] == 'Unknown']
        if not unknown_rows.empty:
            print(f"Warning: {len(unknown_rows)} rows have unrecognized solver configurations in {csv_path}")
            print("Sample of unrecognized configurations:")
            print(unknown_rows[['solver', 'op_PC', 'op_BC', 'op_TR']].head())
        
        # Define suboptimal factors and solver names
        solvers = ['ECBS', 'ECBS+BC', 'ECBS+BC+TR', 'DECBS', 'DECBS+BC', 'DECBS+BC+TR']
        
        # Define colors, line styles, and markers
        colors = sns.color_palette("deep")
        opt_colors = {
            'DECBS': colors[0],
            'DECBS+BC': colors[1],
            'DECBS+BC+TR': colors[2],
            'ECBS': colors[3],
            'ECBS+BC': colors[4],
            'ECBS+BC+TR': colors[5]
        }
        markers = {
            'DECBS': 'o',
            'DECBS+BC': 's',
            'DECBS+BC+TR': 'D',
            'ECBS': 'o',
            'ECBS+BC': 's',
            'ECBS+BC+TR': 'D'
        }
        
        legend_lines = []
        legend_labels = []
        
        # Debug: Print unique values of important columns
        print(f"Unique suboptimal factors in {csv_path}: {df['low_level_suboptimal'].unique()}")
        print(f"Unique solvers in {csv_path}: {df['solver'].unique()}")
        print(f"Unique agent counts in {csv_path}: {sorted(df['num_agents'].unique())}")
        
        # Plot lines for each suboptimal factor and solver
        for factor in subopt_factors:
            factor_data = df[df['low_level_suboptimal'] == factor]
            if factor_data.empty:
                print(f"Warning: No data for suboptimal factor {factor} in {csv_path}")
                continue
                
            for solver_name in solvers:
                solver_data = factor_data[factor_data['full_name'] == solver_name]
                if not solver_data.empty:
                    if len(solver_data['num_agents']) < 2:
                        print(f"Warning: Only {len(solver_data)} data points for {solver_name} ({factor}) in {csv_path}")
                    
                    # Sort the data by num_agents to ensure proper line connections
                    solver_data = solver_data.sort_values(by='num_agents')
                    
                    line, = ax.plot(
                        solver_data['num_agents'], 
                        solver_data['success_rate'],
                        linestyle=line_styles[factor],
                        marker=markers[solver_name],
                        color=opt_colors[solver_name],
                        markerfacecolor='white',
                        markersize=6,
                        linewidth=2,
                        label=f'{solver_name} ({factor})'
                    )
                    if store_legend:
                        legend_lines.append(line)
                        legend_labels.append(f'{solver_name} ({factor})')
        
        # Customize the axis
        ax.set_xlabel('Number of agents', fontsize=12)
        ax.set_ylabel('Success rate', fontsize=12)
        ax.grid(True, linestyle='--', alpha=0.3)
        ax.set_ylim(0, 1.05)  # Fixed y-limits to standard success rate range
        
        # Set x-axis range and ticks based on the filename
        if 'empty' in csv_path:
            ax.set_xlim(110, 270)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(120, 280, 20))
        elif 'random' in csv_path:
            ax.set_xlim(40, 155)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(45, 165, 15))
        elif 'den_312' in csv_path:
            ax.set_xlim(100, 215)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(105, 215, 15))
        elif 'warehouse' in csv_path:
            ax.set_xlim(80, 310)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(90, 300, 30))
        else:
            # Default range
            ax.set_xlim(35, 155)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(45, 150, 15))
            
        ax.tick_params(axis='both', which='major', labelsize=12)
        
        return legend_lines, legend_labels
        
    except Exception as e:
        print(f"Error processing {csv_path}: {str(e)}")
        return [], []

def plot_avg_time(ax, csv_path, store_legend=False):
    """
    Plot average time data from a CSV file on the given axis.
    X-axis is low_level_suboptimal, Y-axis is avg_time.
    Returns legend lines and labels if store_legend is True.
    """
    sns.set_theme(style="whitegrid", font_scale=1.0)
    sns.set_palette("deep")
    
    # Print the CSV filename being processed for debugging
    print(f"Processing average time file: {csv_path}")

    try:
        # Read the CSV file
        df = pd.read_csv(csv_path)
        
        # Check if required columns exist
        required_columns = ['solver', 'op_PC', 'op_BC', 'op_TR', 'avg_time', 'low_level_suboptimal']
        missing_columns = [col for col in required_columns if col not in df.columns]
        if missing_columns:
            print(f"Warning: Missing columns in {csv_path}: {missing_columns}")
            return [], []
        
        # Apply the get_full_name function and handle any errors
        df['full_name'] = df.apply(get_full_name, axis=1)
        
        # Check if any rows have "Unknown" as the full_name
        unknown_rows = df[df['full_name'] == 'Unknown']
        if not unknown_rows.empty:
            print(f"Warning: {len(unknown_rows)} rows have unrecognized solver configurations in {csv_path}")
            print("Sample of unrecognized configurations:")
            print(unknown_rows[['solver', 'op_PC', 'op_BC', 'op_TR']].head())
        
        # Define solver names
        solvers = ['ECBS', 'ECBS+BC', 'ECBS+BC+TR', 'DECBS', 'DECBS+BC', 'DECBS+BC+TR']
        
        # Define colors and markers
        colors = sns.color_palette("deep")
        opt_colors = {
            'DECBS': colors[0],
            'DECBS+BC': colors[1],
            'DECBS+BC+TR': colors[2],
            'ECBS': colors[3],
            'ECBS+BC': colors[4],
            'ECBS+BC+TR': colors[5]
        }
        markers = {
            'DECBS': 'o',
            'DECBS+BC': 's',
            'DECBS+BC+TR': 'D',
            'ECBS': 'o',
            'ECBS+BC': 's',
            'ECBS+BC+TR': 'D'
        }
        
        legend_lines = []
        legend_labels = []
        
        # Debug: Print unique values of important columns
        print(f"Unique low level suboptimal factors in {csv_path}: {df['low_level_suboptimal'].unique()}")
        print(f"Unique solvers in {csv_path}: {df['solver'].unique()}")
        
        # Plot lines for each solver
        for solver_name in solvers:
            solver_data = df[df['full_name'] == solver_name]
            if not solver_data.empty:
                if len(solver_data) < 2:
                    print(f"Warning: Only {len(solver_data)} data points for {solver_name} in {csv_path}")
                
                # Sort the data by low_level_suboptimal to ensure proper line connections
                solver_data = solver_data.sort_values(by='low_level_suboptimal')
                
                line, = ax.plot(
                    solver_data['low_level_suboptimal'], 
                    solver_data['avg_time'],
                    marker=markers[solver_name],
                    color=opt_colors[solver_name],
                    markerfacecolor='white',
                    markersize=6,
                    linewidth=2,
                    label=solver_name
                )
                if store_legend:
                    legend_lines.append(line)
                    legend_labels.append(solver_name)
        
        # Customize the axis
        ax.set_xlabel('Suboptimality factor', fontsize=12)
        ax.set_ylabel('Average time (s)', fontsize=12)
        ax.grid(True, linestyle='--', alpha=0.3)
        
        # Set appropriate y-limits based on data
        if not df.empty and 'avg_time' in df.columns:
            max_time = df['avg_time'].max()
            ax.set_ylim(0, max_time * 1.1)  # Add 10% padding
            if 'empty' in csv_path:
                ax.set_xlim(1, 1.22)
                ax.set_xticks(np.arange(1.02, 1.2, 0.02))
            elif 'random' in csv_path:
                ax.set_xlim(1, 1.22)
                ax.set_xticks(np.arange(1.02, 1.2, 0.02))
            elif 'den_312' in csv_path:
                ax.set_xlim(1, 1.11)
                ax.set_xticks(np.arange(1.01, 1.1, 0.01))
            elif 'warehouse' in csv_path:
                ax.set_xlim(1, 1.11)
                ax.set_xticks(np.arange(1.01, 1.1, 0.01))
            else:
                # Default range
                ax.set_xlim(1, 1.22)
                ax.set_xticks(np.arange(1.02, 1.2, 0.02))
        
        ax.tick_params(axis='both', which='major', labelsize=12)
        
        return legend_lines, legend_labels
        
    except Exception as e:
        print(f"Error processing {csv_path}: {str(e)}")
        return [], []

def create_legend(fig, row_idx=0):
    """
    Create a combined legend for a specific row index with both solvers and suboptimality factors
    
    Parameters:
    fig - The figure to add the legend to
    row_idx - Row index (0 for top row, 1 for bottom row)
    
    Returns:
    legend - The created legend object
    """
    colors = sns.color_palette("deep")
    # Create custom handles for solvers
    solver_handles = [
        Line2D([0], [0], color=colors[3], marker='o', markerfacecolor='white', markersize=6, label='ECBS'),
        Line2D([0], [0], color=colors[4], marker='s', markerfacecolor='white', markersize=6, label='ECBS+BC'),
        Line2D([0], [0], color=colors[5], marker='D', markerfacecolor='white', markersize=6, label='ECBS+BC+TR'),
        Line2D([0], [0], color=colors[0], marker='o', markerfacecolor='white', markersize=6, label='DECBS'),
        Line2D([0], [0], color=colors[1], marker='s', markerfacecolor='white', markersize=6, label='DECBS+BC'),
        Line2D([0], [0], color=colors[2], marker='D', markerfacecolor='white', markersize=6, label='DECBS+BC+TR')
    ]
    
    # Create custom handles for suboptimality factors based on row
    if row_idx == 0:
        # First row: 1.02, 1.1, 1.2
        subopt_handles = [
            Line2D([0], [0], color='gray', linestyle=':', linewidth=2, label='1.02'),
            Line2D([0], [0], color='gray', linestyle='--', linewidth=2, label='1.10'),
            Line2D([0], [0], color='gray', linestyle='-', linewidth=2, label='1.20')
        ]
    else:
        # Second row: 1.01, 1.05, 1.1
        subopt_handles = [
            Line2D([0], [0], color='gray', linestyle=':', linewidth=2, label='1.01'),
            Line2D([0], [0], color='gray', linestyle='--', linewidth=2, label='1.05'),
            Line2D([0], [0], color='gray', linestyle='-', linewidth=2, label='1.10')
        ]
    
    # Combine all handles
    all_handles = solver_handles + subopt_handles
    all_labels = [h.get_label() for h in all_handles]
    
    # Add a single combined legend for the row
    if row_idx == 0:
        legend = fig.legend(handles=all_handles, 
                     labels=all_labels,
                     loc='upper center', 
                     bbox_to_anchor=(0.5, 0.92),
                     ncol=len(all_handles), 
                     fontsize=10, 
                     frameon=True)
    elif row_idx == 1:
        legend = fig.legend(handles=all_handles, 
                     labels=all_labels,
                     loc='upper center', 
                     bbox_to_anchor=(0.5, 0.49),
                     ncol=len(all_handles), 
                     fontsize=10, 
                     frameon=True)
    
    
    return legend

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Create success rate and average time plots from CSV files')
    parser.add_argument('--output_path', type=str, default='fig/combined_plots.png',
                        help='Path to save the output figure')
    
    args = parser.parse_args()
    
    # File paths
    map_files = {
        'random': {
            'stat': 'result/random-32-32-20_stat.csv',
            'time': 'result/random-32-32-20_time.csv'
        },
        'empty': {
            'stat': 'result/empty-32-32-20_stat.csv',
            'time': 'result/empty-32-32-20_time.csv'
        },
        'den_312': {
            'stat': 'result/den_312d_stat.csv',
            'time': 'result/den_312d_time.csv'
        },
        'warehouse': {
            'stat': 'result/warehouse-10-20-10-2-1_stat.csv',
            'time': 'result/warehouse-10-20-10-2-1_time.csv'
        }
    }
    
    # Create a figure with 2 rows and 4 columns
    fig, axes = plt.subplots(2, 4, figsize=(30, 10))

    line_styles1 = {
            1.02: ':',
            1.1: '--',
            1.2: '-'
        }

    line_styles2 = {
            1.01: ':',
            1.05: '--',
            1.1: '-'
        }
    
    # Create all plots according to the specified layout
    # Row 0
    plot_success_rate(axes[0, 0], map_files['random']['stat'], [1.02, 1.1, 1.2], line_styles1)
    plot_avg_time(axes[0, 1], map_files['random']['time'])
    plot_success_rate(axes[0, 2], map_files['empty']['stat'], [1.02, 1.1, 1.2], line_styles1)
    plot_avg_time(axes[0, 3], map_files['empty']['time'])
    
    # Row 1
    plot_success_rate(axes[1, 0], map_files['den_312']['stat'], [1.01, 1.05, 1.1], line_styles2)
    plot_avg_time(axes[1, 1], map_files['den_312']['time'])
    plot_success_rate(axes[1, 2], map_files['warehouse']['stat'], [1.01, 1.05, 1.1], line_styles2)
    plot_avg_time(axes[1, 3], map_files['warehouse']['time'])
    
    # Set titles for each subplot
    map_titles = {
        'random': 'Random 32x32',
        'empty': 'Empty 32x32',
        'den_312': 'Den 312d',
        'warehouse': 'Warehouse'
    }
    
    # Set titles
    axes[0, 0].set_title(f"{map_titles['random']}", fontsize=14)
    axes[0, 1].set_title(f"{map_titles['random']}", fontsize=14)
    axes[0, 2].set_title(f"{map_titles['empty']}", fontsize=14)
    axes[0, 3].set_title(f"{map_titles['empty']}", fontsize=14)
    
    axes[1, 0].set_title(f"{map_titles['den_312']}", fontsize=14)
    axes[1, 1].set_title(f"{map_titles['den_312']}", fontsize=14)
    axes[1, 2].set_title(f"{map_titles['warehouse']}", fontsize=14)
    axes[1, 3].set_title(f"{map_titles['warehouse']}", fontsize=14)
    
    # Create separate legends for each row
    legend1 = create_legend(fig, row_idx=0)
    legend2 = create_legend(fig, row_idx=1)
    
    # Adjust spacing between plots
    plt.subplots_adjust(hspace=0.4, wspace=0.3, top=0.85)
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(args.output_path), exist_ok=True)
    
    # Save the figure
    plt.savefig(args.output_path, dpi=300, bbox_inches='tight')
    print(f"Figure saved to {args.output_path}")