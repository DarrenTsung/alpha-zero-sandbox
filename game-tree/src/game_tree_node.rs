use std::hash::Hash;

pub trait GameTreeNode: Hash + Clone {
    type Node;

    fn children(&self) -> Vec<Self::Node>;
}
