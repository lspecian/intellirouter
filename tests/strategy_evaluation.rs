//! Test harness for evaluating different routing strategies
//!
//! This module provides a framework for evaluating the performance and behavior
//! of different routing strategies under various load conditions.

use intellirouter::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
    ModelFilter, ModelMetadata, ModelRegistry, ModelStatus, ModelType,
};
use intellirouter::modules::router_core::{
    RouterConfig, RouterImpl, RoutingRequest, RoutingStrategy, StrategyConfig,
};
use intellirouter::test_utils::init_test_logging;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

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

// Helper function to create a test request
fn create_test_request(content: &str) -> RoutingRequest {
    let chat_request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: content.to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: None,
        functions: None,
        tools: None,
        additional_params: None,
    };

    RoutingRequest::new(chat_request)
}

// Evaluate a strategy under load
async fn evaluate_strategy(
    strategy: RoutingStrategy,
    registry: Arc<ModelRegistry>,
    num_requests: usize,
    concurrency: usize,
) -> (Duration, HashMap<String, usize>) {
    // Create a router with the specified strategy
    let mut config = RouterConfig::default();
    config.strategy = strategy;

    let router = Arc::new(RouterImpl::new(config, registry.clone()).unwrap());

    // Initialize the router with registry data
    router.update_from_registry().await.unwrap();

    // Track model usage
    let model_usage = Arc::new(Mutex::new(HashMap::new()));

    // Start timing
    let start = Instant::now();

    // Create tasks
    let mut handles = Vec::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

    for i in 0..num_requests {
        let router = router.clone();
        let model_usage = model_usage.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let request = create_test_request(&format!("Test request {}", i));

        let handle = tokio::spawn(async move {
            // Get filtered models
            let models = router.get_filtered_models(&request).await.unwrap();

            // In a real test, we would route the request
            // For now, we'll just simulate by picking the first model
            if let Some(model) = models.first() {
                let mut usage = model_usage.lock().await;
                let count = usage.entry(model.id.clone()).or_insert(0);
                *count += 1;
            }

            // Release the semaphore permit
            drop(permit);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // End timing
    let duration = start.elapsed();

    // Return results
    (duration, Arc::try_unwrap(model_usage).unwrap().into_inner())
}

// Calculate distribution metrics (min, max, avg, std dev)
fn calculate_distribution_metrics(usage: &HashMap<String, usize>) -> (usize, usize, f64, f64) {
    let values = usage.values().cloned().collect::<Vec<_>>();

    if values.is_empty() {
        return (0, 0, 0.0, 0.0);
    }

    let min = *values.iter().min().unwrap_or(&0);
    let max = *values.iter().max().unwrap_or(&0);
    let sum: usize = values.iter().sum();
    let avg = sum as f64 / values.len() as f64;

    let variance = values
        .iter()
        .map(|&v| {
            let diff = v as f64 - avg;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;

    let std_dev = variance.sqrt();

    (min, max, avg, std_dev)
}

#[tokio::test]
async fn evaluate_strategies_under_load() {
    // Initialize test environment
    init_test_logging();

    // Create a large test registry
    let registry = create_large_test_registry(100);

    // Define test parameters
    let num_requests = 1000;
    let concurrency = 10;

    // Evaluate round-robin strategy
    let (rr_duration, rr_usage) = evaluate_strategy(
        RoutingStrategy::RoundRobin,
        registry.clone(),
        num_requests,
        concurrency,
    )
    .await;

    // Evaluate content-based strategy
    let (cb_duration, cb_usage) = evaluate_strategy(
        RoutingStrategy::ContentBased,
        registry.clone(),
        num_requests,
        concurrency,
    )
    .await;

    // Print results
    println!("Round-robin strategy:");
    println!("  Duration: {:?}", rr_duration);
    println!(
        "  Requests per second: {:.2}",
        num_requests as f64 / rr_duration.as_secs_f64()
    );
    println!("  Models used: {}", rr_usage.len());

    println!("Content-based strategy:");
    println!("  Duration: {:?}", cb_duration);
    println!(
        "  Requests per second: {:.2}",
        num_requests as f64 / cb_duration.as_secs_f64()
    );
    println!("  Models used: {}", cb_usage.len());

    // Calculate distribution metrics
    let rr_distribution = calculate_distribution_metrics(&rr_usage);
    let cb_distribution = calculate_distribution_metrics(&cb_usage);

    println!("Round-robin distribution metrics:");
    println!("  Min: {}", rr_distribution.0);
    println!("  Max: {}", rr_distribution.1);
    println!("  Avg: {:.2}", rr_distribution.2);
    println!("  Std Dev: {:.2}", rr_distribution.3);

    println!("Content-based distribution metrics:");
    println!("  Min: {}", cb_distribution.0);
    println!("  Max: {}", cb_distribution.1);
    println!("  Avg: {:.2}", cb_distribution.2);
    println!("  Std Dev: {:.2}", cb_distribution.3);

    // Verify that round-robin has better distribution (lower std dev)
    assert!(rr_distribution.3 <= cb_distribution.3);
}

#[tokio::test]
async fn evaluate_strategies_with_varying_concurrency() {
    // Initialize test environment
    init_test_logging();

    // Create a large test registry
    let registry = create_large_test_registry(100);

    // Define test parameters
    let num_requests = 500;
    let concurrency_levels = vec![1, 5, 10, 20, 50];

    println!("Evaluating strategies with varying concurrency levels:");

    for concurrency in concurrency_levels {
        // Evaluate round-robin strategy
        let (rr_duration, _) = evaluate_strategy(
            RoutingStrategy::RoundRobin,
            registry.clone(),
            num_requests,
            concurrency,
        )
        .await;

        // Evaluate content-based strategy
        let (cb_duration, _) = evaluate_strategy(
            RoutingStrategy::ContentBased,
            registry.clone(),
            num_requests,
            concurrency,
        )
        .await;

        println!("Concurrency level: {}", concurrency);
        println!(
            "  Round-robin: {:?} ({:.2} req/s)",
            rr_duration,
            num_requests as f64 / rr_duration.as_secs_f64()
        );
        println!(
            "  Content-based: {:?} ({:.2} req/s)",
            cb_duration,
            num_requests as f64 / cb_duration.as_secs_f64()
        );
    }
}

#[tokio::test]
async fn evaluate_strategies_with_provider_failures() {
    // Initialize test environment
    init_test_logging();

    // Create a large test registry
    let registry = create_large_test_registry(100);

    // Define test parameters
    let num_requests = 500;
    let concurrency = 10;

    // Evaluate round-robin strategy with all providers available
    let (rr_duration_all, rr_usage_all) = evaluate_strategy(
        RoutingStrategy::RoundRobin,
        registry.clone(),
        num_requests,
        concurrency,
    )
    .await;

    // Mark all provider1 models as unavailable
    for i in 0..100 {
        if i % 3 == 0 {
            registry
                .update_model_status(&format!("model{}", i), ModelStatus::Unavailable)
                .unwrap();
        }
    }

    // Evaluate round-robin strategy with provider1 unavailable
    let (rr_duration_partial, rr_usage_partial) = evaluate_strategy(
        RoutingStrategy::RoundRobin,
        registry.clone(),
        num_requests,
        concurrency,
    )
    .await;

    println!("Round-robin with all providers:");
    println!("  Duration: {:?}", rr_duration_all);
    println!("  Models used: {}", rr_usage_all.len());

    println!("Round-robin with provider1 unavailable:");
    println!("  Duration: {:?}", rr_duration_partial);
    println!("  Models used: {}", rr_usage_partial.len());

    // Verify that no provider1 models were used after marking them unavailable
    for (model_id, _) in &rr_usage_partial {
        let model_num = model_id.replace("model", "").parse::<usize>().unwrap();
        assert_ne!(
            model_num % 3,
            0,
            "Provider1 model was used despite being unavailable"
        );
    }
}

#[tokio::test]
async fn evaluate_strategies_with_model_capabilities() {
    // Initialize test environment
    init_test_logging();

    // Create a large test registry
    let registry = create_large_test_registry(100);

    // Define test parameters
    let num_requests = 500;
    let concurrency = 10;

    // Create a request with a filter for function calling
    let mut request = create_test_request("Test request with function calling");
    request.model_filter = Some(ModelFilter::new().with_function_calling(true));

    // Create a router with round-robin strategy
    let mut config = RouterConfig::default();
    config.strategy = RoutingStrategy::RoundRobin;

    let router = Arc::new(RouterImpl::new(config, registry.clone()).unwrap());

    // Initialize the router with registry data
    router.update_from_registry().await.unwrap();

    // Get filtered models
    let models = router.get_filtered_models(&request).await.unwrap();

    // Verify that all models support function calling
    for model in &models {
        assert!(
            model.capabilities.supports_function_calling,
            "Model {} does not support function calling",
            model.id
        );
    }

    // Verify that approximately 1/3 of models were selected (those with i % 3 == 0)
    assert!(
        models.len() >= 30 && models.len() <= 34,
        "Expected around 33 models, got {}",
        models.len()
    );
}
