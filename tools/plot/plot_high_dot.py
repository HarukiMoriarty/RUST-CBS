import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse
import os
import seaborn as sns
from matplotlib.colors import LinearSegmentedColormap

def plot_expanded_nodes(ax, df, title):
    # Set the Seaborn style
    sns.set_style("whitegrid")
    
    # Create a custom colormap from light blue to dark blue
    # This will make areas with high density appear darker blue
    colors = ["#E6F3FF", "#ADD8E6", "#5CACEE", "#1E90FF", "#0000CD"]
    custom_blue_cmap = LinearSegmentedColormap.from_list("custom_blues", colors)
    
    # Get the data points
    x = df['expanded_decbs']
    y = df['expanded_ecbs']
    
    # Create a 2D histogram with logarithmic bins to show density
    h, xedges, yedges = np.histogram2d(
        np.log10(x), 
        np.log10(y), 
        bins=50,
        range=[[np.log10(10**3), np.log10(10**7)], [np.log10(10**3), np.log10(10**7)]]
    )
    
    # Plot the 2D histogram as an image
    h = h.T  # Transpose for correct orientation
    h = np.log1p(h)  # Log transform counts for better color scaling
    
    # Plot the 2D histogram with blue color gradient
    img = ax.imshow(h, 
               extent=[np.log10(10**3), np.log10(10**7), np.log10(10**3), np.log10(10**7)],
               aspect='auto',
               origin='lower',
               cmap=custom_blue_cmap,
               alpha=0.8)
    
    # Also overlay scatter plot with minimal opacity for individual points
    sns.scatterplot(
        x='expanded_decbs',
        y='expanded_ecbs',
        data=df,
        color='#ADD8E6',
        s=15,  # Smaller points
        alpha=0.3,  # More transparent
        edgecolor='#1E90FF',  
        linewidth=0.3,
        ax=ax
    )
    
    # Set axis limits
    xlims = (10**1, 10**4)
    ylims = (10**1, 10**4)
    
    # Draw a dashed diagonal line representing y=x (1x)
    ax.plot([xlims[0], xlims[1]], [xlims[0], xlims[1]], 'k--', lw=1.5, label='1x')
    
    # Draw a dashed line representing y=5x
    ax.plot([xlims[0], xlims[1]/5], [5*xlims[0], xlims[1]], 'r--', lw=1.5, label='5x')

    # Set specific limits
    ax.set_xlim(xlims)
    ax.set_ylim(ylims)
    
    # Better labels
    ax.set_xlabel('DECBS CT node', fontsize=14)
    ax.set_ylabel('ECBS CT node', fontsize=14)
    
    # Add legend for the reference lines
    ax.legend(loc='upper left', fontsize=16)
    
    # Log scales
    ax.set_xscale('log')
    ax.set_yscale('log')
    
    # Add grid on log scale
    ax.grid(True, which="both", ls="-", alpha=0.2)
    
    # Add tick labels
    ax.set_xticks([10**1, 10**2, 10**3, 10**4])
    ax.set_yticks([10**1, 10**2, 10**3, 10**4])
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
    
    # Check if 'high_level_expanded' exists in the data
    if 'high_level_expanded' not in combined_df.columns:
        print("Error: 'high_level_expanded' column not found in the data.")
        print("Available columns:", combined_df.columns.tolist())
        return

    # Columns that define a unique experiment setting
    merge_cols = ["map_path", "yaml_path", "num_agents", "seed", "low_level_suboptimal"]

    # Group by the unique experiment settings and 'solver', aggregating expanded nodes with the first value
    df_grouped = combined_df.groupby(merge_cols + ['solver'], as_index=False)['high_level_expanded'].first()
    
    # Pivot the DataFrame to create separate columns for decbs and ecbs expanded nodes
    df_pivot = df_grouped.pivot(index=merge_cols, columns='solver', values='high_level_expanded').reset_index()
    
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