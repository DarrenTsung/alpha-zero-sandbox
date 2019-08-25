use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub enum NodeState<N> {
    HasChildren(Vec<N>),
    Reward(i64),
}

pub trait GameTreeNode: Debug + Hash + Clone + Send + Sync {
    type Node: GameTreeNode;

    fn is_self_turn(&self) -> bool;
    fn calculate_state(&self) -> NodeState<Self::Node>;
}
