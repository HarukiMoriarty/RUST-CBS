import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse
import os
import seaborn as sns

def plot_expanded_nodes(ax, df, title):
    # Set the Seaborn style
    sns.set_style("whitegrid")
    
    # Use a nicer color palette
    palette = sns.color_palette("deep")
    
    # Plot points with better aesthetics
    sns.scatterplot(
        x='expanded_decbs',
        y='expanded_ecbs',
        data=df,
        color=palette[1],
        s=30,
        alpha=0.8,
        ax=ax
    )
    
    # Set axis limits
    xlims = (10**3, 10**7)
    ylims = (10**3, 10**7)
    
    # Draw a dashed diagonal line representing y=x (1x)
    ax.plot([xlims[0], xlims[1]], [xlims[0], xlims[1]], 'k--', lw=1.5, label='1x')
    
    # Draw a dashed line representing y=5x
    ax.plot([xlims[0], xlims[1]/5], [5*xlims[0], xlims[1]], 'r--', lw=1.5, label='5x')

    # Set specific limits
    ax.set_xlim(xlims)
    ax.set_ylim(ylims)
    
    # Better labels
    ax.set_xlabel('DECBS low level focal node', fontsize=14)
    ax.set_ylabel('ECBS low level focal node', fontsize=14)
    ax.set_title(title, fontsize=15, pad=15)
    
    # Add legend for the reference lines
    ax.legend(loc='upper left', fontsize=16)
    
    # Log scales
    ax.set_xscale('log')
    ax.set_yscale('log')
    
    # Add grid on log scale
    ax.grid(True, which="both", ls="-", alpha=0.2)
    
    # Add tick labels
    ax.set_xticks([10**3, 10**4, 10**5, 10**6, 10**7])
    ax.set_yticks([10**3, 10**4, 10**5, 10**6, 10**7])
    ax.set_xticklabels(['10³', '10⁴', '10⁵', '10⁶', '10⁷'])
    ax.set_yticklabels(['10³', '10⁴', '10⁵', '10⁶', '10⁷'])
    ax.tick_params(labelsize=14)

def main(data_paths, output_path):
    # Initialize an empty DataFrame to hold combined data
    combined_df = pd.DataFrame()
    
    # Load and combine data from multiple CSV files
    for data_path in data_paths:
        if os.path.exists(data_path):
            df = pd.read_csv(data_path)
            combined_df = pd.concat([combined_df, df], ignore_index=True)
        else:
            print(f"Warning: File {data_path} not found, skipping.")
    
    if combined_df.empty:
        print("Error: No valid data found in the provided files.")
        return
    
    # Check if 'low_level_focal_expanded' exists in the data
    if 'low_level_focal_expanded' not in combined_df.columns:
        print("Error: 'low_level_focal_expanded' column not found in the data.")
        print("Available columns:", combined_df.columns.tolist())
        return

    # Columns that define a unique experiment setting
    merge_cols = ["map_path", "yaml_path", "num_agents", "seed", "low_level_suboptimal"]

    # Group by the unique experiment settings and 'solver', aggregating expanded nodes with the first value
    df_grouped = combined_df.groupby(merge_cols + ['solver'], as_index=False)['low_level_focal_expanded'].first()
    
    # Pivot the DataFrame to create separate columns for decbs and ecbs expanded nodes
    df_pivot = df_grouped.pivot(index=merge_cols, columns='solver', values='low_level_focal_expanded').reset_index()
    
    # Check if we have both 'decbs' and 'ecbs' columns
    if 'decbs' in df_pivot.columns and 'ecbs' in df_pivot.columns:
        # Drop rows where either decbs or ecbs has missing data
        df_pivot = df_pivot.dropna(subset=['decbs', 'ecbs'], how='any')
        
        # Rename solver columns for consistency
        df_pivot = df_pivot.rename(columns={'decbs': 'expanded_decbs', 'ecbs': 'expanded_ecbs'})
        
        # Create a figure with improved aesthetics
        plt.figure(figsize=(10, 8), dpi=100)
        fig, ax = plt.subplots(figsize=(10, 8))
        
        # Apply seaborn styling to the whole figure
        sns.set_context("notebook", font_scale=1.2)
        
        # Plot all data points in one plot
        plot_expanded_nodes(ax, df_pivot, 'DECBS vs ECBS: Low-Level Focal Expanded Nodes')
        
        # Improve overall figure appearance
        plt.tight_layout()
        plt.savefig(output_path, dpi=300, bbox_inches='tight')
        print(f"Figure saved to {output_path}")
    else:
        print("Error: After pivoting, 'decbs' or 'ecbs' columns are missing.")
        print("Available columns:", df_pivot.columns.tolist())

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Plot DECBS vs. ECBS expanded nodes.")
    parser.add_argument("--data_paths", type=str, nargs='+',
                        help="Paths to the CSV data files (space-separated).")
    parser.add_argument("--output_path", type=str, required=True,
                        help="Path to save the output figure (e.g., 'output.png' or 'output.pdf').")
    args = parser.parse_args()

    # If data_paths is not provided, use default
    data_paths = args.data_paths if args.data_paths else ['result/random-32-32-20_result.csv', 'result/decbs_warehouse-10-20-10-2-1_result.csv']
    
    main(data_paths, args.output_path)