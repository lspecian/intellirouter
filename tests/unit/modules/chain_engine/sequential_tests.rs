//! Tests for sequential execution in the Chain Engine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use async_trait::async_trait;
use intellirouter::modules::chain_engine::definition::{
    Chain, ChainStep, DependencyType, Role, StepDependency, StepType,
};
use intellirouter::modules::chain_engine::error::{ChainError, ChainResult};
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
async fn test_sequential_execution() {
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

    // Create a test chain with sequential steps
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
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "step2".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step2".to_string(),
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took at least 300ms (3 steps * 100ms)
    assert!(
        duration >= Duration::from_millis(300),
        "Sequential execution was too fast: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_sequential_execution_with_all_dependency() {
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

    // Create a test chain with sequential steps
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

    chain.steps.insert(step1.id.clone(), step1);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);

    // Add dependencies - step4 depends on both step2 and step3
    chain.dependencies.push(StepDependency {
        dependent_step: "step2".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step4".to_string(),
        dependency_type: DependencyType::All {
            required_steps: vec!["step2".to_string(), "step3".to_string()],
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took at least 400ms (step1 + step2 & step3 in parallel + step4)
    // Since step2 and step3 both depend on step1 but not on each other, they can run in parallel
    // But the engine executes them sequentially in the current implementation
    assert!(
        duration >= Duration::from_millis(400),
        "Sequential execution was too fast: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_sequential_execution_with_any_dependency() {
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

    // Create a test chain with sequential steps
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
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);

    // Add dependencies - step3 depends on either step1 or step2
    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Any {
            required_steps: vec!["step1".to_string(), "step2".to_string()],
        },
    });

    // Execute the chain
    let start = Instant::now();
    let result = engine.execute_chain(&chain, HashMap::new()).await;
    let duration = start.elapsed();

    // Verify the result
    assert!(result.is_ok(), "Chain execution failed: {:?}", result.err());

    // Verify that the execution took at least 200ms (step1 + step3)
    // Since step3 depends on either step1 or step2, and step1 is executed first,
    // step3 can run immediately after step1
    assert!(
        duration >= Duration::from_millis(200),
        "Sequential execution was too fast: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_sequential_execution_with_error() {
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

    // Create a test chain with sequential steps
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

    let step2 = create_test_step(
        "step2",
        StepType::FunctionCall {
            function_name: "test-function".to_string(),
            arguments: HashMap::new(),
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
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "step2".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step2".to_string(),
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
                msg.contains("step2"),
                "Error message should mention step2: {}",
                msg
            );
        }
        _ => panic!("Expected StepExecutionError"),
    }
}
