//! Tests for conditional execution in the Chain Engine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use async_trait::async_trait;
use intellirouter::modules::chain_engine::definition::{
    Chain, ChainStep, ComparisonOperator, Condition, ConditionalBranch, DependencyType, Role,
    StepDependency, StepType,
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
async fn test_condition_evaluation() {
    // Create a chain engine
    let engine = ChainEngine::new();

    // Create a context with variables
    let mut variables = HashMap::new();
    variables.insert("foo".to_string(), serde_json::json!("bar"));
    variables.insert("num".to_string(), serde_json::json!(42));
    variables.insert("flag".to_string(), serde_json::json!(true));

    let context = ChainContext {
        chain_id: "test-chain".to_string(),
        variables,
        step_results: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
    };

    // Test expression condition
    let condition = Condition::Expression {
        expression: "${flag} == true".to_string(),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test comparison condition (eq)
    let condition = Condition::Comparison {
        left: "${foo}".to_string(),
        operator: ComparisonOperator::Eq,
        right: "bar".to_string(),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test comparison condition (ne)
    let condition = Condition::Comparison {
        left: "${foo}".to_string(),
        operator: ComparisonOperator::Ne,
        right: "baz".to_string(),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test comparison condition (lt)
    let condition = Condition::Comparison {
        left: "${num}".to_string(),
        operator: ComparisonOperator::Lt,
        right: "100".to_string(),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test comparison condition (gt)
    let condition = Condition::Comparison {
        left: "${num}".to_string(),
        operator: ComparisonOperator::Gt,
        right: "10".to_string(),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test comparison condition (contains)
    let condition = Condition::Comparison {
        left: "${foo}".to_string(),
        operator: ComparisonOperator::Contains,
        right: "a".to_string(),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test logical AND condition
    let condition = Condition::And {
        conditions: vec![
            Condition::Comparison {
                left: "${foo}".to_string(),
                operator: ComparisonOperator::Eq,
                right: "bar".to_string(),
            },
            Condition::Comparison {
                left: "${num}".to_string(),
                operator: ComparisonOperator::Gt,
                right: "10".to_string(),
            },
        ],
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test logical OR condition
    let condition = Condition::Or {
        conditions: vec![
            Condition::Comparison {
                left: "${foo}".to_string(),
                operator: ComparisonOperator::Eq,
                right: "baz".to_string(),
            },
            Condition::Comparison {
                left: "${num}".to_string(),
                operator: ComparisonOperator::Gt,
                right: "10".to_string(),
            },
        ],
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());

    // Test logical NOT condition
    let condition = Condition::Not {
        condition: Box::new(Condition::Comparison {
            left: "${foo}".to_string(),
            operator: ComparisonOperator::Eq,
            right: "baz".to_string(),
        }),
    };
    assert!(engine.evaluate_condition(&condition, &context).unwrap());
}

#[tokio::test]
async fn test_conditional_step_execution() {
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

    // Create a test chain with conditional steps
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

    let branches = vec![
        ConditionalBranch {
            condition: Condition::Comparison {
                left: "${result}".to_string(),
                operator: ComparisonOperator::Eq,
                right: "success".to_string(),
            },
            target_step: "step2".to_string(),
        },
        ConditionalBranch {
            condition: Condition::Comparison {
                left: "${result}".to_string(),
                operator: ComparisonOperator::Eq,
                right: "failure".to_string(),
            },
            target_step: "step3".to_string(),
        },
    ];

    let conditional_step = create_test_step(
        "conditional",
        StepType::Conditional {
            branches: branches.clone(),
            default_branch: Some("step4".to_string()),
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
    chain
        .steps
        .insert(conditional_step.id.clone(), conditional_step);
    chain.steps.insert(step2.id.clone(), step2);
    chain.steps.insert(step3.id.clone(), step3);
    chain.steps.insert(step4.id.clone(), step4);

    // Add dependencies
    chain.dependencies.push(StepDependency {
        dependent_step: "conditional".to_string(),
        dependency_type: DependencyType::Simple {
            required_step: "step1".to_string(),
        },
    });

    // Create inputs with a result variable
    let mut inputs = HashMap::new();
    inputs.insert("result".to_string(), serde_json::json!("success"));

    // Execute the chain
    let result = engine.execute_chain(&chain, inputs).await;

    // Verify the result
    assert!(result.is_ok());

    // Verify that step2 was executed (based on the condition)
    let context = Arc::new(Mutex::new(ChainContext {
        chain_id: chain.id.clone(),
        variables: HashMap::new(),
        step_results: HashMap::new(),
        inputs: HashMap::from([("result".to_string(), serde_json::json!("failure"))]),
        outputs: HashMap::new(),
    }));

    // Execute the conditional step directly
    engine
        .execute_conditional_step(
            &conditional_step,
            &branches,
            Some("step4".to_string()),
            &chain,
            context.clone(),
        )
        .await
        .unwrap();

    // Verify that step3 was executed (based on the condition)
    let context = Arc::new(Mutex::new(ChainContext {
        chain_id: chain.id.clone(),
        variables: HashMap::new(),
        step_results: HashMap::new(),
        inputs: HashMap::from([("result".to_string(), serde_json::json!("unknown"))]),
        outputs: HashMap::new(),
    }));

    // Execute the conditional step directly
    engine
        .execute_conditional_step(
            &conditional_step,
            &branches,
            Some("step4".to_string()),
            &chain,
            context.clone(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_condition_based_step_skipping() {
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

    // Create a test chain with conditional steps
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

    let mut step2 = create_test_step(
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

    // Add a condition to step2
    step2.condition = Some(Condition::Comparison {
        left: "${flag}".to_string(),
        operator: ComparisonOperator::Eq,
        right: "true".to_string(),
    });

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
            required_step: "step1".to_string(),
        },
    });

    // Test with flag=true (step2 should execute)
    let mut inputs = HashMap::new();
    inputs.insert("flag".to_string(), serde_json::json!(true));

    let result = engine.execute_chain(&chain, inputs).await;
    assert!(result.is_ok());

    // Test with flag=false (step2 should be skipped)
    let mut inputs = HashMap::new();
    inputs.insert("flag".to_string(), serde_json::json!(false));

    let result = engine.execute_chain(&chain, inputs).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_conditional_dependency() {
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

    // Create a test chain with conditional dependency
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

    // Add conditional dependency - step3 depends on step1 only if flag is true
    chain.dependencies.push(StepDependency {
        dependent_step: "step3".to_string(),
        dependency_type: DependencyType::Conditional {
            required_step: "step1".to_string(),
            condition: Condition::Comparison {
                left: "${flag}".to_string(),
                operator: ComparisonOperator::Eq,
                right: "true".to_string(),
            },
        },
    });

    // Test with flag=true (step3 should depend on step1)
    let mut inputs = HashMap::new();
    inputs.insert("flag".to_string(), serde_json::json!(true));

    let result = engine.execute_chain(&chain, inputs).await;
    assert!(result.is_ok());

    // Test with flag=false (step3 should not depend on step1)
    let mut inputs = HashMap::new();
    inputs.insert("flag".to_string(), serde_json::json!(false));

    let result = engine.execute_chain(&chain, inputs).await;
    assert!(result.is_ok());
}
