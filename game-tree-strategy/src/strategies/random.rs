use game_tree::GameTreeNode;
use rand::seq::IteratorRandom;

use crate::Strategy;

pub struct RandomStrategy;

impl<N: GameTreeNode<Node = N>> Strategy<N> for RandomStrategy {
    fn select_child(children: Vec<N>) -> N {
        children
            .into_iter()
            .choose(&mut rand::thread_rng())
            .expect("array is not empty")
    }
}
