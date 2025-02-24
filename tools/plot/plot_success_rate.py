import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse
import seaborn as sns

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

def plot_data_on_axis(ax, csv_path, store_legend=False):
    """
    Plot data from a CSV file on the given axis.
    Returns legend lines and labels if store_legend is True.
    """
    sns.set_theme(style="whitegrid", font_scale=1.0)
    sns.set_palette("deep")
    
    # Print the CSV filename being processed for debugging
    print(f"Processing file: {csv_path}")

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
        subopt_factors = [1.02, 1.1, 1.2]
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
        line_styles = {
            1.02: ':',
            1.1: '--',
            1.2: '-'
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
        else:
            # Default range
            ax.set_xlim(35, 155)
            ax.set_ylim(-0.1, 1.1)
            ax.set_xticks(np.arange(45, 150, 15))
            
        ax.tick_params(axis='both', which='major', labelsize=12)
        
        # Don't manipulate the spine position to avoid distorting the plot
        # ax.spines['bottom'].set_position(('data', -0.05))  # Removed this line
        
        return legend_lines, legend_labels
        
    except Exception as e:
        print(f"Error processing {csv_path}: {str(e)}")
        return [], []

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Create success rate plots from one or more CSV files')
    parser.add_argument('--csv_paths', type=str, nargs='+', default=['result/empty-32-32-20_stat.csv', 'result/random-32-32-20_stat.csv'],
                        help='Paths to the input CSV file(s)')
    parser.add_argument('--output_path', type=str, default='fig/success_rate_plots.png',
                        help='Path to save the output figure')
    
    args = parser.parse_args()
    
    num_files = len(args.csv_paths)
    
    # Create one subplot per CSV file (in one row)
    fig, axes = plt.subplots(1, num_files, figsize=(8*num_files, 5))
    
    # If only one file is provided, ensure axes is a list
    if num_files == 1:
        axes = [axes]
    
    all_legend_lines = []
    all_legend_labels = []
    
    # Loop through each CSV file and plot on its corresponding subplot
    for i, csv_path in enumerate(args.csv_paths):
        # Store legend from all subplots to ensure completeness
        legend_lines, legend_labels = plot_data_on_axis(axes[i], csv_path, store_legend=True)
        
        # Use a more descriptive title (filename without path)
        import os
        title = os.path.basename(csv_path.replace(".csv", ""))
        axes[i].set_title(title, fontsize=14)
        
        # Add to the complete legend collection
        all_legend_lines.extend(legend_lines)
        all_legend_labels.extend(legend_labels)
    
    # Create a common legend for the figure
    if all_legend_lines:
        # Remove duplicate legend entries
        unique_labels = []
        unique_lines = []
        seen_labels = set()
        
        for line, label in zip(all_legend_lines, all_legend_labels):
            if label not in seen_labels:
                seen_labels.add(label)
                unique_labels.append(label)
                unique_lines.append(line)
        
        
        legend = fig.legend(unique_lines, unique_labels,
                    loc='center left',
                    bbox_to_anchor=(0.9, 0.5),
                    fontsize=20,
                    borderaxespad=0.5,
                    markerscale=1.5,
                    ncol=2)
    
    plt.tight_layout(rect=[0, 0, 0.9, 1])
    
    
    plt.savefig(args.output_path, dpi=300, bbox_inches='tight')
    print(f"Figure saved to {args.output_path}")