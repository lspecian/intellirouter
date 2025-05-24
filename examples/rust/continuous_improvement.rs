//! Continuous Improvement Example
//!
//! This example demonstrates how to use the Reporting and Continuous Improvement System
//! to analyze testing results and suggest improvements.

use intellirouter::modules::orchestrator::{
    create_continuous_improvement_system, create_report_generator, AnalysisFinding,
    AnalysisRecommendation, EstimatedImpact, FindingSeverity, ImplementationDifficulty, Mode,
    SuggestionPriority, SuggestionStatus, Task, TaskResult, TaskStatus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Continuous Improvement Example");
    println!("==============================");

    // Create a continuous improvement system
    let improvement_system = create_continuous_improvement_system();

    // Create some tasks for analysis
    let tasks = create_sample_tasks();
    println!("\nAnalyzing {} tasks...", tasks.len());

    // Analyze tasks
    let analysis_results = improvement_system.analyze_tasks(&tasks)?;

    // Print analysis results
    println!("\nAnalysis Results:");
    for (i, result) in analysis_results.iter().enumerate() {
        println!("{}. {} ({})", i + 1, result.name, result.id);

        println!("   Metrics:");
        for (key, value) in &result.metrics {
            println!("   - {}: {:.2}", key, value);
        }

        if !result.findings.is_empty() {
            println!("   Findings:");
            for finding in &result.findings {
                println!("   - {} (Severity: {:?})", finding.title, finding.severity);
                println!("     {}", finding.description);
            }
        }

        if !result.recommendations.is_empty() {
            println!("   Recommendations:");
            for recommendation in &result.recommendations {
                println!(
                    "   - {} (Priority: {:?}, Difficulty: {:?}, Impact: {:?})",
                    recommendation.title,
                    recommendation.priority,
                    recommendation.difficulty,
                    recommendation.impact
                );
                println!("     {}", recommendation.description);
            }
        }

        println!();
    }

    // Get improvement suggestions
    let suggestions = improvement_system.get_all_suggestions()?;

    println!("\nImprovement Suggestions:");
    for (i, suggestion) in suggestions.iter().enumerate() {
        println!(
            "{}. {} (Priority: {:?}, Status: {:?})",
            i + 1,
            suggestion.title,
            suggestion.priority,
            suggestion.status
        );
        println!("   {}", suggestion.description);

        if !suggestion.affected_tasks.is_empty() {
            println!(
                "   Affected Tasks: {}",
                suggestion.affected_tasks.join(", ")
            );
        }

        if !suggestion.affected_workflows.is_empty() {
            println!(
                "   Affected Workflows: {}",
                suggestion.affected_workflows.join(", ")
            );
        }

        println!();
    }

    // Update suggestion status
    if !suggestions.is_empty() {
        let suggestion_id = &suggestions[0].id;
        println!(
            "\nUpdating status of suggestion '{}' to Implemented...",
            suggestion_id
        );
        improvement_system
            .update_suggestion_status(suggestion_id, SuggestionStatus::Implemented)?;

        // Get suggestions by status
        let implemented_suggestions =
            improvement_system.get_suggestions_by_status(SuggestionStatus::Implemented)?;

        println!("\nImplemented Suggestions:");
        for (i, suggestion) in implemented_suggestions.iter().enumerate() {
            println!(
                "{}. {} (Priority: {:?})",
                i + 1,
                suggestion.title,
                suggestion.priority
            );
        }
    }

    // Create a report generator
    let report_generator = create_report_generator();

    // Register a report configuration
    report_generator.register_config(
        "continuous-improvement",
        intellirouter::modules::orchestrator::reporting::ReportConfig::default()
            .with_format(intellirouter::modules::orchestrator::reporting::ReportFormat::Markdown)
            .with_task_details(true)
            .with_task_results(true)
            .with_workflow_details(true),
    )?;

    // Generate a report
    println!("\nGenerating report...");
    let report =
        report_generator.generate_report("continuous-improvement", &create_mock_orchestrator())?;

    println!("\nReport Preview (first 500 characters):");
    println!(
        "{}",
        if report.len() > 500 {
            &report[..500]
        } else {
            &report
        }
    );

    println!("\nContinuous improvement example completed successfully!");

    Ok(())
}

// Create sample tasks for analysis
fn create_sample_tasks() -> Vec<Task> {
    let mut tasks = Vec::new();

    // Debug tasks
    for i in 1..=5 {
        let task = Task::new(
            format!("debug-{}", i),
            format!("Debug Task {}", i),
            format!("A debug task for testing component {}", i),
            Mode::Debug,
        );
        tasks.push(task);
    }

    // Code tasks
    for i in 1..=10 {
        let mut task = Task::new(
            format!("code-{}", i),
            format!("Code Task {}", i),
            format!("A code task for implementing feature {}", i),
            Mode::Code,
        );

        // Add dependencies for some tasks
        if i > 1 {
            task.dependencies.push(format!("code-{}", i - 1));
        }

        tasks.push(task);
    }

    // Test tasks
    for i in 1..=3 {
        let mut task = Task::new(
            format!("test-{}", i),
            format!("Test Task {}", i),
            format!("A test task for validating feature {}", i),
            Mode::Test,
        );

        // Add dependencies on code tasks
        task.dependencies.push(format!("code-{}", i));

        tasks.push(task);
    }

    // Set status for some tasks
    tasks[0].status = TaskStatus::Completed; // debug-1
    tasks[1].status = TaskStatus::Completed; // debug-2
    tasks[2].status = TaskStatus::Failed; // debug-3
    tasks[5].status = TaskStatus::Completed; // code-1
    tasks[6].status = TaskStatus::Completed; // code-2
    tasks[7].status = TaskStatus::InProgress; // code-3

    tasks
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
