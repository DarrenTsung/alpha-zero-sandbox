
use std::time::Duration;
use monte_carlo_tree_search::{SearchConfig, SearchTree};
use game_tree::GameTreeNode;
use game_tree_strategy::Strategy;
use std::fmt;

use crate::StrategyIterator;

pub struct SearchTreeIterationIterator<N> {
    tree: SearchTree,
    root: N,

    step_iterations: u64,
    exploitation_factor: f64,
    total_iterations: usize,

    current_iteration: usize,
}

impl<N> SearchTreeIterationIterator<N> {
    pub fn new(root: N, step_iterations: u64, exploitation_factor: f64, total_iterations: usize) -> Self {
        Self {
            tree: SearchTree::new(),
            root,

            step_iterations,
            exploitation_factor,
            total_iterations,

            current_iteration: 0,
        }
    }
}

impl<N: GameTreeNode<Node = N> + 'static> fmt::Display for SearchTreeIterationIterator<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MCSearchTree(exploit_f={}, iters={}, fully={})",
            self.exploitation_factor,
            self.current_iteration as u64 * self.step_iterations,
            self.tree.number_of_fully_explored_nodes(self.root.clone())
        )
    }
}

impl<N: GameTreeNode<Node = N> + 'static> Iterator for SearchTreeIterationIterator<N> {
    type Item = Box<dyn Strategy<N>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_iteration >= self.total_iterations {
            return None;
        }

        self.current_iteration += 1;

        self.tree.search(self.root.clone(), SearchConfig {
            max_duration: Duration::from_secs(60),
            max_iterations: self.step_iterations,
            exploitation_factor: self.exploitation_factor,
        });

        Some(Box::new(self.tree.clone()))
    }
}

impl<N: GameTreeNode<Node = N> + 'static> StrategyIterator<N> for SearchTreeIterationIterator<N> {}
