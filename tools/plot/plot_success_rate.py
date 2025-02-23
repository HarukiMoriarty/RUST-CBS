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
        1.02: '-.',
        1.04: '--',
        1.06: 'dotted',
        1.08: ':',
        1.1: '--',
        1.12: 'solid',
        1.14: 'dashed',
        1.16: 'dashdot',
        1.2: '-'
    }
    
    # Define markers for different optimization combinations
    markers = {
        'DECBS': 'o',
        'DECBS+BC': 's',
        'DECBS+BC+TR': 'D',
        'ECBS': '*',
        'ECBS+BC': '^',
        'ECBS+BC+TR': 'v'
    }

    # Create figure with subplots
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(20, 16))
    plt.rcParams.update({'font.size': 12})

    # Subplot configurations
    subplot_configs = [
        {
            'ax': ax1,
            'title': 'All Variants',
            'solvers': ['ECBS', 'ECBS+BC', 'ECBS+BC+TR', 'DECBS', 'DECBS+BC', 'DECBS+BC+TR']
        },
        {
            'ax': ax2,
            'title': 'ECBS vs DECBS',
            'solvers': ['ECBS', 'DECBS']
        },
        {
            'ax': ax3,
            'title': 'ECBS+BC vs DECBS+BC',
            'solvers': ['ECBS+BC', 'DECBS+BC']
        },
        {
            'ax': ax4,
            'title': 'ECBS+BC+TR vs DECBS+BC+TR',
            'solvers': ['ECBS+BC+TR', 'DECBS+BC+TR']
        }
    ]

    # Plot each subplot
    for config in subplot_configs:
        ax = config['ax']
        
        # Plot each combination
        for factor in subopt_factors:
            factor_data = df[df['low_level_suboptimal'] == factor]
            
            for solver_name in config['solvers']:
                solver_data = factor_data[factor_data['full_name'] == solver_name]
                if not solver_data.empty:
                    ax.plot(solver_data['num_agents'], 
                           solver_data['success_rate'],
                           linestyle=line_styles[factor],
                           marker=markers[solver_name],
                           color=opt_colors[solver_name],
                           label=f'{solver_name}({factor})',
                           markerfacecolor='white',
                           markersize=6,
                           linewidth=2)
        
        # Customize each subplot
        ax.set_xlabel('Number of agents', fontsize=12)
        ax.set_ylabel('Success rate', fontsize=12)
        ax.grid(True, linestyle='--', alpha=0.3)
        ax.set_title(config['title'], fontsize=14, pad=20)
        
        # Set axis limits with padding
        ax.set_xlim(35, 155)
        ax.set_ylim(-0.05, 1.1)
        
        # Customize x-axis ticks
        ax.set_xticks(np.arange(45, 150, 15))
        ax.tick_params(axis='both', which='major', labelsize=12)
        
        # Move the x-axis slightly down
        ax.spines['bottom'].set_position(('data', -0.05))
        
        # Create legend
        ax.legend(bbox_to_anchor=(1.05, 0.5),
                 loc='center left',
                 fontsize=12,
                 borderaxespad=0,
                 frameon=True,
                 markerscale=1.5)

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