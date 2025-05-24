//! Orchestrator Module
//!
//! This module provides the Boomerang orchestration functionality for coordinating
//! testing strategies across different specialized modes (Debug, Code, Test).
//! It delegates tasks, tracks execution, and integrates results from different modes.


pub mod architecture;
pub mod communication;
pub mod continuous_improvement;
pub mod delegation;
pub mod integration;
pub mod mode_handlers;
pub mod reporting;
pub mod task_manager;
pub mod types;
pub mod workflow;
pub mod workflow_templates;

pub use architecture::{OrchestratorArchitecture, OrchestratorConfig};
pub use communication::{
    CommunicationProtocol, Message, MessageBus, MessageHandler, MessagePriority, MessageType,
};
pub use continuous_improvement::{
    AnalysisFinding, AnalysisRecommendation, AnalysisResult, Analyzer, ContinuousImprovementSystem,
    CoverageAnalyzer, EstimatedImpact, FindingSeverity, ImplementationDifficulty,
    OrchestratorReporting, PerformanceAnalyzer, QualityAnalyzer,
};
pub use delegation::{TaskDelegator, TaskTracker};
pub use integration::{IntegrationFramework, ResultAggregator};
pub use mode_handlers::{CodeModeHandler, DebugModeHandler, ModeHandlerFactory, TestModeHandler};
pub use reporting::{ContinuousImprovement, ReportGenerator};
pub use task_manager::{TaskManager, TaskManagerConfig};
pub use types::{
    DelegationError, IntegrationError, Mode, OrchestratorError, ReportingError, Task, TaskResult,
    TaskStatus, WorkflowError,
};
pub use workflow::{WorkflowExecutor, WorkflowManager};
pub use workflow_templates::{
    ConditionalWorkflowBuilder, FanInWorkflowBuilder, FanOutWorkflowBuilder,
    ParallelWorkflowBuilder, PipelineWorkflowBuilder, SequentialWorkflowBuilder, WorkflowTemplate,
    WorkflowTemplateManager, WorkflowTemplateType,
};

/// Create a new orchestrator with default configuration
pub fn create_orchestrator() -> OrchestratorArchitecture {
    OrchestratorArchitecture::new()
}

/// Create a new orchestrator with custom configuration
pub fn create_orchestrator_with_config(config: OrchestratorConfig) -> OrchestratorArchitecture {
    OrchestratorArchitecture::with_config(config)
}

/// Create a new task delegator
pub fn create_task_delegator() -> TaskDelegator {
    TaskDelegator::new()
}

/// Create a new integration framework
pub fn create_integration_framework() -> IntegrationFramework {
    IntegrationFramework::new()
}

/// Create a new workflow manager
pub fn create_workflow_manager() -> WorkflowManager {
    WorkflowManager::new()
}

/// Create a new report generator
pub fn create_report_generator() -> ReportGenerator {
    ReportGenerator::new()
}

/// Create a new task manager with default configuration
pub fn create_task_manager() -> TaskManager {
    TaskManager::new(TaskManagerConfig::default())
}

/// Create a new task manager with custom configuration
pub fn create_task_manager_with_config(config: TaskManagerConfig) -> TaskManager {
    TaskManager::new(config)
}

/// Create a new communication protocol
pub fn create_communication_protocol() -> CommunicationProtocol {
    CommunicationProtocol::new()
}

/// Create a new workflow template manager
pub fn create_workflow_template_manager() -> WorkflowTemplateManager {
    WorkflowTemplateManager::new()
}

/// Create a new continuous improvement system
pub fn create_continuous_improvement_system() -> ContinuousImprovementSystem {
    ContinuousImprovementSystem::new()
}
