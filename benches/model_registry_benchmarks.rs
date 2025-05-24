//! Performance benchmarks for the model registry module

use criterion::{criterion_group, criterion_main, Criterion};
use intellirouter::modules::model_registry::{
    ModelMetadata, ModelRegistry, ModelStatus, ModelType,
};
use std::sync::Arc;
use std::time::Duration;

mod framework;
use framework::{run_benchmark, BenchmarkConfig, BenchmarkType, Benchmarkable, ResourceUsage};

// Helper function to create a test registry with many models
fn create_large_test_registry(num_models: usize) -> Arc<ModelRegistry> {
    let registry = Arc::new(ModelRegistry::new());

    for i in 0..num_models {
        let provider = if i % 3 == 0 {
            "provider1"
        } else if i % 3 == 1 {
            "provider2"
        } else {
            "provider3"
        };

        let mut model = ModelMetadata::new(
            format!("model{}", i),
            format!("Test Model {}", i),
            provider.to_string(),
            "1.0".to_string(),
            "https://example.com".to_string(),
        );

        model.set_status(ModelStatus::Available);
        model.set_model_type(ModelType::TextGeneration);
        model.capabilities.max_context_length = 4096 * (i % 4 + 1);
        model.capabilities.supports_streaming = i % 2 == 0;
        model.capabilities.supports_function_calling = i % 3 == 0;

        registry.register_model(model).unwrap();
    }

    registry
}

/// Benchmark for model registration
struct ModelRegistrationBenchmark {
    registry: Arc<ModelRegistry>,
    models_to_register: Vec<ModelMetadata>,
}

impl ModelRegistrationBenchmark {
    fn new(num_models: usize) -> Self {
        let registry = Arc::new(ModelRegistry::new());
        let mut models_to_register = Vec::with_capacity(num_models);

        for i in 0..num_models {
            let provider = if i % 3 == 0 {
                "provider1"
            } else if i % 3 == 1 {
                "provider2"
            } else {
                "provider3"
            };

            let mut model = ModelMetadata::new(
                format!("model{}", i),
                format!("Test Model {}", i),
                provider.to_string(),
                "1.0".to_string(),
                "https://example.com".to_string(),
            );

            model.set_status(ModelStatus::Available);
            model.set_model_type(ModelType::TextGeneration);
            model.capabilities.max_context_length = 4096 * (i % 4 + 1);
            model.capabilities.supports_streaming = i % 2 == 0;
            model.capabilities.supports_function_calling = i % 3 == 0;

            models_to_register.push(model);
        }

        Self {
            registry,
            models_to_register,
        }
    }
}

impl Benchmarkable for ModelRegistrationBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Register all models
        for model in &self.models_to_register {
            self.registry.register_model(model.clone())?;
        }

        Ok(start.elapsed())
    }

    fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear the registry before each iteration
        self.registry.clear();
        Ok(())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "model_registration".to_string(),
            description: "Benchmark for registering models in the model registry".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "models".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for model lookup
struct ModelLookupBenchmark {
    registry: Arc<ModelRegistry>,
    model_ids: Vec<String>,
}

impl ModelLookupBenchmark {
    fn new(registry: Arc<ModelRegistry>, num_lookups: usize) -> Self {
        let model_ids: Vec<String> = (0..num_lookups)
            .map(|i| format!("model{}", i % 100)) // Assuming 100 models in registry
            .collect();

        Self {
            registry,
            model_ids,
        }
    }
}

impl Benchmarkable for ModelLookupBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Look up all models
        for model_id in &self.model_ids {
            let _ = self.registry.get_model(model_id)?;
        }

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "model_lookup".to_string(),
            description: "Benchmark for looking up models in the model registry".to_string(),
            benchmark_type: BenchmarkType::Throughput,
            unit: "lookups".to_string(),
            sample_size: Some(100),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for model filtering
struct ModelFilteringBenchmark {
    registry: Arc<ModelRegistry>,
}

impl ModelFilteringBenchmark {
    fn new(registry: Arc<ModelRegistry>) -> Self {
        Self { registry }
    }
}

impl Benchmarkable for ModelFilteringBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Filter models by provider
        let _ = self.registry.filter_models(|model| {
            model.provider == "provider1" && model.capabilities.supports_function_calling
        });

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "model_filtering".to_string(),
            description: "Benchmark for filtering models in the model registry".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "filters".to_string(),
            sample_size: Some(50),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

fn bench_model_registry(c: &mut Criterion) {
    // Model registration benchmark
    let registration_benchmark = Arc::new(ModelRegistrationBenchmark::new(100));
    run_benchmark(c, registration_benchmark);

    // Create a registry for lookup and filtering benchmarks
    let registry = create_large_test_registry(100);

    // Model lookup benchmark
    let lookup_benchmark = Arc::new(ModelLookupBenchmark::new(registry.clone(), 1000));
    run_benchmark(c, lookup_benchmark);

    // Model filtering benchmark
    let filtering_benchmark = Arc::new(ModelFilteringBenchmark::new(registry.clone()));
    run_benchmark(c, filtering_benchmark);
}

criterion_group!(benches, bench_model_registry);
criterion_main!(benches);
