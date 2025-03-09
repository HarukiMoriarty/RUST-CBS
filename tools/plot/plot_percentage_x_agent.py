import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import os
import sys

def analyze_multiple_files(input_files=None, output_file="fig/decbs_vs_ecbs_agent.pdf"):
    """
    Analyze DECBS vs ECBS performance data from multiple CSV files
    and create a 3x2 grid of histogram subplots.
    
    Args:
        input_files (list): List of CSV file paths to analyze
        output_file (str): Path to save the output figure
    """
    # Default list of input files if none provided
    if input_files is None:
        input_files = [
            "result/decbs_random-32-32-20_agents_time.csv",
            "result/decbs_maze-32-32-2_agents_time.csv",
            "result/decbs_den_312d_agents_time.csv",
            "result/decbs_warehouse-10-20-10-2-1_agents_time.csv",
            "result/decbs_den_520d_agents_time.csv",
            "result/decbs_Paris_1_256_agents_time.csv"
        ]
    
    # Standard list of suboptimal values to look for in all files
    # This ensures consistent x-axis across all plots
    std_agents = [[45, 60, 75, 90, 105, 120, 135],
                   [20, 30, 40, 50, 60, 70],
                   [105, 120, 135, 150, 165, 180],
                   [90, 120, 150, 180, 210, 240, 270],
                   [50, 100, 150, 200, 250, 300],
                   [50, 100, 150, 200, 250, 300]]
    
    # Create figure with 3x2 subplots
    fig, axes = plt.subplots(3, 2, figsize=(16, 18))
    axes = axes.flatten()  # Flatten to make indexing easier
    
    # Hide any extra subplots if there are fewer than 6 files
    for i in range(len(input_files), 6):
        if i < len(axes):
            axes[i].set_visible(False)
    
    # Define width for the bars
    bar_width = 0.35
    
    # Set consistent colors
    config1_color = 'steelblue'
    config2_color = 'darkorange'
    avg1_color = 'navy'
    avg2_color = 'darkred'
    
    # Process each file
    for i, file_path in enumerate(input_files):
        agents = std_agents[i]
        if i >= 6:  # Limit to 6 files (3x2 grid)
            print(f"Warning: Only showing the first 6 files. Skipping {file_path}")
            continue
            
        if not os.path.exists(file_path):
            print(f"Warning: File {file_path} does not exist")
            # Show error in subplot
            ax = axes[i]
            ax.text(0.5, 0.5, f"File not found:\n{file_path}", 
                    horizontalalignment='center', verticalalignment='center',
                    transform=ax.transAxes, fontsize=30, color='red')
            continue
        
        # Get title from filename (without extension)
        title = os.path.splitext(os.path.basename(file_path))[0]
        title = title.replace('decbs_', '')
        title = title.replace('_agents_time', '')
        title = title.replace('n_3', 'n3')
        title = title.replace('n_5', 'n5')
        
        try:
            # Read the CSV file
            df = pd.read_csv(file_path)
            
            # Filter data for the specific configurations
            config1 = df[(df['op_PC'] == False) & (df['op_BC'] == False) & (df['op_TR'] == False)]
            config2 = df[(df['op_PC'] == False) & (df['op_BC'] == True) & (df['op_TR'] == True)]
            
            # Check if we have data for both configurations
            if config1.empty or config2.empty:
                raise ValueError(f"Missing data for one or both configurations in file {file_path}")
            
            # Group by solver and num_agents
            pivot1 = config1.pivot_table(index='num_agents', columns='solver', values='avg_time')
            pivot2 = config2.pivot_table(index='num_agents', columns='solver', values='avg_time')
            
            # Check if we have both solvers in the data
            if 'ecbs' not in pivot1.columns or 'decbs' not in pivot1.columns:
                raise ValueError(f"Missing solver data in file {file_path}")
            
            # Calculate improvement percentages
            pivot1['improvement'] = ((pivot1['ecbs'] - pivot1['decbs']) / pivot1['ecbs']) * 100
            pivot2['improvement'] = ((pivot2['ecbs'] - pivot2['decbs']) / pivot2['ecbs']) * 100
            
            # Create dictionaries to store improvement values for standard agent values
            improvements1 = {}
            improvements2 = {}
            
            # Get the agent values available in this file
            available_agents = pivot1.index.tolist()
            
            # Match the standard agent values with available ones
            for agent in agents:
                if agent in available_agents:
                    improvements1[agent] = pivot1.loc[agent, 'improvement']
                    improvements2[agent] = pivot2.loc[agent, 'improvement']
                else:
                    # Use NaN for missing values
                    improvements1[agent] = np.nan
                    improvements2[agent] = np.nan
            
            # Plot on the corresponding subplot
            ax = axes[i]
            
            # Positions for bars
            indices = np.arange(len(agents))
            
            # Extract improvement values for plotting
            imp1_values = [improvements1.get(s, np.nan) for s in agents]
            imp2_values = [improvements2.get(s, np.nan) for s in agents]
            
            # Create the bars - for missing values, no bar will be shown
            ax.bar(indices - bar_width/2, imp1_values,
                  bar_width, label='Config1 (F,F,F)', color=config1_color, alpha=0.7)
            
            ax.bar(indices + bar_width/2, imp2_values,
                  bar_width, label='Config2 (F,T,T)', color=config2_color, alpha=0.7)
            
            # Calculate averages (ignoring NaN values)
            valid_imp1 = [v for v in imp1_values if not np.isnan(v)]
            valid_imp2 = [v for v in imp2_values if not np.isnan(v)]
            
            avg_imp1 = np.mean(valid_imp1) if valid_imp1 else np.nan
            avg_imp2 = np.mean(valid_imp2) if valid_imp2 else np.nan
            
            # Add average lines with the value directly on the line
            if not np.isnan(avg_imp1):
                ax.axhline(y=avg_imp1, color=avg1_color, linestyle='--', linewidth=2)
                
                # Add text box with average value on the line
                # Position it at 80% of x-axis width for the first config
                ax.text(0.8 * len(indices), avg_imp1, 
                       f'{avg_imp1:.1f}%', 
                       backgroundcolor='white',
                       bbox=dict(facecolor='white', alpha=0.8, edgecolor=avg1_color, boxstyle='round,pad=0.3'),
                       ha='center', va='center',
                       fontsize=25,
                       fontweight='bold',
                       color=avg1_color)
            
            if not np.isnan(avg_imp2):
                ax.axhline(y=avg_imp2, color=avg2_color, linestyle='--', linewidth=2)
                
                # Add text box with average value on the line
                # Position it at 20% of x-axis width for the second config to avoid overlap
                ax.text(0.2 * len(indices), avg_imp2, 
                       f'{avg_imp2:.1f}%', 
                       backgroundcolor='white',
                       bbox=dict(facecolor='white', alpha=0.5, edgecolor=avg2_color, boxstyle='round,pad=0.3'),
                       ha='center', va='center',
                       fontsize=25,
                       fontweight='bold',
                       color=avg2_color)
            
            # Add a horizontal line at y=0
            ax.axhline(y=0, color='black', linestyle='-', alpha=0.3)
            
            # Set x-ticks and labels
            ax.set_xticks(indices)
            ax.set_xticklabels([str(s) for s in agents], fontsize=30)
            
            # Set title
            ax.set_title(title, fontsize=30)
            
            # Only add x-axis labels for the bottom row (indices 4, 5)
            if i >= 4:  # Bottom row
                ax.set_xlabel('Number of agents', fontsize=30)
            else:
                ax.set_xlabel('', fontsize=30)  # Empty label for other rows
            
            # Only add y-axis label and tick labels for the leftmost subfigures (indices 0, 2, 4)
            if i % 2 == 0:  # Left column
                ax.set_ylabel('Improvement\npercentage (%)', fontsize=30)
                ax.tick_params(axis='y', labelsize=25)
            else:  # Right column
                ax.set_ylabel('')  # No label for right column
                ax.set_yticklabels([])  # Hide y-tick labels for right column
                
            ax.grid(True, alpha=0.3, axis='y')
            
            if i <= 1:
                ax.set_ylim([-10, 55])
            elif i <= 3:
                ax.set_ylim([-10, 40])
            elif i <= 5:
                ax.set_ylim([-35, 5])
            
            # Print summary stats for this file
            print(f"\nFile: {file_path}")
            print(f"Config1 average improvement: {avg_imp1:.2f}%" if not np.isnan(avg_imp1) else "Config1: No valid data")
            print(f"Config2 average improvement: {avg_imp2:.2f}%" if not np.isnan(avg_imp2) else "Config2: No valid data")
            
        except Exception as e:
            print(f"Error processing file {file_path}: {e}")
            # Add error message to the subplot
            ax = axes[i]
            ax.text(0.5, 0.5, f"Error processing:\n{file_path}\n{str(e)}", 
                    horizontalalignment='center', verticalalignment='center',
                    transform=ax.transAxes, fontsize=30, color='red', wrap=True)
    
    # Create a custom legend for the entire figure
    custom_lines = [
        plt.Line2D([0], [0], color=config1_color, lw=0, marker='s', markersize=15, alpha=0.7),
        plt.Line2D([0], [0], color=config2_color, lw=0, marker='s', markersize=15, alpha=0.7),
        plt.Line2D([0], [0], color=avg1_color, lw=2, linestyle='--'),
        plt.Line2D([0], [0], color=avg2_color, lw=2, linestyle='--')
    ]
    
    custom_labels = [
        'No Optimization', 
        'BC+TR', 
        'No Optimization Average', 
        'BC+TR Average'
    ]
    
    fig.legend(custom_lines, custom_labels, loc='upper center', 
               bbox_to_anchor=(0.5, 1.06), ncol=2, fontsize=30, frameon=True)
    
    # Adjust layout
    plt.tight_layout()
    plt.subplots_adjust(top=0.94, hspace=0.25, wspace=0.05)  # Make room for the legend at the bottom
    
    # Create directory for output if it doesn't exist
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    
    # Save the figure
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"\nPlot saved as {output_file}")
    
    return fig

def main():
    # If files are specified as command-line arguments, use those
    if len(sys.argv) > 1:
        input_files = sys.argv[1:]
        analyze_multiple_files(input_files)
    else:
        # Otherwise, use the defaults
        print("No input files specified. Using default file list.")
        analyze_multiple_files()

if __name__ == "__main__":
    main()