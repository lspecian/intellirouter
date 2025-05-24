//! Workflow Management Example
//!
//! This example demonstrates how to use the Workflow Management System
//! to create and execute workflows with different patterns.

use intellirouter::modules::orchestrator::{
    create_workflow_manager, create_workflow_template_manager, ConditionalWorkflowBuilder,
    FanInWorkflowBuilder, FanOutWorkflowBuilder, Mode, ParallelWorkflowBuilder,
    PipelineWorkflowBuilder, SequentialWorkflowBuilder, Task, TaskResult, TaskStatus,
    WorkflowTemplateType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Workflow Management Example");
    println!("==========================");

    // Create a workflow manager
    let workflow_manager = create_workflow_manager();

    // Create a workflow template manager
    let template_manager = create_workflow_template_manager();

    // List available templates
    println!("\nAvailable Workflow Templates:");
    for template in template_manager.get_all_templates() {
        println!(
            "- {}: {} (Type: {:?})",
            template.id, template.name, template.template_type
        );
    }

    // Create a workflow from a template
    println!("\nCreating a workflow from the 'sequential' template...");
    let sequential_workflow =
        template_manager.create_workflow_from_template("sequential", "workflow-1")?;

    // Register the workflow
    workflow_manager.register_workflow(sequential_workflow)?;

    // Create a sequential workflow using the builder
    println!("\nCreating a sequential workflow using the builder...");
    let task1 = Task::new("task-1", "Task 1", "First task", Mode::Debug);
    let task2 = Task::new("task-2", "Task 2", "Second task", Mode::Code);
    let task3 = Task::new("task-3", "Task 3", "Third task", Mode::Test);

    let sequential_workflow = SequentialWorkflowBuilder::new(
        "workflow-2",
        "Sequential Workflow",
        "A workflow with sequential tasks",
    )
    .add_task(task1)
    .add_task(task2)
    .add_task(task3)
    .build()?;

    // Register the workflow
    workflow_manager.register_workflow(sequential_workflow)?;

    // Create a parallel workflow using the builder
    println!("\nCreating a parallel workflow using the builder...");
    let task4 = Task::new("task-4", "Task 4", "Fourth task", Mode::Debug);
    let task5 = Task::new("task-5", "Task 5", "Fifth task", Mode::Code);
    let task6 = Task::new("task-6", "Task 6", "Sixth task", Mode::Test);

    let parallel_workflow = ParallelWorkflowBuilder::new(
        "workflow-3",
        "Parallel Workflow",
        "A workflow with parallel tasks",
    )
    .add_task(task4)
    .add_task(task5)
    .add_task(task6)
    .build()?;

    // Register the workflow
    workflow_manager.register_workflow(parallel_workflow)?;

    // Create a fan-out workflow using the builder
    println!("\nCreating a fan-out workflow using the builder...");
    let source_task = Task::new(
        "task-7",
        "Source Task",
        "Source task for fan-out",
        Mode::Debug,
    );
    let target_task1 = Task::new("task-8", "Target Task 1", "First target task", Mode::Code);
    let target_task2 = Task::new("task-9", "Target Task 2", "Second target task", Mode::Test);
    let target_task3 = Task::new("task-10", "Target Task 3", "Third target task", Mode::Code);

    let fan_out_workflow = FanOutWorkflowBuilder::new(
        "workflow-4",
        "Fan-Out Workflow",
        "A workflow with fan-out pattern",
    )
    .with_source_task(source_task)
    .add_target_task(target_task1)
    .add_target_task(target_task2)
    .add_target_task(target_task3)
    .build()?;

    // Register the workflow
    workflow_manager.register_workflow(fan_out_workflow)?;

    // Create a fan-in workflow using the builder
    println!("\nCreating a fan-in workflow using the builder...");
    let source_task1 = Task::new("task-11", "Source Task 1", "First source task", Mode::Debug);
    let source_task2 = Task::new("task-12", "Source Task 2", "Second source task", Mode::Code);
    let source_task3 = Task::new("task-13", "Source Task 3", "Third source task", Mode::Test);
    let target_task = Task::new(
        "task-14",
        "Target Task",
        "Target task for fan-in",
        Mode::Boomerang,
    );

    let fan_in_workflow = FanInWorkflowBuilder::new(
        "workflow-5",
        "Fan-In Workflow",
        "A workflow with fan-in pattern",
    )
    .add_source_task(source_task1)
    .add_source_task(source_task2)
    .add_source_task(source_task3)
    .with_target_task(target_task)
    .build()?;

    // Register the workflow
    workflow_manager.register_workflow(fan_in_workflow)?;

    // Create a pipeline workflow using the builder
    println!("\nCreating a pipeline workflow using the builder...");
    let stage1_task1 = Task::new(
        "task-15",
        "Stage 1 Task 1",
        "First task in stage 1",
        Mode::Debug,
    );
    let stage1_task2 = Task::new(
        "task-16",
        "Stage 1 Task 2",
        "Second task in stage 1",
        Mode::Debug,
    );

    let stage2_task1 = Task::new(
        "task-17",
        "Stage 2 Task 1",
        "First task in stage 2",
        Mode::Code,
    );
    let stage2_task2 = Task::new(
        "task-18",
        "Stage 2 Task 2",
        "Second task in stage 2",
        Mode::Code,
    );

    let stage3_task1 = Task::new(
        "task-19",
        "Stage 3 Task 1",
        "First task in stage 3",
        Mode::Test,
    );
    let stage3_task2 = Task::new(
        "task-20",
        "Stage 3 Task 2",
        "Second task in stage 3",
        Mode::Test,
    );

    let pipeline_workflow = PipelineWorkflowBuilder::new(
        "workflow-6",
        "Pipeline Workflow",
        "A workflow with pipeline pattern",
    )
    .add_stage(vec![stage1_task1, stage1_task2])
    .add_stage(vec![stage2_task1, stage2_task2])
    .add_stage(vec![stage3_task1, stage3_task2])
    .build()?;

    // Register the workflow
    workflow_manager.register_workflow(pipeline_workflow)?;

    // Create a conditional workflow using the builder
    println!("\nCreating a conditional workflow using the builder...");
    let source_task = Task::new(
        "task-21",
        "Source Task",
        "Source task for conditional",
        Mode::Debug,
    );
    let success_task = Task::new(
        "task-22",
        "Success Task",
        "Task for success condition",
        Mode::Code,
    );
    let failure_task = Task::new(
        "task-23",
        "Failure Task",
        "Task for failure condition",
        Mode::Test,
    );
    let default_task = Task::new("task-24", "Default Task", "Default task", Mode::Boomerang);

    let conditional_workflow = ConditionalWorkflowBuilder::new(
        "workflow-7",
        "Conditional Workflow",
        "A workflow with conditional pattern",
    )
    .with_source_task(source_task)
    .add_condition_task("success", success_task)
    .add_condition_task("failure", failure_task)
    .with_default_task(default_task)
    .build()?;

    // Register the workflow
    workflow_manager.register_workflow(conditional_workflow)?;

    // List all registered workflows
    println!("\nRegistered Workflows:");
    let workflows = workflow_manager.get_all_workflows()?;

    for (i, workflow) in workflows.iter().enumerate() {
        println!(
            "{}. {} ({}): {}",
            i + 1,
            workflow.id,
            workflow.name,
            workflow.description
        );
        println!("   Tasks: {}", workflow.task_ids.join(", "));

        if let Some(template_type) = workflow.metadata.get("template_type") {
            println!("   Template Type: {}", template_type);
        }

        println!();
    }

    // Execute a workflow
    println!("\nExecuting the sequential workflow (workflow-2)...");
    workflow_manager.execute_workflow("workflow-2", &create_mock_orchestrator())?;

    // Get the workflow result
    println!("\nWorkflow Result:");
    if let Some(result) = workflow_manager.get_workflow_result("workflow-2")? {
        println!("Status: {:?}", result.status);
        println!("Completed Tasks: {}", result.completed_tasks.join(", "));
        println!("Failed Tasks: {}", result.failed_tasks.join(", "));
        println!("Pending Tasks: {}", result.pending_tasks.join(", "));

        if let Some(error) = result.error_message {
            println!("Error: {}", error);
        }
    }

    println!("\nWorkflow management example completed successfully!");

    Ok(())
}

// Create a mock orchestrator for demonstration purposes
fn create_mock_orchestrator(
) -> impl intellirouter::modules::orchestrator::architecture::OrchestratorArchitecture {
    struct MockOrchestrator;

    impl intellirouter::modules::orchestrator::architecture::OrchestratorArchitecture
        for MockOrchestrator
    {
        fn get_task(
            &self,
            task_id: &str,
        ) -> Result<Option<Task>, intellirouter::modules::orchestrator::types::OrchestratorError>
        {
            // Return a mock task
            let task = Task::new(task_id, "Mock Task", "A mock task", Mode::Debug);
            Ok(Some(task))
        }

        // Implement other required methods
        // ...
    }

    MockOrchestrator
}
