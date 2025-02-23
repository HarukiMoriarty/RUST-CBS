import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import argparse

def create_plot(csv_path):
    # Read the CSV file
    df = pd.read_csv(csv_path)
    
    # Convert success rate from percentage to decimal
    df['success_rate'] = df['success_rate'] / 100.0
    
    # Create optimization combination column for each solver
    def get_full_name(row):
        if row['solver'] == 'ecbs' and not row['op_PC'] and not row['op_BC'] and not row['op_TR']:
            return 'ECBS'
        elif row['solver'] == 'decbs':
            if not row['op_PC'] and not row['op_BC'] and not row['op_TR']:
                return 'DECBS'
            elif not row['op_PC'] and row['op_BC'] and not row['op_TR']:
                return 'DECBS+BC'
            elif not row['op_PC'] and row['op_BC'] and row['op_TR']:
                return 'DECBS+BC+TR'
    
    df['full_name'] = df.apply(get_full_name, axis=1)
    
    # Get unique suboptimal factors
    subopt_factors = [1.02, 1.1, 1.2]
    
    # Set up the plot with adjusted figure size
    plt.figure(figsize=(16, 6))
    
    # Create subplot with specific size ratio
    ax = plt.subplot(111)
    
    # Define colors for optimization combinations
    opt_colors = {
        'DECBS': '#1b9e77',
        'DECBS+BC': '#c95f02',
        'DECBS+BC+TR': '#7570b3',
        'ECBS': '#66a61e'  
    }
    
    # Define line styles for different suboptimal factors
    line_styles = {
        1.02: '-',
        1.04: '--',
        1.06: ':',
        1.08: '-.',
        1.1: '--',
        1.12: 'solid',
        1.14: 'dashed',
        1.16: 'dashdot',
        1.2: 'dotted'
    }
    
    # Define markers for different optimization combinations
    markers = {
        'DECBS': 'o',
        'DECBS+BC': 's',
        'DECBS+BC+TR': 'D',
        'ECBS': '*'
    }
    
    # Increase font size for all text elements
    plt.rcParams.update({'font.size': 12})
    
    # Plot each combination
    for factor in subopt_factors:
        factor_data = df[df['low_level_suboptimal'] == factor]
        
        # Plot all solvers including ECBS
        for solver_name in ['ECBS', 'DECBS', 'DECBS+BC', 'DECBS+BC+TR']:
            solver_data = factor_data[factor_data['full_name'] == solver_name]
            if not solver_data.empty:
                plt.plot(solver_data['num_agents'], 
                        solver_data['success_rate'],
                        linestyle=line_styles[factor],
                        marker=markers[solver_name],
                        color=opt_colors[solver_name],
                        label=f'{solver_name}({factor})',
                        markersize=6)
    
    # Customize the plot
    plt.xlabel('Number of agents', fontsize=12)
    plt.ylabel('Success rate', fontsize=12)
    plt.grid(True, linestyle='--', alpha=0.3)
    
    # Adjust legend position and box
    box = ax.get_position()
    ax.set_position([box.x0, box.y0, box.width * 0.7, box.height])
    
    # Create legend with larger font size and more height
    legend = plt.legend(bbox_to_anchor=(1.05, 0.5),
                       loc='center left',
                       fontsize=12,
                       borderaxespad=0,
                       bbox_transform=ax.transAxes,
                       frameon=True,
                       markerscale=1.5)
    
    # Set axis limits with padding
    ax.set_xlim(35, 155)
    ax.set_ylim(-0.05, 1.1)
    
    # Customize x-axis ticks
    ax.set_xticks(np.arange(45, 150, 15))
    ax.tick_params(axis='both', which='major', labelsize=12)
    
    # Move the x-axis slightly down
    ax.spines['bottom'].set_position(('data', -0.05))
    
    return plt

# Usage
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Create success rate plot')
    parser.add_argument('--csv_path', type=str, default='result/stat.csv',
                       help='Path to the input CSV file')
    parser.add_argument('--output_path', type=str, default='fig/success_rate_plot.png',
                       help='Path to save the output plot')
    
    args = parser.parse_args()
    
    plt = create_plot(args.csv_path)
    plt.tight_layout()
    plt.savefig(args.output_path, dpi=300, bbox_inches='tight')