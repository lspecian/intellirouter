//! Benchmark Example
//!
//! This example demonstrates how to use the IntelliRouter benchmarking system.

use intellirouter::modules::test_harness::{
    benchmark::{BenchmarkConfig, BenchmarkRunner, BenchmarkSuite, BenchmarkType},
    reporting::{ReportConfig, ReportGenerator, ReportManager, TestRun},
    types::TestHarnessError,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Benchmark Example");
    println!("===============================");

    // Create a report configuration
    let report_config = ReportConfig {
        title: "IntelliRouter Benchmark Report".to_string(),
        description: Some("Example benchmark report for IntelliRouter".to_string()),
        output_dir: PathBuf::from("reports"),
        formats: vec![intellirouter::modules::test_harness::reporting::ExportFormat::Html],
        ..ReportConfig::default()
    };
    println!("Created report configuration");

    // Create a report generator
    let report_generator = Arc::new(ReportGenerator::new(report_config));
    println!("Created report generator");

    // Create a report manager
    let report_manager = ReportManager::new("reports", Arc::clone(&report_generator));
    println!("Created report manager");

    // Start a test run
    let run_id = format!("benchmark-{}", Utc::now().timestamp());
    report_manager
        .start_run(&run_id, "Benchmark Test Run")
        .await?;
    println!("Started benchmark test run: {}", run_id);

    // Create benchmark configurations
    println!("\nCreating benchmark configurations...");

    // Throughput benchmark
    let throughput_config = BenchmarkConfig::new(
        "throughput-bench",
        "Throughput Benchmark",
        BenchmarkType::Throughput,
    )
    .with_description("Measures the maximum throughput of the system")
    .with_duration(Duration::from_secs(2))
    .with_warmup_duration(Duration::from_millis(500))
    .with_cooldown_duration(Duration::from_millis(500))
    .with_concurrency(10)
    .with_tag("throughput")
    .with_tag("performance");

    println!("Created throughput benchmark configuration");

    // Latency benchmark
    let latency_config =
        BenchmarkConfig::new("latency-bench", "Latency Benchmark", BenchmarkType::Latency)
            .with_description("Measures the response latency of the system")
            .with_duration(Duration::from_secs(2))
            .with_warmup_duration(Duration::from_millis(500))
            .with_cooldown_duration(Duration::from_millis(500))
            .with_concurrency(1)
            .with_tag("latency")
            .with_tag("performance");

    println!("Created latency benchmark configuration");

    // Create benchmark functions
    println!("\nCreating benchmark functions...");

    // Throughput benchmark function
    let throughput_fn = || {
        // Simulate a fast operation
        thread::sleep(Duration::from_millis(1));
        Ok(Duration::from_millis(1))
    };

    // Latency benchmark function
    let latency_fn = || {
        // Simulate a variable latency operation
        let mut rng = rand::thread_rng();
        let latency = rng.gen_range(5..20);
        thread::sleep(Duration::from_millis(latency));

        // Occasionally simulate an error
        if rng.gen_bool(0.05) {
            Err("Simulated error".to_string())
        } else {
            Ok(Duration::from_millis(latency))
        }
    };

    // Create benchmark runners
    println!("\nCreating benchmark runners...");

    let throughput_runner = BenchmarkRunner::new(throughput_config, throughput_fn);
    println!("Created throughput benchmark runner");

    let latency_runner = BenchmarkRunner::new(latency_config, latency_fn);
    println!("Created latency benchmark runner");

    // Run benchmarks
    println!("\nRunning benchmarks...");

    println!("Running throughput benchmark...");
    let throughput_result = throughput_runner.run().await?;
    println!("Throughput benchmark completed");
    println!("  Throughput: {:.2} ops/sec", throughput_result.throughput);
    println!(
        "  Latency (avg): {:.2} ms",
        throughput_result.latency.avg_duration.as_secs_f64() * 1000.0
    );
    println!(
        "  Latency (p95): {:.2} ms",
        throughput_result.latency.p95_duration.as_secs_f64() * 1000.0
    );

    println!("\nRunning latency benchmark...");
    let latency_result = latency_runner.run().await?;
    println!("Latency benchmark completed");
    println!("  Throughput: {:.2} ops/sec", latency_result.throughput);
    println!(
        "  Latency (avg): {:.2} ms",
        latency_result.latency.avg_duration.as_secs_f64() * 1000.0
    );
    println!(
        "  Latency (p95): {:.2} ms",
        latency_result.latency.p95_duration.as_secs_f64() * 1000.0
    );
    println!("  Error rate: {:.2}%", latency_result.error_rate * 100.0);

    // Add benchmark results to the test run
    println!("\nAdding benchmark results to test run...");

    let throughput_test_result = throughput_result.to_test_result();
    report_manager.add_result(throughput_test_result).await?;
    println!("Added throughput benchmark result to test run");

    let latency_test_result = latency_result.to_test_result();
    report_manager.add_result(latency_test_result).await?;
    println!("Added latency benchmark result to test run");

    // End the test run
    let test_run = report_manager.end_run().await?;
    println!("\nEnded benchmark test run: {}", test_run.id);

    // Generate reports
    println!("\nGenerating benchmark reports...");
    report_manager.generate_report().await?;
    println!("Benchmark reports generated successfully in the 'reports' directory");

    // Create a benchmark suite
    println!("\nCreating benchmark suite...");

    let suite = BenchmarkSuite::new("perf-suite", "Performance Benchmark Suite")
        .with_description("A suite of performance benchmarks for IntelliRouter")
        .with_benchmark(throughput_result.config)
        .with_benchmark(latency_result.config)
        .with_tag("performance");

    println!("Created benchmark suite: {}", suite.name());
    println!("  Benchmarks: {}", suite.benchmarks().len());
    println!("  Tags: {}", suite.tags().join(", "));

    println!("\nBenchmark example completed successfully!");
    Ok(())
}
