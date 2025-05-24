//! Validation Reporting
//!
//! This module provides functionality for generating validation reports.

use std::collections::HashMap;

use serde_json::json;
use tracing::{info, warn};

use crate::modules::audit::report::AuditReport;
use crate::modules::audit::types::AuditError;

use super::types::{ValidationResult, ValidationType};

/// Generate a validation report from validation results
pub async fn generate_validation_report(
    results: &[ValidationResult],
    report: &mut AuditReport,
) -> Result<(), AuditError> {
    info!(
        "Generating validation report from {} results",
        results.len()
    );

    if results.is_empty() {
        warn!("No validation results to generate report from");
        return Ok(());
    }

    // Count results by validation type and status
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut validation_counts = HashMap::new();

    for result in results {
        let validation_type = format!("{}", result.validation_type);

        // Update validation type counts
        let count = validation_counts.entry(validation_type).or_insert(0);
        *count += 1;

        // Update overall counts
        if result.success {
            success_count += 1;
        } else {
            failure_count += 1;
        }
    }

    // Calculate overall success rate
    let total_count = success_count + failure_count;
    let success_rate = if total_count > 0 {
        (success_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    // Add validation summary to report
    report.add_section(
        "validation_summary",
        json!({
            "total_validations": total_count,
            "successful_validations": success_count,
            "failed_validations": failure_count,
            "success_rate": success_rate,
            "validation_counts": validation_counts,
        }),
    );

    // Add detailed validation results to report
    let validation_details = results
        .iter()
        .map(|r| {
            json!({
                "type": format!("{}", r.validation_type),
                "success": r.success,
                "error": r.error,
                "duration_ms": r.duration_ms,
                "timestamp": r.timestamp,
                "details": r.details,
            })
        })
        .collect::<Vec<_>>();

    report.add_section("validation_details", json!(validation_details));

    // Add validation recommendations based on results
    let mut recommendations = Vec::new();

    for result in results {
        if !result.success {
            match result.validation_type {
                ValidationType::ServiceDiscovery => {
                    recommendations.push(
                        "Check service discovery configuration and ensure all services are running."
                            .to_string(),
                    );
                }
                ValidationType::DirectCommunication => {
                    recommendations.push(
                        "Verify network connectivity and firewall rules between services."
                            .to_string(),
                    );
                }
                ValidationType::EndToEndFlow => {
                    recommendations.push(
                        "Review end-to-end flow configurations and service dependencies."
                            .to_string(),
                    );
                }
                ValidationType::DataIntegrity => {
                    recommendations.push(
                        "Check data consistency mechanisms and validation rules.".to_string(),
                    );
                }
                ValidationType::ErrorHandling => {
                    recommendations.push(
                        "Improve error handling and recovery mechanisms in services.".to_string(),
                    );
                }
                ValidationType::Security => {
                    recommendations.push(
                        "Review security configurations and authentication mechanisms.".to_string(),
                    );
                }
            }
        }
    }

    if !recommendations.is_empty() {
        report.add_section("validation_recommendations", json!(recommendations));
    }

    info!("Validation report generated successfully");
    Ok(())
}

/// Generate a summary of validation results
pub fn _summarize_validation_results(results: &[ValidationResult]) -> String {
    let mut summary = String::new();

    if results.is_empty() {
        return "No validation results available.".to_string();
    }

    // Count results by validation type and status
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut validation_counts = HashMap::new();

    for result in results {
        let validation_type = format!("{}", result.validation_type);

        // Update validation type counts
        let count = validation_counts.entry(validation_type).or_insert(0);
        *count += 1;

        // Update overall counts
        if result.success {
            success_count += 1;
        } else {
            failure_count += 1;
        }
    }

    // Calculate overall success rate
    let total_count = success_count + failure_count;
    let success_rate = if total_count > 0 {
        (success_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    // Build summary string
    summary.push_str(&format!(
        "Validation Summary: {:.1}% Success Rate ({} of {} validations passed)\n\n",
        success_rate, success_count, total_count
    ));

    // Add validation type breakdown
    summary.push_str("Validation Types:\n");
    for (validation_type, count) in validation_counts {
        summary.push_str(&format!("- {}: {} validations\n", validation_type, count));
    }

    // Add failed validations
    let failed_validations: Vec<_> = results.iter().filter(|r| !r.success).collect();

    if !failed_validations.is_empty() {
        summary.push_str("\nFailed Validations:\n");
        for result in failed_validations {
            summary.push_str(&format!(
                "- {} failed: {}\n",
                result.validation_type,
                result.error.as_deref().unwrap_or("Unknown error")
            ));
        }
    }

    summary
}
