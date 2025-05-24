//! Validation Configuration
//!
//! This module provides configuration structures for the validation workflow.

use serde::{Deserialize, Serialize};

/// Validation workflow configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationConfig {
    /// Whether to validate service discovery
    pub validate_service_discovery: bool,
    /// Whether to validate direct communication
    pub validate_direct_communication: bool,
    /// Whether to validate end-to-end flows
    pub validate_end_to_end_flows: bool,
    /// Whether to validate data integrity
    pub validate_data_integrity: bool,
    /// Whether to validate error handling
    pub validate_error_handling: bool,
    /// Whether to validate security
    pub validate_security: bool,
    /// Timeout for validation operations in seconds
    pub validation_timeout_secs: u64,
    /// Whether to fail fast on validation errors
    pub fail_fast: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_service_discovery: true,
            validate_direct_communication: true,
            validate_end_to_end_flows: true,
            validate_data_integrity: true,
            validate_error_handling: true,
            validate_security: true,
            validation_timeout_secs: 120, // 2 minutes
            fail_fast: true,
        }
    }
}

/// Create a new validation configuration with custom settings
pub fn create_validation_config(
    validate_service_discovery: bool,
    validate_direct_communication: bool,
    validate_end_to_end_flows: bool,
    validate_data_integrity: bool,
    validate_error_handling: bool,
    validate_security: bool,
    validation_timeout_secs: u64,
    fail_fast: bool,
) -> ValidationConfig {
    ValidationConfig {
        validate_service_discovery,
        validate_direct_communication,
        validate_end_to_end_flows,
        validate_data_integrity,
        validate_error_handling,
        validate_security,
        validation_timeout_secs,
        fail_fast,
    }
}
