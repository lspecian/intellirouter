//! Performance benchmarks for the RAG manager module

use criterion::{criterion_group, criterion_main, Criterion};
use intellirouter::modules::rag_manager::{
    ContextChunk, FileContextSource, RagDocument, RagManager,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

mod framework;
use framework::{run_benchmark, BenchmarkConfig, BenchmarkType, Benchmarkable, ResourceUsage};

// Helper function to create test documents
fn create_test_documents(num_docs: usize) -> Vec<RagDocument> {
    let mut documents = Vec::with_capacity(num_docs);

    for i in 0..num_docs {
        let metadata = HashMap::new();

        let content = format!(
            "This is test document {} content. It contains information about topic {} and category {}. \
            This document is meant to be used for benchmarking the RAG manager module. \
            It has some keywords like intellirouter, performance, benchmarking, and testing.",
            i, i % 10, i % 5
        );

        let document = RagDocument {
            id: format!("doc_{}", i),
            content,
            metadata,
        };

        documents.push(document);
    }

    documents
}

// Helper function to create test context chunks
fn create_test_context_chunks(num_chunks: usize) -> Vec<ContextChunk> {
    let mut chunks = Vec::with_capacity(num_chunks);

    for i in 0..num_chunks {
        let chunk = ContextChunk {
            content: format!("This is test chunk {} content.", i),
            source: format!("source_{}", i % 5),
            relevance_score: 0.9 - (i as f32 * 0.01),
            metadata: HashMap::new(),
        };

        chunks.push(chunk);
    }

    chunks
}

/// Benchmark for RAG manager creation
struct RagManagerCreationBenchmark {}

impl RagManagerCreationBenchmark {
    fn new() -> Self {
        Self {}
    }
}

impl Benchmarkable for RagManagerCreationBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Create a RAG manager
        let _manager = RagManager::new();

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "rag_manager_creation".to_string(),
            description: "Benchmark for creating a RAG manager".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "managers".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for adding sources to RAG manager
struct RagManagerAddSourceBenchmark {
    num_sources: usize,
}

impl RagManagerAddSourceBenchmark {
    fn new(num_sources: usize) -> Self {
        Self { num_sources }
    }
}

impl Benchmarkable for RagManagerAddSourceBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Create a RAG manager
        let mut manager = RagManager::new();

        // Add sources
        for i in 0..self.num_sources {
            let content = format!("This is test source {} content.", i);
            let name = format!("source_{}", i);
            let source = Arc::new(FileContextSource::new(content, name));
            manager.add_source(source);
        }

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "rag_manager_add_source".to_string(),
            description: "Benchmark for adding sources to a RAG manager".to_string(),
            benchmark_type: BenchmarkType::Throughput,
            unit: "sources".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for retrieving context
struct RagManagerRetrieveContextBenchmark {
    manager: RagManager,
    queries: Vec<String>,
}

impl RagManagerRetrieveContextBenchmark {
    fn new(num_sources: usize, num_queries: usize) -> Self {
        let mut manager = RagManager::new();

        // Add sources
        for i in 0..num_sources {
            let content = format!(
                "This is test source {} content. It contains information about topic {} and category {}. \
                This document is meant to be used for benchmarking the RAG manager module. \
                It has some keywords like intellirouter, performance, benchmarking, and testing.",
                i, i % 10, i % 5
            );
            let name = format!("source_{}", i);
            let source = Arc::new(FileContextSource::new(content, name));
            manager.add_source(source);
        }

        // Create queries
        let queries: Vec<String> = (0..num_queries)
            .map(|i| format!("Query about topic {} and category {}", i % 10, i % 5))
            .collect();

        Self { manager, queries }
    }
}

impl Benchmarkable for RagManagerRetrieveContextBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Retrieve context for all queries
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            for query in &self.queries {
                let _ = self.manager.retrieve_context(query, 5).await?;
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "rag_manager_retrieve_context".to_string(),
            description: "Benchmark for retrieving context from a RAG manager".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "retrievals".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for fusing context
struct RagManagerFuseContextBenchmark {
    manager: RagManager,
    chunks_sets: Vec<Vec<ContextChunk>>,
}

impl RagManagerFuseContextBenchmark {
    fn new(num_sets: usize, chunks_per_set: usize) -> Self {
        let manager = RagManager::new();

        // Create sets of chunks
        let chunks_sets: Vec<Vec<ContextChunk>> = (0..num_sets)
            .map(|_| create_test_context_chunks(chunks_per_set))
            .collect();

        Self {
            manager,
            chunks_sets,
        }
    }
}

impl Benchmarkable for RagManagerFuseContextBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Fuse context for all chunk sets
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            for chunks in &self.chunks_sets {
                let _ = self.manager.fuse_context(chunks, 1000).await?;
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "rag_manager_fuse_context".to_string(),
            description: "Benchmark for fusing context in a RAG manager".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "fusions".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

fn bench_rag_manager(c: &mut Criterion) {
    // RAG manager creation benchmark
    let creation_benchmark = Arc::new(RagManagerCreationBenchmark::new());
    run_benchmark(c, creation_benchmark);

    // RAG manager add source benchmark
    let add_source_benchmark = Arc::new(RagManagerAddSourceBenchmark::new(10));
    run_benchmark(c, add_source_benchmark);

    // RAG manager retrieve context benchmark
    let retrieve_context_benchmark = Arc::new(RagManagerRetrieveContextBenchmark::new(10, 5));
    run_benchmark(c, retrieve_context_benchmark);

    // RAG manager fuse context benchmark
    let fuse_context_benchmark = Arc::new(RagManagerFuseContextBenchmark::new(5, 10));
    run_benchmark(c, fuse_context_benchmark);
}

criterion_group!(benches, bench_rag_manager);
criterion_main!(benches);
