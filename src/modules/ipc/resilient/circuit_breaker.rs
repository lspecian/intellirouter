//! Circuit Breaker Implementation
//!
//! This module provides a circuit breaker implementation for resilient IPC clients.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tracing::{debug, warn};

use crate::modules::ipc::IpcError;
use crate::modules::router_core::retry::{
    CircuitBreakerConfig, DegradedServiceMode,
};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are not allowed
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current state
    state: Arc<Mutex<CircuitState>>,
    /// Number of consecutive failures
    consecutive_failures: AtomicU32,
    /// Number of consecutive successes
    consecutive_successes: AtomicU32,
    /// Total number of successful operations
    successful_operations: AtomicU64,
    /// Total number of failed operations
    failed_operations: AtomicU64,
    /// Last failure time
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    /// Last success time
    last_success_time: Arc<Mutex<Option<Instant>>>,
    /// Whether the circuit breaker is enabled
    enabled: AtomicBool,
    /// Degraded service mode
    degraded_mode: Arc<Mutex<DegradedServiceMode>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig, degraded_mode: DegradedServiceMode) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            consecutive_failures: AtomicU32::new(0),
            consecutive_successes: AtomicU32::new(0),
            successful_operations: AtomicU64::new(0),
            failed_operations: AtomicU64::new(0),
            last_failure_time: Arc::new(Mutex::new(None)),
            last_success_time: Arc::new(Mutex::new(None)),
            enabled: AtomicBool::new(true),
            degraded_mode: Arc::new(Mutex::new(degraded_mode)),
        }
    }

    /// Check if the circuit is allowed to execute
    pub fn allow_execution(&self) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return true;
        }

        match *self.state.lock().unwrap() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                // Check if reset timeout has elapsed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    let elapsed = last_failure.elapsed().as_millis() as u64;
                    if elapsed >= self.config.reset_timeout_ms {
                        debug!("Circuit breaker reset timeout elapsed, transitioning to half-open");
                        *self.state.lock().unwrap() = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    // No recorded failure time, allow execution
                    true
                }
            }
        }
    }

    /// Record a successful execution
    pub fn record_success(&self) {
        self.successful_operations.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.fetch_add(1, Ordering::Relaxed);
        *self.last_success_time.lock().unwrap() = Some(Instant::now());

        // If in half-open state and success threshold reached, transition to closed
        if *self.state.lock().unwrap() == CircuitState::HalfOpen {
            let successes = self.consecutive_successes.load(Ordering::Relaxed);
            if successes >= self.config.success_threshold {
                debug!("Circuit breaker success threshold reached, transitioning to closed");
                *self.state.lock().unwrap() = CircuitState::Closed;
                self.consecutive_successes.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Record a failed execution
    pub fn record_failure(&self, error: &IpcError) -> IpcError {
        self.failed_operations.fetch_add(1, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());

        // If failure threshold reached, open the circuit
        if failures >= self.config.failure_threshold && self.enabled.load(Ordering::Relaxed) {
            if *self.state.lock().unwrap() != CircuitState::Open {
                warn!("Circuit breaker failure threshold reached, transitioning to open");
                *self.state.lock().unwrap() = CircuitState::Open;
            }
        }

        // Return the original error
        error.clone()
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        *self.state.lock().unwrap() = CircuitState::Closed;
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        debug!("Circuit breaker reset to closed state");
    }

    /// Get the current state
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    /// Get the number of consecutive failures
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// Get the number of successful operations
    pub fn successful_operations(&self) -> u64 {
        self.successful_operations.load(Ordering::Relaxed)
    }

    /// Get the number of failed operations
    pub fn failed_operations(&self) -> u64 {
        self.failed_operations.load(Ordering::Relaxed)
    }

    /// Set the degraded service mode
    pub fn set_degraded_mode(&self, mode: DegradedServiceMode) {
        *self.degraded_mode.lock().unwrap() = mode;
    }

    /// Get the degraded service mode
    pub fn degraded_mode(&self) -> DegradedServiceMode {
        self.degraded_mode.lock().unwrap().clone()
    }

    /// Enable or disable the circuit breaker
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if the circuit breaker is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}
