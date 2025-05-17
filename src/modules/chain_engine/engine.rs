//! Chain Engine
//!
//! This module provides the core execution engine for chains.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

use crate::modules::chain_engine::chain_definition::{
    Chain, ChainStep, Condition, DependencyType, StepType,
};
use crate::modules::chain_engine::condition_evaluator::ConditionEvaluator;
use crate::modules::chain_engine::context::{ChainContext, StepResult};
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::executors::{
    conditional::ConditionalExecutor, custom::CustomExecutor, function::FunctionCallExecutor,
    llm::LLMInferenceExecutor, loop_executor::LoopExecutor, parallel::ParallelExecutor,
    tool::ToolUseExecutor, StepExecutor,
};
use crate::modules::chain_engine::validation::validate_chain;

/// Chain engine for executing chains
#[derive(Clone)]
pub struct ChainEngine {
    executors: Arc<RwLock<HashMap<String, Arc<dyn StepExecutor>>>>,
}

impl ChainEngine {
    /// Create a new chain engine
    pub fn new() -> Self {
        Self {
            executors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a step executor for a specific step type
    pub fn register_executor(&self, step_type: &str, executor: Arc<dyn StepExecutor>) {
        let mut executors = self.executors.write().unwrap();
        executors.insert(step_type.to_string(), executor);
    }

    /// Execute a chain with the given inputs
    pub async fn execute_chain(
        &self,
        chain: &Chain,
        inputs: HashMap<String, serde_json::Value>,
    ) -> ChainResult<HashMap<String, serde_json::Value>> {
        // Validate the chain
        validate_chain(chain)?;

        // Create execution context
        let context = Arc::new(Mutex::new(ChainContext {
            chain_id: chain.id.clone(),
            variables: chain
                .variables
                .iter()
                .filter_map(|(name, var)| {
                    var.initial_value.clone().map(|value| (name.clone(), value))
                })
                .collect(),
            step_results: HashMap::new(),
            inputs,
            outputs: HashMap::new(),
        }));

        // Build execution plan
        let execution_plan = self.build_execution_plan(chain)?;

        // Execute the plan
        self.execute_plan(chain, execution_plan, context.clone())
            .await?;

        // Return the outputs
        let final_context = context.lock().await;
        Ok(final_context.outputs.clone())
    }

    /// Build an execution plan for a chain
    fn build_execution_plan(&self, chain: &Chain) -> ChainResult<Vec<String>> {
        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_graph: HashMap<String, Vec<String>> = HashMap::new();

        // Add all steps to the graph
        for step_id in chain.steps.keys() {
            graph.insert(step_id.clone(), Vec::new());
            reverse_graph.insert(step_id.clone(), Vec::new());
        }

        // Add dependencies to the graph
        for dependency in &chain.dependencies {
            let dependent_step = &dependency.dependent_step;

            match &dependency.dependency_type {
                DependencyType::Simple { required_step } => {
                    graph
                        .get_mut(required_step)
                        .unwrap()
                        .push(dependent_step.clone());
                    reverse_graph
                        .get_mut(dependent_step)
                        .unwrap()
                        .push(required_step.clone());
                }
                DependencyType::All { required_steps } => {
                    for step_id in required_steps {
                        graph.get_mut(step_id).unwrap().push(dependent_step.clone());
                        reverse_graph
                            .get_mut(dependent_step)
                            .unwrap()
                            .push(step_id.clone());
                    }
                }
                DependencyType::Any { required_steps } => {
                    for step_id in required_steps {
                        graph.get_mut(step_id).unwrap().push(dependent_step.clone());
                        reverse_graph
                            .get_mut(dependent_step)
                            .unwrap()
                            .push(step_id.clone());
                    }
                }
                DependencyType::Conditional { required_step, .. } => {
                    graph
                        .get_mut(required_step)
                        .unwrap()
                        .push(dependent_step.clone());
                    reverse_graph
                        .get_mut(dependent_step)
                        .unwrap()
                        .push(required_step.clone());
                }
            }
        }

        // Perform topological sort
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for step_id in chain.steps.keys() {
            if !visited.contains(step_id) {
                self.topological_sort(&reverse_graph, step_id, &mut visited, &mut stack)?;
            }
        }

        Ok(stack)
    }

    /// Perform topological sort using DFS
    fn topological_sort(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> ChainResult<()> {
        visited.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.topological_sort(graph, neighbor, visited, stack)?;
                }
            }
        } else {
            return Err(ChainError::StepNotFound(node.to_string()));
        }

        stack.push(node.to_string());
        Ok(())
    }

    /// Execute a plan
    async fn execute_plan(
        &self,
        chain: &Chain,
        plan: Vec<String>,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        // Track completed steps
        let completed_steps = Arc::new(Mutex::new(HashSet::new()));

        // Execute steps in the plan
        for step_id in plan {
            let step = chain.steps.get(&step_id).ok_or_else(|| {
                ChainError::StepNotFound(format!("Step not found in execution plan: {}", step_id))
            })?;

            // Check if the step should be executed
            if let Some(condition) = &step.condition {
                let context_guard = context.lock().await;
                if !self.evaluate_condition(condition, &context_guard)? {
                    continue;
                }
            }

            // Check if dependencies are satisfied
            let mut dependencies_satisfied = true;
            for dependency in &chain.dependencies {
                if dependency.dependent_step == step_id {
                    match &dependency.dependency_type {
                        DependencyType::Simple { required_step } => {
                            let completed = completed_steps.lock().await;
                            if !completed.contains(required_step) {
                                dependencies_satisfied = false;
                                break;
                            }
                        }
                        DependencyType::All { required_steps } => {
                            let completed = completed_steps.lock().await;
                            if !required_steps.iter().all(|s| completed.contains(s)) {
                                dependencies_satisfied = false;
                                break;
                            }
                        }
                        DependencyType::Any { required_steps } => {
                            let completed = completed_steps.lock().await;
                            if !required_steps.iter().any(|s| completed.contains(s)) {
                                dependencies_satisfied = false;
                                break;
                            }
                        }
                        DependencyType::Conditional {
                            required_step,
                            condition,
                        } => {
                            let completed = completed_steps.lock().await;
                            if !completed.contains(required_step) {
                                dependencies_satisfied = false;
                                break;
                            }

                            let context_guard = context.lock().await;
                            if !self.evaluate_condition(condition, &context_guard)? {
                                dependencies_satisfied = false;
                                break;
                            }
                        }
                    }
                }
            }

            if !dependencies_satisfied {
                continue;
            }

            // Execute the step
            match &step.step_type {
                StepType::LLMInference { .. } => {
                    self.execute_llm_inference_step(step, context.clone())
                        .await?;
                }
                StepType::FunctionCall { .. } => {
                    self.execute_function_call_step(step, context.clone())
                        .await?;
                }
                StepType::ToolUse { .. } => {
                    self.execute_tool_use_step(step, context.clone()).await?;
                }
                StepType::Conditional {
                    branches,
                    default_branch,
                } => {
                    self.execute_conditional_step(
                        step,
                        branches,
                        default_branch.clone(),
                        chain,
                        context.clone(),
                    )
                    .await?;
                }
                StepType::Parallel {
                    steps,
                    wait_for_all,
                } => {
                    self.execute_parallel_step(step, steps, *wait_for_all, chain, context.clone())
                        .await?;
                }
                StepType::Loop {
                    iteration_variable,
                    max_iterations,
                    steps,
                    break_condition,
                } => {
                    self.execute_loop_step(
                        step,
                        iteration_variable,
                        *max_iterations,
                        steps,
                        break_condition.as_ref(),
                        chain,
                        context.clone(),
                    )
                    .await?;
                }
                StepType::Custom { handler, config } => {
                    self.execute_custom_step(step, handler, config, context.clone())
                        .await?;
                }
            }

            // Mark the step as completed
            completed_steps.lock().await.insert(step_id);
        }

        Ok(())
    }

    /// Execute an LLM inference step
    pub async fn execute_llm_inference_step(
        &self,
        step: &ChainStep,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = LLMInferenceExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Execute a function call step
    pub async fn execute_function_call_step(
        &self,
        step: &ChainStep,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = FunctionCallExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Execute a tool use step
    pub async fn execute_tool_use_step(
        &self,
        step: &ChainStep,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = ToolUseExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Execute a conditional step
    pub async fn execute_conditional_step(
        &self,
        step: &ChainStep,
        branches: &[crate::modules::chain_engine::chain_definition::ConditionalBranch],
        default_branch: Option<String>,
        chain: &Chain,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = ConditionalExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Execute a parallel step
    pub async fn execute_parallel_step(
        &self,
        step: &ChainStep,
        steps: &[String],
        wait_for_all: bool,
        chain: &Chain,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = ParallelExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Execute a loop step
    pub async fn execute_loop_step(
        &self,
        step: &ChainStep,
        iteration_variable: &str,
        max_iterations: Option<u32>,
        steps: &[String],
        break_condition: Option<&Condition>,
        chain: &Chain,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = LoopExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Execute a custom step
    pub async fn execute_custom_step(
        &self,
        step: &ChainStep,
        handler: &str,
        config: &HashMap<String, serde_json::Value>,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let executor = CustomExecutor::new();
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard.step_results.insert(step.id.clone(), result);

        Ok(())
    }

    /// Evaluate a condition
    pub fn evaluate_condition(
        &self,
        condition: &Condition,
        context: &ChainContext,
    ) -> ChainResult<bool> {
        let evaluator = ConditionEvaluator::new();
        evaluator.evaluate_condition(condition, context)
    }
}
