use structopt::{self, StructOpt};
use std::fmt;
use strum_macros::EnumString;
use game_tree_strategy::{strategies::random::RandomStrategy, Strategy};
use game_tree::{NodeState, GameTreeNode};
use std::collections::HashMap;

mod search_tree;
use self::search_tree::SearchTreeIterationIterator;

#[derive(Debug, StructOpt)]
#[structopt(name = "compare-strategies", about = "A CLI tool to help compare game tree strategies.")]
struct Opt {
    #[structopt(short = "s", long = "strategy")]
    pub strategy_to_compare: StrategyType,

    #[structopt(short = "g", long = "game")]
    pub game: GameType,
}

#[derive(Debug, EnumString)]
enum StrategyType {
    #[strum(serialize="MCTS")]
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

trait StrategyIterator<N>: fmt::Display + Iterator<Item = Box<dyn Strategy<N>>> {}

fn main_ty<N: GameTreeNode<Node = N> + 'static>(opt: Opt, root_node: N) {
    let strategy_iterators: Vec<Box<dyn StrategyIterator<N>>> = match opt.strategy_to_compare {
        StrategyType::MonteCarloSearchTree => {
            [0.1, 0.3, 0.5, 0.7, 0.9].iter().map(|exploration_factor| {
                Box::new(SearchTreeIterationIterator::new(root_node.clone(), 100, *exploration_factor, 30)) as Box<dyn StrategyIterator<N>>
            }).collect::<Vec<_>>()
        },
    };

    let random_strategy = RandomStrategy;
    for mut s_iterator in strategy_iterators {
        while let Some(strategy) = s_iterator.next() {
            let mut total_reward = 0;
            let mut reward_counts = HashMap::new();

            // Play 1,000 games against random strategy
            for _ in 0..1_000 {
                let mut current = root_node.clone();

                loop {
                    match current.calculate_state() {
                        NodeState::Reward(reward) => {
                            total_reward += reward;
                            *reward_counts.entry(reward).or_insert(0) += 1;
                            break;
                        },
                        NodeState::HasChildren(children) => {
                            current = if current.is_self_turn() {
                                strategy.select_child(children)
                            } else {
                                random_strategy.select_child(children)
                            };
                        },
                    }
                }
            }

            let mut reward_counts_strings = reward_counts.into_iter().map(|(reward, count)| {
                format!("{} - {}", reward, count)
            }).collect::<Vec<_>>();
            reward_counts_strings.sort();
            println!("{} - total={} ({})", s_iterator, total_reward, reward_counts_strings.join(", "));
        }
    }
}
