use std::hash::Hash;

pub trait GameTreeNode: Hash + Clone + Send + Sync {
    type Node;

    fn children(&self) -> Vec<Self::Node>;
}
