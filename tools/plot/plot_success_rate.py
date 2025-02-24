import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse
import seaborn as sns

def get_full_name(row):
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

def create_plots(csv_path):
    sns.set_theme(style="whitegrid", font_scale=1.0)
    sns.set_palette("deep")

    # Read the CSV file
    df = pd.read_csv(csv_path)
    
    # Convert success rate from percentage to decimal
    df['success_rate'] = df['success_rate'] / 100.0
    
    # Add full name column
    df['full_name'] = df.apply(get_full_name, axis=1)
    
    # Get unique suboptimal factors
    subopt_factors = [1.02, 1.1, 1.2]
    
    # Define colors for optimization combinations
    colors = sns.color_palette("deep")
    opt_colors = {
        'DECBS': colors[0],
        'DECBS+BC': colors[1],
        'DECBS+BC+TR': colors[2],
        'ECBS': colors[3],
        'ECBS+BC': colors[4],
        'ECBS+BC+TR': colors[5]
    }
    
    # Define line styles for different suboptimal factors
    line_styles = {
        1.02: ':',
        1.1: '--',
        1.2: '-'
    }
    
    # Define markers for different optimization combinations
    markers = {
        'DECBS': 'o',
        'DECBS+BC': 's',
        'DECBS+BC+TR': 'D',
        'ECBS': 'o',
        'ECBS+BC': 's',
        'ECBS+BC+TR': 'D'
    }

    # Create figure with subplots in one row
    fig, (ax1, ax2, ax3, ax4) = plt.subplots(1, 4, figsize=(24, 5))
    plt.rcParams.update({'font.size': 12})

    # Subplot configurations
    subplot_configs = [
        {
            'ax': ax1,
            'solvers': ['ECBS', 'ECBS+BC', 'ECBS+BC+TR', 'DECBS', 'DECBS+BC', 'DECBS+BC+TR']
        },
        {
            'ax': ax2,
            'solvers': ['ECBS', 'DECBS']
        },
        {
            'ax': ax3,
            'solvers': ['ECBS+BC', 'DECBS+BC']
        },
        {
            'ax': ax4,
            'solvers': ['ECBS+BC+TR', 'DECBS+BC+TR']
        }
    ]

    # Store all lines for the legend
    legend_lines = []
    legend_labels = []

    # Plot each subplot
    for config in subplot_configs:
        ax = config['ax']
        
        # Plot each combination
        for factor in subopt_factors:
            factor_data = df[df['low_level_suboptimal'] == factor]
            
            for solver_name in config['solvers']:
                solver_data = factor_data[factor_data['full_name'] == solver_name]
                if not solver_data.empty:
                    line = ax.plot(solver_data['num_agents'], 
                                 solver_data['success_rate'],
                                 linestyle=line_styles[factor],
                                 marker=markers[solver_name],
                                 color=opt_colors[solver_name],
                                 markerfacecolor='white',
                                 markersize=6,
                                 linewidth=2)
                    
                    # Only store for legend if it's from the first subplot
                    if ax == ax1:
                        legend_lines.append(line[0])
                        legend_labels.append(f'{solver_name}({factor})')
        
        # Customize each subplot
        ax.set_xlabel('Number of agents', fontsize=12)
        if ax == ax1:  # Only add y-label to the first subplot
            ax.set_ylabel('Success rate', fontsize=12)
        ax.grid(True, linestyle='--', alpha=0.3)
        
        # Set axis limits with padding
        ax.set_xlim(35, 155)
        ax.set_ylim(-0.05, 1.1)
        
        # Customize x-axis ticks
        ax.set_xticks(np.arange(45, 150, 15))
        ax.tick_params(axis='both', which='major', labelsize=12)
        
        # Move the x-axis slightly down
        ax.spines['bottom'].set_position(('data', -0.05))

    # Create a single legend for all subplots
    fig.legend(legend_lines, legend_labels,
              loc='center left',
              bbox_to_anchor=(0.1, 0.5),
              fontsize=22,
              borderaxespad=0,
              frameon=True,
              markerscale=1.5,
              ncol=2)

    # Adjust layout to prevent overlap
    plt.tight_layout()
    return plt

# Usage
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Create success rate plots')
    parser.add_argument('--csv_path', type=str, default='result/stat.csv',
                       help='Path to the input CSV file')
    parser.add_argument('--output_path', type=str, default='fig/success_rate_plots.png',
                       help='Path to save the output plot')
    
    args = parser.parse_args()
    
    plt = create_plots(args.csv_path)
    plt.savefig(args.output_path, dpi=300, bbox_inches='tight')