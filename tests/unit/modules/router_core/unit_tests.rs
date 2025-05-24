//! Unit tests for the Router Core module

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::test_utils::{self, mocks::MockRouter};
    use mockall::predicate::*;
    use test_case::test_case;

    #[test]
    fn test_routing_strategy_enum() {
        // Test that all enum variants are accessible
        let strategies = vec![
            RoutingStrategy::RoundRobin,
            RoutingStrategy::LoadBalanced,
            RoutingStrategy::ContentBased,
            RoutingStrategy::CostOptimized,
            RoutingStrategy::LatencyOptimized,
            RoutingStrategy::Custom,
        ];

        assert_eq!(strategies.len(), 6);
    }

    #[test]
    fn test_router_config() {
        let config = RouterConfig {
            strategy: RoutingStrategy::ContentBased,
            strategy_config: Some(StrategyConfig::default()),
            fallback_strategies: vec![RoutingStrategy::RoundRobin],
            global_timeout_ms: 10000,
            max_routing_attempts: 3,
            cache_routing_decisions: true,
            max_cache_size: 1000,
            collect_metrics: true,
            additional_config: HashMap::new(),
        };

        assert!(matches!(config.strategy, RoutingStrategy::ContentBased));
        assert!(config.strategy_config.is_some());
        assert_eq!(config.fallback_strategies.len(), 1);
        assert_eq!(config.global_timeout_ms, 10000);
    }

    #[test]
    fn test_init_success() {
        let config = RouterConfig::default();
        let result = init(config);
        assert!(result.is_ok());
    }

    #[test_case("simple request" => Ok(String::from("response")) ; "simple request routes successfully")]
    #[test_case("error request" => Err(String::from("error")) ; "error request returns error")]
    fn test_route_request(request: &str) -> Result<String, String> {
        // This is a placeholder test until actual routing logic is implemented
        if request.contains("error") {
            Err("error".to_string())
        } else {
            Ok("response".to_string())
        }
    }

    #[test]
    fn test_with_mock_router() {
        let mut mock_router = MockRouter::new();

        // Set up expectations
        mock_router
            .expect_route()
            .with(eq("test request"))
            .times(1)
            .returning(|_| Ok("mocked response".to_string()));

        // Use the mock
        let result = mock_router.route("test request");
        assert_eq!(result, Ok("mocked response".to_string()));
    }

    #[test]
    fn test_routing_context() {
        use crate::modules::model_registry::ChatCompletionRequest;

        let request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: Vec::new(),
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        let context = RoutingContext::new(request.clone())
            .with_user_id("test-user")
            .with_org_id("test-org")
            .with_priority(5)
            .with_tag("test-tag")
            .with_parameter("param1", "value1");

        assert_eq!(context.user_id, Some("test-user".to_string()));
        assert_eq!(context.org_id, Some("test-org".to_string()));
        assert_eq!(context.priority, 5);
        assert_eq!(context.tags, vec!["test-tag".to_string()]);
        assert_eq!(
            context.parameters.get("param1"),
            Some(&"value1".to_string())
        );
    }

    #[test]
    fn test_routing_request() {
        use crate::modules::model_registry::{ChatCompletionRequest, ModelFilter};
        use std::time::Duration;

        let request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: Vec::new(),
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        let filter = ModelFilter::new();
        let routing_request = RoutingRequest::new(request.clone())
            .with_model_filter(filter)
            .with_preferred_model("preferred-model")
            .exclude_model("excluded-model")
            .with_max_attempts(5)
            .with_timeout(Duration::from_secs(60));

        assert!(routing_request.model_filter.is_some());
        assert_eq!(
            routing_request.preferred_model_id,
            Some("preferred-model".to_string())
        );
        assert_eq!(
            routing_request.excluded_model_ids,
            vec!["excluded-model".to_string()]
        );
        assert_eq!(routing_request.max_attempts, 5);
        assert_eq!(routing_request.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_strategy_configs() {
        let round_robin_config = RoundRobinConfig::default();
        assert!(!round_robin_config.weighted);

        let load_balanced_config = LoadBalancedConfig::default();
        assert_eq!(load_balanced_config.capacity_weight, 0.4);
        assert_eq!(load_balanced_config.load_weight, 0.4);
        assert_eq!(load_balanced_config.performance_weight, 0.2);

        let content_based_config = ContentBasedConfig::default();
        assert!(content_based_config.analyze_content);
        assert!(content_based_config.match_capabilities);
        assert!(content_based_config.language_detection);

        let cost_optimized_config = CostOptimizedConfig::default();
        assert!(cost_optimized_config.estimate_token_count);
        assert_eq!(cost_optimized_config.performance_cost_balance, 0.3);

        let latency_optimized_config = LatencyOptimizedConfig::default();
        assert!(latency_optimized_config.use_historical_data);
        assert_eq!(latency_optimized_config.quality_latency_balance, 0.3);
    }

    // Property-based test using proptest
    #[cfg(feature = "proptest")]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn doesnt_crash_on_any_string_input(s in "\\PC*") {
                let _ = route_request(&s);
            }
        }
    }
}
