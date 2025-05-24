//! Cross-Mode Communication Protocol
//!
//! This module provides a communication protocol for exchanging information
//! between different specialized modes (Debug, Code, Test) in the Boomerang orchestrator.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::modules::orchestrator::types::{
    IntegrationError, Mode, OrchestratorError, Task, TaskResult, TaskStatus,
};

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Task delegation
    TaskDelegation,
    /// Task status update
    TaskStatusUpdate,
    /// Task result
    TaskResult,
    /// Mode status update
    ModeStatusUpdate,
    /// Command
    Command,
    /// Query
    Query,
    /// Response
    Response,
    /// Notification
    Notification,
    /// Error
    Error,
}

/// Message
#[derive(Debug, Clone)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// Sender mode
    pub sender: Mode,
    /// Receiver mode
    pub receiver: Mode,
    /// Message type
    pub message_type: MessageType,
    /// Message priority
    pub priority: MessagePriority,
    /// Message content
    pub content: String,
    /// Related task ID (if any)
    pub task_id: Option<String>,
    /// Timestamp
    pub timestamp: Instant,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(
        id: impl Into<String>,
        sender: Mode,
        receiver: Mode,
        message_type: MessageType,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            sender,
            receiver,
            message_type,
            priority: MessagePriority::default(),
            content: content.into(),
            task_id: None,
            timestamp: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set the message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the related task ID
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Message handler
pub trait MessageHandler: Send + Sync {
    /// Handle a message
    fn handle_message(&self, message: Message) -> Result<Option<Message>, OrchestratorError>;
}

/// Message bus for routing messages between modes
pub struct MessageBus {
    /// Message handlers
    handlers: Mutex<HashMap<Mode, Box<dyn MessageHandler + Send + Sync>>>,
    /// Message history
    history: Mutex<Vec<Message>>,
    /// Maximum history size
    max_history_size: usize,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(HashMap::new()),
            history: Mutex::new(Vec::new()),
            max_history_size: 1000,
        }
    }

    /// Set the maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
    }

    /// Register a message handler
    pub fn register_handler(
        &self,
        mode: Mode,
        handler: Box<dyn MessageHandler + Send + Sync>,
    ) -> Result<(), OrchestratorError> {
        let mut handlers = self.handlers.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on handlers".to_string())
        })?;

        handlers.insert(mode, handler);
        Ok(())
    }

    /// Send a message
    pub fn send_message(&self, message: Message) -> Result<Option<Message>, OrchestratorError> {
        // Add the message to the history
        {
            let mut history = self.history.lock().map_err(|_| {
                OrchestratorError::Other("Failed to acquire lock on history".to_string())
            })?;

            history.push(message.clone());

            // Trim the history if it exceeds the maximum size
            if history.len() > self.max_history_size {
                let drain_count = history.len() - self.max_history_size;
                history.drain(0..drain_count);
            }
        }

        // Get the handler for the receiver
        let handlers = self.handlers.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on handlers".to_string())
        })?;

        let handler = handlers.get(&message.receiver).ok_or_else(|| {
            OrchestratorError::Integration(IntegrationError::Other(format!(
                "No handler registered for mode: {}",
                message.receiver
            )))
        })?;

        // Handle the message
        handler.handle_message(message)
    }

    /// Broadcast a message to all modes
    pub fn broadcast_message(
        &self,
        sender: Mode,
        message_type: MessageType,
        content: impl Into<String>,
    ) -> Result<(), OrchestratorError> {
        let content = content.into();
        let handlers = self.handlers.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on handlers".to_string())
        })?;

        for (mode, _) in handlers.iter() {
            if *mode != sender {
                let message = Message::new(
                    format!(
                        "broadcast-{}-{}",
                        sender,
                        Instant::now().elapsed().as_millis()
                    ),
                    sender,
                    *mode,
                    message_type,
                    content.clone(),
                );

                self.send_message(message)?;
            }
        }

        Ok(())
    }

    /// Get the message history
    pub fn get_history(&self) -> Result<Vec<Message>, OrchestratorError> {
        let history = self.history.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on history".to_string())
        })?;

        Ok(history.clone())
    }

    /// Get the message history for a specific mode
    pub fn get_history_for_mode(&self, mode: Mode) -> Result<Vec<Message>, OrchestratorError> {
        let history = self.history.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on history".to_string())
        })?;

        Ok(history
            .iter()
            .filter(|m| m.sender == mode || m.receiver == mode)
            .cloned()
            .collect())
    }

    /// Get the message history for a specific task
    pub fn get_history_for_task(&self, task_id: &str) -> Result<Vec<Message>, OrchestratorError> {
        let history = self.history.lock().map_err(|_| {
            OrchestratorError::Other("Failed to acquire lock on history".to_string())
        })?;

        Ok(history
            .iter()
            .filter(|m| m.task_id.as_ref().map_or(false, |id| id == task_id))
            .cloned()
            .collect())
    }
}

/// Default message handlers for each mode
pub mod handlers {
    use super::*;

    /// Debug mode message handler
    pub struct DebugModeHandler;

    impl MessageHandler for DebugModeHandler {
        fn handle_message(&self, message: Message) -> Result<Option<Message>, OrchestratorError> {
            // In a real implementation, this would handle messages for the Debug mode
            // For now, we'll just log the message
            println!("Debug mode received message: {:?}", message);

            // For query messages, send a response
            if message.message_type == MessageType::Query {
                let response = Message::new(
                    format!("response-{}", message.id),
                    Mode::Debug,
                    message.sender,
                    MessageType::Response,
                    format!("Response to query: {}", message.content),
                )
                .with_task_id(message.task_id.unwrap_or_default());

                return Ok(Some(response));
            }

            Ok(None)
        }
    }

    /// Code mode message handler
    pub struct CodeModeHandler;

    impl MessageHandler for CodeModeHandler {
        fn handle_message(&self, message: Message) -> Result<Option<Message>, OrchestratorError> {
            // In a real implementation, this would handle messages for the Code mode
            // For now, we'll just log the message
            println!("Code mode received message: {:?}", message);

            // For query messages, send a response
            if message.message_type == MessageType::Query {
                let response = Message::new(
                    format!("response-{}", message.id),
                    Mode::Code,
                    message.sender,
                    MessageType::Response,
                    format!("Response to query: {}", message.content),
                )
                .with_task_id(message.task_id.unwrap_or_default());

                return Ok(Some(response));
            }

            Ok(None)
        }
    }

    /// Test mode message handler
    pub struct TestModeHandler;

    impl MessageHandler for TestModeHandler {
        fn handle_message(&self, message: Message) -> Result<Option<Message>, OrchestratorError> {
            // In a real implementation, this would handle messages for the Test mode
            // For now, we'll just log the message
            println!("Test mode received message: {:?}", message);

            // For query messages, send a response
            if message.message_type == MessageType::Query {
                let response = Message::new(
                    format!("response-{}", message.id),
                    Mode::Test,
                    message.sender,
                    MessageType::Response,
                    format!("Response to query: {}", message.content),
                )
                .with_task_id(message.task_id.unwrap_or_default());

                return Ok(Some(response));
            }

            Ok(None)
        }
    }

    /// Boomerang mode message handler
    pub struct BoomerangModeHandler;

    impl MessageHandler for BoomerangModeHandler {
        fn handle_message(&self, message: Message) -> Result<Option<Message>, OrchestratorError> {
            // In a real implementation, this would handle messages for the Boomerang mode
            // For now, we'll just log the message
            println!("Boomerang mode received message: {:?}", message);

            // For query messages, send a response
            if message.message_type == MessageType::Query {
                let response = Message::new(
                    format!("response-{}", message.id),
                    Mode::Boomerang,
                    message.sender,
                    MessageType::Response,
                    format!("Response to query: {}", message.content),
                )
                .with_task_id(message.task_id.unwrap_or_default());

                return Ok(Some(response));
            }

            Ok(None)
        }
    }
}

/// Communication protocol for exchanging information between modes
pub struct CommunicationProtocol {
    /// Message bus
    message_bus: Arc<MessageBus>,
}

impl CommunicationProtocol {
    /// Create a new communication protocol
    pub fn new() -> Self {
        let message_bus = Arc::new(MessageBus::new());

        // Register default handlers
        message_bus
            .register_handler(Mode::Debug, Box::new(handlers::DebugModeHandler))
            .expect("Failed to register Debug mode handler");

        message_bus
            .register_handler(Mode::Code, Box::new(handlers::CodeModeHandler))
            .expect("Failed to register Code mode handler");

        message_bus
            .register_handler(Mode::Test, Box::new(handlers::TestModeHandler))
            .expect("Failed to register Test mode handler");

        message_bus
            .register_handler(Mode::Boomerang, Box::new(handlers::BoomerangModeHandler))
            .expect("Failed to register Boomerang mode handler");

        Self { message_bus }
    }

    /// Get the message bus
    pub fn message_bus(&self) -> Arc<MessageBus> {
        self.message_bus.clone()
    }

    /// Send a task delegation message
    pub fn send_task_delegation(
        &self,
        sender: Mode,
        receiver: Mode,
        task: &Task,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("task-delegation-{}", task.id),
            sender,
            receiver,
            MessageType::TaskDelegation,
            format!("Task delegation: {}", task.title),
        )
        .with_task_id(task.id.clone())
        .with_priority(MessagePriority::High);

        self.message_bus.send_message(message)?;

        Ok(())
    }

    /// Send a task status update message
    pub fn send_task_status_update(
        &self,
        sender: Mode,
        receiver: Mode,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("task-status-update-{}", task_id),
            sender,
            receiver,
            MessageType::TaskStatusUpdate,
            format!("Task status update: {}", status),
        )
        .with_task_id(task_id)
        .with_metadata("status", format!("{:?}", status));

        self.message_bus.send_message(message)?;

        Ok(())
    }

    /// Send a task result message
    pub fn send_task_result(
        &self,
        sender: Mode,
        receiver: Mode,
        result: &TaskResult,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("task-result-{}", result.task_id),
            sender,
            receiver,
            MessageType::TaskResult,
            format!("Task result: {}", result.message),
        )
        .with_task_id(result.task_id.clone())
        .with_metadata("status", format!("{:?}", result.status));

        // Add result data as metadata
        let mut message_with_data = message;
        for (key, value) in &result.data {
            message_with_data = message_with_data.with_metadata(key, value);
        }

        self.message_bus.send_message(message_with_data)?;

        Ok(())
    }

    /// Send a mode status update message
    pub fn send_mode_status_update(
        &self,
        sender: Mode,
        receiver: Mode,
        status: &str,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("mode-status-update-{}", sender),
            sender,
            receiver,
            MessageType::ModeStatusUpdate,
            format!("Mode status update: {}", status),
        );

        self.message_bus.send_message(message)?;

        Ok(())
    }

    /// Send a command message
    pub fn send_command(
        &self,
        sender: Mode,
        receiver: Mode,
        command: &str,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("command-{}", Instant::now().elapsed().as_millis()),
            sender,
            receiver,
            MessageType::Command,
            format!("Command: {}", command),
        );

        self.message_bus.send_message(message)?;

        Ok(())
    }

    /// Send a query message and wait for a response
    pub fn send_query(
        &self,
        sender: Mode,
        receiver: Mode,
        query: &str,
    ) -> Result<Option<Message>, OrchestratorError> {
        let message = Message::new(
            format!("query-{}", Instant::now().elapsed().as_millis()),
            sender,
            receiver,
            MessageType::Query,
            format!("Query: {}", query),
        );

        self.message_bus.send_message(message)
    }

    /// Send a notification message
    pub fn send_notification(
        &self,
        sender: Mode,
        receiver: Mode,
        notification: &str,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("notification-{}", Instant::now().elapsed().as_millis()),
            sender,
            receiver,
            MessageType::Notification,
            format!("Notification: {}", notification),
        );

        self.message_bus.send_message(message)?;

        Ok(())
    }

    /// Send an error message
    pub fn send_error(
        &self,
        sender: Mode,
        receiver: Mode,
        error: &str,
    ) -> Result<(), OrchestratorError> {
        let message = Message::new(
            format!("error-{}", Instant::now().elapsed().as_millis()),
            sender,
            receiver,
            MessageType::Error,
            format!("Error: {}", error),
        )
        .with_priority(MessagePriority::High);

        self.message_bus.send_message(message)?;

        Ok(())
    }

    /// Broadcast a notification to all modes
    pub fn broadcast_notification(
        &self,
        sender: Mode,
        notification: &str,
    ) -> Result<(), OrchestratorError> {
        self.message_bus.broadcast_message(
            sender,
            MessageType::Notification,
            format!("Notification: {}", notification),
        )
    }

    /// Get the message history
    pub fn get_message_history(&self) -> Result<Vec<Message>, OrchestratorError> {
        self.message_bus.get_history()
    }

    /// Get the message history for a specific mode
    pub fn get_message_history_for_mode(
        &self,
        mode: Mode,
    ) -> Result<Vec<Message>, OrchestratorError> {
        self.message_bus.get_history_for_mode(mode)
    }

    /// Get the message history for a specific task
    pub fn get_message_history_for_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<Message>, OrchestratorError> {
        self.message_bus.get_history_for_task(task_id)
    }
}
