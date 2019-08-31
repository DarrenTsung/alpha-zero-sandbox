use ccl::dhashmap::DHashMap;
use game_tree::{GameTreeNode, NodeState};
use ordered_float::OrderedFloat;
use rand::seq::{IteratorRandom, SliceRandom};
use rayon::prelude::*;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::node_metadata::NodeMetadata;
use crate::SearchConfig;

type MetadataMap = DHashMap<u64, Arc<NodeMetadata>>;

#[derive(Clone)]
pub struct SearchTree {
    node_metadata: Arc<MetadataMap>,
}

impl SearchTree {
    pub fn new() -> Self {
        Self {
            node_metadata: Arc::new(DHashMap::default()),
        }
    }

    /// Explore more of the SearchTree from the node provided
    /// with the given search configuration.
    pub fn search<N: GameTreeNode<Node = N> + 'static>(&self, node: N, config: SearchConfig) {
        let number_iterations = Arc::new(AtomicU64::new(0));
        let _: () = (0..num_cpus::get())
            .into_par_iter()
            .map(|_| {
                let node = node.clone();
                let task = SearchTask {
                    tree: self.clone(),
                    number_iterations: Arc::clone(&number_iterations),
                    config: config.clone(),
                };
                task.run(node);
                ()
            })
            .collect();
    }

    pub fn number_of_fully_expanded_nodes<N: GameTreeNode<Node = N> + 'static>(
        &self,
        node: N,
    ) -> u64 {
        let mut count = 0;
        let mut queue = vec![node];
        while let Some(node) = queue.pop() {
            let is_fully_expanded = self
                .node_metadata
                .get(&hash(&node))
                .map(|meta| meta.is_fully_expanded())
                .unwrap_or(false);
            if !is_fully_expanded {
                continue;
            }

            count += 1;
            if let NodeState::HasChildren(children) = node.calculate_state() {
                queue.extend(children);
            }
        }
        count
    }

    pub fn select_most_visited_child<N: GameTreeNode>(&self, children: Vec<N>) -> (u32, N) {
        children
            .into_iter()
            .map(|c| (self.get_number_of_visits(&c), c))
            .max_by_key(|(number_of_visits, _c)| *number_of_visits)
            .expect("array is not empty")
    }

    pub fn get_number_of_visits<N: GameTreeNode>(&self, node: &N) -> u32 {
        self.node_metadata
            .get(&hash(node))
            .map(|meta| meta.number_of_visits())
            .unwrap_or(0)
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

        let mut state_cache = lru::LruCache::new(100_000);
        macro_rules! load_metadata {
            ($node:expr) => {
                Self::load_metadata($node, &self.tree.node_metadata)
            };
        }

        #[derive(Debug, PartialEq, Clone, Copy)]
        enum State {
            NodesFullyExpanded,
            ChooseSimulationStart,
            InSimulation,
        }

        'run: loop {
            if self.number_iterations.load(Ordering::SeqCst) > self.config.max_iterations {
                break 'run;
            }

            struct Current<N> {
                node: N,
                metadata: Arc<NodeMetadata>,

                /// The action arriving to this node was from
                /// a node whose turn was 'self'.
                parent_was_self: bool,
                /// The node was reached through State::InSimulation
                chosen_from_simulation: bool,
            }

            let mut visited = vec![];
            let mut current = Current {
                node: node.clone(),
                metadata: load_metadata!(&node),
                parent_was_self: false,
                chosen_from_simulation: false,
            };

            let mut state = State::NodesFullyExpanded;
            let reward = loop {
                if start.elapsed() > self.config.max_duration {
                    break 'run;
                }

                let current_node_hash = hash(&current.node);
                let node_state: Cow<NodeState<N>> =
                    if let Some(node_state) = state_cache.get(&current_node_hash) {
                        Cow::Borrowed(node_state)
                    } else {
                        let state = current.node.calculate_state();
                        state_cache.put(current_node_hash, state);
                        Cow::Borrowed(state_cache.get(&current_node_hash).expect("exists"))
                    };

                use std::borrow::Borrow;
                let mut children = match node_state.borrow() {
                    NodeState::Reward(reward) => {
                        // Ensure that current node is marked as visited
                        // before reporting the reward.
                        visited.push(current);
                        break *reward;
                    }
                    NodeState::HasChildren(children) => children
                        .iter()
                        .map(|c| {
                            let metadata = load_metadata!(c);
                            (c, metadata)
                        })
                        .collect::<Vec<_>>(),
                };

                debug_assert!(!children.is_empty());

                let mut chosen_from_simulation = false;
                let chosen_child = loop {
                    match state {
                        State::NodesFullyExpanded => {
                            if !current.metadata.is_fully_expanded() {
                                // If cached check fails, ensure that it is
                                // truly not fully expanded.
                                let all_children_visited =
                                    children.iter().all(|(_, meta)| meta.is_visited());
                                if all_children_visited {
                                    current.metadata.set_fully_expanded();
                                } else {
                                    state = State::ChooseSimulationStart;
                                    continue;
                                }
                            }

                            break children
                                .into_iter()
                                .max_by_key(|(_, meta)| {
                                    OrderedFloat(
                                        meta.uct(&current.metadata, self.config.exploration_factor),
                                    )
                                })
                                .expect("array is not empty");
                        }
                        State::ChooseSimulationStart => {
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
                            state = State::InSimulation;
                            break children.remove(*index);
                        }
                        State::InSimulation => {
                            chosen_from_simulation = true;
                            break children
                                .into_iter()
                                .choose(&mut rand)
                                .expect("array is not empty");
                        }
                    }
                };

                let parent_was_self = current.node.is_self_turn();
                visited.push(current);
                current = Current {
                    node: chosen_child.0.clone(),
                    metadata: chosen_child.1,
                    parent_was_self,
                    chosen_from_simulation,
                };
            };

            // After search has reached some terminal node with a reward,
            // back-propagate the reward along any fully expanded nodes.
            for Current {
                node: _node,
                metadata,
                parent_was_self,
                chosen_from_simulation,
            } in visited
            {
                // Don't record nodes where were reached through simulation.
                if chosen_from_simulation {
                    continue;
                }

                // The reward is only positive if the action taken was
                // from the perspective of the self player.
                let self_factor = if parent_was_self { 1 } else { -1 };
                metadata.record_result(self_factor * reward);
            }

            self.number_iterations.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn load_metadata(node: &impl GameTreeNode, map: &MetadataMap) -> Arc<NodeMetadata> {
        let hash = hash(node);
        map.get_or_insert_with(&hash, || Arc::new(NodeMetadata::new()))
            .clone()
    }
}

fn hash(node: &impl GameTreeNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}
