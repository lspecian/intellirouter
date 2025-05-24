//! Resilient Client Traits
//!
//! This module defines common traits for resilient IPC clients.

use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;

use crate::modules::ipc::IpcResult;
use crate::modules::router_core::retry::{CircuitBreakerConfig, RetryPolicy};

/// Common trait for all resilient clients
pub trait ResilientClient: Send + Sync + Debug {
    /// Get the name of the service this client connects to
    fn service_name(&self) -> &str;

    /// Get the retry policy for this client
    fn retry_policy(&self) -> &RetryPolicy;

    /// Get the circuit breaker configuration for this client
    fn circuit_breaker_config(&self) -> &CircuitBreakerConfig;

    /// Check if the client is in a healthy state
    fn is_healthy(&self) -> bool;

    /// Get the last successful connection time
    fn last_successful_connection(&self) -> Option<Duration>;

    /// Get the number of consecutive failures
    fn consecutive_failures(&self) -> u32;

    /// Get the number of successful operations
    fn successful_operations(&self) -> u64;

    /// Get the number of failed operations
    fn failed_operations(&self) -> u64;

    /// Reset the circuit breaker
    fn reset_circuit_breaker(&mut self);
}

/// Trait for resilient clients that support health checks
#[async_trait]
pub trait HealthCheckable {
    /// Perform a health check
    async fn health_check(&self) -> IpcResult<()>;
}

/// Trait for resilient clients that support graceful shutdown
#[async_trait]
pub trait GracefulShutdown {
    /// Perform a graceful shutdown
    async fn shutdown(&self) -> IpcResult<()>;
}
