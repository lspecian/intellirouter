//! Validation Types
//!
//! This module provides type definitions for the validation workflow.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationResult {
    /// Validation type
    pub validation_type: ValidationType,
    /// Whether the validation was successful
    pub success: bool,
    /// Error message if the validation failed
    pub error: Option<String>,
    /// Validation duration in milliseconds
    pub duration_ms: u64,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Validation details
    pub details: HashMap<String, serde_json::Value>,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success(
        validation_type: ValidationType,
        duration_ms: u64,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            validation_type,
            success: true,
            error: None,
            duration_ms,
            timestamp: chrono::Utc::now(),
            details,
        }
    }

    /// Create a new failed validation result
    pub fn failure(
        validation_type: ValidationType,
        error: String,
        duration_ms: u64,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            validation_type,
            success: false,
            error: Some(error),
            duration_ms,
            timestamp: chrono::Utc::now(),
            details,
        }
    }
}

/// Validation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ValidationType {
    /// Service discovery validation
    ServiceDiscovery,
    /// Direct communication validation
    DirectCommunication,
    /// End-to-end flow validation
    EndToEndFlow,
    /// Data integrity validation
    DataIntegrity,
    /// Error handling validation
    ErrorHandling,
    /// Security validation
    Security,
}

impl std::fmt::Display for ValidationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationType::ServiceDiscovery => write!(f, "Service Discovery"),
            ValidationType::DirectCommunication => write!(f, "Direct Communication"),
            ValidationType::EndToEndFlow => write!(f, "End-to-End Flow"),
            ValidationType::DataIntegrity => write!(f, "Data Integrity"),
            ValidationType::ErrorHandling => write!(f, "Error Handling"),
            ValidationType::Security => write!(f, "Security"),
        }
    }
}

/// Validation test result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationTestResult {
    /// Test name
    pub name: String,
    /// Test status
    pub success: bool,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Test duration in milliseconds
    pub duration_ms: u64,
    /// Test details
    pub details: HashMap<String, serde_json::Value>,
}

impl ValidationTestResult {
    /// Create a new successful test result
    pub fn _success(
        name: String,
        duration_ms: u64,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            name,
            success: true,
            error: None,
            duration_ms,
            details,
        }
    }

    /// Create a new failed test result
    pub fn _failure(
        name: String,
        error: String,
        duration_ms: u64,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            name,
            success: false,
            error: Some(error),
            duration_ms,
            details,
        }
    }
}

/// Test flow step
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestFlow {
    /// Flow name
    pub name: String,
    /// Flow description
    pub description: String,
    /// Flow steps
    pub steps: Vec<Step>,
}

/// Test flow step
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Step {
    /// API call step
    Step {
        /// Step name
        name: String,
        /// Service type
        service_type: crate::modules::audit::types::ServiceType,
        /// Endpoint
        endpoint: String,
        /// HTTP method
        method: String,
        /// Request payload
        payload: Option<serde_json::Value>,
        /// Expected status code
        expected_status: u16,
    },
}
