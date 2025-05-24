//! Security Testing Framework
//!
//! This module provides utilities for security testing, including
//! vulnerability scanning, penetration testing, and security auditing.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::{
    TestCategory, TestContext, TestHarnessError, TestOutcome, TestResult,
};

/// Security vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    /// Informational severity
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl fmt::Display for VulnerabilitySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VulnerabilitySeverity::Info => write!(f, "Info"),
            VulnerabilitySeverity::Low => write!(f, "Low"),
            VulnerabilitySeverity::Medium => write!(f, "Medium"),
            VulnerabilitySeverity::High => write!(f, "High"),
            VulnerabilitySeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Security vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Vulnerability ID
    pub id: String,
    /// Vulnerability name
    pub name: String,
    /// Vulnerability description
    pub description: String,
    /// Vulnerability severity
    pub severity: VulnerabilitySeverity,
    /// Vulnerability location
    pub location: String,
    /// Vulnerability evidence
    pub evidence: Option<String>,
    /// Vulnerability remediation
    pub remediation: Option<String>,
    /// Vulnerability references
    pub references: Vec<String>,
    /// Vulnerability tags
    pub tags: Vec<String>,
    /// Vulnerability metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Vulnerability {
    /// Create a new vulnerability
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        severity: VulnerabilitySeverity,
        location: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            severity,
            location: location.into(),
            evidence: None,
            remediation: None,
            references: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the vulnerability evidence
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }

    /// Set the vulnerability remediation
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    /// Add a reference
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.references.push(reference.into());
        self
    }

    /// Add multiple references
    pub fn with_references(
        mut self,
        references: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.references
            .extend(references.into_iter().map(|r| r.into()));
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Add metadata
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.metadata.insert(key.into(), value);
        Ok(self)
    }
}

/// Security test parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTestParams {
    /// Target URL or host
    pub target: String,
    /// Authentication credentials
    pub credentials: Option<HashMap<String, String>>,
    /// Test timeout
    pub timeout: Duration,
    /// Maximum scan depth
    pub max_depth: Option<usize>,
    /// Scan scope
    pub scope: Vec<String>,
    /// Excluded paths
    pub excluded_paths: Vec<String>,
    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for SecurityTestParams {
    fn default() -> Self {
        Self {
            target: "http://localhost".to_string(),
            credentials: None,
            timeout: Duration::from_secs(300),
            max_depth: None,
            scope: Vec::new(),
            excluded_paths: Vec::new(),
            custom_params: HashMap::new(),
        }
    }
}

impl SecurityTestParams {
    /// Create new security test parameters
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            ..Default::default()
        }
    }

    /// Set the authentication credentials
    pub fn with_credentials(mut self, credentials: HashMap<String, String>) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Add a credential
    pub fn with_credential(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let credentials = self.credentials.get_or_insert_with(HashMap::new);
        credentials.insert(key.into(), value.into());
        self
    }

    /// Set the test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum scan depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Add a scope path
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope.push(scope.into());
        self
    }

    /// Add multiple scope paths
    pub fn with_scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scope.extend(scopes.into_iter().map(|s| s.into()));
        self
    }

    /// Add an excluded path
    pub fn with_excluded_path(mut self, path: impl Into<String>) -> Self {
        self.excluded_paths.push(path.into());
        self
    }

    /// Add multiple excluded paths
    pub fn with_excluded_paths(
        mut self,
        paths: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.excluded_paths
            .extend(paths.into_iter().map(|p| p.into()));
        self
    }

    /// Add a custom parameter
    pub fn with_custom_param(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.custom_params.insert(key.into(), value);
        Ok(self)
    }
}

/// Security test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTestResult {
    /// Test name
    pub name: String,
    /// Test description
    pub description: Option<String>,
    /// Test parameters
    pub params: SecurityTestParams,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Test start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Test end time
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Test duration
    pub duration: Duration,
    /// Test outcome
    pub outcome: TestOutcome,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Test summary
    pub summary: HashMap<String, serde_json::Value>,
}

impl SecurityTestResult {
    /// Create a new security test result
    pub fn new(name: impl Into<String>, params: SecurityTestParams, outcome: TestOutcome) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            description: None,
            params,
            vulnerabilities: Vec::new(),
            start_time: now,
            end_time: now,
            duration: Duration::from_secs(0),
            outcome,
            error: None,
            summary: HashMap::new(),
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a vulnerability
    pub fn with_vulnerability(mut self, vulnerability: Vulnerability) -> Self {
        self.vulnerabilities.push(vulnerability);
        self
    }

    /// Add multiple vulnerabilities
    pub fn with_vulnerabilities(mut self, vulnerabilities: Vec<Vulnerability>) -> Self {
        self.vulnerabilities.extend(vulnerabilities);
        self
    }

    /// Set the test start time
    pub fn with_start_time(mut self, start_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.start_time = start_time;
        self
    }

    /// Set the test end time
    pub fn with_end_time(mut self, end_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.end_time = end_time;
        self
    }

    /// Set the test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Add a summary value
    pub fn with_summary_value(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.summary.insert(key.into(), value);
        Ok(self)
    }

    /// Calculate summary statistics
    pub fn calculate_summary(&mut self) -> Result<(), TestHarnessError> {
        // Count vulnerabilities by severity
        let mut severity_counts = HashMap::new();
        for vuln in &self.vulnerabilities {
            *severity_counts.entry(vuln.severity).or_insert(0) += 1;
        }

        // Add severity counts to summary
        for (severity, count) in severity_counts {
            self.summary.insert(
                format!("{}_count", severity.to_string().to_lowercase()),
                serde_json::to_value(count).map_err(TestHarnessError::SerializationError)?,
            );
        }

        // Calculate total vulnerabilities
        let total_vulns = self.vulnerabilities.len();
        self.summary.insert(
            "total_vulnerabilities".to_string(),
            serde_json::to_value(total_vulns).map_err(TestHarnessError::SerializationError)?,
        );

        // Calculate risk score (weighted sum of vulnerabilities)
        let risk_score = self.vulnerabilities.iter().fold(0.0, |score, vuln| {
            score
                + match vuln.severity {
                    VulnerabilitySeverity::Info => 0.1,
                    VulnerabilitySeverity::Low => 1.0,
                    VulnerabilitySeverity::Medium => 3.0,
                    VulnerabilitySeverity::High => 6.0,
                    VulnerabilitySeverity::Critical => 10.0,
                }
        });

        self.summary.insert(
            "risk_score".to_string(),
            serde_json::to_value(risk_score).map_err(TestHarnessError::SerializationError)?,
        );

        Ok(())
    }

    /// Check if the test passed
    pub fn passed(&self) -> bool {
        self.outcome == TestOutcome::Passed
    }

    /// Check if the test failed
    pub fn failed(&self) -> bool {
        self.outcome == TestOutcome::Failed
    }

    /// Check if the test has critical vulnerabilities
    pub fn has_critical_vulnerabilities(&self) -> bool {
        self.vulnerabilities
            .iter()
            .any(|v| v.severity == VulnerabilitySeverity::Critical)
    }

    /// Check if the test has high vulnerabilities
    pub fn has_high_vulnerabilities(&self) -> bool {
        self.vulnerabilities
            .iter()
            .any(|v| v.severity == VulnerabilitySeverity::High)
    }

    /// Get vulnerabilities by severity
    pub fn get_vulnerabilities_by_severity(
        &self,
        severity: VulnerabilitySeverity,
    ) -> Vec<&Vulnerability> {
        self.vulnerabilities
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }
}

/// Security test interface
#[async_trait]
pub trait SecurityTest: Send + Sync {
    /// Get the test name
    fn name(&self) -> &str;

    /// Get the test description
    fn description(&self) -> Option<&str>;

    /// Execute the test with the given parameters
    async fn execute(
        &self,
        params: &SecurityTestParams,
    ) -> Result<SecurityTestResult, TestHarnessError>;
}

/// Security test builder
pub struct SecurityTestBuilder {
    /// Test name
    name: String,
    /// Test description
    description: Option<String>,
    /// Test execution function
    execute_fn: Option<
        Box<
            dyn Fn(
                    &SecurityTestParams,
                )
                    -> BoxFuture<'static, Result<SecurityTestResult, TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
}

impl SecurityTestBuilder {
    /// Create a new security test builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            execute_fn: None,
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the test execution function
    pub fn with_execute_fn(
        mut self,
        execute_fn: impl Fn(
                &SecurityTestParams,
            ) -> BoxFuture<'static, Result<SecurityTestResult, TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.execute_fn = Some(Box::new(execute_fn));
        self
    }

    /// Build the security test
    pub fn build(self) -> Box<dyn SecurityTest> {
        let execute_fn = self.execute_fn.unwrap_or_else(|| {
            Box::new(|params| {
                async move {
                    Ok(SecurityTestResult::new(
                        self.name.clone(),
                        params.clone(),
                        TestOutcome::Passed,
                    ))
                }
                .boxed()
            })
        });

        Box::new(BasicSecurityTest {
            name: self.name,
            description: self.description,
            execute_fn,
        })
    }
}

/// Basic security test implementation
struct BasicSecurityTest {
    /// Test name
    name: String,
    /// Test description
    description: Option<String>,
    /// Test execution function
    execute_fn: Box<
        dyn Fn(
                &SecurityTestParams,
            ) -> BoxFuture<'static, Result<SecurityTestResult, TestHarnessError>>
            + Send
            + Sync,
    >,
}

#[async_trait]
impl SecurityTest for BasicSecurityTest {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn execute(
        &self,
        params: &SecurityTestParams,
    ) -> Result<SecurityTestResult, TestHarnessError> {
        (self.execute_fn)(params).await
    }
}

/// Security test suite
#[derive(Debug)]
pub struct SecurityTestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: Option<String>,
    /// Test parameters
    pub params: Vec<SecurityTestParams>,
    /// Tests to run
    pub tests: Vec<Box<dyn SecurityTest>>,
    /// Whether to run tests in parallel
    pub parallel: bool,
    /// Whether to fail fast on the first test failure
    pub fail_fast: bool,
}

impl SecurityTestSuite {
    /// Create a new security test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            params: Vec::new(),
            tests: Vec::new(),
            parallel: false,
            fail_fast: false,
        }
    }

    /// Set the suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add test parameters
    pub fn with_params(mut self, params: SecurityTestParams) -> Self {
        self.params.push(params);
        self
    }

    /// Add multiple test parameters
    pub fn with_multiple_params(mut self, params: Vec<SecurityTestParams>) -> Self {
        self.params.extend(params);
        self
    }

    /// Add a test
    pub fn with_test(mut self, test: Box<dyn SecurityTest>) -> Self {
        self.tests.push(test);
        self
    }

    /// Add multiple tests
    pub fn with_tests(mut self, tests: Vec<Box<dyn SecurityTest>>) -> Self {
        self.tests.extend(tests);
        self
    }

    /// Set whether to run tests in parallel
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set whether to fail fast on the first test failure
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Execute the test suite
    pub async fn execute(&self) -> Result<Vec<SecurityTestResult>, TestHarnessError> {
        info!("Executing security test suite: {}", self.name);

        let mut results = Vec::new();

        for params in &self.params {
            info!("Running security tests for target: {}", params.target);

            for test in &self.tests {
                info!("Executing test: {}", test.name());

                let start_time = Instant::now();
                let result = match test.execute(params).await {
                    Ok(mut result) => {
                        // Calculate summary statistics
                        if let Err(e) = result.calculate_summary() {
                            warn!("Failed to calculate summary statistics: {}", e);
                        }
                        result
                    }
                    Err(e) => {
                        error!("Test failed: {}: {}", test.name(), e);
                        let result = SecurityTestResult::new(
                            test.name(),
                            params.clone(),
                            TestOutcome::Failed,
                        )
                        .with_error(format!("Test execution error: {}", e))
                        .with_duration(start_time.elapsed());

                        results.push(result.clone());

                        if self.fail_fast {
                            return Ok(results);
                        } else {
                            continue;
                        }
                    }
                };

                // Check result
                if result.outcome == TestOutcome::Failed {
                    if self.fail_fast {
                        results.push(result);
                        return Ok(results);
                    }
                }

                results.push(result);
            }
        }

        Ok(results)
    }
}

/// Create a test case from a security test suite
pub fn create_test_case_from_security_suite(
    suite: SecurityTestSuite,
) -> crate::modules::test_harness::types::TestCase {
    let suite_name = suite.name.clone();

    crate::modules::test_harness::types::TestCase::new(
        TestContext::new(TestCategory::Security, suite_name.clone()),
        move |_| {
            let suite = suite.clone();
            async move {
                let start_time = Instant::now();
                let start_datetime = chrono::Utc::now();

                let results = suite.execute().await?;

                let duration = start_time.elapsed();
                let end_datetime = chrono::Utc::now();

                let all_passed = results.iter().all(|r| r.outcome == TestOutcome::Passed);
                let outcome = if all_passed {
                    TestOutcome::Passed
                } else {
                    TestOutcome::Failed
                };

                let mut test_result = TestResult::new(&suite_name, TestCategory::Security, outcome)
                    .with_start_time(start_datetime)
                    .with_end_time(end_datetime)
                    .with_duration(duration);

                // Add security test results as custom data
                test_result = test_result
                    .with_custom_data("security_test_results", &results)
                    .map_err(|e| {
                        TestHarnessError::ExecutionError(format!(
                            "Failed to add security test results: {}",
                            e
                        ))
                    })?;

                // Add error message if any tests failed
                if !all_passed {
                    let failed_tests: Vec<String> = results
                        .iter()
                        .filter(|r| r.outcome == TestOutcome::Failed)
                        .map(|r| {
                            if let Some(error) = &r.error {
                                format!("{}: {}", r.name, error)
                            } else {
                                format!("{}: Failed", r.name)
                            }
                        })
                        .collect();

                    test_result = test_result.with_error(format!(
                        "Security test suite failed with {} failed tests: {}",
                        failed_tests.len(),
                        failed_tests.join(", ")
                    ));
                }

                Ok(test_result)
            }
            .boxed()
        },
    )
}

/// Create a new security test
pub fn create_security_test(name: impl Into<String>) -> SecurityTestBuilder {
    SecurityTestBuilder::new(name)
}

/// Create new security test parameters
pub fn create_security_test_params(target: impl Into<String>) -> SecurityTestParams {
    SecurityTestParams::new(target)
}

/// Create a new security test suite
pub fn create_security_test_suite(name: impl Into<String>) -> SecurityTestSuite {
    SecurityTestSuite::new(name)
}

/// Create a new vulnerability
pub fn create_vulnerability(
    id: impl Into<String>,
    name: impl Into<String>,
    description: impl Into<String>,
    severity: VulnerabilitySeverity,
    location: impl Into<String>,
) -> Vulnerability {
    Vulnerability::new(id, name, description, severity, location)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future;

    #[tokio::test]
    async fn test_security_test_params() {
        let params = create_security_test_params("http://example.com")
            .with_credential("username", "admin")
            .with_credential("password", "password")
            .with_timeout(Duration::from_secs(120))
            .with_max_depth(5)
            .with_scope("/api")
            .with_excluded_path("/api/admin");

        assert_eq!(params.target, "http://example.com");
        assert_eq!(
            params
                .credentials
                .as_ref()
                .unwrap()
                .get("username")
                .unwrap(),
            "admin"
        );
        assert_eq!(
            params
                .credentials
                .as_ref()
                .unwrap()
                .get("password")
                .unwrap(),
            "password"
        );
        assert_eq!(params.timeout, Duration::from_secs(120));
        assert_eq!(params.max_depth, Some(5));
        assert_eq!(params.scope, vec!["/api"]);
        assert_eq!(params.excluded_paths, vec!["/api/admin"]);
    }

    #[tokio::test]
    async fn test_vulnerability() {
        let vuln = create_vulnerability(
            "CVE-2021-1234",
            "SQL Injection",
            "SQL injection vulnerability in login form",
            VulnerabilitySeverity::High,
            "/login",
        )
        .with_evidence("' OR 1=1 --")
        .with_remediation("Use prepared statements")
        .with_reference("https://example.com/cve-2021-1234")
        .with_tag("injection");

        assert_eq!(vuln.id, "CVE-2021-1234");
        assert_eq!(vuln.name, "SQL Injection");
        assert_eq!(
            vuln.description,
            "SQL injection vulnerability in login form"
        );
        assert_eq!(vuln.severity, VulnerabilitySeverity::High);
        assert_eq!(vuln.location, "/login");
        assert_eq!(vuln.evidence, Some("' OR 1=1 --".to_string()));
        assert_eq!(
            vuln.remediation,
            Some("Use prepared statements".to_string())
        );
        assert_eq!(vuln.references, vec!["https://example.com/cve-2021-1234"]);
        assert_eq!(vuln.tags, vec!["injection"]);
    }

    #[tokio::test]
    async fn test_security_test() {
        // Create a security test
        let test = create_security_test("test_security")
            .with_description("Test security test")
            .with_execute_fn(|params| {
                async move {
                    // Create some test vulnerabilities
                    let vulnerabilities = vec![
                        create_vulnerability(
                            "CVE-2021-1234",
                            "SQL Injection",
                            "SQL injection vulnerability in login form",
                            VulnerabilitySeverity::High,
                            "/login",
                        ),
                        create_vulnerability(
                            "CVE-2021-5678",
                            "XSS",
                            "Cross-site scripting vulnerability in search form",
                            VulnerabilitySeverity::Medium,
                            "/search",
                        ),
                    ];

                    let mut result = SecurityTestResult::new(
                        "test_security",
                        params.clone(),
                        TestOutcome::Passed,
                    )
                    .with_vulnerabilities(vulnerabilities)
                    .with_duration(Duration::from_secs(1));

                    if let Err(e) = result.calculate_summary() {
                        return Err(TestHarnessError::ExecutionError(format!(
                            "Failed to calculate summary: {}",
                            e
                        )));
                    }

                    Ok(result)
                }
                .boxed()
            })
            .build();

        // Create test parameters
        let params = create_security_test_params("http://example.com");

        // Execute the test
        let result = test.execute(&params).await.unwrap();

        // Check the result
        assert_eq!(result.name, "test_security");
        assert_eq!(result.outcome, TestOutcome::Passed);
        assert_eq!(result.vulnerabilities.len(), 2);

        // Check summary statistics
        assert!(result.summary.contains_key("total_vulnerabilities"));
        assert!(result.summary.contains_key("high_count"));
        assert!(result.summary.contains_key("medium_count"));
        assert!(result.summary.contains_key("risk_score"));
    }
}
