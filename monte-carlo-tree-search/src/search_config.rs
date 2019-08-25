use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub max_duration: Duration,

    pub exploitation_factor: f64,
}
