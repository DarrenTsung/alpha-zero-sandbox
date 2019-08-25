use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};

pub struct NodeMetadata {
    total_reward: AtomicI64,
    number_of_visits: AtomicU32,

    visited: AtomicBool,
    fully_expanded: AtomicBool,
}

impl NodeMetadata {
    pub fn new() -> Self {
        Self {
            total_reward: AtomicI64::new(0),
            number_of_visits: AtomicU32::new(0),

            visited: AtomicBool::new(false),
            fully_expanded: AtomicBool::new(false),
        }
    }

    pub fn record_result(&self, reward: i64) {
        self.number_of_visits.fetch_add(1, Ordering::SeqCst);
        self.total_reward.fetch_add(reward, Ordering::SeqCst);

        self.visited.store(true, Ordering::SeqCst);
    }

    fn number_of_visits(&self) -> u32 {
        self.number_of_visits.load(Ordering::SeqCst)
    }

    pub fn uct(&self, parent_metadata: &NodeMetadata, exploration_factor: f64) -> f64 {
        // Potentially inaccurate due to the non-atomic loading of all of these values
        // But IMO not harmful to the guarantees of the search.
        let parent_number_of_visits = parent_metadata.number_of_visits() as f64;
        let number_of_visits = self.number_of_visits() as f64;
        let total_reward = self.total_reward.load(Ordering::SeqCst) as f64;

        let exploitation_component = total_reward / number_of_visits;
        let exploration_component =
            exploration_factor * (number_of_visits.log10() / parent_number_of_visits).sqrt();

        exploitation_component + exploration_component
    }

    pub fn is_visited(&self) -> bool {
        self.visited.load(Ordering::SeqCst)
    }

    pub fn set_fully_expanded(&self) {
        self.fully_expanded.store(true, Ordering::SeqCst);
    }

    pub fn is_fully_expanded(&self) -> bool {
        self.fully_expanded.load(Ordering::SeqCst)
    }
}
