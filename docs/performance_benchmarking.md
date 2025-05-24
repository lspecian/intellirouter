# Performance Benchmarking System

This document describes the performance benchmarking system for the IntelliRouter project. The system is designed to measure the performance of key components, track performance metrics over time, and identify performance regressions.

## Overview

The performance benchmarking system consists of the following components:

1. **Benchmarking Framework**: A reusable framework for creating and running benchmarks.
2. **Component Benchmarks**: Specific benchmarks for each key component of the system.
3. **Performance Tracking**: Tools for storing and analyzing benchmark results over time.
4. **CI Integration**: Integration with the CI pipeline to run benchmarks and detect regressions.
5. **Reporting**: Tools for generating performance reports and visualizations.

## Benchmarking Framework

The benchmarking framework is located in the `benches/framework/` directory and provides common functionality for all benchmarks:

- `harness.rs`: Core benchmarking functionality, including the `Benchmarkable` trait and benchmark configuration.
- `metrics.rs`: Tools for collecting and analyzing benchmark metrics.
- `reporters.rs`: Tools for generating reports and visualizations from benchmark results.

### Key Concepts

- **Benchmark Types**: The framework supports different types of benchmarks:
  - **Throughput**: Measures operations per second.
  - **Latency**: Measures time per operation.
  - **Resource Usage**: Measures memory and CPU usage.

- **Benchmarkable Trait**: All benchmarks implement the `Benchmarkable` trait, which defines methods for running iterations, setup, teardown, and configuration.

- **Benchmark Configuration**: Each benchmark has a configuration that specifies its name, description, type, measurement unit, and other parameters.

- **Benchmark Results**: Results include metrics such as median, mean, standard deviation, min, max, and throughput.

## Component Benchmarks

The system includes benchmarks for the following key components:

### Router Core (`benches/router_benchmarks.rs`)

- **Router Creation**: Measures the time to create a router instance.
- **Update from Registry**: Measures the time to update the router from the model registry.
- **Get Filtered Models**: Measures the time to filter models based on a request.
- **Routing Strategies**: Measures the performance of different routing strategies (round-robin, content-based).

### Model Registry (`benches/model_registry_benchmarks.rs`)

- **Model Registration**: Measures the time to register models in the registry.
- **Model Lookup**: Measures the throughput of model lookups.
- **Model Filtering**: Measures the time to filter models based on criteria.

### Chain Engine (`benches/chain_engine_benchmarks.rs`)

- **Chain Creation**: Measures the time to create chains.
- **Chain Validation**: Measures the throughput of chain validation.
- **Chain Execution**: Measures the latency of chain execution.

### Memory (`benches/memory_benchmarks.rs`)

- **Memory Creation**: Measures the time to create memory instances.
- **Add Entry**: Measures the throughput of adding entries to memory.
- **Get Messages**: Measures the latency of retrieving messages from memory.
- **Time Range Retrieval**: Measures the latency of retrieving entries by time range.

### RAG Manager (`benches/rag_manager_benchmarks.rs`)

- **RAG Manager Creation**: Measures the time to create a RAG manager.
- **Add Source**: Measures the throughput of adding sources to the RAG manager.
- **Retrieve Context**: Measures the latency of retrieving context.
- **Fuse Context**: Measures the latency of fusing context chunks.

## Running Benchmarks

The benchmarks can be run using the `scripts/run_benchmarks.sh` script, which:

1. Runs all benchmarks using Criterion.
2. Stores the results in the `metrics/performance/` directory.
3. Generates performance reports and charts.
4. Checks for performance regressions compared to previous runs.

### Command Line

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run a specific benchmark
cargo bench --bench router_benchmarks

# Run a specific benchmark with verbose output
cargo bench --bench router_benchmarks -- --verbose

# Run a specific benchmark function
cargo bench --bench router_benchmarks -- bench_router_creation
```

## Performance Tracking

Benchmark results are stored in the `metrics/performance/` directory:

- `benchmark_results_*.csv`: CSV files containing benchmark results with timestamps.
- `reports/`: Directory containing performance reports and regression reports.
- `charts/`: Directory containing performance charts.

### Metrics Collected

For each benchmark, the following metrics are collected:

- **Median**: The median execution time.
- **Mean**: The mean execution time.
- **Standard Deviation**: The standard deviation of execution times.
- **Min**: The minimum execution time.
- **Max**: The maximum execution time.
- **Throughput**: For throughput benchmarks, the operations per second.

## CI Integration

The benchmarking system is integrated with the CI pipeline using GitHub Actions:

- Benchmarks are run daily and on pushes to the main branch.
- Results are stored as artifacts.
- Performance regressions are detected and reported.
- If regressions are detected, an issue is created automatically.

The CI configuration is located in `.github/workflows/benchmarks.yml`.

## Performance Reports

The system generates two types of reports:

### Performance Report

The performance report includes:

- A summary of all benchmarks.
- Detailed results for each component and benchmark.
- Performance charts showing trends over time.

### Regression Report

The regression report includes:

- A comparison with previous benchmark results.
- Identification of performance regressions.
- Recommendations for addressing regressions.

## Adding New Benchmarks

To add a new benchmark:

1. Create a new benchmark file in the `benches/` directory.
2. Import the benchmarking framework.
3. Define a struct that implements the `Benchmarkable` trait.
4. Add the benchmark to the `criterion_group!` macro.
5. Update the `scripts/run_benchmarks.sh` script to include the new benchmark.

Example:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use std::time::Duration;

mod framework;
use framework::{
    Benchmarkable, BenchmarkConfig, BenchmarkType, ResourceUsage, run_benchmark,
};

struct MyBenchmark {}

impl MyBenchmark {
    fn new() -> Self {
        Self {}
    }
}

impl Benchmarkable for MyBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        
        // Benchmark code here
        
        Ok(start.elapsed())
    }
    
    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "my_benchmark".to_string(),
            description: "My benchmark description".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "operations".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

fn bench_my_component(c: &mut Criterion) {
    let benchmark = Arc::new(MyBenchmark::new());
    run_benchmark(c, benchmark);
}

criterion_group!(benches, bench_my_component);
criterion_main!(benches);
```

## Best Practices

When working with the benchmarking system, follow these best practices:

1. **Isolation**: Ensure benchmarks are isolated from each other and from external factors.
2. **Reproducibility**: Make benchmarks reproducible by using fixed seeds for random operations.
3. **Warm-up**: Include warm-up iterations to avoid cold-start effects.
4. **Realistic Workloads**: Use realistic workloads that reflect actual usage patterns.
5. **Resource Cleanup**: Clean up resources after benchmarks to avoid affecting subsequent benchmarks.
6. **Documentation**: Document what each benchmark measures and how it relates to real-world performance.
7. **Regular Runs**: Run benchmarks regularly to track performance over time.
8. **Investigate Regressions**: Investigate and address performance regressions promptly.

## Future Improvements

Planned improvements to the benchmarking system include:

1. **More Detailed Resource Monitoring**: Add more detailed monitoring of memory, CPU, and I/O usage.
2. **Distributed Benchmarking**: Support for running benchmarks across multiple machines.
3. **Load Testing**: Add support for load testing with multiple concurrent users.
4. **Profiling Integration**: Integrate with profiling tools to identify bottlenecks.
5. **Custom Metrics**: Support for custom application-specific metrics.
6. **Interactive Dashboard**: Create an interactive dashboard for exploring benchmark results.
7. **Anomaly Detection**: Add anomaly detection to identify unusual performance patterns.
8. **Benchmark Comparison**: Add tools for comparing benchmarks across different configurations or versions.

## Conclusion

The performance benchmarking system provides a comprehensive framework for measuring, tracking, and improving the performance of the IntelliRouter system. By regularly running benchmarks and addressing performance regressions, we can ensure that the system remains performant as it evolves.