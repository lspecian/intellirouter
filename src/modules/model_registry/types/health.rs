//! Model health status types
//!
//! This module defines types related to model health status.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Model health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelHealthStatus {
    /// Model is healthy and ready to use
    Healthy,
    /// Model is degraded but still operational
    Degraded(String),
    /// Model is unhealthy and should not be used
    Unhealthy(String),
}

impl Default for ModelHealthStatus {
    fn default() -> Self {
        ModelHealthStatus::Healthy
    }
}

impl fmt::Display for ModelHealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelHealthStatus::Healthy => write!(f, "Healthy"),
            ModelHealthStatus::Degraded(reason) => write!(f, "Degraded: {}", reason),
            ModelHealthStatus::Unhealthy(reason) => write!(f, "Unhealthy: {}", reason),
        }
    }
}

impl From<super::super::health::HealthCheckResult> for ModelHealthStatus {
    fn from(result: super::super::health::HealthCheckResult) -> Self {
        if result.success {
            ModelHealthStatus::Healthy
        } else {
            if let Some(error) = result.error_message {
                ModelHealthStatus::Unhealthy(error)
            } else {
                ModelHealthStatus::Unhealthy("Unknown error".to_string())
            }
        }
    }
}
