//! Tests for parallel execution in the Chain Engine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::modules::chain_engine::chain_definition::{
    Chain, ChainStep, DependencyType, Role, StepDependency, StepType,
};
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::{ChainContext, ChainEngine, StepExecutor, StepResult};
use async_trait::async_trait;

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
            crate::modules::chain_engine::chain_definition::ErrorHandlingStrategy::StopOnError,
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
async fn test_parallel_execution() {
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

    // Create a test chain with parallel steps
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

    let parallel_steps = vec![
        "step2".to_string(),
        "step3".to_string(),
        "step4".to_string(),
    ];

    let parallel_step = create_test_step(
        "parallel",
        StepType::Parallel {
            steps: parallel_steps.clone(),
            wait_for_all: true,
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

    let step5 = create_test_step(
        "step5",
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
    chain.steps.insert(parallel_step.id.clone(), parallel_step);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);
    chain.steps.insert(step5.id.clone(), step5);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "parallel".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step5".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "parallel".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took less than 500ms
    // (100ms for step1 + 100ms for parallel steps + 100ms for step5)
    assert!(
        duration < Duration::from_millis(500),
        "Parallel execution was too slow: {:?}",
        duration
    );

    // Verify that the execution took at least 300ms
    // (100ms for step1 + 100ms for parallel steps + 100ms for step5)
    assert!(
        duration >= Duration::from_millis(300),
        "Execution was too fast: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_parallel_execution_with_error() {
    // Create a chain engine
    let engine = ChainEngine::new();

    // Register a mock executor that succeeds
    let mock_success_executor = Arc::new(MockStepExecutor::new(
        Duration::from_millis(100),
        HashMap::from([("result".to_string(), serde_json::json!("success"))]),
    ));
    engine
        .register_executor("LLMInference", mock_success_executor.clone())
        .await;

    // Register a mock executor that fails
    struct FailingExecutor;

    #[async_trait]
    impl StepExecutor for FailingExecutor {
        async fn execute_step(
            &self,
            step: &ChainStep,
            _context: &ChainContext,
        ) -> ChainResult<StepResult> {
            Err(ChainError::StepExecutionError(format!(
                "Failed to execute step {}",
                step.id
            )))
        }
    }

    engine
        .register_executor("FunctionCall", Arc::new(FailingExecutor))
        .await;

    // Create a test chain with parallel steps, one of which will fail
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

    let parallel_steps = vec![
        "step2".to_string(),
        "step3".to_string(),
        "step4".to_string(),
    ];

    let parallel_step = create_test_step(
        "parallel",
        StepType::Parallel {
            steps: parallel_steps.clone(),
            wait_for_all: true,
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
        StepType::FunctionCall {
            function_name: "test-function".to_string(),
            arguments: HashMap::new(),
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
    chain.steps.insert(parallel_step.id.clone(), parallel_step);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "parallel".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    // Execute the chain
    let result = engine.execute_chain(&chain, HashMap::new()).await;

    // Verify that the execution failed
    assert!(result.is_err(), "Chain execution should have failed");

    // Verify the error message
    match result {
        Err(ChainError::StepExecutionError(msg)) => {
            assert!(
                msg.contains("step3"),
                "Error message should mention step3: {}",
                msg
            );
        }
        _ => panic!("Expected StepExecutionError"),
    }
}

#[tokio::test]
async fn test_parallel_execution_without_waiting() {
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

    // Create a test chain with parallel steps
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

    let parallel_steps = vec![
        "step2".to_string(),
        "step3".to_string(),
        "step4".to_string(),
    ];

    let parallel_step = create_test_step(
        "parallel",
        StepType::Parallel {
            steps: parallel_steps.clone(),
            wait_for_all: false, // Don't wait for parallel steps to complete
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

    let step5 = create_test_step(
        "step5",
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
    chain.steps.insert(parallel_step.id.clone(), parallel_step);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);
    chain.steps.insert(step5.id.clone(), step5);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "parallel".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step5".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "parallel".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took less than 300ms
    // (100ms for step1 + ~0ms for launching parallel steps + 100ms for step5)
    assert!(
        duration < Duration::from_millis(300),
        "Execution was too slow: {:?}",
        duration
    );

    // Verify that the execution took at least 200ms
    // (100ms for step1 + ~0ms for launching parallel steps + 100ms for step5)
    assert!(
        duration >= Duration::from_millis(200),
        "Execution was too fast: {:?}",
        duration
    );

    // Sleep a bit to allow the parallel steps to complete
    tokio::time::sleep(Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_nested_parallel_execution() {
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

    // Create a test chain with nested parallel steps
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

    // First level parallel steps
    let parallel_steps1 = vec!["nested_parallel".to_string(), "step3".to_string()];

    let parallel_step1 = create_test_step(
        "parallel1",
        StepType::Parallel {
            steps: parallel_steps1.clone(),
            wait_for_all: true,
        },
    );

    // Nested parallel steps
    let parallel_steps2 = vec!["step4".to_string(), "step5".to_string()];

    let parallel_step2 = create_test_step(
        "nested_parallel",
        StepType::Parallel {
            steps: parallel_steps2.clone(),
            wait_for_all: true,
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

    let step5 = create_test_step(
        "step5",
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

    let step6 = create_test_step(
        "step6",
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
        .insert(parallel_step1.id.clone(), parallel_step1);
    chain
        .steps
        .insert(parallel_step2.id.clone(), parallel_step2);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);
    chain.steps.insert(step5.id.clone(), step5);
    chain.steps.insert(step6.id.clone(), step6);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "parallel1".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step6".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "parallel1".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took less than 400ms
    // (100ms for step1 + 100ms for parallel steps (which includes nested parallel) + 100ms for step6)
    assert!(
        duration < Duration::from_millis(400),
        "Nested parallel execution was too slow: {:?}",
        duration
    );

    // Verify that the execution took at least 300ms
    // (100ms for step1 + 100ms for parallel steps + 100ms for step6)
    assert!(
        duration >= Duration::from_millis(300),
        "Execution was too fast: {:?}",
        duration
    );
}
