//! Chain Engine core
//!
//! This module provides the core execution engine for chains.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

use crate::modules::chain_engine::chain_definition::{
    Chain, ChainStep, ComparisonOperator, Condition, DependencyType, StepType,
};
use crate::modules::chain_engine::error::{ChainError, ChainResult};
use crate::modules::chain_engine::validation::validate_chain;

/// Result of a step execution
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub outputs: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: Duration,
}

/// Context for chain execution
#[derive(Debug, Clone)]
pub struct ChainContext {
    pub chain_id: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub step_results: HashMap<String, StepResult>,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: HashMap<String, serde_json::Value>,
}

/// Interface for step executors
#[async_trait]
pub trait StepExecutor: Send + Sync {
    async fn execute_step(
        &self,
        step: &ChainStep,
        context: &ChainContext,
    ) -> ChainResult<StepResult>;
}

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
    pub async fn register_executor(&self, step_type: &str, executor: Arc<dyn StepExecutor>) {
        let mut executors = self.executors.write().await;
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
    async fn execute_llm_inference_step(
        &self,
        step: &ChainStep,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        // Get the executor for LLM inference
        let executors = self.executors.read().await;
        let executor = executors.get("LLMInference").ok_or_else(|| {
            ChainError::Other("No executor registered for LLM inference".to_string())
        })?;

        // Execute the step
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard
            .step_results
            .insert(step.id.clone(), result.clone());

        // Process outputs
        for output in &step.outputs {
            let value = result.outputs.get(&output.name).ok_or_else(|| {
                ChainError::Other(format!("Output not found in step result: {}", output.name))
            })?;

            // Apply transformation if specified
            let transformed_value = if let Some(transform) = &output.transform {
                // This would call a function to apply the transformation
                // apply_transform(transform, value)?
                value.clone() // Simplified for now
            } else {
                value.clone()
            };

            // Store the output according to the target
            match &output.target {
                crate::modules::chain_engine::chain_definition::DataTarget::Variable {
                    variable_name,
                } => {
                    context_guard
                        .variables
                        .insert(variable_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::ChainOutput {
                    output_name,
                } => {
                    context_guard
                        .outputs
                        .insert(output_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::StepInput {
                    ..
                } => {
                    // This will be handled when the target step is executed
                }
            }
        }

        Ok(())
    }

    /// Execute a function call step
    async fn execute_function_call_step(
        &self,
        step: &ChainStep,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        // Get the executor for function calls
        let executors = self.executors.read().await;
        let executor = executors.get("FunctionCall").ok_or_else(|| {
            ChainError::Other("No executor registered for function calls".to_string())
        })?;

        // Execute the step
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard
            .step_results
            .insert(step.id.clone(), result.clone());

        // Process outputs
        for output in &step.outputs {
            let value = result.outputs.get(&output.name).ok_or_else(|| {
                ChainError::Other(format!("Output not found in step result: {}", output.name))
            })?;

            // Apply transformation if specified
            let transformed_value = if let Some(transform) = &output.transform {
                // This would call a function to apply the transformation
                // apply_transform(transform, value)?
                value.clone() // Simplified for now
            } else {
                value.clone()
            };

            // Store the output according to the target
            match &output.target {
                crate::modules::chain_engine::chain_definition::DataTarget::Variable {
                    variable_name,
                } => {
                    context_guard
                        .variables
                        .insert(variable_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::ChainOutput {
                    output_name,
                } => {
                    context_guard
                        .outputs
                        .insert(output_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::StepInput {
                    ..
                } => {
                    // This will be handled when the target step is executed
                }
            }
        }

        Ok(())
    }

    /// Execute a tool use step
    async fn execute_tool_use_step(
        &self,
        step: &ChainStep,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        // Get the executor for tool use
        let executors = self.executors.read().await;
        let executor = executors
            .get("ToolUse")
            .ok_or_else(|| ChainError::Other("No executor registered for tool use".to_string()))?;

        // Execute the step
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard
            .step_results
            .insert(step.id.clone(), result.clone());

        // Process outputs
        for output in &step.outputs {
            let value = result.outputs.get(&output.name).ok_or_else(|| {
                ChainError::Other(format!("Output not found in step result: {}", output.name))
            })?;

            // Apply transformation if specified
            let transformed_value = if let Some(transform) = &output.transform {
                // This would call a function to apply the transformation
                // apply_transform(transform, value)?
                value.clone() // Simplified for now
            } else {
                value.clone()
            };

            // Store the output according to the target
            match &output.target {
                crate::modules::chain_engine::chain_definition::DataTarget::Variable {
                    variable_name,
                } => {
                    context_guard
                        .variables
                        .insert(variable_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::ChainOutput {
                    output_name,
                } => {
                    context_guard
                        .outputs
                        .insert(output_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::StepInput {
                    ..
                } => {
                    // This will be handled when the target step is executed
                }
            }
        }

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
        // Evaluate conditions and find the matching branch
        let context_guard = context.lock().await;
        let mut target_step = None;

        for branch in branches {
            if self.evaluate_condition(&branch.condition, &context_guard)? {
                target_step = Some(branch.target_step.clone());
                break;
            }
        }

        // If no branch matched, use the default branch
        let target_step = target_step.or_else(|| default_branch.clone());

        // Execute the target step if found
        if let Some(target_step) = target_step {
            let target = chain.steps.get(&target_step).ok_or_else(|| {
                ChainError::StepNotFound(format!("Target step not found: {}", target_step))
            })?;

            drop(context_guard);

            // Execute the target step based on its type
            match &target.step_type {
                StepType::LLMInference { .. } => {
                    self.execute_llm_inference_step(target, context.clone())
                        .await?;
                }
                StepType::FunctionCall { .. } => {
                    self.execute_function_call_step(target, context.clone())
                        .await?;
                }
                StepType::ToolUse { .. } => {
                    self.execute_tool_use_step(target, context.clone()).await?;
                }
                StepType::Conditional {
                    branches,
                    default_branch,
                } => {
                    self.execute_conditional_step(
                        target,
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
                    self.execute_parallel_step(
                        target,
                        steps,
                        *wait_for_all,
                        chain,
                        context.clone(),
                    )
                    .await?;
                }
                StepType::Loop {
                    iteration_variable,
                    max_iterations,
                    steps,
                    break_condition,
                } => {
                    self.execute_loop_step(
                        target,
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
                    self.execute_custom_step(target, handler, config, context.clone())
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Execute a parallel step
    async fn execute_parallel_step(
        &self,
        step: &ChainStep,
        steps: &[String],
        wait_for_all: bool,
        chain: &Chain,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        // Create a vector to hold the join handles for parallel tasks
        let mut handles = Vec::new();

        // Create a shared context for the parallel steps
        let shared_context = context.clone();

        // Launch each step in parallel
        for step_id in steps {
            let target_step = chain.steps.get(step_id).ok_or_else(|| {
                ChainError::StepNotFound(format!("Parallel step not found: {}", step_id))
            })?;

            // Clone the necessary data for the async task
            let target_step_clone = target_step.clone();
            let chain_clone = chain.clone();
            let context_clone = shared_context.clone();
            let self_clone = self.clone();

            // Spawn a new task for this step
            let handle = tokio::spawn(async move {
                match &target_step_clone.step_type {
                    StepType::LLMInference { .. } => {
                        self_clone
                            .execute_llm_inference_step(&target_step_clone, context_clone)
                            .await
                    }
                    StepType::FunctionCall { .. } => {
                        self_clone
                            .execute_function_call_step(&target_step_clone, context_clone)
                            .await
                    }
                    StepType::ToolUse { .. } => {
                        self_clone
                            .execute_tool_use_step(&target_step_clone, context_clone)
                            .await
                    }
                    StepType::Conditional {
                        branches,
                        default_branch,
                    } => {
                        self_clone
                            .execute_conditional_step(
                                &target_step_clone,
                                branches,
                                default_branch.clone(),
                                &chain_clone,
                                context_clone,
                            )
                            .await
                    }
                    StepType::Parallel {
                        steps: nested_steps,
                        wait_for_all: nested_wait,
                    } => {
                        self_clone
                            .execute_parallel_step(
                                &target_step_clone,
                                nested_steps,
                                *nested_wait,
                                &chain_clone,
                                context_clone,
                            )
                            .await
                    }
                    StepType::Loop {
                        iteration_variable,
                        max_iterations,
                        steps: loop_steps,
                        break_condition,
                    } => {
                        self_clone
                            .execute_loop_step(
                                &target_step_clone,
                                iteration_variable,
                                *max_iterations,
                                loop_steps,
                                break_condition.as_ref(),
                                &chain_clone,
                                context_clone,
                            )
                            .await
                    }
                    StepType::Custom { handler, config } => {
                        self_clone
                            .execute_custom_step(&target_step_clone, handler, config, context_clone)
                            .await
                    }
                }
            });

            handles.push((step_id.clone(), handle));
        }

        // Wait for all steps to complete if required
        if wait_for_all {
            for (step_id, handle) in handles {
                match handle.await {
                    Ok(result) => {
                        if let Err(err) = result {
                            return Err(ChainError::StepExecutionError(format!(
                                "Error executing parallel step {}: {}",
                                step_id, err
                            )));
                        }
                    }
                    Err(err) => {
                        return Err(ChainError::StepExecutionError(format!(
                            "Error joining parallel step {}: {}",
                            step_id, err
                        )));
                    }
                }
            }
        } else {
            // If we don't need to wait for all steps, just fire and forget
            // This is useful for non-blocking parallel steps
            // We'll still log any errors that occur
            tokio::spawn(async move {
                for (step_id, handle) in handles {
                    if let Err(err) = handle.await {
                        eprintln!("Error joining parallel step {}: {}", step_id, err);
                    }
                }
            });
        }

        Ok(())
    }

    /// Execute a loop step
    async fn execute_loop_step(
        &self,
        step: &ChainStep,
        iteration_variable: &str,
        max_iterations: Option<u32>,
        steps: &[String],
        break_condition: Option<&Condition>,
        chain: &Chain,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        let mut iteration = 0;

        loop {
            // Check max iterations
            if let Some(max) = max_iterations {
                if iteration >= max {
                    break;
                }
            }

            // Set iteration variable
            {
                let mut context_guard = context.lock().await;
                context_guard.variables.insert(
                    iteration_variable.to_string(),
                    serde_json::Value::Number(serde_json::Number::from(iteration)),
                );
            }

            // Check break condition
            if let Some(condition) = break_condition {
                let context_guard = context.lock().await;
                if self.evaluate_condition(condition, &context_guard)? {
                    break;
                }
            }

            // Execute steps
            for step_id in steps {
                let target = chain.steps.get(step_id).ok_or_else(|| {
                    ChainError::StepNotFound(format!("Loop step not found: {}", step_id))
                })?;

                match &target.step_type {
                    StepType::LLMInference { .. } => {
                        self.execute_llm_inference_step(target, context.clone())
                            .await?;
                    }
                    StepType::FunctionCall { .. } => {
                        self.execute_function_call_step(target, context.clone())
                            .await?;
                    }
                    StepType::ToolUse { .. } => {
                        self.execute_tool_use_step(target, context.clone()).await?;
                    }
                    StepType::Conditional {
                        branches,
                        default_branch,
                    } => {
                        self.execute_conditional_step(
                            target,
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
                        self.execute_parallel_step(
                            target,
                            steps,
                            *wait_for_all,
                            chain,
                            context.clone(),
                        )
                        .await?;
                    }
                    StepType::Loop {
                        iteration_variable,
                        max_iterations,
                        steps,
                        break_condition,
                    } => {
                        self.execute_loop_step(
                            target,
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
                        self.execute_custom_step(target, handler, config, context.clone())
                            .await?;
                    }
                }
            }

            iteration += 1;
        }

        Ok(())
    }

    /// Execute a custom step
    async fn execute_custom_step(
        &self,
        step: &ChainStep,
        handler: &str,
        config: &HashMap<String, serde_json::Value>,
        context: Arc<Mutex<ChainContext>>,
    ) -> ChainResult<()> {
        // Get the executor for custom steps
        let executors = self.executors.read().await;
        let executor = executors.get("Custom").ok_or_else(|| {
            ChainError::Other("No executor registered for custom steps".to_string())
        })?;

        // Execute the step
        let context_guard = context.lock().await;
        let result = executor.execute_step(step, &context_guard).await?;

        // Update the context with the result
        drop(context_guard);
        let mut context_guard = context.lock().await;
        context_guard
            .step_results
            .insert(step.id.clone(), result.clone());

        // Process outputs
        for output in &step.outputs {
            let value = result.outputs.get(&output.name).ok_or_else(|| {
                ChainError::Other(format!("Output not found in step result: {}", output.name))
            })?;

            // Apply transformation if specified
            let transformed_value = if let Some(transform) = &output.transform {
                // This would call a function to apply the transformation
                // apply_transform(transform, value)?
                value.clone() // Simplified for now
            } else {
                value.clone()
            };

            // Store the output according to the target
            match &output.target {
                crate::modules::chain_engine::chain_definition::DataTarget::Variable {
                    variable_name,
                } => {
                    context_guard
                        .variables
                        .insert(variable_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::ChainOutput {
                    output_name,
                } => {
                    context_guard
                        .outputs
                        .insert(output_name.clone(), transformed_value);
                }
                crate::modules::chain_engine::chain_definition::DataTarget::StepInput {
                    ..
                } => {
                    // This will be handled when the target step is executed
                }
            }
        }

        Ok(())
    }
    /// Evaluate a condition
    pub fn evaluate_condition(
        &self,
        condition: &Condition,
        context: &ChainContext,
    ) -> ChainResult<bool> {
        match condition {
            Condition::Expression { expression } => self.evaluate_expression(expression, context),
            Condition::Comparison {
                left,
                operator,
                right,
            } => self.evaluate_comparison(left, operator, right, context),
            Condition::And { conditions } => {
                for condition in conditions {
                    if !self.evaluate_condition(condition, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or { conditions } => {
                for condition in conditions {
                    if self.evaluate_condition(condition, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not { condition } => {
                let result = self.evaluate_condition(condition, context)?;
                Ok(!result)
            }
            // Delegate to the existing evaluate_condition function for other condition types
            _ => crate::modules::chain_engine::executors::evaluate_condition(condition, context),
        }
    }

    /// Evaluate an expression
    fn evaluate_expression(&self, expression: &str, context: &ChainContext) -> ChainResult<bool> {
        // For now, we'll implement a simple expression evaluator
        // that supports variable substitution and basic boolean expressions

        // Replace variables in the expression
        let mut expr = expression.to_string();

        // Find all variable references in the expression
        let var_regex = regex::Regex::new(r"\$\{([^}]+)\}")
            .map_err(|e| ChainError::Other(format!("Invalid variable reference regex: {}", e)))?;

        for capture in var_regex.captures_iter(expression) {
            let var_name = &capture[1];
            let var_value = context
                .variables
                .get(var_name)
                .ok_or_else(|| ChainError::VariableNotFound(var_name.to_string()))?;

            // Convert the value to a string
            let value_str = match var_value {
                serde_json::Value::String(s) => s.clone(),
                _ => var_value.to_string(),
            };

            // Replace the variable reference with its value
            expr = expr.replace(&format!("${{{}}}", var_name), &value_str);
        }

        // Evaluate the expression
        // For now, we'll just support simple boolean expressions
        match expr.trim().to_lowercase().as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                // Try to evaluate as a boolean expression
                match expr.parse::<bool>() {
                    Ok(value) => Ok(value),
                    Err(_) => {
                        // Try to evaluate as a numeric comparison
                        if expr.contains("==") {
                            let parts: Vec<&str> = expr.split("==").collect();
                            if parts.len() == 2 {
                                let left = parts[0].trim();
                                let right = parts[1].trim();
                                return Ok(left == right);
                            }
                        } else if expr.contains("!=") {
                            let parts: Vec<&str> = expr.split("!=").collect();
                            if parts.len() == 2 {
                                let left = parts[0].trim();
                                let right = parts[1].trim();
                                return Ok(left != right);
                            }
                        } else if expr.contains(">=") {
                            let parts: Vec<&str> = expr.split(">=").collect();
                            if parts.len() == 2 {
                                let left = parts[0].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[0]
                                    ))
                                })?;
                                let right = parts[1].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[1]
                                    ))
                                })?;
                                return Ok(left >= right);
                            }
                        } else if expr.contains("<=") {
                            let parts: Vec<&str> = expr.split("<=").collect();
                            if parts.len() == 2 {
                                let left = parts[0].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[0]
                                    ))
                                })?;
                                let right = parts[1].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[1]
                                    ))
                                })?;
                                return Ok(left <= right);
                            }
                        } else if expr.contains(">") {
                            let parts: Vec<&str> = expr.split(">").collect();
                            if parts.len() == 2 {
                                let left = parts[0].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[0]
                                    ))
                                })?;
                                let right = parts[1].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[1]
                                    ))
                                })?;
                                return Ok(left > right);
                            }
                        } else if expr.contains("<") {
                            let parts: Vec<&str> = expr.split("<").collect();
                            if parts.len() == 2 {
                                let left = parts[0].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[0]
                                    ))
                                })?;
                                let right = parts[1].trim().parse::<f64>().map_err(|_| {
                                    ChainError::Other(format!(
                                        "Invalid numeric value: {}",
                                        parts[1]
                                    ))
                                })?;
                                return Ok(left < right);
                            }
                        }

                        Err(ChainError::Other(format!(
                            "Invalid boolean expression: {}",
                            expr
                        )))
                    }
                }
            }
        }
    }

    /// Evaluate a comparison
    fn evaluate_comparison(
        &self,
        left: &str,
        operator: &ComparisonOperator,
        right: &str,
        context: &ChainContext,
    ) -> ChainResult<bool> {
        // Replace variables in the left and right operands
        let left_value = self.resolve_value(left, context)?;
        let right_value = self.resolve_value(right, context)?;

        // Perform the comparison
        match operator {
            ComparisonOperator::Eq => Ok(left_value == right_value),
            ComparisonOperator::Ne => Ok(left_value != right_value),
            ComparisonOperator::Lt => {
                let left_num = self.to_number(&left_value)?;
                let right_num = self.to_number(&right_value)?;
                Ok(left_num < right_num)
            }
            ComparisonOperator::Lte => {
                let left_num = self.to_number(&left_value)?;
                let right_num = self.to_number(&right_value)?;
                Ok(left_num <= right_num)
            }
            ComparisonOperator::Gt => {
                let left_num = self.to_number(&left_value)?;
                let right_num = self.to_number(&right_value)?;
                Ok(left_num > right_num)
            }
            ComparisonOperator::Gte => {
                let left_num = self.to_number(&left_value)?;
                let right_num = self.to_number(&right_value)?;
                Ok(left_num >= right_num)
            }
            ComparisonOperator::Contains => {
                let left_str = self.to_string(&left_value)?;
                let right_str = self.to_string(&right_value)?;
                Ok(left_str.contains(&right_str))
            }
            ComparisonOperator::StartsWith => {
                let left_str = self.to_string(&left_value)?;
                let right_str = self.to_string(&right_value)?;
                Ok(left_str.starts_with(&right_str))
            }
            ComparisonOperator::EndsWith => {
                let left_str = self.to_string(&left_value)?;
                let right_str = self.to_string(&right_value)?;
                Ok(left_str.ends_with(&right_str))
            }
            ComparisonOperator::Matches => {
                let left_str = self.to_string(&left_value)?;
                let right_str = self.to_string(&right_value)?;
                let regex = regex::Regex::new(&right_str)
                    .map_err(|e| ChainError::Other(format!("Invalid regex pattern: {}", e)))?;
                Ok(regex.is_match(&left_str))
            }
        }
    }

    /// Resolve a value from a string
    fn resolve_value(&self, value: &str, context: &ChainContext) -> ChainResult<serde_json::Value> {
        // Check if the value is a variable reference
        if value.starts_with("${") && value.ends_with("}") {
            let var_name = &value[2..value.len() - 1];
            context
                .variables
                .get(var_name)
                .cloned()
                .ok_or_else(|| ChainError::VariableNotFound(var_name.to_string()))
        } else {
            // Try to parse as JSON
            match serde_json::from_str(value) {
                Ok(json_value) => Ok(json_value),
                Err(_) => Ok(serde_json::Value::String(value.to_string())),
            }
        }
    }

    /// Convert a JSON value to a number
    fn to_number(&self, value: &serde_json::Value) -> ChainResult<f64> {
        match value {
            serde_json::Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| ChainError::Other(format!("Invalid numeric value: {}", n))),
            serde_json::Value::String(s) => s
                .parse::<f64>()
                .map_err(|_| ChainError::Other(format!("Invalid numeric value: {}", s))),
            _ => Err(ChainError::Other(format!(
                "Cannot convert to number: {}",
                value
            ))),
        }
    }

    /// Convert a JSON value to a string
    fn to_string(&self, value: &serde_json::Value) -> ChainResult<String> {
        match value {
            serde_json::Value::String(s) => Ok(s.clone()),
            _ => Ok(value.to_string()),
        }
    }
}

// Clone implementation is now derived automatically with #[derive(Clone)]
