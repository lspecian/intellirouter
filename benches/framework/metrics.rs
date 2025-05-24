//! Metrics collection and analysis for benchmarks
//!
//! This module provides functionality for collecting, storing, and analyzing benchmark metrics.

use crate::framework::harness::{BenchmarkResult, BenchmarkType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Metrics storage for benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStorage {
    /// Path to the metrics directory
    #[serde(skip)]
    metrics_dir: PathBuf,
    /// Benchmark results by component and benchmark name
    results: HashMap<String, HashMap<String, Vec<BenchmarkResultRecord>>>,
}

/// Serializable benchmark result record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResultRecord {
    /// Name of the benchmark
    pub name: String,
    /// Type of benchmark
    pub benchmark_type: String,
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
    pub timestamp: DateTime<Utc>,
    /// Git commit hash when the benchmark was run
    pub commit_hash: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl From<BenchmarkResult> for BenchmarkResultRecord {
    fn from(result: BenchmarkResult) -> Self {
        Self {
            name: result.name,
            benchmark_type: format!("{:?}", result.benchmark_type),
            unit: result.unit,
            median: result.median,
            mean: result.mean,
            std_dev: result.std_dev,
            min: result.min,
            max: result.max,
            throughput: result.throughput,
            timestamp: result.timestamp,
            commit_hash: get_current_commit_hash(),
            metadata: result.metadata,
        }
    }
}

impl MetricsStorage {
    /// Create a new metrics storage
    pub fn new<P: AsRef<Path>>(metrics_dir: P) -> io::Result<Self> {
        let metrics_dir = metrics_dir.as_ref().to_path_buf();
        fs::create_dir_all(&metrics_dir)?;

        let index_path = metrics_dir.join("index.json");

        if index_path.exists() {
            // Load existing index
            let file = File::open(&index_path)?;
            let reader = BufReader::new(file);
            let mut storage: Self = serde_json::from_reader(reader)?;
            storage.metrics_dir = metrics_dir;
            Ok(storage)
        } else {
            // Create new index
            Ok(Self {
                metrics_dir,
                results: HashMap::new(),
            })
        }
    }

    /// Save the metrics storage
    pub fn save(&self) -> io::Result<()> {
        let index_path = self.metrics_dir.join("index.json");
        let file = File::create(&index_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Add a benchmark result
    pub fn add_result(&mut self, component: &str, result: BenchmarkResult) -> io::Result<()> {
        let record = BenchmarkResultRecord::from(result);

        // Add to in-memory index
        let component_results = self.results.entry(component.to_string()).or_default();
        let benchmark_results = component_results.entry(record.name.clone()).or_default();
        benchmark_results.push(record.clone());

        // Save to component-specific file
        let component_dir = self.metrics_dir.join(component);
        fs::create_dir_all(&component_dir)?;

        let benchmark_file = component_dir.join(format!("{}.json", record.name));

        let mut records = if benchmark_file.exists() {
            let file = File::open(&benchmark_file)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        } else {
            Vec::new()
        };

        records.push(record);

        let file = File::create(&benchmark_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &records)?;

        // Save the index
        self.save()?;

        Ok(())
    }

    /// Get benchmark results for a component
    pub fn get_component_results(
        &self,
        component: &str,
    ) -> Option<&HashMap<String, Vec<BenchmarkResultRecord>>> {
        self.results.get(component)
    }

    /// Get benchmark results for a specific benchmark in a component
    pub fn get_benchmark_results(
        &self,
        component: &str,
        benchmark: &str,
    ) -> Option<&Vec<BenchmarkResultRecord>> {
        self.results.get(component).and_then(|c| c.get(benchmark))
    }

    /// Get all components
    pub fn get_components(&self) -> Vec<&str> {
        self.results.keys().map(|s| s.as_str()).collect()
    }

    /// Get all benchmarks for a component
    pub fn get_benchmarks(&self, component: &str) -> Option<Vec<&str>> {
        self.results
            .get(component)
            .map(|c| c.keys().map(|s| s.as_str()).collect())
    }

    /// Detect performance regressions
    pub fn detect_regressions(&self, threshold: f64) -> Vec<PerformanceRegression> {
        let mut regressions = Vec::new();

        for (component, benchmarks) in &self.results {
            for (benchmark_name, results) in benchmarks {
                if results.len() < 2 {
                    continue;
                }

                // Sort by timestamp
                let mut sorted_results = results.clone();
                sorted_results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                // Compare the latest result with the previous one
                let latest = &sorted_results[sorted_results.len() - 1];
                let previous = &sorted_results[sorted_results.len() - 2];

                // Check for regression based on benchmark type
                let regression = match latest.benchmark_type.as_str() {
                    "Throughput" => {
                        // For throughput, lower is worse
                        if previous.mean > 0.0 && latest.mean < previous.mean * (1.0 - threshold) {
                            Some(PerformanceRegression {
                                component: component.clone(),
                                benchmark: benchmark_name.clone(),
                                previous_value: previous.mean,
                                current_value: latest.mean,
                                change_percentage: (latest.mean / previous.mean - 1.0) * 100.0,
                                previous_timestamp: previous.timestamp,
                                current_timestamp: latest.timestamp,
                                regression_type: RegressionType::Throughput,
                            })
                        } else {
                            None
                        }
                    }
                    "Latency" => {
                        // For latency, higher is worse
                        if latest.mean > previous.mean * (1.0 + threshold) {
                            Some(PerformanceRegression {
                                component: component.clone(),
                                benchmark: benchmark_name.clone(),
                                previous_value: previous.mean,
                                current_value: latest.mean,
                                change_percentage: (latest.mean / previous.mean - 1.0) * 100.0,
                                previous_timestamp: previous.timestamp,
                                current_timestamp: latest.timestamp,
                                regression_type: RegressionType::Latency,
                            })
                        } else {
                            None
                        }
                    }
                    "ResourceUsage" => {
                        // For resource usage, higher is worse
                        if latest.mean > previous.mean * (1.0 + threshold) {
                            Some(PerformanceRegression {
                                component: component.clone(),
                                benchmark: benchmark_name.clone(),
                                previous_value: previous.mean,
                                current_value: latest.mean,
                                change_percentage: (latest.mean / previous.mean - 1.0) * 100.0,
                                previous_timestamp: previous.timestamp,
                                current_timestamp: latest.timestamp,
                                regression_type: RegressionType::ResourceUsage,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(reg) = regression {
                    regressions.push(reg);
                }
            }
        }

        regressions
    }
}

/// Performance regression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Component name
    pub component: String,
    /// Benchmark name
    pub benchmark: String,
    /// Previous value
    pub previous_value: f64,
    /// Current value
    pub current_value: f64,
    /// Change percentage
    pub change_percentage: f64,
    /// Previous timestamp
    pub previous_timestamp: DateTime<Utc>,
    /// Current timestamp
    pub current_timestamp: DateTime<Utc>,
    /// Type of regression
    pub regression_type: RegressionType,
}

/// Type of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    /// Throughput regression (operations per second decreased)
    Throughput,
    /// Latency regression (time per operation increased)
    Latency,
    /// Resource usage regression (memory or CPU usage increased)
    ResourceUsage,
}

/// Get the current git commit hash
fn get_current_commit_hash() -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}
