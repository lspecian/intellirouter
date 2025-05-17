//! Performance benchmarks for the router core module

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intellirouter::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatMessage, MessageRole},
    ModelFilter, ModelMetadata, ModelRegistry, ModelStatus, ModelType,
};
use intellirouter::modules::router_core::{
    RouterConfig, RouterImpl, RoutingRequest, RoutingStrategy, StrategyConfig,
};
use std::sync::Arc;
use std::time::Duration;

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
fn create_test_request() -> RoutingRequest {
    let chat_request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello, world!".to_string(),
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

fn bench_router_creation(c: &mut Criterion) {
    let registry = create_large_test_registry(100);

    c.bench_function("router_creation", |b| {
        b.iter(|| {
            let config = RouterConfig::default();
            let _router = RouterImpl::new(config, registry.clone()).unwrap();
        })
    });
}

fn bench_update_from_registry(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    c.bench_function("update_from_registry", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = router.update_from_registry().await.unwrap();
            });
        })
    });
}

fn bench_get_filtered_models(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();
    let request = create_test_request();

    c.bench_function("get_filtered_models", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = router.get_filtered_models(&request).await.unwrap();
            });
        })
    });
}

fn bench_get_filtered_models_with_filter(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    let mut request = create_test_request();
    request.model_filter = Some(ModelFilter::new().with_provider("provider1".to_string()));

    c.bench_function("get_filtered_models_with_filter", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = router.get_filtered_models(&request).await.unwrap();
            });
        })
    });
}

fn bench_get_filtered_models_with_complex_filter(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    let mut request = create_test_request();
    request.model_filter = Some(
        ModelFilter::new()
            .with_provider("provider1".to_string())
            .with_function_calling(true)
            .with_min_context_length(8192),
    );

    c.bench_function("get_filtered_models_with_complex_filter", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = router.get_filtered_models(&request).await.unwrap();
            });
        })
    });
}

fn bench_round_robin_strategy(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let mut config = RouterConfig::default();
    config.strategy = RoutingStrategy::RoundRobin;

    let router = RouterImpl::new(config, registry.clone()).unwrap();
    let request = create_test_request();

    // Initialize the router with registry data
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        router.update_from_registry().await.unwrap();
    });

    c.bench_function("round_robin_strategy", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // We can't actually route the request because we don't have real connectors
                // But we can get filtered models and simulate the strategy selection
                let models = router.get_filtered_models(&request).await.unwrap();
                assert!(!models.is_empty());
            });
        })
    });
}

fn bench_content_based_strategy(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let mut config = RouterConfig::default();
    config.strategy = RoutingStrategy::ContentBased;

    let router = RouterImpl::new(config, registry.clone()).unwrap();
    let request = create_test_request();

    // Initialize the router with registry data
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        router.update_from_registry().await.unwrap();
    });

    c.bench_function("content_based_strategy", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // We can't actually route the request because we don't have real connectors
                // But we can get filtered models and simulate the strategy selection
                let models = router.get_filtered_models(&request).await.unwrap();
                assert!(!models.is_empty());
            });
        })
    });
}

fn bench_router_with_excluded_models(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    let mut request = create_test_request();

    // Exclude half of the models
    for i in 0..50 {
        request.excluded_model_ids.push(format!("model{}", i));
    }

    c.bench_function("router_with_excluded_models", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let models = router.get_filtered_models(&request).await.unwrap();
                assert_eq!(models.len(), 50);
            });
        })
    });
}

fn bench_router_with_preferred_model(c: &mut Criterion) {
    let registry = create_large_test_registry(100);
    let config = RouterConfig::default();
    let router = RouterImpl::new(config, registry.clone()).unwrap();

    let mut request = create_test_request();
    request.preferred_model_id = Some("model42".to_string());

    c.bench_function("router_with_preferred_model", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let models = router.get_filtered_models(&request).await.unwrap();
                assert_eq!(models.len(), 1);
                assert_eq!(models[0].id, "model42");
            });
        })
    });
}

criterion_group!(
    benches,
    bench_router_creation,
    bench_update_from_registry,
    bench_get_filtered_models,
    bench_get_filtered_models_with_filter,
    bench_get_filtered_models_with_complex_filter,
    bench_round_robin_strategy,
    bench_content_based_strategy,
    bench_router_with_excluded_models,
    bench_router_with_preferred_model
);
criterion_main!(benches);
