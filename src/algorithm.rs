mod astar;
mod astarfocal;

pub(crate) use astar::{a_star_search, standard_a_star_search};
pub(crate) use astarfocal::focal_a_star_search;

use std::collections::{HashMap, HashSet};

use crate::common::{Agent, Constraint, Mdd, MddNode, Path};
use crate::map::Map;

type Trace = HashMap<((usize, usize), usize), ((usize, usize), usize)>;

// TODO: different kinds of hc
// h1: Number of conflicts
// h2: Number of conflicting agents
// h3: Number of pairs
// h4: Vertex Cover
// h5: Alternating heuristic
fn heuristic_focal(
    agent: usize,
    position: (usize, usize),
    prev_position: (usize, usize),
    time: usize,
    paths: &[Path],
) -> usize {
    // Tricky: we never call this function when time step is 0.
    assert_ne!(time, 0);

    let mut conflict_count = 0;

    for (agent_id, path) in paths.iter().enumerate() {
        if agent_id == agent {
            continue; // Skip the current agent to avoid self-conflict.
        }

        let other_position = path.get(time).unwrap_or_else(|| path.last().unwrap());

        // Check for vertex conflict.
        if *other_position == position {
            conflict_count += 1;
        }

        // Check for edge conflict.
        if time >= path.len() {
            continue;
        }
        let other_prev_position = path.get(time - 1).unwrap();
        if (*other_position == prev_position) && (*other_prev_position == position) {
            conflict_count += 1;
        }
    }

    conflict_count
}

fn construct_path(trace: &Trace, mut current: ((usize, usize), usize)) -> Path {
    let mut path = vec![current.0];
    while let Some(&(pos, time)) = trace.get(&current) {
        path.push(pos);
        current = (pos, time);
    }
    path.reverse();
    path
}

fn construct_mdd(
    map: &Map,
    agent: &Agent,
    constraints: &HashSet<Constraint>,
    optimal_cost: usize,
) -> Mdd {
    let mut mdd = vec![HashMap::new(); optimal_cost + 1];

    mdd[0].insert(
        agent.start,
        MddNode {
            parents: HashSet::new(),
            children: HashSet::new(),
        },
    );

    // Forward pass
    for depth in 0..optimal_cost {
        for (&pos, _) in mdd[depth].clone().iter() {
            for neighbor in map.get_neighbors(pos.0, pos.1, true) {
                if constraints
                    .iter()
                    .any(|c| c.is_violated(pos, neighbor, depth + 1))
                {
                    continue;
                }

                if map.heuristic[agent.id][neighbor.0][neighbor.1] <= optimal_cost - (depth + 1) {
                    let next_node = mdd[depth + 1].entry(neighbor).or_insert(MddNode {
                        parents: HashSet::new(),
                        children: HashSet::new(),
                    });
                    next_node.parents.insert(pos);
                    mdd[depth].get_mut(&pos).unwrap().children.insert(neighbor);
                }
            }
        }
    }

    assert_eq!(mdd[optimal_cost].len(), 1);
    assert!(mdd[optimal_cost].contains_key(&agent.goal));

    // Backward pass
    for depth in (0..optimal_cost).rev() {
        let next_layer = mdd[depth + 1].clone();
        mdd[depth].retain(|_, node| {
            node.children
                .iter()
                .any(|child| next_layer.contains_key(child))
        });
    }

    mdd
}
