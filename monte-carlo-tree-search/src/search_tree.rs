use antidote::Mutex;
use game_tree::{GameTreeNode, NodeState};
use game_tree_strategy::Strategy;
use ordered_float::OrderedFloat;
use rand::seq::{IteratorRandom, SliceRandom};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crate::node_metadata::NodeMetadata;
use crate::SearchConfig;

type MetadataMap = HashMap<u64, Arc<NodeMetadata>>;

#[derive(Clone)]
pub struct SearchTree {
    node_metadata: Arc<Mutex<MetadataMap>>,
}

impl SearchTree {
    pub fn new() -> Self {
        Self {
            node_metadata: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Explore more of the SearchTree from the node provided
    /// with the given search configuration.
    pub fn search<N: GameTreeNode<Node = N> + 'static>(&self, node: N, config: SearchConfig) {
        let number_iterations = Arc::new(AtomicU64::new(0));
        let mut handles = vec![];
        for _ in 0..num_cpus::get() {
            let node = node.clone();
            let task = SearchTask {
                tree: self.clone(),
                number_iterations: Arc::clone(&number_iterations),
                config: config.clone(),
            };
            handles.push(thread::spawn(|| task.run(node)));
        }

        for h in handles {
            h.join().expect("no panics");
        }
    }
}

impl<N: GameTreeNode> Strategy<N> for SearchTree {
    fn select_child(&mut self, children: Vec<N>) -> N {
        let node_metadata = self.node_metadata.lock();
        children
            .into_iter()
            .max_by_key(|c| {
                node_metadata
                    .get(&hash(c))
                    .map(|meta| meta.number_of_visits())
                    .unwrap_or(0)
            })
            .expect("array is not empty")
    }
}

struct SearchTask {
    tree: SearchTree,
    number_iterations: Arc<AtomicU64>,
    config: SearchConfig,
}

impl SearchTask {
    fn run<N: GameTreeNode<Node = N>>(self, node: N) {
        let start = Instant::now();
        let mut rand = rand::thread_rng();

        let mut local_node_metadata = HashMap::new();
        macro_rules! load_metadata {
            ($node:expr) => {
                Self::load_metadata($node, &mut local_node_metadata, &self.tree.node_metadata)
            };
        }

        #[derive(Debug, PartialEq)]
        enum State {
            NodesFullyExpanded,
            SelectSimulation,
            InSimulation,
        }

        'run: loop {
            if self.number_iterations.load(Ordering::SeqCst) > self.config.max_iterations {
                break 'run;
            }

            let mut visited = vec![];
            let mut current = (node.clone(), load_metadata!(&node));

            let mut state = State::NodesFullyExpanded;
            let reward = loop {
                if start.elapsed() > self.config.max_duration {
                    break 'run;
                }

                let mut children = match current.0.calculate_state() {
                    NodeState::Reward(reward) => break reward,
                    NodeState::HasChildren(children) => children
                        .into_iter()
                        .map(|c| {
                            let metadata = load_metadata!(&c);
                            (c, metadata)
                        })
                        .collect::<Vec<_>>(),
                };

                debug_assert!(!children.is_empty());

                let chosen_child = loop {
                    match state {
                        State::NodesFullyExpanded => {
                            if !current.1.is_fully_expanded() {
                                // If cached check fails, ensure that it is
                                // truly not fully expanded.
                                let all_children_visited =
                                    children.iter().all(|(_, meta)| meta.is_visited());
                                if all_children_visited {
                                    current.1.set_fully_expanded();
                                } else {
                                    state = State::SelectSimulation;
                                    continue;
                                }
                            }

                            break children
                                .into_iter()
                                .max_by_key(|(_, meta)| {
                                    OrderedFloat(
                                        meta.uct(&current.1, self.config.exploitation_factor),
                                    )
                                })
                                .expect("array is not empty");
                        }
                        State::SelectSimulation => {
                            let non_visited_indices = children
                                .iter()
                                .enumerate()
                                .filter_map(|(i, (_, meta))| {
                                    if meta.is_visited() {
                                        return None;
                                    }

                                    Some(i)
                                })
                                .collect::<Vec<_>>();

                            if non_visited_indices.is_empty() {
                                state = State::NodesFullyExpanded;
                                continue;
                            }

                            let index = non_visited_indices.choose(&mut rand).expect("not empty");
                            break children.remove(*index);
                        }
                        State::InSimulation => {
                            break children
                                .into_iter()
                                .choose(&mut rand)
                                .expect("array is not empty");
                        }
                    }
                };

                visited.push((current, state == State::InSimulation));
                current = chosen_child;
            };

            // After search has reached some terminal node with a reward,
            // back-propagate the reward along any fully expanded nodes.
            for ((_node, metadata), in_simulation) in visited {
                // Don't record nodes where were reached through simulation.
                if in_simulation {
                    continue;
                }

                metadata.record_result(reward);
            }

            self.number_iterations.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn load_metadata(
        node: &impl GameTreeNode,
        local: &mut MetadataMap,
        synchronized: &Arc<Mutex<MetadataMap>>,
    ) -> Arc<NodeMetadata> {
        let hash = hash(node);
        if let Some(metadata) = local.get(&hash) {
            return Arc::clone(&metadata);
        }

        let metadata = synchronized
            .lock()
            .entry(hash)
            .or_insert_with(|| Arc::new(NodeMetadata::new()))
            .clone();
        local.insert(hash, Arc::clone(&metadata));
        metadata
    }
}

fn hash(node: &impl GameTreeNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}
