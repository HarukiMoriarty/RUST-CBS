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

def plot_success_rate(ax, csv_path, subopt_factors, line_styles, store_legend=False):
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
        ax.set_xlabel('Number of agents', fontsize=18)
        ax.set_ylabel('Success rate', fontsize=18)
        ax.grid(True, linestyle='--', alpha=0.3)
        ax.set_ylim(0, 1.05)  # Fixed y-limits to standard success rate range
        
        # Set x-axis range and ticks based on the filename
        if 'empty' in csv_path:
            ax.set_xlim(140, 340)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(150, 340, 20))
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
            ax.set_xticks(np.arange(90, 310, 30))
        elif 'den_520' in csv_path:
            ax.set_xlim(40, 360)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(50, 360, 50))
        elif 'Paris' in csv_path:
            ax.set_xlim(40, 360)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(50, 360, 50))
        else:
            # Default range
            ax.set_xlim(35, 155)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(45, 150, 15))
        
        ax.tick_params(axis='both', which='major', labelsize=16)
        return legend_lines, legend_labels
        
    except Exception as e:
        print(f"Error processing {csv_path}: {str(e)}")
        return [], []

def plot_case(ax, dfs, color_map, labels):
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
                      linewidth=2.0)  # Thick outline

    ax.set_xlabel('DECBS runtime (s)', fontsize=18)
    ax.set_ylabel('ECBS runtime (s)', fontsize=18)
    ax.set_xlim(0, 60)
    ax.set_ylim(0, 60)

    ax.set_xscale('log')
    ax.set_yscale('log')

    # Draw reference lines (but don't add legends for them)
    xlims = ax.get_xlim()
    ylims = ax.get_ylim()
    lower = min(xlims[0], ylims[0])
    upper = max(xlims[1], ylims[1])
    
    # Equal runtime (y = x)
    ax.plot([lower, upper], [lower, upper], 'k--', lw=1.5)
    
    # 2x runtime (y = 2x)
    ax.plot([lower, upper], [2*lower, 2*upper], 'k--', lw=2)

    # Reset the limits
    ax.set_xlim(xlims)
    ax.set_ylim(ylims)

    ax.tick_params(axis='both', which='major', labelsize=16)

def plot_time(ax, data_path, show_agent_legend=True):
    """
    Process CSV data and create scatter plot for runtime comparison.
    
    Parameters:
    ax - The axis to plot on
    data_path - Path to the CSV data file
    show_agent_legend - Whether to show the agent legend (default: True)
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
    plot_case(ax, [df1, df2, df3], color_map, config_labels)
    
    # Create and display agent legend only if requested
    if show_agent_legend:
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
    
    # Note: Removed the individual ratio legends
    
    return unique_agents, color_map

def create_success_rate_legend(fig, row_idx=0):
    """
    Create a row-specific legend for the success rate figure.
    
    Parameters:
    fig - The figure to add the legend to
    row_idx - Row index (0 for top row, 1 for middle row, 2 for bottom row)
    
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
            Line2D([0], [0], color='gray', linestyle='-', linewidth=2, label='1.034')
        ]
    
    # Combine all handles
    all_handles = solver_handles + subopt_handles
    all_labels = [h.get_label() for h in all_handles]
    
    # Add a row-specific legend
    bbox_anchors = {0: (0.5, 0.95), 1: (0.5, 0.655), 2: (0.5, 0.36)}
    
    legend = fig.legend(handles=all_handles, 
                 labels=all_labels,
                 loc='upper center', 
                 bbox_to_anchor=bbox_anchors[row_idx],
                 ncol=len(all_handles), 
                 fontsize=18,
                 frameon=True)
    
    return legend

def create_runtime_legend(fig, color_map=None):
    """
    Create a comprehensive legend for the runtime figure.
    """
    # Create custom handles for the average marker
    legend_handles = [Line2D([0], [0], marker='X', color='w', 
                      markersize=8, markeredgecolor='black', markeredgewidth=1.5, label='Average')]
    
    # Create custom handles for ratio
    ratio_handles = [
        Line2D([0], [0], linestyle='--', color='k', lw=1.5, label='1x'),
        Line2D([0], [0], linestyle='--', color='k', lw=2, label='2x')
    ]
    
    # Combine handles
    all_handles = legend_handles + ratio_handles
    all_labels = [h.get_label() for h in all_handles]
    
    # Add a single combined legend for the figure at the top
    legend = fig.legend(handles=all_handles, 
                 labels=all_labels,
                 loc='upper center', 
                 bbox_to_anchor=(0.5, 0.96),
                 ncol=len(all_handles), 
                 fontsize=18,
                 frameon=True)
    
    return legend

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Create separate success rate and runtime plots')
    parser.add_argument('--success_rate_output', type=str, default='fig/success_rate_plots.pdf',
                       help='Path to save the success rate figure')
    parser.add_argument('--runtime_output', type=str, default='fig/runtime_plots.pdf',
                       help='Path to save the runtime figure')
    
    args = parser.parse_args()
    
    # File paths
    map_files = {
        'random': {
            'stat': 'result/decbs_random-32-32-20_stat.csv',
            'time': 'result/decbs_random-32-32-20_result.csv'
        },
        'empty': {
            'stat': 'result/decbs_empty_32_32_stat.csv',
            'time': 'result/decbs_empty_32_32_result.csv'
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
    
    # Map titles for plots
    map_titles = {
        'random': 'random-32-32-20',
        'empty': 'empty-32-32',
        'den_312': 'den_312d',
        'warehouse': 'warehouse-10-20-10-2-1',
        'den_520': 'den_520d',
        'Paris': 'Paris_1_256'
    }
    
    # Define line styles for different suboptimality factors
    line_styles1 = {1.02: ':', 1.1: '--', 1.2: '-'}  # For random and empty
    line_styles2 = {1.01: ':', 1.05: '--', 1.1: '-'}  # For den_312, warehouse, den_520, Paris
    line_styles3 = {1.002: ':', 1.018: '--', 1.038: '-'}  # For other maps (currently unused)
    
    # ---------------------------
    # SUCCESS RATE PLOT (3x3 grid)
    # ---------------------------
    fig_sr, axes_sr = plt.subplots(3, 3, figsize=(24, 18))
    
    # Row 0: random, empty, (empty placeholder)
    plot_success_rate(axes_sr[0, 0], map_files['random']['stat'], [1.02, 1.1, 1.2], line_styles1)
    axes_sr[0, 0].set_title(f"{map_titles['random']}", fontsize=18)
    
    plot_success_rate(axes_sr[0, 1], map_files['empty']['stat'], [1.02, 1.1, 1.2], line_styles1)
    axes_sr[0, 1].set_title(f"{map_titles['empty']}", fontsize=18)
    
    # Make the third column empty but still have a title box
    axes_sr[0, 2].set_visible(False)
    
    # Row 1: den_312, warehouse, (empty placeholder)
    plot_success_rate(axes_sr[1, 0], map_files['den_312']['stat'], [1.01, 1.05, 1.1], line_styles2)
    axes_sr[1, 0].set_title(f"{map_titles['den_312']}", fontsize=18)
    
    plot_success_rate(axes_sr[1, 1], map_files['warehouse']['stat'], [1.01, 1.05, 1.1], line_styles2)
    axes_sr[1, 1].set_title(f"{map_titles['warehouse']}", fontsize=18)
    
    # Make the third column empty but still have a title box
    axes_sr[1, 2].set_visible(False)
    
    # Row 2: den_520, Paris, (empty placeholder)
    plot_success_rate(axes_sr[2, 0], map_files['den_520']['stat'], [1.002, 1.018, 1.038], line_styles3)
    axes_sr[2, 0].set_title(f"{map_titles['den_520']}", fontsize=18)
    
    plot_success_rate(axes_sr[2, 1], map_files['Paris']['stat'], [1.002, 1.018, 1.038], line_styles3)
    axes_sr[2, 1].set_title(f"{map_titles['Paris']}", fontsize=18)
    
    # Make the third column empty but still have a title box
    axes_sr[2, 2].set_visible(False)
    
    # Add row-specific legends for the success rate plot
    legend1 = create_success_rate_legend(fig_sr, row_idx=0)
    legend2 = create_success_rate_legend(fig_sr, row_idx=1)
    legend3 = create_success_rate_legend(fig_sr, row_idx=2)
    
    # Adjust layout
    plt.subplots_adjust(hspace=0.4, wspace=0.3, bottom=0.1, top=0.9)
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(args.success_rate_output), exist_ok=True)
    
    # Save the success rate figure as PDF
    plt.savefig(args.success_rate_output, format='pdf', bbox_inches='tight')
    print(f"Success rate figure saved to {args.success_rate_output}")
    
    # ---------------------------
    # RUNTIME PLOT (3x3 grid)
    # ---------------------------
    fig_rt, axes_rt = plt.subplots(3, 3, figsize=(24, 18))
    
    # Row 0: random, empty, (empty placeholder)
    agents, color_map = plot_time(axes_rt[0, 0], map_files['random']['time'], show_agent_legend=True)
    axes_rt[0, 0].set_title(f"{map_titles['random']}", fontsize=18)
    
    plot_time(axes_rt[0, 1], map_files['empty']['time'], show_agent_legend=True)
    axes_rt[0, 1].set_title(f"{map_titles['empty']}", fontsize=18)
    
    # Make the third column empty but still have a title box
    axes_rt[0, 2].set_visible(False)
    
    # Row 1: den_312, warehouse, (empty placeholder)
    plot_time(axes_rt[1, 0], map_files['den_312']['time'], show_agent_legend=True)
    axes_rt[1, 0].set_title(f"{map_titles['den_312']}", fontsize=18)
    
    plot_time(axes_rt[1, 1], map_files['warehouse']['time'], show_agent_legend=True)
    axes_rt[1, 1].set_title(f"{map_titles['warehouse']}", fontsize=18)
    
    # Make the third column empty but still have a title box
    axes_rt[1, 2].set_visible(False)
    
    # Row 2: den_520, Paris, (empty placeholder)
    plot_time(axes_rt[2, 0], map_files['den_520']['time'], show_agent_legend=True)
    axes_rt[2, 0].set_title(f"{map_titles['den_520']}", fontsize=18)
    
    plot_time(axes_rt[2, 1], map_files['Paris']['time'], show_agent_legend=True)
    axes_rt[2, 1].set_title(f"{map_titles['Paris']}", fontsize=18)
    
    # Make the third column empty but still have a title box
    axes_rt[2, 2].set_visible(False)
    
    # No main title for runtime figure
    
    # Add overall legend for the runtime plot
    runtime_legend = create_runtime_legend(fig_rt, color_map)
    
    # Adjust layout
    plt.subplots_adjust(hspace=0.3, wspace=0.3, bottom=0.1, top=0.9)
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(args.runtime_output), exist_ok=True)
    
    # Save the runtime figure as PDF
    plt.savefig(args.runtime_output, format='pdf', bbox_inches='tight')
    print(f"Runtime figure saved to {args.runtime_output}")