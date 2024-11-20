mod astar;
mod astarfocal;

pub(super) use astar::a_star_search;
pub(super) use astarfocal::{focal_a_star_double_search, focal_a_star_search};

use std::{collections::HashMap, usize};

type Trace = HashMap<((usize, usize), usize), ((usize, usize), usize)>;

fn heuristic_focal(
    agent: usize,
    position: (usize, usize),
    time: usize,
    paths: &[Vec<(usize, usize)>],
) -> usize {
    let mut conflict_count = 0;

    for (agent_id, path) in paths.iter().enumerate() {
        if agent_id == agent {
            continue; // Skip the current agent to avoid self-conflict.
        }

        let other_position = path.get(time).unwrap_or_else(|| path.last().unwrap());

        // Check if the current agent's position conflicts with the other agent's position.
        if *other_position == position {
            conflict_count += 1;
        }
    }

    conflict_count
}

fn construct_path(trace: &Trace, mut current: ((usize, usize), usize)) -> Vec<(usize, usize)> {
    let mut path = vec![current.0];
    while let Some(&(pos, time)) = trace.get(&current) {
        path.push(pos);
        current = (pos, time);
    }
    path.reverse();
    path
}
