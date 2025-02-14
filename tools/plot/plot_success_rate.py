import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

def create_plot(csv_path):
    # Read the CSV file
    df = pd.read_csv(csv_path)
    
    # Convert success rate from percentage to decimal
    df['success_rate'] = df['success_rate'] / 100.0
    
    # Create optimization combination column for each solver
    def get_full_name(row):
        base_solver = 'DECBS' if row['solver'] == 'decbs' else 'ECBS'
        if base_solver == 'ECBS':
            return 'ECBS'  # ECBS is always without optimization
        return get_opt_name(base_solver, row['op_BC'], row['op_PC'], row['op_TR'])
    
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
        'DECBS+BC+PC': '#7570b3',
        'DECBS+BC+PC+TR': '#e7298a',
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
        'DECBS+BC+PC': '^',
        'DECBS+BC+PC+TR': 'D',
        'ECBS': '*'
    }
    
    # Increase font size for all text elements
    plt.rcParams.update({'font.size': 12})
    
    # Plot each combination
    for factor in subopt_factors:
        factor_data = df[df['low_level_suboptimal'] == factor]
        
        # Plot all solvers including ECBS
        for solver_name in ['ECBS', 'DECBS', 'DECBS+BC', 'DECBS+BC+PC', 'DECBS+BC+PC+TR']:
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
    ax.set_xticks(np.arange(40, 151, 15))
    ax.tick_params(axis='both', which='major', labelsize=12)
    
    # Move the x-axis slightly down
    ax.spines['bottom'].set_position(('data', -0.05))
    
    return plt

def get_opt_name(base_solver, bc, pc, tr):
    if not bc:
        return base_solver
    elif bc and not pc:
        return f'{base_solver}+BC'
    elif bc and pc and not tr:
        return f'{base_solver}+BC+PC'
    elif bc and pc and tr:
        return f'{base_solver}+BC+PC+TR'
    return base_solver

# Usage
if __name__ == "__main__":
    plt = create_plot('result/stat.csv')
    plt.tight_layout()
    plt.savefig('fig/success_rate_plot.png', dpi=300, bbox_inches='tight')
    plt.show()