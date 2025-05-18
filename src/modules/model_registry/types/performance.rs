//! Model performance characteristics

use serde::{Deserialize, Serialize};

/// Model performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPerformance {
    /// Average latency in milliseconds
    pub avg_latency_ms: Option<f64>,
    /// P95 latency in milliseconds
    pub p95_latency_ms: Option<f64>,
    /// P99 latency in milliseconds
    pub p99_latency_ms: Option<f64>,
    /// Tokens per second for generation
    pub tokens_per_second: Option<f64>,
    /// Maximum requests per minute
    pub max_requests_per_minute: Option<u32>,
    /// Maximum tokens per minute
    pub max_tokens_per_minute: Option<u32>,
}
