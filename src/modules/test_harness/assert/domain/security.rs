//! Security assertions for the assertion framework.
//!
//! This module provides assertions for security testing.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents the severity of a vulnerability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    /// Informational severity.
    Info,
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// High severity.
    High,
    /// Critical severity.
    Critical,
}

impl VulnerabilitySeverity {
    /// Returns whether the severity is critical.
    pub fn is_critical(&self) -> bool {
        *self == VulnerabilitySeverity::Critical
    }

    /// Returns whether the severity is high.
    pub fn is_high(&self) -> bool {
        *self == VulnerabilitySeverity::High
    }

    /// Returns whether the severity is medium.
    pub fn is_medium(&self) -> bool {
        *self == VulnerabilitySeverity::Medium
    }

    /// Returns whether the severity is low.
    pub fn is_low(&self) -> bool {
        *self == VulnerabilitySeverity::Low
    }

    /// Returns whether the severity is informational.
    pub fn is_info(&self) -> bool {
        *self == VulnerabilitySeverity::Info
    }
}

/// Represents a vulnerability for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// The vulnerability ID.
    pub id: String,
    /// The vulnerability name.
    pub name: String,
    /// The vulnerability description.
    pub description: String,
    /// The vulnerability severity.
    pub severity: VulnerabilitySeverity,
    /// The vulnerability location.
    pub location: String,
    /// The vulnerability evidence.
    pub evidence: Option<String>,
    /// The vulnerability remediation.
    pub remediation: Option<String>,
    /// The vulnerability references.
    pub references: Vec<String>,
    /// The vulnerability metadata.
    pub metadata: Value,
}

impl Vulnerability {
    /// Creates a new vulnerability.
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        severity: VulnerabilitySeverity,
        location: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            severity,
            location: location.to_string(),
            evidence: None,
            remediation: None,
            references: Vec::new(),
            metadata: Value::Null,
        }
    }

    /// Sets the vulnerability evidence.
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence = Some(evidence.to_string());
        self
    }

    /// Sets the vulnerability remediation.
    pub fn with_remediation(mut self, remediation: &str) -> Self {
        self.remediation = Some(remediation.to_string());
        self
    }

    /// Adds a reference to the vulnerability.
    pub fn with_reference(mut self, reference: &str) -> Self {
        self.references.push(reference.to_string());
        self
    }

    /// Sets the vulnerability metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a security scan result for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    /// The scan name.
    pub name: String,
    /// The scan description.
    pub description: String,
    /// The scan target.
    pub target: String,
    /// The scan start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// The scan end time.
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// The scan duration.
    pub duration: Duration,
    /// The scan vulnerabilities.
    pub vulnerabilities: Vec<Vulnerability>,
    /// The scan parameters.
    pub parameters: Value,
    /// The scan metadata.
    pub metadata: Value,
    /// The scan summary.
    pub summary: HashMap<String, usize>,
}

impl SecurityScanResult {
    /// Creates a new security scan result.
    pub fn new(name: &str, target: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            description: "".to_string(),
            target: target.to_string(),
            start_time: now,
            end_time: now,
            duration: Duration::default(),
            vulnerabilities: Vec::new(),
            parameters: Value::Null,
            metadata: Value::Null,
            summary: HashMap::new(),
        }
    }

    /// Sets the scan description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the scan start time.
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Sets the scan end time.
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Sets the scan duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Adds a vulnerability to the scan result.
    pub fn with_vulnerability(mut self, vulnerability: Vulnerability) -> Self {
        self.vulnerabilities.push(vulnerability);
        self
    }

    /// Sets the scan parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the scan metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Calculates summary statistics for the scan result.
    pub fn calculate_summary(&mut self) -> Result<(), TestHarnessError> {
        // Count vulnerabilities by severity
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        let mut info_count = 0;

        for vulnerability in &self.vulnerabilities {
            match vulnerability.severity {
                VulnerabilitySeverity::Critical => critical_count += 1,
                VulnerabilitySeverity::High => high_count += 1,
                VulnerabilitySeverity::Medium => medium_count += 1,
                VulnerabilitySeverity::Low => low_count += 1,
                VulnerabilitySeverity::Info => info_count += 1,
            }
        }

        self.summary.insert("critical".to_string(), critical_count);
        self.summary.insert("high".to_string(), high_count);
        self.summary.insert("medium".to_string(), medium_count);
        self.summary.insert("low".to_string(), low_count);
        self.summary.insert("info".to_string(), info_count);
        self.summary
            .insert("total".to_string(), self.vulnerabilities.len());

        Ok(())
    }

    /// Returns whether the scan result has critical vulnerabilities.
    pub fn has_critical_vulnerabilities(&self) -> bool {
        self.vulnerabilities
            .iter()
            .any(|v| v.severity.is_critical())
    }

    /// Returns whether the scan result has high vulnerabilities.
    pub fn has_high_vulnerabilities(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity.is_high())
    }

    /// Returns whether the scan result has medium vulnerabilities.
    pub fn has_medium_vulnerabilities(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity.is_medium())
    }

    /// Returns whether the scan result has low vulnerabilities.
    pub fn has_low_vulnerabilities(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity.is_low())
    }

    /// Returns whether the scan result has informational vulnerabilities.
    pub fn has_info_vulnerabilities(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity.is_info())
    }

    /// Returns the number of critical vulnerabilities.
    pub fn critical_count(&self) -> usize {
        self.summary.get("critical").copied().unwrap_or(0)
    }

    /// Returns the number of high vulnerabilities.
    pub fn high_count(&self) -> usize {
        self.summary.get("high").copied().unwrap_or(0)
    }

    /// Returns the number of medium vulnerabilities.
    pub fn medium_count(&self) -> usize {
        self.summary.get("medium").copied().unwrap_or(0)
    }

    /// Returns the number of low vulnerabilities.
    pub fn low_count(&self) -> usize {
        self.summary.get("low").copied().unwrap_or(0)
    }

    /// Returns the number of informational vulnerabilities.
    pub fn info_count(&self) -> usize {
        self.summary.get("info").copied().unwrap_or(0)
    }

    /// Returns the total number of vulnerabilities.
    pub fn total_count(&self) -> usize {
        self.summary.get("total").copied().unwrap_or(0)
    }
}

/// Assertions for security testing.
#[derive(Debug, Clone)]
pub struct SecurityAssertions;

impl SecurityAssertions {
    /// Creates a new security assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a scan result has no vulnerabilities.
    pub fn assert_no_vulnerabilities(&self, result: &SecurityScanResult) -> AssertionResult {
        if result.vulnerabilities.is_empty() {
            AssertionResult::new("No vulnerabilities", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("No vulnerabilities", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has vulnerabilities",
                    "0 vulnerabilities",
                    &format!("{} vulnerabilities", result.vulnerabilities.len()),
                ),
            )
        }
    }

    /// Asserts that a scan result has no critical vulnerabilities.
    pub fn assert_no_critical_vulnerabilities(
        &self,
        result: &SecurityScanResult,
    ) -> AssertionResult {
        if !result.has_critical_vulnerabilities() {
            AssertionResult::new("No critical vulnerabilities", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("No critical vulnerabilities", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Scan result has critical vulnerabilities",
                        "0 critical vulnerabilities",
                        &format!("{} critical vulnerabilities", result.critical_count()),
                    ),
                )
        }
    }

    /// Asserts that a scan result has no high vulnerabilities.
    pub fn assert_no_high_vulnerabilities(&self, result: &SecurityScanResult) -> AssertionResult {
        if !result.has_high_vulnerabilities() {
            AssertionResult::new("No high vulnerabilities", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("No high vulnerabilities", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has high vulnerabilities",
                    "0 high vulnerabilities",
                    &format!("{} high vulnerabilities", result.high_count()),
                ),
            )
        }
    }

    /// Asserts that a scan result has no medium vulnerabilities.
    pub fn assert_no_medium_vulnerabilities(&self, result: &SecurityScanResult) -> AssertionResult {
        if !result.has_medium_vulnerabilities() {
            AssertionResult::new("No medium vulnerabilities", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("No medium vulnerabilities", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has medium vulnerabilities",
                    "0 medium vulnerabilities",
                    &format!("{} medium vulnerabilities", result.medium_count()),
                ),
            )
        }
    }

    /// Asserts that a scan result has at most a specific number of vulnerabilities.
    pub fn assert_max_vulnerabilities(
        &self,
        result: &SecurityScanResult,
        max: usize,
    ) -> AssertionResult {
        let count = result.total_count();
        if count <= max {
            AssertionResult::new(
                &format!("At most {} vulnerabilities", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("At most {} vulnerabilities", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has too many vulnerabilities",
                    &format!("<= {}", max),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a scan result has at most a specific number of critical vulnerabilities.
    pub fn assert_max_critical_vulnerabilities(
        &self,
        result: &SecurityScanResult,
        max: usize,
    ) -> AssertionResult {
        let count = result.critical_count();
        if count <= max {
            AssertionResult::new(
                &format!("At most {} critical vulnerabilities", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("At most {} critical vulnerabilities", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has too many critical vulnerabilities",
                    &format!("<= {}", max),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a scan result has at most a specific number of high vulnerabilities.
    pub fn assert_max_high_vulnerabilities(
        &self,
        result: &SecurityScanResult,
        max: usize,
    ) -> AssertionResult {
        let count = result.high_count();
        if count <= max {
            AssertionResult::new(
                &format!("At most {} high vulnerabilities", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("At most {} high vulnerabilities", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has too many high vulnerabilities",
                    &format!("<= {}", max),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a scan result has at most a specific number of medium vulnerabilities.
    pub fn assert_max_medium_vulnerabilities(
        &self,
        result: &SecurityScanResult,
        max: usize,
    ) -> AssertionResult {
        let count = result.medium_count();
        if count <= max {
            AssertionResult::new(
                &format!("At most {} medium vulnerabilities", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("At most {} medium vulnerabilities", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result has too many medium vulnerabilities",
                    &format!("<= {}", max),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a scan result has no vulnerabilities of a specific type.
    pub fn assert_no_vulnerability_type(
        &self,
        result: &SecurityScanResult,
        vulnerability_type: &str,
    ) -> AssertionResult {
        let vulnerabilities: Vec<&Vulnerability> = result
            .vulnerabilities
            .iter()
            .filter(|v| v.name == vulnerability_type)
            .collect();

        if vulnerabilities.is_empty() {
            AssertionResult::new(
                &format!("No '{}' vulnerabilities", vulnerability_type),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("No '{}' vulnerabilities", vulnerability_type),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Scan result has '{}' vulnerabilities", vulnerability_type),
                    &format!("0 '{}' vulnerabilities", vulnerability_type),
                    &format!(
                        "{} '{}' vulnerabilities",
                        vulnerabilities.len(),
                        vulnerability_type
                    ),
                ),
            )
        }
    }

    /// Asserts that a scan result has no vulnerabilities in a specific location.
    pub fn assert_no_vulnerabilities_in_location(
        &self,
        result: &SecurityScanResult,
        location: &str,
    ) -> AssertionResult {
        let vulnerabilities: Vec<&Vulnerability> = result
            .vulnerabilities
            .iter()
            .filter(|v| v.location == location)
            .collect();

        if vulnerabilities.is_empty() {
            AssertionResult::new(
                &format!("No vulnerabilities in '{}'", location),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("No vulnerabilities in '{}'", location),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Scan result has vulnerabilities in '{}'", location),
                    &format!("0 vulnerabilities in '{}'", location),
                    &format!(
                        "{} vulnerabilities in '{}'",
                        vulnerabilities.len(),
                        location
                    ),
                ),
            )
        }
    }

    /// Asserts that a scan result passes a security policy.
    /// This is a placeholder for a more sophisticated security policy check.
    pub fn assert_passes_security_policy(&self, result: &SecurityScanResult) -> AssertionResult {
        // In a real implementation, this would check against a security policy.
        // For now, we just check that there are no critical or high vulnerabilities.
        if !result.has_critical_vulnerabilities() && !result.has_high_vulnerabilities() {
            AssertionResult::new("Passes security policy", AssertionOutcome::Passed)
        } else {
            let critical_count = result.critical_count();
            let high_count = result.high_count();
            AssertionResult::new("Passes security policy", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Scan result does not pass security policy",
                    "0 critical and 0 high vulnerabilities",
                    &format!(
                        "{} critical and {} high vulnerabilities",
                        critical_count, high_count
                    ),
                ),
            )
        }
    }
}

impl Default for SecurityAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new security assertions instance.
pub fn create_security_assertions() -> SecurityAssertions {
    SecurityAssertions::new()
}

/// Creates a new vulnerability.
pub fn create_vulnerability(
    id: &str,
    name: &str,
    description: &str,
    severity: VulnerabilitySeverity,
    location: &str,
) -> Vulnerability {
    Vulnerability::new(id, name, description, severity, location)
}

/// Creates a new security scan result.
pub fn create_security_scan_result(name: &str, target: &str) -> SecurityScanResult {
    SecurityScanResult::new(name, target)
}
