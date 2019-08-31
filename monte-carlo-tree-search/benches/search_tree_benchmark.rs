use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use criterion::black_box;

use game_tree::games::tic_tac_toe::TicTacToeState;
use monte_carlo_tree_search::{SearchConfig, SearchTree};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SearchTree - TicTacToe - 1,000 iters", |b| b.iter(|| {
        let root = TicTacToeState::new();
        let tree = SearchTree::new();
        black_box(tree.search(root, SearchConfig {
            max_duration: Duration::from_secs(60),
            max_iterations: 1_000,

            exploration_factor: 2.0_f64.sqrt(),
        }));
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
