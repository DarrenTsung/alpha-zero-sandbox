use game_tree::GameTreeNode;

pub trait Strategy<N: GameTreeNode<Node = N>> {
    fn select_child(children: Vec<N>) -> N;
}
