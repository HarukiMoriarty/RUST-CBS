import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import os
import sys

def analyze_combined_data(agent_files=None, subopt_files=None, output_file="fig/decbs_vs_ecbs_combined.pdf"):
    """
    Analyze DECBS vs ECBS performance data from multiple CSV files
    and create a 4x3 grid combining agent and suboptimality analyses.
    
    Args:
        agent_files (list): List of CSV file paths for agent analysis
        subopt_files (list): List of CSV file paths for suboptimality analysis
        output_file (str): Path to save the output figure
    """
    # Default list of input files if none provided
    if agent_files is None:
        agent_files = [
            "result/decbs_random-32-32-20_agents_time.csv",
            "result/decbs_maze-32-32-2_agents_time.csv",
            "result/decbs_den_312d_agents_time.csv",
            "result/decbs_warehouse-10-20-10-2-1_agents_time.csv",
            "result/decbs_den_520d_agents_time.csv",
            "result/decbs_Paris_1_256_agents_time.csv"
        ]
    
    if subopt_files is None:
        subopt_files = [
            "result/decbs_random-32-32-20_time.csv",
            "result/decbs_maze-32-32-2_time.csv",
            "result/decbs_den_312d_time.csv",
            "result/decbs_warehouse-10-20-10-2-1_time.csv",
            "result/decbs_den_520d_time.csv",
            "result/decbs_Paris_1_256_time.csv"
        ]
    
    # Standard list of agent values to look for in all files
    std_agents = [
        [45, 60, 75, 90, 105, 120, 135],
        [20, 30, 40, 50, 60, 70],
        [105, 120, 135, 150, 165, 180],
        [90, 120, 150, 180, 210, 240, 270],
        [50, 100, 150, 200, 250, 300],
        [50, 100, 150, 200, 250, 300]
    ]
    
    # Standard list of suboptimal values to look for in all files
    std_suboptimal_values = [
        [1.06, 1.08, 1.1, 1.12, 1.14, 1.16, 1.18, 1.2],
        [1.06, 1.08, 1.1, 1.12, 1.14, 1.16, 1.18, 1.2],
        [1.04, 1.05, 1.06, 1.07, 1.08, 1.09, 1.1],
        [1.04, 1.05, 1.06, 1.07, 1.08, 1.09, 1.1],
        [1.01, 1.014, 1.018, 1.022, 1.026, 1.03, 1.034, 1.038],
        [1.01, 1.014, 1.018, 1.022, 1.026, 1.03, 1.034, 1.038]
    ]
    
    # Define the number of maps to process
    num_maps = min(len(agent_files), len(subopt_files), 6)  # Maximum 6 maps
    
    # Create figure with 3x4 subplots
    # Each row will have 2 maps, each with an agent plot and a subopt plot side by side
    fig, axes = plt.subplots(3, 4, figsize=(24, 18))
    
    # Define width for the bars
    bar_width = 0.35
    
    # Set consistent colors
    config1_color = 'steelblue'
    config2_color = 'darkorange'
    avg1_color = 'navy'
    avg2_color = 'darkred'
    
    # Process each map (up to 6 maps)
    for i in range(num_maps):
        agent_file = agent_files[i]
        subopt_file = subopt_files[i]
        
        # Calculate row and column for agent plot
        map_row = i // 2  # Maps 0,1 in row 0, 2,3 in row 1, 4,5 in row 2
        map_col_base = (i % 2) * 2  # Maps 0,2,4 start at column 0, Maps 1,3,5 start at column 2
        
        # Agent plot is always at the column base
        agent_row = map_row
        agent_col = map_col_base
        
        # Subopt plot is always at column base + 1
        subopt_row = map_row
        subopt_col = map_col_base + 1
        
        # Get the axes for this map
        agent_ax = axes[agent_row, agent_col]
        subopt_ax = axes[subopt_row, subopt_col]
        
        # Get map title from filename (without extension)
        if os.path.exists(agent_file):
            title = os.path.splitext(os.path.basename(agent_file))[0]
            title = title.replace('decbs_', '').replace('_agents_time', '')
            title = title.replace('_time', '').replace('n_3', 'n3').replace('n_5', 'n5')
        elif os.path.exists(subopt_file):
            title = os.path.splitext(os.path.basename(subopt_file))[0]
            title = title.replace('decbs_', '').replace('_time', '')
            title = title.replace('n_3', 'n3').replace('n_5', 'n5')
        else:
            title = f"Map {i+1}"
            
        # Use the same title for both plots of this map
        map_title = title

        # UPDATED Y-AXIS LIMITS HERE:
        ylim_by_row = [[-10, 50], [-10, 40], [-35, 5]]  # Modified Y-limits by row
        
        # Process agent file
        process_file(
            file_path=agent_file,
            ax=agent_ax,
            bar_width=bar_width,
            config1_color=config1_color,
            config2_color=config2_color,
            avg1_color=avg1_color,
            avg2_color=avg2_color,
            std_values=std_agents[i],
            value_type='num_agents',
            title=map_title,
            x_label="Number of agents",
            show_y_label=(agent_col == 0),  # Only show y label for leftmost column
            show_x_label=(agent_row == 2),  # Only show x label for bottom row
            ylim_by_row=ylim_by_row  # Updated y-limits
        )
        
        # Process suboptimality file
        process_file(
            file_path=subopt_file,
            ax=subopt_ax,
            bar_width=bar_width,
            config1_color=config1_color,
            config2_color=config2_color,
            avg1_color=avg1_color,
            avg2_color=avg2_color,
            std_values=std_suboptimal_values[i],
            value_type='low_level_suboptimal',
            title=map_title,  # Use same title for suboptimality plot
            x_label="Suboptimality factor",
            show_y_label=False,  # Never show y label for subopt plots
            show_x_label=(subopt_row == 2),  # Only show x label for bottom row
            ylim_by_row=ylim_by_row,  # Updated y-limits
            format_subopt_labels=True  # Format suboptimality labels specially
        )
    
    # Hide any unused subplots
    for i in range(3):
        for j in range(4):
            # Calculate which map this subplot belongs to
            map_index = (i * 2) + (j // 2)
            if map_index >= num_maps:
                axes[i, j].set_visible(False)
    
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
               bbox_to_anchor=(0.5, 1.03), ncol=4, fontsize=25, frameon=True)
    
    # Adjust layout
    plt.tight_layout()
    plt.subplots_adjust(top=0.94, left=0.07, hspace=0.3, wspace=0.15)
    
    # Create directory for output if it doesn't exist
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    
    # Save the figure
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"\nPlot saved as {output_file}")
    
    return fig

def process_file(file_path, ax, bar_width, config1_color, config2_color, 
                 avg1_color, avg2_color, std_values, value_type, title, 
                 x_label, show_y_label, show_x_label, ylim_by_row, format_subopt_labels=False):
    """
    Process a single CSV file and create a bar plot on the given axis.
    
    Args:
        file_path (str): Path to the CSV file
        ax (matplotlib.axes.Axes): Axis to plot on
        bar_width (float): Width of the bars
        config1_color (str): Color for config1 bars
        config2_color (str): Color for config2 bars
        avg1_color (str): Color for config1 average line
        avg2_color (str): Color for config2 average line
        std_values (list): Standard values to look for
        value_type (str): Type of x-axis value ('num_agents' or 'low_level_suboptimal')
        title (str): Title for the plot
        x_label (str): Label for x-axis
        show_y_label (bool): Whether to show y-axis label
        show_x_label (bool): Whether to show x-axis label
        ylim_by_row (list): Y-axis limits by row
        format_subopt_labels (bool): Whether to format suboptimality labels specially
    """
    if not os.path.exists(file_path):
        print(f"Warning: File {file_path} does not exist")
        # Show error in subplot
        ax.text(0.5, 0.5, f"File not found:\n{file_path}", 
                horizontalalignment='center', verticalalignment='center',
                transform=ax.transAxes, fontsize=25, color='red')
        return
    
    try:
        # Read the CSV file
        df = pd.read_csv(file_path)
        
        # Filter data for the specific configurations
        config1 = df[(df['op_PC'] == False) & (df['op_BC'] == False) & (df['op_TR'] == False)]
        config2 = df[(df['op_PC'] == False) & (df['op_BC'] == True) & (df['op_TR'] == True)]
        
        # Check if we have data for both configurations
        if config1.empty or config2.empty:
            raise ValueError(f"Missing data for one or both configurations in file {file_path}")
        
        # Group by solver and num_agents/low_level_suboptimal
        pivot1 = config1.pivot_table(index=value_type, columns='solver', values='avg_time')
        pivot2 = config2.pivot_table(index=value_type, columns='solver', values='avg_time')
        
        # Check if we have both solvers in the data
        if 'ecbs' not in pivot1.columns or 'decbs' not in pivot1.columns:
            raise ValueError(f"Missing solver data in file {file_path}")
        
        # Calculate improvement percentages
        pivot1['improvement'] = ((pivot1['ecbs'] - pivot1['decbs']) / pivot1['ecbs']) * 100
        pivot2['improvement'] = ((pivot2['ecbs'] - pivot2['decbs']) / pivot2['ecbs']) * 100
        
        # Create dictionaries to store improvement values for standard values
        improvements1 = {}
        improvements2 = {}
        
        # Get the values available in this file
        available_values = pivot1.index.tolist()
        
        # Match the standard values with available ones
        for val in std_values:
            if val in available_values:
                improvements1[val] = pivot1.loc[val, 'improvement']
                improvements2[val] = pivot2.loc[val, 'improvement']
            else:
                # Use NaN for missing values
                improvements1[val] = np.nan
                improvements2[val] = np.nan
        
        # Positions for bars
        indices = np.arange(len(std_values))
        
        # Extract improvement values for plotting
        imp1_values = [improvements1.get(s, np.nan) for s in std_values]
        imp2_values = [improvements2.get(s, np.nan) for s in std_values]
        
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
                   bbox=dict(facecolor='white', alpha=0.8, edgecolor=avg2_color, boxstyle='round,pad=0.3'),
                   ha='center', va='center',
                   fontsize=25,
                   fontweight='bold',
                   color=avg2_color)
        
        # Add a horizontal line at y=0
        ax.axhline(y=0, color='black', linestyle='-', alpha=0.3)
        
        # Set x-ticks and labels
        ax.set_xticks(indices)
        
        if format_subopt_labels:
            # Format x-tick labels to show every other tick and remove leading 1 for suboptimality
            formatted_labels = []
            for j, s in enumerate(std_values):
                # Format to remove leading 1 and show only every other tick
                if j % 2 == 0:  # Show every other tick
                    formatted_labels.append(f'.{str(s).split(".")[-1]}')
                else:
                    formatted_labels.append('')  # Empty string for ticks to hide
            ax.set_xticklabels(formatted_labels, fontsize=25)
        else:
            # For agent values, show every two ticks
            formatted_labels = []
            for j, s in enumerate(std_values):
                if j % 2 == 0:  # Show every other tick
                    formatted_labels.append(str(s))
                else:
                    formatted_labels.append('')  # Empty string for ticks to hide
            ax.set_xticklabels(formatted_labels, fontsize=25)
        
        # Set title
        ax.set_title(title, fontsize=25)
        
        # Set axis labels based on flags
        if show_x_label:
            ax.set_xlabel(x_label, fontsize=25)
        else:
            ax.set_xlabel('', fontsize=25)
            
        if show_y_label:
            ax.set_ylabel('Improvement\npercentage (%)', fontsize=25)
            ax.tick_params(axis='y', labelsize=25)  # Updated to font size 25
        else:
            ax.set_ylabel('')
            ax.set_yticklabels([])
            
        # Add grid
        ax.grid(True, alpha=0.3, axis='y')
        
        # Set y-limits based on map type
        if "den520d" in title or "den_520d" in title or "Paris" in title:
            row_index = 2  # Use the third set of limits for den_520d and Paris maps
        elif "den312d" in title or "den_312d" in title or "warehouse" in title:
            row_index = 1  # Use the second set of limits for den_312d and warehouse maps
        else:
            row_index = 0  # Use the first set of limits for random and maze maps
        ax.set_ylim(ylim_by_row[row_index])
        print(f"Setting y-limits for {title} to {ylim_by_row[row_index]}")
        
        # Print summary stats for this file
        print(f"\nFile: {file_path}")
        print(f"Config1 average improvement: {avg_imp1:.2f}%" if not np.isnan(avg_imp1) else "Config1: No valid data")
        print(f"Config2 average improvement: {avg_imp2:.2f}%" if not np.isnan(avg_imp2) else "Config2: No valid data")
        
    except Exception as e:
        print(f"Error processing file {file_path}: {e}")
        # Add error message to the subplot
        ax.text(0.5, 0.5, f"Error processing:\n{file_path}\n{str(e)}", 
                horizontalalignment='center', verticalalignment='center',
                transform=ax.transAxes, fontsize=25, color='red', wrap=True)

def main():
    # Check if agent files and subopt files are specified as command-line arguments
    if len(sys.argv) > 2:
        agent_files = []
        subopt_files = []
        
        # Assume alternating agent and subopt files
        for i in range(1, len(sys.argv), 2):
            if i < len(sys.argv):
                agent_files.append(sys.argv[i])
            if i+1 < len(sys.argv):
                subopt_files.append(sys.argv[i+1])
                
        analyze_combined_data(agent_files, subopt_files)
    else:
        # Otherwise, use the defaults
        print("Not enough input files specified. Using default file lists.")
        analyze_combined_data()
        
    print("\nLayout summary:")
    print("- 3x4 grid with 6 maps total (2 maps per row)")
    print("- Each map has agent analysis on the left, suboptimality analysis on the right")
    print("- Maps are arranged in row-major order: random, maze (row 1); den_312d, warehouse (row 2); den_520d, Paris (row 3)")
    print("- Y-axis labels only on leftmost plots")
    print("- X-axis labels only on bottom row")

if __name__ == "__main__":
    main()