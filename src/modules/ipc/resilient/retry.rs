//! Retry Logic Implementation
//!
//! This module provides retry logic for resilient IPC clients.

use std::future::Future;
use std::time::{Duration, Instant};

use tokio::time::sleep;
use tracing::{debug, warn};

use crate::modules::ipc::IpcError;
use crate::modules::router_core::retry::RetryPolicy;

/// Retry handler for resilient clients
#[derive(Debug, Clone)]
pub struct RetryHandler {
    /// Retry policy
    policy: RetryPolicy,
}

impl RetryHandler {
    /// Create a new retry handler
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, IpcError>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = Result<T, IpcError>> + Send,
        T: Send,
    {
        let start_time = Instant::now();
        let mut attempt = 0;
        let max_retries = self.max_retries();

        loop {
            attempt += 1;
            debug!(
                "Executing operation, attempt {}/{}",
                attempt,
                max_retries + 1
            );

            match operation().await {
                Ok(result) => {
                    let elapsed = start_time.elapsed();
                    debug!(
                        "Operation succeeded after {} attempts in {:?}",
                        attempt, elapsed
                    );
                    return Ok(result);
                }
                Err(error) => {
                    // Check if error is retryable
                    if !self.is_retryable(&error) {
                        debug!("Non-retryable error: {:?}", error);
                        return Err(error);
                    }

                    // Check if max retries reached
                    if attempt > max_retries {
                        warn!(
                            "Max retries ({}) reached after {:?}",
                            max_retries,
                            start_time.elapsed()
                        );
                        return Err(error);
                    }

                    // Calculate backoff duration
                    let backoff = self.calculate_backoff(attempt);
                    debug!(
                        "Operation failed, retrying in {:?} (attempt {}/{})",
                        backoff,
                        attempt,
                        max_retries + 1
                    );

                    // Wait for backoff duration
                    sleep(backoff).await;
                }
            }
        }
    }

    /// Calculate backoff duration based on retry policy and attempt number
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        match &self.policy {
            RetryPolicy::None => Duration::from_millis(0),
            RetryPolicy::Fixed { interval_ms, .. } => Duration::from_millis(*interval_ms),
            RetryPolicy::ExponentialBackoff {
                initial_interval_ms,
                backoff_factor,
                max_interval_ms,
                ..
            } => {
                let interval = (*initial_interval_ms as f64
                    * backoff_factor.powf((attempt - 1) as f64))
                    as u64;
                let capped_interval = interval.min(*max_interval_ms);
                Duration::from_millis(capped_interval)
            }
        }
    }

    /// Get the maximum number of retries from the policy
    fn max_retries(&self) -> u32 {
        match &self.policy {
            RetryPolicy::None => 0,
            RetryPolicy::Fixed { max_retries, .. } => *max_retries,
            RetryPolicy::ExponentialBackoff { max_retries, .. } => *max_retries,
        }
    }

    /// Check if an error is retryable
    fn is_retryable(&self, error: &IpcError) -> bool {
        match error {
            IpcError::ConnectionError(_) => true,
            IpcError::Timeout(_) => true,
            IpcError::Unavailable(_) => true,
            IpcError::TransportError(_) => true,
            // Other errors are generally not retryable
            _ => false,
        }
    }

    /// Get the retry policy
    pub fn policy(&self) -> &RetryPolicy {
        &self.policy
    }
}
