import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse
from matplotlib.lines import Line2D

def plot_case(ax, df, color_map, title):
    # Plot points for each group of num_agents using the provided color_map.
    for agent, color in color_map.items():
        sub = df[df['num_agents'] == agent]
        ax.scatter(sub['time_decbs'], 
                   sub['time_ecbs'],
                   color=color, 
                   s=5, 
                   label=f'{agent} agents')
    
    avg_points = df.groupby('num_agents')[['time_decbs', 'time_ecbs']].mean().reset_index()
    for idx, row in avg_points.iterrows():
        agent = row['num_agents']
        # Plot the average point with a larger "X" marker.
        ax.scatter(row['time_decbs'], row['time_ecbs'],
                   color=color_map.get(agent, 'k'),
                   s=100, marker='X', edgecolor='k', linewidth=1.5)

    ax.set_xlabel('DECBS runtime (s)')
    ax.set_ylabel('ECBS runtime (s)')
    ax.set_title(title)
    
    ax.set_xscale('log')
    ax.set_yscale('log')

    # Draw a dashed diagonal line representing y=x.
    xlims = ax.get_xlim()
    ylims = ax.get_ylim()
    lower = min(xlims[0], ylims[0])
    upper = max(xlims[1], ylims[1])
    ax.plot([lower, upper], [lower, upper], 'k--', lw=1)

    # Reset the limits.
    ax.set_xlim(xlims)
    ax.set_ylim(ylims)

def main(data_path, output_path, suboptimal):
    # Load the combined data from the CSV file.
    df = pd.read_csv(data_path)

    # If a suboptimal value is specified, filter the data.
    if suboptimal is not None:
        suboptimal = [float(x) for x in suboptimal]
        df = df[df['low_level_suboptimal'].isin(suboptimal)]

    # Convert the 'time(us)' column to numeric; non-numeric entries become NaN,
    # then replace them with a large value (converted to seconds).
    df['time(us)'] = pd.to_numeric(df['time(us)'], errors='coerce') / 1_000_000

    # Columns that define a unique experiment setting.
    merge_cols = ["map_path", "yaml_path", 
                  "num_agents", "seed",
                  "low_level_suboptimal",
                  "op_PC", "op_BC", "op_TR"]

    # Group by the unique experiment settings and 'solver', aggregating time with the first value.
    df_grouped = df.groupby(merge_cols + ['solver'], as_index=False)['time(us)'].first()

    # Pivot the DataFrame to create separate columns for decbs and ecbs times.
    df_pivot = df_grouped.pivot(index=merge_cols, columns='solver', values='time(us)').reset_index()

    df_pivot = df_pivot.dropna(subset=['decbs','ecbs'], how='any')

    # For any remaining missing time (if only one solver is missing), fill with a default value (e.g., 60 seconds).
    df_pivot = df_pivot.fillna(60)

    # Rename solver columns for consistency.
    df_pivot = df_pivot.rename(columns={'decbs': 'time_decbs', 'ecbs': 'time_ecbs'})

    # Define the three cases.
    df1 = df_pivot[(df_pivot['op_PC'] == False) & 
                   (df_pivot['op_BC'] == False) & 
                   (df_pivot['op_TR'] == False)]
    df2 = df_pivot[(df_pivot['op_PC'] == False) & 
                   (df_pivot['op_BC'] == True) & 
                   (df_pivot['op_TR'] == False)]
    df3 = df_pivot[(df_pivot['op_PC'] == False) & 
                   (df_pivot['op_BC'] == True) & 
                   (df_pivot['op_TR'] == True)]

    # Create a global color mapping for num_agents.
    unique_agents = sorted(df_pivot['num_agents'].unique())
    colors = plt.cm.jet(np.linspace(0, 1, len(unique_agents)))
    color_map = {agent: color for agent, color in zip(unique_agents, colors)}

    # Create a figure with three subplots.
    fig, axs = plt.subplots(1, 3, figsize=(18, 7))

    # Plot each case using the same color mapping.
    plot_case(axs[0], df1, color_map, '')
    plot_case(axs[1], df2, color_map, 'BC')
    plot_case(axs[2], df3, color_map, 'BC+TR')

    plt.tight_layout(rect=[0, 0, 1, 0.85])  # leave space at the top for the legend

    # Create global legend handles.
    legend_handles = [Line2D([0], [0], marker='o', color='w',
                             markerfacecolor=color_map[agent], markersize=8)
                      for agent in unique_agents]
    legend_labels = [f'{agent} agents' for agent in unique_agents]
    # Place the legend at the top in one row.
    fig.legend(legend_handles, legend_labels, loc='upper center', ncol=len(legend_handles),
               bbox_to_anchor=(0.5, 0.95), fontsize=12)

    plt.savefig(output_path)
    print(f"Figure saved to {output_path}")

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Plot DECBS vs. ECBS running times.")
    parser.add_argument("--data_path", type=str, required=True,
                        help="Path to the CSV data file.")
    parser.add_argument("--output_path", type=str, required=True,
                        help="Path to save the output figure (e.g., 'output.png' or 'output.pdf').")
    args = parser.parse_args()
    main(args.data_path, args.output_path, ['1.02', '1.04', '1.06', '1.08', '1.10', '1.12', '1.14', '1.16', '1.18', '1.2'])
