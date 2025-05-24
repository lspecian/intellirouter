//! Benchmarking harness for IntelliRouter
//!
//! This module provides common functionality for benchmarking IntelliRouter components.

use criterion::{BenchmarkId, Criterion, Throughput};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Benchmark types supported by the framework
#[derive(Debug, Clone, Copy)]
pub enum BenchmarkType {
    /// Measure throughput (operations per second)
    Throughput,
    /// Measure latency (time per operation)
    Latency,
    /// Measure resource usage (memory, CPU, etc.)
    ResourceUsage,
}

/// Configuration for a benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Name of the benchmark
    pub name: String,
    /// Description of the benchmark
    pub description: String,
    /// Type of benchmark
    pub benchmark_type: BenchmarkType,
    /// Measurement unit (e.g., "requests", "operations", "bytes")
    pub unit: String,
    /// Sample size for the benchmark
    pub sample_size: Option<usize>,
    /// Warm-up time for the benchmark
    pub warm_up_time: Option<Duration>,
    /// Measurement time for the benchmark
    pub measurement_time: Option<Duration>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            description: "".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "operations".to_string(),
            sample_size: None,
            warm_up_time: None,
            measurement_time: None,
        }
    }
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Type of benchmark
    pub benchmark_type: BenchmarkType,
    /// Measurement unit
    pub unit: String,
    /// Median value
    pub median: f64,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Throughput (operations per second) if applicable
    pub throughput: Option<f64>,
    /// Timestamp when the benchmark was run
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Trait for benchmarkable components
pub trait Benchmarkable: Send + Sync {
    /// Run a single iteration of the benchmark
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>>;

    /// Setup before running the benchmark
    fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Teardown after running the benchmark
    fn teardown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Get the benchmark configuration
    fn config(&self) -> BenchmarkConfig;
}

/// Run a benchmark using Criterion
pub fn run_benchmark<B: Benchmarkable + 'static>(c: &mut Criterion, benchmarkable: Arc<B>) {
    let config = benchmarkable.config();

    let mut group = c.benchmark_group(&config.name);

    if let Some(sample_size) = config.sample_size {
        group.sample_size(sample_size);
    }

    if let Some(warm_up_time) = config.warm_up_time {
        group.warm_up_time(warm_up_time);
    }

    if let Some(measurement_time) = config.measurement_time {
        group.measurement_time(measurement_time);
    }

    match config.benchmark_type {
        BenchmarkType::Throughput => {
            group.throughput(Throughput::Elements(1));
        }
        BenchmarkType::Latency => {
            // Default Criterion configuration is good for latency
        }
        BenchmarkType::ResourceUsage => {
            // For resource usage, we might need custom measurements
            // This is a placeholder for now
        }
    }

    let bench_id = BenchmarkId::new(&config.name, "");
    group.bench_with_input(bench_id, &benchmarkable, |b, benchmarkable| {
        // Setup
        let _ = benchmarkable.setup();

        b.iter(|| {
            let result = benchmarkable.run_iteration();
            assert!(
                result.is_ok(),
                "Benchmark iteration failed: {:?}",
                result.err()
            );
        });

        // Teardown
        let _ = benchmarkable.teardown();
    });

    group.finish();
}

/// Measure resource usage during a benchmark
pub fn measure_resource_usage<F>(f: F) -> (Duration, ResourceUsage)
where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
{
    // Record initial resource usage
    let initial_usage = ResourceUsage::current();

    // Measure execution time
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();

    // Record final resource usage
    let final_usage = ResourceUsage::current();

    // Calculate difference
    let usage_diff = final_usage - initial_usage;

    assert!(
        result.is_ok(),
        "Function execution failed: {:?}",
        result.err()
    );

    (duration, usage_diff)
}

/// Resource usage metrics
#[derive(Debug, Clone, Copy)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// CPU usage percentage (0-100)
    pub cpu_percentage: f64,
}

impl ResourceUsage {
    /// Get current resource usage
    pub fn current() -> Self {
        // This is a placeholder implementation
        // In a real implementation, we would use system APIs to get actual resource usage
        // For example, using the `sysinfo` crate
        Self {
            memory_bytes: 0,
            cpu_percentage: 0.0,
        }
    }
}

impl std::ops::Sub for ResourceUsage {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            memory_bytes: self.memory_bytes.saturating_sub(other.memory_bytes),
            cpu_percentage: (self.cpu_percentage - other.cpu_percentage).max(0.0),
        }
    }
}

/// Save benchmark results to a file
pub fn save_benchmark_result(result: &BenchmarkResult, path: &str) -> std::io::Result<()> {
    use std::fs::{File, OpenOptions};
    use std::io::Write;

    // Create directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Check if file exists
    let file_exists = std::path::Path::new(path).exists();

    let mut file = if file_exists {
        OpenOptions::new().append(true).open(path)?
    } else {
        let mut file = File::create(path)?;
        // Write header if creating a new file
        writeln!(
            file,
            "name,type,unit,median,mean,std_dev,min,max,throughput,timestamp"
        )?;
        file
    };

    // Write result as CSV
    writeln!(
        file,
        "{},{:?},{},{},{},{},{},{},{},{}",
        result.name,
        result.benchmark_type,
        result.unit,
        result.median,
        result.mean,
        result.std_dev,
        result.min,
        result.max,
        result.throughput.unwrap_or(0.0),
        result.timestamp.to_rfc3339()
    )?;

    Ok(())
}
