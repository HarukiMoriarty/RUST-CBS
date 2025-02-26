import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import os
import sys

def analyze_multiple_files(input_files=None, output_file="fig/decbs_vs_ecbs_histogram.pdf"):
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
            "result/decbs_random-32-32-20_time.csv",
            "result/decbs_empty_32_32_time.csv",
            "result/decbs_den_312d_time.csv",
            "result/decbs_warehouse-10-20-10-2-1_time.csv",
            "result/decbs_den_520d_time.csv",
            "result/decbs_Paris_1_256_time.csv"
        ]
    
    # Standard list of suboptimal values to look for in all files
    # This ensures consistent x-axis across all plots
    std_suboptimal_values = [[1.06, 1.08, 1.1, 1.12, 1.14, 1.16, 1.18, 1.2],
                             [1.06, 1.08, 1.1, 1.12, 1.14, 1.16, 1.18, 1.2],
                             [1.03, 1.04, 1.05, 1.06, 1.07, 1.08, 1.09, 1.1],
                             [1.03, 1.04, 1.05, 1.06, 1.07, 1.08, 1.09, 1.1],
                             [1.01, 1.014, 1.018, 1.022, 1.026, 1.03, 1.034, 1.038],
                             [1.01, 1.014, 1.018, 1.022, 1.026, 1.03, 1.034, 1.038]]
    
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
        suboptimal = std_suboptimal_values[i]
        if i >= 6:  # Limit to 6 files (3x2 grid)
            print(f"Warning: Only showing the first 6 files. Skipping {file_path}")
            continue
            
        if not os.path.exists(file_path):
            print(f"Warning: File {file_path} does not exist")
            # Show error in subplot
            ax = axes[i]
            ax.text(0.5, 0.5, f"File not found:\n{file_path}", 
                    horizontalalignment='center', verticalalignment='center',
                    transform=ax.transAxes, fontsize=14, color='red')
            continue
        
        # Get title from filename (without extension)
        title = os.path.splitext(os.path.basename(file_path))[0]
        
        try:
            # Read the CSV file
            df = pd.read_csv(file_path)
            
            # Filter data for the specific configurations
            config1 = df[(df['op_PC'] == False) & (df['op_BC'] == False) & (df['op_TR'] == False)]
            config2 = df[(df['op_PC'] == False) & (df['op_BC'] == True) & (df['op_TR'] == True)]
            
            # Check if we have data for both configurations
            if config1.empty or config2.empty:
                raise ValueError(f"Missing data for one or both configurations in file {file_path}")
            
            # Group by solver and low_level_suboptimal
            pivot1 = config1.pivot_table(index='low_level_suboptimal', columns='solver', values='avg_time')
            pivot2 = config2.pivot_table(index='low_level_suboptimal', columns='solver', values='avg_time')
            
            # Check if we have both solvers in the data
            if 'ecbs' not in pivot1.columns or 'decbs' not in pivot1.columns:
                raise ValueError(f"Missing solver data in file {file_path}")
            
            # Calculate improvement percentages
            pivot1['improvement'] = ((pivot1['ecbs'] - pivot1['decbs']) / pivot1['ecbs']) * 100
            pivot2['improvement'] = ((pivot2['ecbs'] - pivot2['decbs']) / pivot2['ecbs']) * 100
            
            # Create dictionaries to store improvement values for standard suboptimal values
            improvements1 = {}
            improvements2 = {}
            
            # Get the suboptimal values available in this file
            available_suboptimal = pivot1.index.tolist()
            
            # Match the standard suboptimal values with available ones
            for subopt in suboptimal:
                if subopt in available_suboptimal:
                    improvements1[subopt] = pivot1.loc[subopt, 'improvement']
                    improvements2[subopt] = pivot2.loc[subopt, 'improvement']
                else:
                    # Use NaN for missing values
                    improvements1[subopt] = np.nan
                    improvements2[subopt] = np.nan
            
            # Plot on the corresponding subplot
            ax = axes[i]
            
            # Positions for bars
            indices = np.arange(len(suboptimal))
            
            # Extract improvement values for plotting
            imp1_values = [improvements1.get(s, np.nan) for s in suboptimal]
            imp2_values = [improvements2.get(s, np.nan) for s in suboptimal]
            
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
            
            # Add average lines
            if not np.isnan(avg_imp1):
                ax.axhline(y=avg_imp1, color=avg1_color, linestyle='--', 
                           label=f'Avg Config1: {avg_imp1:.1f}%', linewidth=2)
            
            if not np.isnan(avg_imp2):
                ax.axhline(y=avg_imp2, color=avg2_color, linestyle='--', 
                           label=f'Avg Config2: {avg_imp2:.1f}%', linewidth=2)
            
            # Add a horizontal line at y=0
            ax.axhline(y=0, color='black', linestyle='-', alpha=0.3)
            
            # Set x-ticks and labels
            ax.set_xticks(indices)
            ax.set_xticklabels([str(s) for s in suboptimal], fontsize=14)
            
            # Set title and labels
            ax.set_title(title, fontsize=14)
            ax.set_xlabel('Suboptimality Factor', fontsize=14)
            ax.set_ylabel('Improvement (%)', fontsize=14)
            ax.grid(True, alpha=0.3, axis='y')
            ax.tick_params(axis='y', labelsize=14)
            
            # Set reasonable y-axis limits
            max_val = max([v for v in list(improvements1.values()) + list(improvements2.values()) if not np.isnan(v)], default=50) + 10
            min_val = min([v for v in list(improvements1.values()) + list(improvements2.values()) if not np.isnan(v)], default=-50) - 10
            
            # Ensure we have reasonable limits in case of all NaN
            if np.isnan(max_val) or np.isnan(min_val):
                max_val, min_val = 50, -50
                
            # Make sure y-limits are appropriate
            min_val = min(-10, min_val)  # At least show some negative space
            ax.set_ylim([min_val, max_val])
            
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
                    transform=ax.transAxes, fontsize=14, color='red', wrap=True)
    
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
               bbox_to_anchor=(0.5, 0.98), ncol=4, fontsize=14, frameon=True)
    
    # Adjust layout
    plt.tight_layout()
    plt.subplots_adjust(top=0.94)  # Make room for the legend
    
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