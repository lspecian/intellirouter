//! Workflow Templates
//!
//! This module provides predefined workflow templates and patterns that can be
//! reused across different testing scenarios.

use std::collections::HashMap;

use crate::modules::orchestrator::types::{Task, WorkflowError};
use crate::modules::orchestrator::workflow::Workflow;

/// Workflow template type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowTemplateType {
    /// Sequential workflow
    Sequential,
    /// Parallel workflow
    Parallel,
    /// Fan-out workflow
    FanOut,
    /// Fan-in workflow
    FanIn,
    /// Pipeline workflow
    Pipeline,
    /// Iterative workflow
    Iterative,
    /// Conditional workflow
    Conditional,
    /// Custom workflow
    Custom,
}

/// Workflow template
#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template type
    pub template_type: WorkflowTemplateType,
    /// Template parameters
    pub parameters: HashMap<String, String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl WorkflowTemplate {
    /// Create a new workflow template
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        template_type: WorkflowTemplateType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            template_type,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Create a workflow from the template
    pub fn create_workflow(
        &self,
        workflow_id: impl Into<String>,
    ) -> Result<Workflow, WorkflowError> {
        let workflow_id = workflow_id.into();
        let mut workflow = Workflow::new(
            workflow_id.clone(),
            self.name.clone(),
            self.description.clone(),
        );

        // Add metadata from the template
        for (key, value) in &self.metadata {
            workflow = workflow.with_metadata(key, value);
        }

        // Add template type as metadata
        workflow = workflow.with_metadata("template_type", format!("{:?}", self.template_type));

        Ok(workflow)
    }
}

/// Workflow template manager
pub struct WorkflowTemplateManager {
    /// Templates
    templates: HashMap<String, WorkflowTemplate>,
}

impl WorkflowTemplateManager {
    /// Create a new workflow template manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Register default templates
        manager.register_default_templates();

        manager
    }

    /// Register default templates
    fn register_default_templates(&mut self) {
        // Sequential workflow template
        let sequential_template = WorkflowTemplate::new(
            "sequential",
            "Sequential Workflow",
            "A workflow where tasks are executed in sequence",
            WorkflowTemplateType::Sequential,
        )
        .with_parameter("max_tasks", "10")
        .with_metadata(
            "description",
            "Tasks are executed one after another in a predefined order",
        );

        self.templates
            .insert(sequential_template.id.clone(), sequential_template);

        // Parallel workflow template
        let parallel_template = WorkflowTemplate::new(
            "parallel",
            "Parallel Workflow",
            "A workflow where tasks are executed in parallel",
            WorkflowTemplateType::Parallel,
        )
        .with_parameter("max_concurrent_tasks", "5")
        .with_metadata(
            "description",
            "Tasks are executed concurrently without dependencies",
        );

        self.templates
            .insert(parallel_template.id.clone(), parallel_template);

        // Fan-out workflow template
        let fan_out_template = WorkflowTemplate::new(
            "fan-out",
            "Fan-Out Workflow",
            "A workflow where a single task spawns multiple parallel tasks",
            WorkflowTemplateType::FanOut,
        )
        .with_parameter("max_fan_out", "10")
        .with_metadata(
            "description",
            "A single task triggers multiple parallel tasks",
        );

        self.templates
            .insert(fan_out_template.id.clone(), fan_out_template);

        // Fan-in workflow template
        let fan_in_template = WorkflowTemplate::new(
            "fan-in",
            "Fan-In Workflow",
            "A workflow where multiple parallel tasks converge to a single task",
            WorkflowTemplateType::FanIn,
        )
        .with_parameter("max_fan_in", "10")
        .with_metadata(
            "description",
            "Multiple parallel tasks converge to a single task",
        );

        self.templates
            .insert(fan_in_template.id.clone(), fan_in_template);

        // Pipeline workflow template
        let pipeline_template = WorkflowTemplate::new(
            "pipeline",
            "Pipeline Workflow",
            "A workflow where tasks are organized in a pipeline",
            WorkflowTemplateType::Pipeline,
        )
        .with_parameter("pipeline_stages", "5")
        .with_metadata(
            "description",
            "Tasks are organized in a pipeline with multiple stages",
        );

        self.templates
            .insert(pipeline_template.id.clone(), pipeline_template);

        // Iterative workflow template
        let iterative_template = WorkflowTemplate::new(
            "iterative",
            "Iterative Workflow",
            "A workflow where tasks are executed iteratively",
            WorkflowTemplateType::Iterative,
        )
        .with_parameter("max_iterations", "5")
        .with_metadata(
            "description",
            "Tasks are executed iteratively until a condition is met",
        );

        self.templates
            .insert(iterative_template.id.clone(), iterative_template);

        // Conditional workflow template
        let conditional_template = WorkflowTemplate::new(
            "conditional",
            "Conditional Workflow",
            "A workflow where task execution depends on conditions",
            WorkflowTemplateType::Conditional,
        )
        .with_parameter("condition_type", "status")
        .with_metadata("description", "Task execution depends on conditions");

        self.templates
            .insert(conditional_template.id.clone(), conditional_template);
    }

    /// Register a template
    pub fn register_template(&mut self, template: WorkflowTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get a template
    pub fn get_template(&self, template_id: &str) -> Option<&WorkflowTemplate> {
        self.templates.get(template_id)
    }

    /// Get all templates
    pub fn get_all_templates(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }

    /// Create a workflow from a template
    pub fn create_workflow_from_template(
        &self,
        template_id: &str,
        workflow_id: impl Into<String>,
    ) -> Result<Workflow, WorkflowError> {
        let template = self.get_template(template_id).ok_or_else(|| {
            WorkflowError::InvalidWorkflow(format!("Template not found: {}", template_id))
        })?;

        template.create_workflow(workflow_id)
    }
}

/// Sequential workflow builder
pub struct SequentialWorkflowBuilder {
    /// Workflow
    workflow: Workflow,
    /// Tasks
    tasks: Vec<Task>,
}

impl SequentialWorkflowBuilder {
    /// Create a new sequential workflow builder
    pub fn new(
        workflow_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let workflow = Workflow::new(workflow_id, name, description)
            .with_metadata("template_type", "Sequential");

        Self {
            workflow,
            tasks: Vec::new(),
        }
    }

    /// Add a task
    pub fn add_task(mut self, task: Task) -> Self {
        self.tasks.push(task);
        self
    }

    /// Build the workflow
    pub fn build(mut self) -> Result<Workflow, WorkflowError> {
        // Add tasks to the workflow
        for task in &self.tasks {
            self.workflow = self.workflow.with_task(task.id.clone());
        }

        // Create dependencies for sequential execution
        for i in 1..self.tasks.len() {
            let prev_task = &self.tasks[i - 1];
            let curr_task = &self.tasks[i];

            // Add dependency
            let mut curr_task = curr_task.clone();
            curr_task.dependencies.push(prev_task.id.clone());

            // Update the task in the list
            self.tasks[i] = curr_task;
        }

        Ok(self.workflow)
    }
}

/// Parallel workflow builder
pub struct ParallelWorkflowBuilder {
    /// Workflow
    workflow: Workflow,
    /// Tasks
    tasks: Vec<Task>,
}

impl ParallelWorkflowBuilder {
    /// Create a new parallel workflow builder
    pub fn new(
        workflow_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let workflow = Workflow::new(workflow_id, name, description)
            .with_metadata("template_type", "Parallel");

        Self {
            workflow,
            tasks: Vec::new(),
        }
    }

    /// Add a task
    pub fn add_task(mut self, task: Task) -> Self {
        self.tasks.push(task);
        self
    }

    /// Build the workflow
    pub fn build(mut self) -> Result<Workflow, WorkflowError> {
        // Add tasks to the workflow
        for task in &self.tasks {
            self.workflow = self.workflow.with_task(task.id.clone());
        }

        Ok(self.workflow)
    }
}

/// Fan-out workflow builder
pub struct FanOutWorkflowBuilder {
    /// Workflow
    workflow: Workflow,
    /// Source task
    source_task: Option<Task>,
    /// Target tasks
    target_tasks: Vec<Task>,
}

impl FanOutWorkflowBuilder {
    /// Create a new fan-out workflow builder
    pub fn new(
        workflow_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let workflow =
            Workflow::new(workflow_id, name, description).with_metadata("template_type", "FanOut");

        Self {
            workflow,
            source_task: None,
            target_tasks: Vec::new(),
        }
    }

    /// Set the source task
    pub fn with_source_task(mut self, task: Task) -> Self {
        self.source_task = Some(task);
        self
    }

    /// Add a target task
    pub fn add_target_task(mut self, task: Task) -> Self {
        self.target_tasks.push(task);
        self
    }

    /// Build the workflow
    pub fn build(mut self) -> Result<Workflow, WorkflowError> {
        // Check if source task is set
        let source_task = self.source_task.ok_or_else(|| {
            WorkflowError::InvalidWorkflow("Source task not set for fan-out workflow".to_string())
        })?;

        // Add source task to the workflow
        self.workflow = self.workflow.with_task(source_task.id.clone());

        // Add target tasks to the workflow and collect their IDs
        let task_ids: Vec<String> = self.target_tasks.iter().map(|t| t.id.clone()).collect();

        // First, add all tasks to the workflow
        for task_id in &task_ids {
            self.workflow = self.workflow.with_task(task_id.clone());
        }

        // Then update the tasks with dependencies
        for i in 0..self.target_tasks.len() {
            let mut task = self.target_tasks[i].clone();
            task.dependencies.push(source_task.id.clone());
            self.target_tasks[i] = task;
        }

        Ok(self.workflow)
    }
}

/// Fan-in workflow builder
pub struct FanInWorkflowBuilder {
    /// Workflow
    workflow: Workflow,
    /// Source tasks
    source_tasks: Vec<Task>,
    /// Target task
    target_task: Option<Task>,
}

impl FanInWorkflowBuilder {
    /// Create a new fan-in workflow builder
    pub fn new(
        workflow_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let workflow =
            Workflow::new(workflow_id, name, description).with_metadata("template_type", "FanIn");

        Self {
            workflow,
            source_tasks: Vec::new(),
            target_task: None,
        }
    }

    /// Add a source task
    pub fn add_source_task(mut self, task: Task) -> Self {
        self.source_tasks.push(task);
        self
    }

    /// Set the target task
    pub fn with_target_task(mut self, task: Task) -> Self {
        self.target_task = Some(task);
        self
    }

    /// Build the workflow
    pub fn build(mut self) -> Result<Workflow, WorkflowError> {
        // Check if target task is set
        let mut target_task = self.target_task.ok_or_else(|| {
            WorkflowError::InvalidWorkflow("Target task not set for fan-in workflow".to_string())
        })?;

        // Add source tasks to the workflow
        for task in &self.source_tasks {
            self.workflow = self.workflow.with_task(task.id.clone());

            // Add dependency on source task
            target_task.dependencies.push(task.id.clone());
        }

        // Add target task to the workflow
        self.workflow = self.workflow.with_task(target_task.id.clone());

        Ok(self.workflow)
    }
}

/// Pipeline workflow builder
pub struct PipelineWorkflowBuilder {
    /// Workflow
    workflow: Workflow,
    /// Pipeline stages
    stages: Vec<Vec<Task>>,
}

impl PipelineWorkflowBuilder {
    /// Create a new pipeline workflow builder
    pub fn new(
        workflow_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let workflow = Workflow::new(workflow_id, name, description)
            .with_metadata("template_type", "Pipeline");

        Self {
            workflow,
            stages: Vec::new(),
        }
    }

    /// Add a stage
    pub fn add_stage(mut self, tasks: Vec<Task>) -> Self {
        self.stages.push(tasks);
        self
    }

    /// Build the workflow
    pub fn build(mut self) -> Result<Workflow, WorkflowError> {
        // Check if there are stages
        if self.stages.is_empty() {
            return Err(WorkflowError::InvalidWorkflow(
                "No stages defined for pipeline workflow".to_string(),
            ));
        }

        // Add all tasks to the workflow
        for stage in &self.stages {
            for task in stage {
                self.workflow = self.workflow.with_task(task.id.clone());
            }
        }

        // Create dependencies between stages
        // First collect all the task IDs we need
        let mut stage_task_ids = Vec::new();
        for stage in &self.stages {
            let stage_ids: Vec<String> = stage.iter().map(|t| t.id.clone()).collect();
            stage_task_ids.push(stage_ids);
        }

        // Now create the dependencies
        for i in 1..self.stages.len() {
            let prev_stage_ids = &stage_task_ids[i - 1];

            // Update each task in the current stage
            for task_idx in 0..self.stages[i].len() {
                let mut curr_task = self.stages[i][task_idx].clone();

                // Add dependencies on all tasks in the previous stage
                for prev_id in prev_stage_ids {
                    curr_task.dependencies.push(prev_id.clone());
                }

                // Update the task in the list
                self.stages[i][task_idx] = curr_task;
            }
        }

        Ok(self.workflow)
    }
}

/// Conditional workflow builder
pub struct ConditionalWorkflowBuilder {
    /// Workflow
    workflow: Workflow,
    /// Source task
    source_task: Option<Task>,
    /// Condition tasks
    condition_tasks: HashMap<String, Task>,
    /// Default task
    default_task: Option<Task>,
}

impl ConditionalWorkflowBuilder {
    /// Create a new conditional workflow builder
    pub fn new(
        workflow_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let workflow = Workflow::new(workflow_id, name, description)
            .with_metadata("template_type", "Conditional");

        Self {
            workflow,
            source_task: None,
            condition_tasks: HashMap::new(),
            default_task: None,
        }
    }

    /// Set the source task
    pub fn with_source_task(mut self, task: Task) -> Self {
        self.source_task = Some(task);
        self
    }

    /// Add a condition task
    pub fn add_condition_task(mut self, condition: impl Into<String>, task: Task) -> Self {
        self.condition_tasks.insert(condition.into(), task);
        self
    }

    /// Set the default task
    pub fn with_default_task(mut self, task: Task) -> Self {
        self.default_task = Some(task);
        self
    }

    /// Build the workflow
    pub fn build(mut self) -> Result<Workflow, WorkflowError> {
        // Check if source task is set
        let source_task = self.source_task.ok_or_else(|| {
            WorkflowError::InvalidWorkflow(
                "Source task not set for conditional workflow".to_string(),
            )
        })?;

        // Add source task to the workflow
        self.workflow = self.workflow.with_task(source_task.id.clone());

        // First collect all condition tasks and their conditions
        let mut condition_entries = Vec::new();
        for (condition, task) in &self.condition_tasks {
            self.workflow = self.workflow.with_task(task.id.clone());
            condition_entries.push((condition.clone(), task.clone()));
        }

        // Now update the tasks with dependencies and metadata
        for (condition, mut task) in condition_entries {
            // Add dependency on source task
            task.dependencies.push(source_task.id.clone());

            // Add condition as metadata
            task.metadata
                .insert("condition".to_string(), condition.clone());

            // Update the task in the map
            self.condition_tasks.insert(condition, task);
        }

        // Add default task to the workflow if set
        if let Some(default_task) = &self.default_task {
            self.workflow = self.workflow.with_task(default_task.id.clone());

            // Add dependency on source task
            let mut default_task = default_task.clone();
            default_task.dependencies.push(source_task.id.clone());

            // Add metadata
            default_task
                .metadata
                .insert("default".to_string(), "true".to_string());

            // Update the default task
            self.default_task = Some(default_task);
        }

        Ok(self.workflow)
    }
}
