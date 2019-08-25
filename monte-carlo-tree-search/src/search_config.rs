use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub max_duration: Duration,
    pub max_iterations: u64,

    pub exploitation_factor: f64,
}
