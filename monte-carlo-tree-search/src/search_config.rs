use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub max_duration: Duration,
    pub max_iterations: u64,

    pub exploration_factor: f64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::from_secs(1),
            max_iterations: 1_000,

            exploration_factor: 0.5,
        }
    }
}
