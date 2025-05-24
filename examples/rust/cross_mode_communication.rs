//! Cross-Mode Communication Example
//!
//! This example demonstrates how to use the Cross-Mode Communication Protocol
//! to exchange information between different specialized modes.

use intellirouter::modules::orchestrator::{
    create_communication_protocol, MessagePriority, MessageType, Mode, Task, TaskResult, TaskStatus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cross-Mode Communication Example");
    println!("================================");

    // Create a communication protocol
    let protocol = create_communication_protocol();

    // Get the message bus
    let message_bus = protocol.message_bus();

    // Send a task delegation message
    println!("\nSending task delegation message...");
    let task = Task::new(
        "task-1",
        "Example Task",
        "A task for demonstration",
        Mode::Code,
    );
    protocol.send_task_delegation(Mode::Boomerang, Mode::Code, &task)?;

    // Send a task status update message
    println!("\nSending task status update message...");
    protocol.send_task_status_update(
        Mode::Code,
        Mode::Boomerang,
        "task-1",
        TaskStatus::InProgress,
    )?;

    // Send a task result message
    println!("\nSending task result message...");
    let result = TaskResult::new(
        "task-1",
        TaskStatus::Completed,
        "Task completed successfully",
    )
    .with_data("execution_time", "120ms")
    .with_data("memory_usage", "10MB");
    protocol.send_task_result(Mode::Code, Mode::Boomerang, &result)?;

    // Send a mode status update message
    println!("\nSending mode status update message...");
    protocol.send_mode_status_update(Mode::Code, Mode::Boomerang, "Ready")?;

    // Send a command message
    println!("\nSending command message...");
    protocol.send_command(Mode::Boomerang, Mode::Debug, "analyze_performance")?;

    // Send a query message and get a response
    println!("\nSending query message...");
    let response = protocol.send_query(Mode::Boomerang, Mode::Test, "get_test_coverage")?;
    if let Some(response) = response {
        println!("Received response: {}", response.content);
    }

    // Send a notification message
    println!("\nSending notification message...");
    protocol.send_notification(
        Mode::Boomerang,
        Mode::Code,
        "New code style guidelines available",
    )?;

    // Send an error message
    println!("\nSending error message...");
    protocol.send_error(
        Mode::Debug,
        Mode::Boomerang,
        "Memory leak detected in module X",
    )?;

    // Broadcast a notification to all modes
    println!("\nBroadcasting notification to all modes...");
    protocol
        .broadcast_notification(Mode::Boomerang, "System maintenance scheduled for tomorrow")?;

    // Get the message history
    println!("\nMessage History:");
    let history = protocol.get_message_history()?;

    for (i, message) in history.iter().enumerate() {
        println!(
            "Message {}: {} -> {} - Type: {:?} - Content: {}",
            i + 1,
            message.sender,
            message.receiver,
            message.message_type,
            message.content
        );
    }

    // Get the message history for a specific mode
    println!("\nMessage History for Code Mode:");
    let code_history = protocol.get_message_history_for_mode(Mode::Code)?;

    for (i, message) in code_history.iter().enumerate() {
        println!(
            "Message {}: {} -> {} - Type: {:?} - Content: {}",
            i + 1,
            message.sender,
            message.receiver,
            message.message_type,
            message.content
        );
    }

    // Get the message history for a specific task
    println!("\nMessage History for Task 1:");
    let task_history = protocol.get_message_history_for_task("task-1")?;

    for (i, message) in task_history.iter().enumerate() {
        println!(
            "Message {}: {} -> {} - Type: {:?} - Content: {}",
            i + 1,
            message.sender,
            message.receiver,
            message.message_type,
            message.content
        );
    }

    println!("\nCross-mode communication example completed successfully!");

    Ok(())
}
