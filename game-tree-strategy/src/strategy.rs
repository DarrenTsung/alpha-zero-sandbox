use game_tree::GameTreeNode;

pub trait Strategy<N: GameTreeNode> {
    fn select_child(&mut self, children: Vec<N>) -> N;
}
