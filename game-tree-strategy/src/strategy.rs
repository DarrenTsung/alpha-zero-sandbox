use game_tree::GameTreeNode;

pub trait Strategy<N: GameTreeNode> {
    fn select_child(&self, parent: N, children: Vec<N>) -> N;
}
