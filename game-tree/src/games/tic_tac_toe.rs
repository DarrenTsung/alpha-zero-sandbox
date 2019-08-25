use crate::{GameTreeNode, NodeState};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TicTacToeState {
    /// The state of a game of tic-tac-toe can be
    /// represented as an length-9 array of slot states (integers).
    board: [Option<Player>; 9],

    /// The player whose turn it is.
    current_player: Player,
}

impl TicTacToeState {
    const WIN_INDICES: [[usize; 3]; 8] = [
        // Rows
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        // Columns
        [0, 3, 6],
        [1, 4, 7],
        [2, 5, 8],
        // Diagonals
        [0, 4, 8],
        [6, 4, 2],
    ];

    fn winner(&self) -> Option<Player> {
        'indices: for indices in &Self::WIN_INDICES {
            let mut winner_for_indices = None;
            for &index in indices {
                let slot = self.board[index];
                if slot.is_none() {
                    continue 'indices;
                }

                let player = slot.expect("is occupied");
                if let Some(winner_player) = winner_for_indices {
                    if winner_player != player {
                        continue 'indices;
                    }
                }

                winner_for_indices = Some(player);
            }

            if let Some(winner) = winner_for_indices {
                return Some(winner);
            }
        }

        None
    }
}

impl GameTreeNode for TicTacToeState {
    type Node = TicTacToeState;

    fn calculate_state(&self) -> NodeState<Self::Node> {
        // Return reward if there is a winner.
        match self.winner() {
            Some(Player::X) => return NodeState::Reward(1),
            Some(Player::O) => return NodeState::Reward(-1),
            _ => (),
        }

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

        // If no possible moves and no winners, than it is a tie.
        if child_nodes.is_empty() {
            NodeState::Reward(0)
        } else {
            NodeState::HasChildren(child_nodes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(s: Option<char>) -> Option<Player> {
        match s.expect("char exists") {
            'O' => Some(Player::O),
            'X' => Some(Player::X),
            ' ' => None,
            c => panic!("Unexpected input for slot(..) - got '{}'", c),
        }
    }

    fn board(row_0: &str, row_1: &str, row_2: &str) -> [Option<Player>; 9] {
        debug_assert_eq!(row_0.len(), 3);
        debug_assert_eq!(row_1.len(), 3);
        debug_assert_eq!(row_2.len(), 3);

        let mut board = [None; 9];
        board[0] = slot(row_0.chars().nth(0));
        board[1] = slot(row_0.chars().nth(1));
        board[2] = slot(row_0.chars().nth(2));
        board[3] = slot(row_1.chars().nth(0));
        board[4] = slot(row_1.chars().nth(1));
        board[5] = slot(row_1.chars().nth(2));
        board[6] = slot(row_2.chars().nth(0));
        board[7] = slot(row_2.chars().nth(1));
        board[8] = slot(row_2.chars().nth(2));
        board
    }

    #[test]
    fn state_has_children_works() {
        #[rustfmt::skip]
        let initial_board = board(
            " XX",
            "OOX",
            " XO"
        );

        let initial_state = TicTacToeState {
            board: initial_board,
            current_player: Player::X,
        };

        let children = match initial_state.calculate_state() {
            NodeState::HasChildren(children) => children,
            s => panic!("expected NodeState::HasChildren, got: {:?}", s),
        };
        assert_eq!(children.len(), 2, "expected 2 possible new states");

        #[rustfmt::skip]
        assert_eq!(children[0],
            TicTacToeState {
                board: board(
                    "XXX",
                    "OOX",
                    " XO",
                ),
                current_player: Player::O,
            }
        );

        #[rustfmt::skip]
        assert_eq!(children[1],
            TicTacToeState {
                board: board(
                    " XX",
                    "OOX",
                    "XXO",
                ),
                current_player: Player::O,
            }
        );
    }

    #[test]
    fn get_reward_from_finished_tictactoe_game() {
        #[rustfmt::skip]
        let initial_board = board(
            " XX",
            "OXO",
            "X O"
        );

        let initial_state = TicTacToeState {
            board: initial_board,
            current_player: Player::O,
        };

        let children = match initial_state.calculate_state() {
            NodeState::Reward(reward) => assert_eq!(reward, 1, "player x gives 1 reward"),
            s => panic!("expected NodeState::Reward, got {:?}", s),
        };
    }

    #[test]
    fn get_no_reward_from_tied_tictactoe_game() {
        #[rustfmt::skip]
        let initial_board = board(
            "OOX",
            "XXO",
            "OXO"
        );

        let initial_state = TicTacToeState {
            board: initial_board,
            current_player: Player::O,
        };

        let children = match initial_state.calculate_state() {
            NodeState::Reward(reward) => assert_eq!(reward, 0, "ties give 0 reward"),
            s => panic!("expected NodeState::Reward, got {:?}", s),
        };
    }
}
