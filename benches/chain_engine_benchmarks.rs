//! Performance benchmarks for the chain engine module

use criterion::{criterion_group, criterion_main, Criterion};
use intellirouter::modules::chain_engine::{
    Chain, ChainConfig, ChainEngine, ChainNode, ChainNodeConfig, ChainNodeType, ChainResult,
};
use std::sync::Arc;
use std::time::Duration;

mod framework;
use framework::{run_benchmark, BenchmarkConfig, BenchmarkType, Benchmarkable, ResourceUsage};

// Helper function to create a test chain with many nodes
fn create_test_chain(num_nodes: usize) -> Chain {
    let mut chain = Chain::new("test_chain".to_string(), ChainConfig::default());

    for i in 0..num_nodes {
        let node_type = match i % 3 {
            0 => ChainNodeType::Prompt,
            1 => ChainNodeType::LLM,
            _ => ChainNodeType::Tool,
        };

        let node_config = ChainNodeConfig {
            node_type,
            name: format!("node_{}", i),
            description: format!("Test node {}", i),
            config: serde_json::json!({
                "key": format!("value_{}", i),
            }),
        };

        let node = ChainNode::new(node_config);
        chain.add_node(node);
    }

    // Connect nodes in sequence
    for i in 0..(num_nodes - 1) {
        chain.connect_nodes(format!("node_{}", i), format!("node_{}", i + 1));
    }

    chain
}

/// Benchmark for chain creation
struct ChainCreationBenchmark {
    engine: Arc<ChainEngine>,
    chains: Vec<Chain>,
}

impl ChainCreationBenchmark {
    fn new(num_chains: usize, nodes_per_chain: usize) -> Self {
        let engine = Arc::new(ChainEngine::new());
        let mut chains = Vec::with_capacity(num_chains);

        for i in 0..num_chains {
            let mut chain = create_test_chain(nodes_per_chain);
            chain.id = format!("chain_{}", i);
            chains.push(chain);
        }

        Self { engine, chains }
    }
}

impl Benchmarkable for ChainCreationBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Register all chains
        for chain in &self.chains {
            self.engine.register_chain(chain.clone())?;
        }

        Ok(start.elapsed())
    }

    fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear the engine before each iteration
        self.engine.clear_chains();
        Ok(())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "chain_creation".to_string(),
            description: "Benchmark for creating chains in the chain engine".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "chains".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for chain validation
struct ChainValidationBenchmark {
    engine: Arc<ChainEngine>,
    chains: Vec<Chain>,
}

impl ChainValidationBenchmark {
    fn new(num_chains: usize, nodes_per_chain: usize) -> Self {
        let engine = Arc::new(ChainEngine::new());
        let mut chains = Vec::with_capacity(num_chains);

        for i in 0..num_chains {
            let mut chain = create_test_chain(nodes_per_chain);
            chain.id = format!("chain_{}", i);
            chains.push(chain);
        }

        Self { engine, chains }
    }
}

impl Benchmarkable for ChainValidationBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Validate all chains
        for chain in &self.chains {
            self.engine.validate_chain(chain)?;
        }

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "chain_validation".to_string(),
            description: "Benchmark for validating chains in the chain engine".to_string(),
            benchmark_type: BenchmarkType::Throughput,
            unit: "validations".to_string(),
            sample_size: Some(50),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for chain execution
struct ChainExecutionBenchmark {
    engine: Arc<ChainEngine>,
    chain_ids: Vec<String>,
    inputs: Vec<serde_json::Value>,
}

impl ChainExecutionBenchmark {
    fn new(engine: Arc<ChainEngine>, num_executions: usize) -> Self {
        // Register some chains
        for i in 0..10 {
            let mut chain = create_test_chain(5);
            chain.id = format!("chain_{}", i);
            engine.register_chain(chain).unwrap();
        }

        let chain_ids: Vec<String> = (0..num_executions)
            .map(|i| format!("chain_{}", i % 10))
            .collect();

        let inputs: Vec<serde_json::Value> = (0..num_executions)
            .map(|i| {
                serde_json::json!({
                    "input": format!("test_input_{}", i),
                    "parameters": {
                        "param1": i,
                        "param2": format!("value_{}", i),
                    }
                })
            })
            .collect();

        Self {
            engine,
            chain_ids,
            inputs,
        }
    }
}

impl Benchmarkable for ChainExecutionBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Execute a chain
        let chain_id = &self.chain_ids[0];
        let input = &self.inputs[0];

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let _ = self.engine.execute_chain(chain_id, input.clone()).await?;
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "chain_execution".to_string(),
            description: "Benchmark for executing chains in the chain engine".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "executions".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

fn bench_chain_engine(c: &mut Criterion) {
    // Chain creation benchmark
    let creation_benchmark = Arc::new(ChainCreationBenchmark::new(10, 5));
    run_benchmark(c, creation_benchmark);

    // Chain validation benchmark
    let validation_benchmark = Arc::new(ChainValidationBenchmark::new(10, 5));
    run_benchmark(c, validation_benchmark);

    // Chain execution benchmark
    let engine = Arc::new(ChainEngine::new());
    let execution_benchmark = Arc::new(ChainExecutionBenchmark::new(engine, 10));
    run_benchmark(c, execution_benchmark);
}

criterion_group!(benches, bench_chain_engine);
criterion_main!(benches);
