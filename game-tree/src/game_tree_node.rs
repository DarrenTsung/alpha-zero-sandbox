use std::hash::Hash;

pub trait GameTreeNode: Hash {
    type Node;

    fn children(&self) -> Vec<Self::Node>;
}
