mod astar;
mod astarfocal;

pub(crate) use astar::{a_star_search, standard_a_star_search};
pub(crate) use astarfocal::focal_a_star_search;

use std::collections::HashMap;

use crate::common::Path;

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
