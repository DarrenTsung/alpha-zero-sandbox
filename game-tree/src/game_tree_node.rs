use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub enum NodeState<N> {
    HasChildren(Vec<N>),
    Reward(i64),
}

pub trait GameTreeNode: Debug + Hash + Clone + Send + Sync {
    type Node: GameTreeNode;

    fn calculate_state(&self) -> NodeState<Self::Node>;
}
