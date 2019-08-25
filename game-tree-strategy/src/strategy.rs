use game_tree::GameTreeNode;

pub trait Strategy<N: GameTreeNode> {
    fn select_child(&self, children: Vec<N>) -> N;
}
