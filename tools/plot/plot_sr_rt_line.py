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
    """
    if row['solver'] == 'ecbs':
        if not row['op_PC'] and not row['op_BC'] and not row['op_TR']:
            return 'ECBS'
        elif not row['op_PC'] and row['op_BC'] and not row['op_TR']:
            return 'ECBS+BC'
        elif not row['op_PC'] and row['op_BC'] and row['op_TR']:
            return 'ECBS+BC+TR'
    elif row['solver'] == 'decbs':
        if not row['op_PC'] and not row['op_BC'] and not row['op_TR']:
            return 'DECBS'
        elif not row['op_PC'] and row['op_BC'] and not row['op_TR']:
            return 'DECBS+BC'
        elif not row['op_PC'] and row['op_BC'] and row['op_TR']:
            return 'DECBS+BC+TR'
    return "Unknown"

def plot_success_rate(ax, csv_path, subopt_factors, line_styles, store_legend=False, last_row=False):
    """
    Plot success rate data from a CSV file on the given axis.
    """
    sns.set_theme(style="whitegrid", font_scale=1.0)
    sns.set_palette("deep")
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
        
        # Apply the get_full_name function
        df['full_name'] = df.apply(get_full_name, axis=1)
        
        # Define solvers and styles
        solvers = ['ECBS', 'ECBS+BC', 'ECBS+BC+TR', 'DECBS', 'DECBS+BC', 'DECBS+BC+TR']
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
        
        # Plot lines for each suboptimal factor and solver
        for factor in subopt_factors:
            factor_data = df[df['low_level_suboptimal'] == factor]
            if factor_data.empty:
                print(f"Warning: No data for suboptimal factor {factor} in {csv_path}")
                continue
                
            for solver_name in solvers:
                solver_data = factor_data[factor_data['full_name'] == solver_name]
                if not solver_data.empty:
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
        if last_row:
            ax.set_xlabel('Number of agents', fontsize=23)
        ax.set_ylabel('Success rate', fontsize=23)
        ax.grid(True, linestyle='--', alpha=0.3)
        ax.set_ylim(0, 1.05)  # Fixed y-limits to standard success rate range
        
        # Set x-axis range and ticks based on the filename
        if 'maze' in csv_path:
            ax.set_xlim(5, 85)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(10, 85, 20))
        elif 'random' in csv_path:
            ax.set_xlim(40, 155)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(45, 165, 30))
        elif 'den_312' in csv_path:
            ax.set_xlim(100, 215)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(105, 215, 30))
        elif 'warehouse' in csv_path:
            ax.set_xlim(80, 310)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(90, 310, 60))
        elif 'den_520' in csv_path:
            ax.set_xlim(40, 360)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(50, 360, 100))
        elif 'Paris' in csv_path:
            ax.set_xlim(40, 360)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(50, 360, 100))
        else:
            # Default range
            ax.set_xlim(35, 155)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(45, 150, 15))
        
        ax.tick_params(axis='both', which='major', labelsize=23)
        return legend_lines, legend_labels
        
    except Exception as e:
        print(f"Error processing {csv_path}: {str(e)}")
        return [], []

def plot_case(ax, dfs, color_map, labels, last_row=False):
    """
    Plot scatter points for runtime comparison with a single global average per agent.
    """
    # Create a dictionary to collect all data points for each agent
    all_data_by_agent = {}
    
    # First, plot all individual data points for each configuration
    for i, df in enumerate(dfs):
        for agent, color in color_map.items():
            sub = df[df['num_agents'] == agent]
            if not sub.empty:
                # Create a softer version of the color
                softer_color = tuple(c * 0.8 + 0.2 for c in color[:3]) + (0.7,)  # Lighter + some transparency
                
                ax.scatter(sub['time_decbs'], 
                           sub['time_ecbs'],
                           color=softer_color, 
                           s=5, 
                           label=f'{agent} agents {labels[i]}' if i == 0 else None)
                
                # Collect data for global average calculation
                if agent not in all_data_by_agent:
                    all_data_by_agent[agent] = {'time_decbs': [], 'time_ecbs': []}
                
                all_data_by_agent[agent]['time_decbs'].extend(sub['time_decbs'].tolist())
                all_data_by_agent[agent]['time_ecbs'].extend(sub['time_ecbs'].tolist())
    
    # Now calculate and plot the global average for each agent (across all configurations)
    for agent, data in all_data_by_agent.items():
        avg_decbs = sum(data['time_decbs']) / len(data['time_decbs']) if data['time_decbs'] else 0
        avg_ecbs = sum(data['time_ecbs']) / len(data['time_ecbs']) if data['time_ecbs'] else 0
        
        if avg_decbs > 0 and avg_ecbs > 0:  # Only plot if we have valid data
            ax.scatter(avg_decbs, avg_ecbs,
                      color=color_map.get(agent, 'k'),
                      s=150,  # Large size
                      marker='X', 
                      edgecolor='black', 
                      linewidth=2.0,
                      zorder=20)  # Thick outline

    if last_row:
        ax.set_xlabel('DECBS runtime (s)', fontsize=23)
    ax.set_ylabel('ECBS runtime (s)', fontsize=23)
    ax.set_xlim(0, 60)
    ax.set_ylim(0, 60)

    ax.set_xscale('log')
    ax.set_yscale('log')

    # Draw reference lines
    xlims = ax.get_xlim()
    ylims = ax.get_ylim()
    lower = min(xlims[0], ylims[0])
    upper = max(xlims[1], ylims[1])
    
    # Equal runtime (y = x)
    ax.plot([lower, upper], [lower, upper], 'k--', lw=1.5, zorder=1)
    
    # 2x runtime (y = 2x)
    ax.plot([lower, upper], [2*lower, 2*upper], 'k--', lw=2, zorder=1)

    # Reset the limits
    ax.set_xlim(xlims)
    ax.set_ylim(ylims)

    ax.tick_params(axis='both', which='major', labelsize=23)

def plot_time(ax, data_path, last_row = False):
    """
    Process CSV data and create scatter plot for runtime comparison.
    """
    # Load the data
    df = pd.read_csv(data_path)
    df['time(us)'] = pd.to_numeric(df['time(us)'], errors='coerce') / 1_000_000

    # Columns that define a unique experiment setting
    merge_cols = ["map_path", "yaml_path", 
                  "num_agents", "seed",
                  "low_level_suboptimal",
                  "op_PC", "op_BC", "op_TR"]

    # Group and pivot data
    df_grouped = df.groupby(merge_cols + ['solver'], as_index=False)['time(us)'].first()
    df_pivot = df_grouped.pivot(index=merge_cols, columns='solver', values='time(us)').reset_index()
    df_pivot = df_pivot.dropna(subset=['decbs','ecbs'], how='any')
    df_pivot = df_pivot.fillna(60)  # Fill missing values
    df_pivot = df_pivot.rename(columns={'decbs': 'time_decbs', 'ecbs': 'time_ecbs'})

    # Define the different configuration cases
    df1 = df_pivot[(df_pivot['op_PC'] == False) & 
                   (df_pivot['op_BC'] == False) & 
                   (df_pivot['op_TR'] == False)]
    df2 = df_pivot[(df_pivot['op_PC'] == False) & 
                   (df_pivot['op_BC'] == True) & 
                   (df_pivot['op_TR'] == False)]
    df3 = df_pivot[(df_pivot['op_PC'] == False) & 
                   (df_pivot['op_BC'] == True) & 
                   (df_pivot['op_TR'] == True)]

    # Create a color mapping for num_agents
    unique_agents = sorted(df_pivot['num_agents'].unique())
    colors = plt.cm.jet(np.linspace(0, 1, len(unique_agents)))
    color_map = {agent: color for agent, color in zip(unique_agents, colors)}

    # Plot the data
    config_labels = ['', 'BC', 'BC+TR']
    if 'maze' in data_path:
        plot_case(ax, [df3], color_map, config_labels, last_row=last_row)
    else:
        plot_case(ax, [df2, df3], color_map, config_labels, last_row=last_row)
    
    # Create legend handles for agent numbers
    legend_handles = []
    legend_labels = []
    
    for agent in unique_agents:
        softer_color = tuple(c * 0.8 + 0.2 for c in color_map[agent][:3]) + (0.7,)
        handle = Line2D([0], [0], marker='o', color='w', 
                        markerfacecolor=softer_color, markersize=6)
        legend_handles.append(handle)
        legend_labels.append(f'{agent}')
    
    # Place the agent legend at the upper left of the plot
    agent_legend = ax.legend(legend_handles, legend_labels, 
                        loc='upper left',
                        fontsize=14,
                        framealpha=0.7,
                        ncol=2,
                        handletextpad=0.1,  
                        columnspacing=0.3,)
    ax.add_artist(agent_legend)
    
    # Add ratio legend at bottom right
    ratio_handles = [
        Line2D([0], [0], linestyle='--', color='k', lw=1.5),
        Line2D([0], [0], linestyle='--', color='k', lw=2)
    ]
    ratio_labels = ['1x', '2x']
    ratio_legend = ax.legend(ratio_handles, ratio_labels, 
                         loc='lower right', 
                         fontsize=16)
    ax.add_artist(ratio_legend)
    
    return unique_agents, color_map

def create_legend(fig, row_idx=0, color_map=None):
    """
    Create a combined legend for a specific row, including the Average marker.
    
    Parameters:
    fig - The figure to add the legend to
    row_idx - Row index (0 for top row, 1 for bottom row)
    color_map - Color mapping for agents (to include Average marker)
    
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
    elif row_idx == 1:
        # Second row: 1.01, 1.05, 1.1
        subopt_handles = [
            Line2D([0], [0], color='gray', linestyle=':', linewidth=2, label='1.01'),
            Line2D([0], [0], color='gray', linestyle='--', linewidth=2, label='1.05'),
            Line2D([0], [0], color='gray', linestyle='-', linewidth=2, label='1.10')
        ]
    elif row_idx == 2:
        # Third row: 1.002, 1.018, 1.034
        subopt_handles = [
            Line2D([0], [0], color='gray', linestyle=':', linewidth=2, label='1.002'),
            Line2D([0], [0], color='gray', linestyle='--', linewidth=2, label='1.018'),
            Line2D([0], [0], color='gray', linestyle='-', linewidth=2, label='1.038')
        ]
    
    # Add the Average marker if color_map is provided
    if color_map is not None:
        # Add average marker
        avg_handle = Line2D([0], [0], marker='X', color='w', 
                           markersize=8, markeredgecolor='gray', markeredgewidth=1.5, label='Average')
        solver_handles.append(avg_handle)
    
    # Combine all handles
    all_handles = solver_handles + subopt_handles
    all_labels = [h.get_label() for h in all_handles]
    
    # Add a single combined legend for the row
    bbox_anchors = {0: (0.5, 0.95), 1: (0.5, 0.66), 2: (0.5, 0.375)}
    
    legend = fig.legend(handles=all_handles, 
                 labels=all_labels,
                 loc='upper center', 
                 bbox_to_anchor=bbox_anchors[row_idx],
                 ncol=len(all_handles) / 2, 
                 fontsize=23,
                 frameon=True)
    
    return legend

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Create success rate and runtime plots from CSV files')
    parser.add_argument('--output_path', type=str, default='fig/combined_plots.png',
                        help='Path to save the output figure')
    
    args = parser.parse_args()
    
    # File paths
    map_files = {
        'random': {
            'stat': 'result/decbs_random-32-32-20_stat.csv',
            'time': 'result/decbs_random-32-32-20_result.csv'
        },
        'maze': {
            'stat': 'result/decbs_maze-32-32-2_stat.csv',
            'time': 'result/decbs_maze-32-32-2_result.csv'
        },
        'den_312': {
            'stat': 'result/decbs_den_312d_stat.csv',
            'time': 'result/decbs_den_312d_result.csv'
        },
        'warehouse': {
            'stat': 'result/decbs_warehouse-10-20-10-2-1_stat.csv',
            'time': 'result/decbs_warehouse-10-20-10-2-1_result.csv'
        },
        'den_520': {
            'stat': 'result/decbs_den_520d_stat.csv',
            'time': 'result/decbs_den_520d_result.csv'
        },
        'Paris': {
            'stat': 'result/decbs_Paris_1_256_stat.csv',
            'time': 'result/decbs_Paris_1_256_result.csv'
        }
    }
    
    # Create a figure with 3 rows and 4 columns
    fig, axes = plt.subplots(3, 4, figsize=(28, 18))

    # Define line styles for different suboptimality factors
    line_styles1 = {1.02: ':', 1.1: '--', 1.2: '-'}
    line_styles2 = {1.01: ':', 1.05: '--', 1.1: '-'}
    line_styles3 = {1.002: ':', 1.018: '--', 1.034: '-'}
    
    # Store color maps for each row to use in the row legends
    color_maps = {0: None, 1: None, 2: None}
    
    # Create all plots according to the specified layout
    # Row 0
    plot_success_rate(axes[0, 0], map_files['random']['stat'], [1.02, 1.1, 1.2], line_styles1)
    agents, color_map = plot_time(axes[0, 1], map_files['random']['time'])
    color_maps[0] = color_map
    plot_success_rate(axes[0, 2], map_files['maze']['stat'], [1.02, 1.1, 1.2], line_styles1)
    plot_time(axes[0, 3], map_files['maze']['time'])
    
    # Row 1
    plot_success_rate(axes[1, 0], map_files['den_312']['stat'], [1.01, 1.05, 1.1], line_styles2)
    agents, color_map = plot_time(axes[1, 1], map_files['den_312']['time'])
    color_maps[1] = color_map
    plot_success_rate(axes[1, 2], map_files['warehouse']['stat'], [1.01, 1.05, 1.1], line_styles2)
    plot_time(axes[1, 3], map_files['warehouse']['time'])

    # Row 2
    plot_success_rate(axes[2, 0], map_files['den_520']['stat'], [1.002, 1.018, 1.034], line_styles3, last_row=True)
    agents, color_map = plot_time(axes[2, 1], map_files['den_520']['time'], last_row=True)
    color_maps[2] = color_map
    plot_success_rate(axes[2, 2], map_files['Paris']['stat'], [1.002, 1.018, 1.034], line_styles3, last_row=True)
    plot_time(axes[2, 3], map_files['Paris']['time'], last_row=True)
    
    # Set titles for each subplot
    map_titles = {
        'random': 'random-32-32-20',
        'maze': 'maze-32-32-20-2',
        'den_312': 'den312d',
        'warehouse': 'warehouse-10-20-10-2-1',
        'den_512': 'den520d',
        'Paris': 'Paris_1_256'
    }
    
    # Apply titles to all subplots
    for col, map_key in enumerate(['random', 'random', 'maze', 'maze']):
        axes[0, col].set_title(f"{map_titles[map_key]}", fontsize=23)
    
    for col, map_key in enumerate(['den_312', 'den_312', 'warehouse', 'warehouse']):
        axes[1, col].set_title(f"{map_titles[map_key]}", fontsize=23)

    for col, map_key in enumerate(['den_512', 'den_512', 'Paris', 'Paris']):
        axes[2, col].set_title(f"{map_titles[map_key]}", fontsize=23)
    
    # Create separate legends for each row, including the Average marker
    legend1 = create_legend(fig, row_idx=0, color_map=color_maps[0])
    legend2 = create_legend(fig, row_idx=1, color_map=color_maps[1])
    legend3 = create_legend(fig, row_idx=2, color_map=color_maps[2])
    
    # Adjust subplot spacing
    plt.subplots_adjust(hspace=0.7, wspace=0.31, top=0.85)
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(args.output_path), exist_ok=True)
    
    # Save the figure
    plt.savefig(args.output_path, dpi=300, bbox_inches='tight')
    print(f"Figure saved to {args.output_path}")