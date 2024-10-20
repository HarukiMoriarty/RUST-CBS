use super::common::HighLevelNode;
use super::Solver;
use crate::common::{Agent, Path, Solution};
use crate::map::Map;

use std::collections::BinaryHeap;
pub struct CBS {
    agents: Vec<Agent>,
    map: Map,
}

impl Solver for CBS {
    fn new(agents: Vec<Agent>, map: Map) -> Self {
        CBS { agents, map }
    }

    fn solve(&self) -> Solution {
        let mut open = BinaryHeap::new();
        let root = HighLevelNode::new(&self.agents, &self.map);

        open.push(root);

        while let Some(current_node) = open.pop() {
            if let Some(conflicts) = current_node.detect_conflicts() {
                for conflict in conflicts {
                    let child_1 = current_node.update_constraint(&conflict, true, &self.map);
                    let child_2 = current_node.update_constraint(&conflict, false, &self.map);

                    open.push(child_1);
                    open.push(child_2);
                }
            }
            else {
                // No conflicts, return solution
                let paths = current_node.paths.into_values()
                    .map(|path| Path { steps: path })
                    .collect();
                return Solution { paths };
            }
        }

        Solution { paths: Vec::new() }
    }
}
