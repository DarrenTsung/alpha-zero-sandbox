pub trait GameTreeNode {
    type Node;

    fn children(&self) -> Vec<Self::Node>;
}
