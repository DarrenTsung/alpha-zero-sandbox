use game_tree::GameTreeNode;
use rand::seq::IteratorRandom;

use crate::Strategy;

pub struct RandomStrategy;

impl<N: GameTreeNode> Strategy<N> for RandomStrategy {
    fn select_child(&self, children: Vec<N>) -> N {
        children
            .into_iter()
            .choose(&mut rand::thread_rng())
            .expect("array is not empty")
    }
}
