//! Resilient IPC Client Configuration
//!
//! This module provides configuration structures and default settings for
//! retry policies and circuit breakers used by resilient IPC clients.

use std::time::Duration;

use crate::modules::router_core::retry::{CircuitBreakerConfig, DegradedServiceMode, RetryPolicy};

/// Default retry policy for IPC clients
pub fn default_retry_policy() -> RetryPolicy {
    RetryPolicy::ExponentialBackoff {
        initial_interval_ms: 100,
        backoff_factor: 2.0,
        max_retries: 3,
        max_interval_ms: 5000,
    }
}

/// Default circuit breaker configuration for IPC clients
pub fn default_circuit_breaker_config() -> CircuitBreakerConfig {
    CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        reset_timeout_ms: 30000, // 30 seconds
        enabled: true,
    }
}

/// Configuration for resilient clients
#[derive(Debug, Clone)]
pub struct ResilientClientConfig {
    /// Retry policy
    pub retry_policy: RetryPolicy,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Degraded service mode
    pub degraded_mode: DegradedServiceMode,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for ResilientClientConfig {
    fn default() -> Self {
        Self {
            retry_policy: default_retry_policy(),
            circuit_breaker: default_circuit_breaker_config(),
            degraded_mode: DegradedServiceMode::FailFast,
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
        }
    }
}
