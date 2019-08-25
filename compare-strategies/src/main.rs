use game_tree::{GameTreeNode, NodeState};
use game_tree_strategy::strategies::random::RandomStrategy;
use game_tree_strategy::strategies::search_tree::LearningSearchTreeStrategy;
use game_tree_strategy::Strategy;
use std::collections::HashMap;
use std::fmt;
use structopt::{self, StructOpt};
use strum_macros::EnumString;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "compare-strategies",
    about = "A CLI tool to help compare game tree strategies."
)]
struct Opt {
    #[structopt(short = "s", long = "strategy")]
    pub strategy_to_compare: StrategyType,

    #[structopt(short = "g", long = "game")]
    pub game: GameType,
}

#[derive(Debug, EnumString)]
enum StrategyType {
    #[strum(serialize = "MCTS")]
    MonteCarloSearchTree,
}

#[derive(Debug, EnumString)]
enum GameType {
    TicTacToe,
}

fn main() {
    let opt = Opt::from_args();
    let root_node = match opt.game {
        GameType::TicTacToe => game_tree::games::tic_tac_toe::TicTacToeState::new(),
    };

    main_ty(opt, root_node);
}

fn main_ty<N: GameTreeNode<Node = N> + 'static>(opt: Opt, root_node: N) {
    match opt.strategy_to_compare {
        StrategyType::MonteCarloSearchTree => {
            let iterations_per_search = 1_000;
            let exploration_factor = 2.0_f64.sqrt();
            let strategies = vec![LearningSearchTreeStrategy::new(
                root_node.clone(),
                iterations_per_search,
                exploration_factor,
            )];

            run_games(root_node, strategies);
        }
    }
}

fn run_games<N: GameTreeNode<Node = N> + 'static, S: fmt::Display + Strategy<N>>(
    root_node: N,
    strategies: Vec<S>,
) {
    let random_strategy = RandomStrategy;
    for strategy in strategies {
        let mut total_reward = 0;
        let mut reward_counts = HashMap::new();

        // Play games against random strategy
        for _ in 0..100 {
            let mut current = root_node.clone();

            loop {
                match current.calculate_state() {
                    NodeState::Reward(reward) => {
                        total_reward += reward;
                        *reward_counts.entry(reward).or_insert(0) += 1;
                        break;
                    }
                    NodeState::HasChildren(children) => {
                        current = if current.is_self_turn() {
                            strategy.select_child(current, children)
                        } else {
                            random_strategy.select_child(current, children)
                        };
                    }
                }
            }
        }

        let mut reward_counts_strings = reward_counts
            .into_iter()
            .map(|(reward, count)| format!("{} - {}", reward, count))
            .collect::<Vec<_>>();
        reward_counts_strings.sort();
        println!(
            "{} - total={} ({})",
            strategy,
            total_reward,
            reward_counts_strings.join(", ")
        );
    }
}
