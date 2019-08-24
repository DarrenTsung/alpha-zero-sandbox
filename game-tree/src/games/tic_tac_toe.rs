use crate::GameTreeNode;

#[derive(Debug, Clone, Copy)]
pub enum Player {
    X = 0,
    O = 1,
}

impl Player {
    fn next(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

pub struct TicTacToeState {
    /// The state of a game of tic-tac-toe can be
    /// represented as an length-9 array of slot states (integers).
    board: [Option<Player>; 9],

    /// The player whose turn it is.
    current_player: Player,
}

impl GameTreeNode for TicTacToeState {
    type Node = TicTacToeState;

    fn children(&self) -> Vec<Self::Node> {
        let mut child_nodes = vec![];

        for (i, slot) in self.board.iter().enumerate() {
            // Cannot play any actions if the slot is occupied.
            if slot.is_some() {
                continue;
            }

            let mut new_board = self.board.clone();
            new_board[i] = Some(self.current_player);
            child_nodes.push(TicTacToeState {
                board: new_board,
                current_player: self.current_player.next(),
            });
        }

        child_nodes
    }
}
