//! Boomerang Orchestration Example
//!
//! This example demonstrates how to use the Boomerang orchestration functionality
//! to coordinate testing strategies across different specialized modes.

use intellirouter::modules::orchestrator::{
    create_task_manager, Mode, Task, TaskManagerConfig, TaskResult, TaskStatus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Boomerang Orchestration Example");
    println!("===============================");

    // Create a task manager with default configuration
    let task_manager = create_task_manager();

    // Create some tasks
    let task1 = Task::new(
        "task-1",
        "Debug Task",
        "A task for the Debug mode",
        Mode::Debug,
    );
    let task2 = Task::new(
        "task-2",
        "Code Task",
        "A task for the Code mode",
        Mode::Code,
    );
    let task3 = Task::new(
        "task-3",
        "Test Task",
        "A task for the Test mode",
        Mode::Test,
    );

    // Create a task with dependencies
    let task4 = Task::new(
        "task-4",
        "Dependent Task",
        "A task that depends on other tasks",
        Mode::Code,
    )
    .with_dependency("task-1")
    .with_dependency("task-2");

    // Add the tasks to the task manager
    println!("Adding tasks to the task manager...");
    task_manager.add_task(task1)?;
    task_manager.add_task(task2)?;
    task_manager.add_task(task3)?;
    task_manager.add_task(task4)?;

    // Simulate task execution and results
    println!("\nSimulating task execution and results...");

    // Simulate task 1 completion
    println!("Task 1 (Debug) completed");
    let result1 = TaskResult::new(
        "task-1",
        TaskStatus::Completed,
        "Debug task completed successfully",
    )
    .with_data("debug_info", "Some debug information");
    task_manager.process_task_result("task-1", result1)?;

    // Simulate task 2 completion
    println!("Task 2 (Code) completed");
    let result2 = TaskResult::new(
        "task-2",
        TaskStatus::Completed,
        "Code task completed successfully",
    )
    .with_data("code_info", "Some code information");
    task_manager.process_task_result("task-2", result2)?;

    // Simulate task 3 failure
    println!("Task 3 (Test) failed");
    let result3 = TaskResult::new("task-3", TaskStatus::Failed, "Test task failed")
        .with_data("error", "Test assertion failed");
    task_manager.process_task_result("task-3", result3)?;

    // Task 4 should be automatically executed after tasks 1 and 2 are completed
    println!("\nTask 4 should be automatically executed after tasks 1 and 2 are completed");

    // Simulate task 4 completion
    println!("Task 4 (Code) completed");
    let result4 = TaskResult::new(
        "task-4",
        TaskStatus::Completed,
        "Dependent task completed successfully",
    )
    .with_data(
        "dependency_info",
        "Executed after dependencies were satisfied",
    );
    task_manager.process_task_result("task-4", result4)?;

    // Get task events
    println!("\nTask Events:");
    let tracker = task_manager.tracker();
    let events = tracker.get_all_events()?;

    for (i, event) in events.iter().enumerate() {
        println!(
            "Event {}: Task {} - Mode: {} - Status: {} - Message: {}",
            i + 1,
            event.task_id,
            event.mode,
            event.status,
            event.message
        );
    }

    println!("\nBoomerang orchestration example completed successfully!");

    Ok(())
}
