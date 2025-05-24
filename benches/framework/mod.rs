//! Benchmarking framework for IntelliRouter
//!
//! This module provides a framework for benchmarking IntelliRouter components.

pub mod harness;
pub mod metrics;
pub mod reporters;

pub use harness::{
    measure_resource_usage, run_benchmark, save_benchmark_result, BenchmarkConfig, BenchmarkResult,
    BenchmarkType, Benchmarkable, ResourceUsage,
};
