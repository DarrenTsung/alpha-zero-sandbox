use game_tree::GameTreeNode;
use monte_carlo_tree_search::{SearchConfig, SearchTree};
use std::fmt;
use std::time::Duration;

use crate::Strategy;

pub struct LearningSearchTreeStrategy<N> {
    tree: SearchTree,
    root: N,

    iterations_per_select: u64,
    exploration_factor: f64,
}

impl<N> LearningSearchTreeStrategy<N> {
    pub fn new(root: N, iterations_per_select: u64, exploration_factor: f64) -> Self {
        Self {
            tree: SearchTree::new(),
            root,

            iterations_per_select,
            exploration_factor,
        }
    }
}

impl<N: GameTreeNode<Node = N> + 'static> fmt::Display for LearningSearchTreeStrategy<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MCSearchTree(explore_f={}, fully={})",
            self.exploration_factor,
            self.tree.number_of_fully_expanded_nodes(self.root.clone())
        )
    }
}

impl<N: GameTreeNode<Node = N> + 'static> Strategy<N> for LearningSearchTreeStrategy<N> {
    fn select_child(&self, parent: N, children: Vec<N>) -> N {
        self.tree.search(
            parent.clone(),
            SearchConfig {
                max_duration: Duration::from_secs(5),
                max_iterations: self.iterations_per_select,

                exploration_factor: self.exploration_factor,
            },
        );

        self.tree.select_most_visited_child(children)
    }
}
