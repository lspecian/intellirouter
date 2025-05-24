//! Scenario Testing Framework
//!
//! This module provides a framework for scenario-based testing, allowing
//! the definition of complex test scenarios with multiple steps, preconditions,
//! and assertions.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::assert::{AssertionContext, AssertionResult};
use crate::modules::test_harness::types::{
    TestCategory, TestContext, TestHarnessError, TestOutcome, TestResult,
};

/// Scenario step status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScenarioStepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
}

impl fmt::Display for ScenarioStepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioStepStatus::Pending => write!(f, "Pending"),
            ScenarioStepStatus::Running => write!(f, "Running"),
            ScenarioStepStatus::Completed => write!(f, "Completed"),
            ScenarioStepStatus::Failed => write!(f, "Failed"),
            ScenarioStepStatus::Skipped => write!(f, "Skipped"),
        }
    }
}

/// Scenario step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStepResult {
    /// Step name
    pub name: String,
    /// Step description
    pub description: Option<String>,
    /// Step status
    pub status: ScenarioStepStatus,
    /// Error message if the step failed
    pub error: Option<String>,
    /// Step duration
    pub duration: Duration,
    /// Step output data
    pub output: HashMap<String, serde_json::Value>,
    /// Assertion results
    pub assertions: Vec<AssertionResult>,
}

impl ScenarioStepResult {
    /// Create a new scenario step result
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            status: ScenarioStepStatus::Pending,
            error: None,
            duration: Duration::from_secs(0),
            output: HashMap::new(),
            assertions: Vec::new(),
        }
    }

    /// Set the step description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the step status
    pub fn with_status(mut self, status: ScenarioStepStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set the step duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add output data
    pub fn with_output(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.output.insert(key.into(), value);
        Ok(self)
    }

    /// Add an assertion result
    pub fn with_assertion(mut self, assertion: AssertionResult) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Add multiple assertion results
    pub fn with_assertions(mut self, assertions: Vec<AssertionResult>) -> Self {
        self.assertions.extend(assertions);
        self
    }

    /// Check if the step passed
    pub fn passed(&self) -> bool {
        self.status == ScenarioStepStatus::Completed && self.assertions.iter().all(|a| a.passed())
    }

    /// Check if the step failed
    pub fn failed(&self) -> bool {
        self.status == ScenarioStepStatus::Failed || self.assertions.iter().any(|a| a.failed())
    }
}

/// Scenario step interface
#[async_trait]
pub trait ScenarioStep: Send + Sync {
    /// Get the step name
    fn name(&self) -> &str;

    /// Get the step description
    fn description(&self) -> Option<&str>;

    /// Check if the step should be skipped
    async fn should_skip(&self, context: &ScenarioContext) -> Result<bool, TestHarnessError>;

    /// Execute the step
    async fn execute(
        &self,
        context: &ScenarioContext,
    ) -> Result<ScenarioStepResult, TestHarnessError>;

    /// Get the step dependencies
    fn dependencies(&self) -> &[String];

    /// Get the step timeout
    fn timeout(&self) -> Option<Duration>;
}

/// Scenario step builder
pub struct ScenarioStepBuilder {
    /// Step name
    name: String,
    /// Step description
    description: Option<String>,
    /// Step dependencies
    dependencies: Vec<String>,
    /// Step timeout
    timeout: Option<Duration>,
    /// Step execution function
    execute_fn: Option<
        Box<
            dyn Fn(
                    &ScenarioContext,
                )
                    -> BoxFuture<'static, Result<ScenarioStepResult, TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
    /// Step skip function
    skip_fn: Option<
        Box<
            dyn Fn(&ScenarioContext) -> BoxFuture<'static, Result<bool, TestHarnessError>>
                + Send
                + Sync,
        >,
    >,
}

impl ScenarioStepBuilder {
    /// Create a new scenario step builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            dependencies: Vec::new(),
            timeout: None,
            execute_fn: None,
            skip_fn: None,
        }
    }

    /// Set the step description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a dependency on another step
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies on other steps
    pub fn with_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.dependencies
            .extend(dependencies.into_iter().map(|d| d.into()));
        self
    }

    /// Set the step timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the step execution function
    pub fn with_execute_fn(
        mut self,
        execute_fn: impl Fn(&ScenarioContext) -> BoxFuture<'static, Result<ScenarioStepResult, TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.execute_fn = Some(Box::new(execute_fn));
        self
    }

    /// Set the step skip function
    pub fn with_skip_fn(
        mut self,
        skip_fn: impl Fn(&ScenarioContext) -> BoxFuture<'static, Result<bool, TestHarnessError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.skip_fn = Some(Box::new(skip_fn));
        self
    }

    /// Build the scenario step
    pub fn build(self) -> Box<dyn ScenarioStep> {
        let execute_fn = self.execute_fn.unwrap_or_else(|| {
            Box::new(|ctx| {
                async move {
                    Ok(ScenarioStepResult::new(ctx.name.clone())
                        .with_status(ScenarioStepStatus::Completed))
                }
                .boxed()
            })
        });

        let skip_fn = self
            .skip_fn
            .unwrap_or_else(|| Box::new(|_| async move { Ok(false) }.boxed()));

        Box::new(BasicScenarioStep {
            name: self.name,
            description: self.description,
            dependencies: self.dependencies,
            timeout: self.timeout,
            execute_fn,
            skip_fn,
        })
    }
}

/// Basic scenario step implementation
struct BasicScenarioStep {
    /// Step name
    name: String,
    /// Step description
    description: Option<String>,
    /// Step dependencies
    dependencies: Vec<String>,
    /// Step timeout
    timeout: Option<Duration>,
    /// Step execution function
    execute_fn: Box<
        dyn Fn(&ScenarioContext) -> BoxFuture<'static, Result<ScenarioStepResult, TestHarnessError>>
            + Send
            + Sync,
    >,
    /// Step skip function
    skip_fn: Box<
        dyn Fn(&ScenarioContext) -> BoxFuture<'static, Result<bool, TestHarnessError>>
            + Send
            + Sync,
    >,
}

#[async_trait]
impl ScenarioStep for BasicScenarioStep {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn should_skip(&self, context: &ScenarioContext) -> Result<bool, TestHarnessError> {
        (self.skip_fn)(context).await
    }

    async fn execute(
        &self,
        context: &ScenarioContext,
    ) -> Result<ScenarioStepResult, TestHarnessError> {
        (self.execute_fn)(context).await
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

/// Scenario context for sharing data between steps
#[derive(Debug, Clone)]
pub struct ScenarioContext {
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: Option<String>,
    /// Shared data between steps
    pub data: HashMap<String, serde_json::Value>,
    /// Step results
    pub step_results: HashMap<String, ScenarioStepResult>,
    /// Test context
    pub test_context: TestContext,
}

impl ScenarioContext {
    /// Create a new scenario context
    pub fn new(name: impl Into<String>, test_context: TestContext) -> Self {
        Self {
            name: name.into(),
            description: None,
            data: HashMap::new(),
            step_results: HashMap::new(),
            test_context,
        }
    }

    /// Set the scenario description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add shared data
    pub fn with_data(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.data.insert(key.into(), value);
        Ok(self)
    }

    /// Get shared data
    pub fn get_data<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        if let Some(value) = self.data.get(key) {
            let value = serde_json::from_value(value.clone())
                .map_err(TestHarnessError::SerializationError)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Add a step result
    pub fn add_step_result(&mut self, result: ScenarioStepResult) {
        self.step_results.insert(result.name.clone(), result);
    }

    /// Get a step result
    pub fn get_step_result(&self, step_name: &str) -> Option<&ScenarioStepResult> {
        self.step_results.get(step_name)
    }

    /// Check if all steps passed
    pub fn all_steps_passed(&self) -> bool {
        self.step_results.values().all(|r| r.passed())
    }

    /// Check if any steps failed
    pub fn any_steps_failed(&self) -> bool {
        self.step_results.values().any(|r| r.failed())
    }
}

/// Scenario definition
#[derive(Debug)]
pub struct Scenario {
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: Option<String>,
    /// Scenario steps
    pub steps: Vec<Box<dyn ScenarioStep>>,
    /// Whether to run steps in parallel
    pub parallel: bool,
    /// Whether to fail fast on the first step failure
    pub fail_fast: bool,
    /// Scenario category
    pub category: TestCategory,
}

impl Scenario {
    /// Create a new scenario
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            steps: Vec::new(),
            parallel: false,
            fail_fast: false,
            category: TestCategory::Integration,
        }
    }

    /// Set the scenario description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a step to the scenario
    pub fn with_step(mut self, step: Box<dyn ScenarioStep>) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps to the scenario
    pub fn with_steps(mut self, steps: Vec<Box<dyn ScenarioStep>>) -> Self {
        self.steps.extend(steps);
        self
    }

    /// Set whether to run steps in parallel
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set whether to fail fast on the first step failure
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set the scenario category
    pub fn with_category(mut self, category: TestCategory) -> Self {
        self.category = category;
        self
    }

    /// Execute the scenario
    pub async fn execute(&self, test_context: TestContext) -> Result<TestResult, TestHarnessError> {
        let start_time = std::time::Instant::now();
        let start_datetime = chrono::Utc::now();

        info!("Executing scenario: {}", self.name);

        // Create scenario context
        let mut context = ScenarioContext::new(&self.name, test_context);
        if let Some(desc) = &self.description {
            context = context.with_description(desc.clone());
        }

        // Sort steps by dependencies
        let sorted_steps = self.sort_steps_by_dependencies()?;

        // Execute steps
        let mut all_passed = true;
        for step in &sorted_steps {
            let step_name = step.name().to_string();
            info!("Executing step: {}", step_name);

            // Check if step should be skipped
            let should_skip = step.should_skip(&context).await?;
            if should_skip {
                info!("Skipping step: {}", step_name);
                let result =
                    ScenarioStepResult::new(&step_name).with_status(ScenarioStepStatus::Skipped);
                context.add_step_result(result);
                continue;
            }

            // Execute step
            let step_start = std::time::Instant::now();
            let step_result = match step.execute(&context).await {
                Ok(result) => result,
                Err(e) => {
                    error!("Step failed: {}: {}", step_name, e);
                    all_passed = false;
                    let result = ScenarioStepResult::new(&step_name)
                        .with_status(ScenarioStepStatus::Failed)
                        .with_error(format!("Step execution error: {}", e))
                        .with_duration(step_start.elapsed());
                    context.add_step_result(result);

                    if self.fail_fast {
                        break;
                    } else {
                        continue;
                    }
                }
            };

            // Check step result
            if step_result.failed() {
                all_passed = false;
                if self.fail_fast {
                    break;
                }
            }

            // Add step result to context
            context.add_step_result(step_result);
        }

        // Create test result
        let duration = start_time.elapsed();
        let end_datetime = chrono::Utc::now();

        let outcome = if all_passed {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed
        };

        let mut test_result = TestResult::new(&self.name, self.category, outcome)
            .with_start_time(start_datetime)
            .with_end_time(end_datetime)
            .with_duration(duration);

        // Add step results as custom data
        let step_results: HashMap<String, ScenarioStepResult> = context.step_results.clone();
        test_result = test_result
            .with_custom_data("step_results", step_results)
            .map_err(|e| {
                TestHarnessError::ExecutionError(format!("Failed to add step results: {}", e))
            })?;

        // Add error message if any steps failed
        if !all_passed {
            let failed_steps: Vec<String> = context
                .step_results
                .values()
                .filter(|r| r.failed())
                .map(|r| {
                    if let Some(error) = &r.error {
                        format!("{}: {}", r.name, error)
                    } else {
                        format!("{}: Failed", r.name)
                    }
                })
                .collect();

            test_result = test_result.with_error(format!(
                "Scenario failed with {} failed steps: {}",
                failed_steps.len(),
                failed_steps.join(", ")
            ));
        }

        Ok(test_result)
    }

    /// Sort steps by dependencies
    fn sort_steps_by_dependencies(&self) -> Result<Vec<&Box<dyn ScenarioStep>>, TestHarnessError> {
        // Build a map of step names to indices
        let mut name_to_index = std::collections::HashMap::new();
        for (i, step) in self.steps.iter().enumerate() {
            name_to_index.insert(step.name().to_string(), i);
        }

        // Build a dependency graph
        let mut graph = vec![Vec::new(); self.steps.len()];
        let mut in_degree = vec![0; self.steps.len()];

        for (i, step) in self.steps.iter().enumerate() {
            for dep in step.dependencies() {
                if let Some(&dep_idx) = name_to_index.get(dep) {
                    graph[dep_idx].push(i);
                    in_degree[i] += 1;
                } else {
                    return Err(TestHarnessError::ExecutionError(format!(
                        "Step {} depends on unknown step {}",
                        step.name(),
                        dep
                    )));
                }
            }
        }

        // Topological sort
        let mut queue = std::collections::VecDeque::new();
        for (i, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(i);
            }
        }

        let mut sorted_indices = Vec::new();
        while let Some(i) = queue.pop_front() {
            sorted_indices.push(i);
            for &j in &graph[i] {
                in_degree[j] -= 1;
                if in_degree[j] == 0 {
                    queue.push_back(j);
                }
            }
        }

        // Check for cycles
        if sorted_indices.len() != self.steps.len() {
            return Err(TestHarnessError::ExecutionError(
                "Cyclic dependencies detected in scenario steps".to_string(),
            ));
        }

        // Create sorted steps
        let sorted_steps = sorted_indices.into_iter().map(|i| &self.steps[i]).collect();

        Ok(sorted_steps)
    }
}

/// Create a new scenario
pub fn create_scenario(name: impl Into<String>) -> Scenario {
    Scenario::new(name)
}

/// Create a new scenario step
pub fn create_scenario_step(name: impl Into<String>) -> ScenarioStepBuilder {
    ScenarioStepBuilder::new(name)
}

/// Create a test case from a scenario
pub fn create_test_case_from_scenario(
    scenario: Scenario,
) -> crate::modules::test_harness::types::TestCase {
    let scenario_name = scenario.name.clone();
    let scenario_category = scenario.category;

    crate::modules::test_harness::types::TestCase::new(
        TestContext::new(scenario_category, scenario_name.clone()),
        move |ctx| {
            let scenario = scenario.clone();
            async move { scenario.execute(ctx.clone()).await }.boxed()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future;

    #[tokio::test]
    async fn test_scenario_execution() {
        // Create a simple scenario with two steps
        let step1 = create_scenario_step("step1")
            .with_description("First step")
            .with_execute_fn(|_| {
                async move {
                    Ok(ScenarioStepResult::new("step1").with_status(ScenarioStepStatus::Completed))
                }
                .boxed()
            })
            .build();

        let step2 = create_scenario_step("step2")
            .with_description("Second step")
            .with_dependency("step1")
            .with_execute_fn(|ctx| {
                async move {
                    // Check that step1 completed successfully
                    let step1_result = ctx.get_step_result("step1").unwrap();
                    assert_eq!(step1_result.status, ScenarioStepStatus::Completed);

                    Ok(ScenarioStepResult::new("step2").with_status(ScenarioStepStatus::Completed))
                }
                .boxed()
            })
            .build();

        let scenario = create_scenario("test_scenario")
            .with_description("Test scenario")
            .with_step(step1)
            .with_step(step2);

        // Execute the scenario
        let test_context = TestContext::new(TestCategory::Integration, "test_scenario".to_string());
        let result = scenario.execute(test_context).await.unwrap();

        // Check the result
        assert_eq!(result.outcome, TestOutcome::Passed);
    }

    #[tokio::test]
    async fn test_scenario_with_failing_step() {
        // Create a scenario with a failing step
        let step1 = create_scenario_step("step1")
            .with_description("First step")
            .with_execute_fn(|_| {
                async move {
                    Ok(ScenarioStepResult::new("step1")
                        .with_status(ScenarioStepStatus::Failed)
                        .with_error("Step failed".to_string()))
                }
                .boxed()
            })
            .build();

        let step2 = create_scenario_step("step2")
            .with_description("Second step")
            .with_dependency("step1")
            .with_execute_fn(|_| {
                async move {
                    Ok(ScenarioStepResult::new("step2").with_status(ScenarioStepStatus::Completed))
                }
                .boxed()
            })
            .build();

        let scenario = create_scenario("test_scenario")
            .with_description("Test scenario")
            .with_step(step1)
            .with_step(step2)
            .with_fail_fast(true);

        // Execute the scenario
        let test_context = TestContext::new(TestCategory::Integration, "test_scenario".to_string());
        let result = scenario.execute(test_context).await.unwrap();

        // Check the result
        assert_eq!(result.outcome, TestOutcome::Failed);
    }

    #[tokio::test]
    async fn test_scenario_with_skipped_step() {
        // Create a scenario with a skipped step
        let step1 = create_scenario_step("step1")
            .with_description("First step")
            .with_skip_fn(|_| async move { Ok(true) }.boxed())
            .build();

        let step2 = create_scenario_step("step2")
            .with_description("Second step")
            .with_execute_fn(|_| {
                async move {
                    Ok(ScenarioStepResult::new("step2").with_status(ScenarioStepStatus::Completed))
                }
                .boxed()
            })
            .build();

        let scenario = create_scenario("test_scenario")
            .with_description("Test scenario")
            .with_step(step1)
            .with_step(step2);

        // Execute the scenario
        let test_context = TestContext::new(TestCategory::Integration, "test_scenario".to_string());
        let result = scenario.execute(test_context).await.unwrap();

        // Check the result
        assert_eq!(result.outcome, TestOutcome::Passed);
    }
}
