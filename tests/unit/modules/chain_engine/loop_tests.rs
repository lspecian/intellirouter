//! Tests for loop execution in the Chain Engine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use intellirouter::modules::chain_engine::definition::{
    Chain, ChainStep, ComparisonOperator, Condition, DependencyType, Role, StepDependency, StepType,
};
use intellirouter::modules::chain_engine::error::ChainResult;
use intellirouter::modules::chain_engine::{ChainContext, ChainEngine, StepExecutor, StepResult};

/// Mock step executor for testing
struct MockStepExecutor {
    delay: Duration,
    result: HashMap<String, serde_json::Value>,
}

impl MockStepExecutor {
    fn new(delay: Duration, result: HashMap<String, serde_json::Value>) -> Self {
        Self { delay, result }
    }
}

#[async_trait]
impl StepExecutor for MockStepExecutor {
    async fn execute_step(
        &self,
        step: &ChainStep,
        _context: &ChainContext,
    ) -> ChainResult<StepResult> {
        // Simulate some work
        tokio::time::sleep(self.delay).await;

        Ok(StepResult {
            step_id: step.id.clone(),
            outputs: self.result.clone(),
            error: None,
            execution_time: self.delay,
        })
    }
}

/// Helper function to create a test chain
fn create_test_chain() -> Chain {
    Chain {
        id: "test-chain".to_string(),
        name: "Test Chain".to_string(),
        description: "A test chain".to_string(),
        version: "1.0.0".to_string(),
        tags: vec!["test".to_string()],
        metadata: HashMap::new(),
        steps: HashMap::new(),
        dependencies: Vec::new(),
        variables: HashMap::new(),
        error_handling:
            intellirouter::modules::chain_engine::definition::ErrorHandlingStrategy::StopOnError,
        max_parallel_steps: None,
        timeout: None,
    }
}

/// Helper function to create a test step
fn create_test_step(id: &str, step_type: StepType) -> ChainStep {
    ChainStep {
        id: id.to_string(),
        name: format!("Step {}", id),
        description: format!("Test step {}", id),
        step_type,
        role: Role::System,
        inputs: Vec::new(),
        outputs: Vec::new(),
        condition: None,
        retry_policy: None,
        timeout: None,
        error_handler: None,
    }
}

#[tokio::test]
async fn test_loop_step_execution() {
    // Create a chain engine
    let engine = ChainEngine::new();

    // Register a mock executor
    let mock_executor = Arc::new(MockStepExecutor::new(
        Duration::from_millis(100),
        HashMap::from([("result".to_string(), serde_json::json!("success"))]),
    ));
    engine
        .register_executor("LLMInference", mock_executor.clone())
        .await;

    // Create a test chain with loop steps
    let mut chain = create_test_chain();

    // Add steps to the chain
    let step1 = create_test_step(
        "step1",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    let loop_steps = vec!["step2".to_string()];

    let loop_step = create_test_step(
        "loop",
        StepType::Loop {
            iteration_variable: "i".to_string(),
            max_iterations: Some(3),
            steps: loop_steps.clone(),
            break_condition: None,
        },
    );

    let step2 = create_test_step(
        "step2",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    let step3 = create_test_step(
        "step3",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    chain.steps.insert(step1.id.clone(), step1);
    chain.steps.insert(loop_step.id.clone(), loop_step);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "loop".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "loop".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took at least 500ms
    // (100ms for step1 + 3 * 100ms for loop steps + 100ms for step3)
    assert!(
        duration >= Duration::from_millis(500),
        "Loop execution was too fast: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_loop_step_with_break_condition() {
    // Create a chain engine
    let engine = ChainEngine::new();

    // Register a mock executor
    let mock_executor = Arc::new(MockStepExecutor::new(
        Duration::from_millis(100),
        HashMap::from([("result".to_string(), serde_json::json!("success"))]),
    ));
    engine
        .register_executor("LLMInference", mock_executor.clone())
        .await;

    // Create a test chain with loop steps
    let mut chain = create_test_chain();

    // Add steps to the chain
    let step1 = create_test_step(
        "step1",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    let loop_steps = vec!["step2".to_string()];

    let loop_step = create_test_step(
        "loop",
        StepType::Loop {
            iteration_variable: "i".to_string(),
            max_iterations: Some(10), // Set a high max to test break condition
            steps: loop_steps.clone(),
            break_condition: Some(Condition::Comparison {
                left: "${i}".to_string(),
                operator: ComparisonOperator::Gte,
                right: "2".to_string(),
            }),
        },
    );

    let step2 = create_test_step(
        "step2",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    let step3 = create_test_step(
        "step3",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    chain.steps.insert(step1.id.clone(), step1);
    chain.steps.insert(loop_step.id.clone(), loop_step);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "loop".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "loop".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took at least 400ms
    // (100ms for step1 + 2 * 100ms for loop steps + 100ms for step3)
    assert!(
        duration >= Duration::from_millis(400),
        "Loop execution was too fast: {:?}",
        duration
    );

    // Verify that the execution took less than 500ms
    // (100ms for step1 + 2 * 100ms for loop steps + 100ms for step3)
    // This ensures that the break condition was triggered
    assert!(
        duration < Duration::from_millis(500),
        "Break condition was not triggered: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_nested_loop_step_execution() {
    // Create a chain engine
    let engine = ChainEngine::new();

    // Register a mock executor
    let mock_executor = Arc::new(MockStepExecutor::new(
        Duration::from_millis(100),
        HashMap::from([("result".to_string(), serde_json::json!("success"))]),
    ));
    engine
        .register_executor("LLMInference", mock_executor.clone())
        .await;

    // Create a test chain with nested loop steps
    let mut chain = create_test_chain();

    // Add steps to the chain
    let step1 = create_test_step(
        "step1",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    let inner_loop_steps = vec!["step3".to_string()];

    let inner_loop_step = create_test_step(
        "inner_loop",
        StepType::Loop {
            iteration_variable: "j".to_string(),
            max_iterations: Some(2),
            steps: inner_loop_steps.clone(),
            break_condition: None,
        },
    );

    let outer_loop_steps = vec!["inner_loop".to_string()];

    let outer_loop_step = create_test_step(
        "outer_loop",
        StepType::Loop {
            iteration_variable: "i".to_string(),
            max_iterations: Some(2),
            steps: outer_loop_steps.clone(),
            break_condition: None,
        },
    );

    let step3 = create_test_step(
        "step3",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    let step4 = create_test_step(
        "step4",
        StepType::LLMInference {
            model: "test-model".to_string(),
            system_prompt: Some("Test prompt".to_string()),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
            additional_params: HashMap::new(),
        },
    );

    chain.steps.insert(step1.id.clone(), step1);
    chain
        .steps
        .insert(outer_loop_step.id.clone(), outer_loop_step);
    chain
        .steps
        .insert(inner_loop_step.id.clone(), inner_loop_step);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "outer_loop".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step4".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "outer_loop".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took at least 600ms
    // (100ms for step1 + 2 * 2 * 100ms for nested loop steps + 100ms for step4)
    assert!(
        duration >= Duration::from_millis(600),
        "Nested loop execution was too fast: {:?}",
        duration
    );
}
