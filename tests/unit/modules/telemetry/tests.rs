#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cost_calculator() {
        let calculator = crate::modules::telemetry::cost::CostCalculator::new();

        // Test GPT-4 cost calculation
        let cost = calculator.calculate_cost("gpt-4", 1000, 500).unwrap();
        let expected_cost = (0.03 * 1000.0 / 1000.0) + (0.06 * 500.0 / 1000.0);
        assert_eq!(cost, expected_cost);

        // Test GPT-3.5 Turbo cost calculation
        let cost = calculator
            .calculate_cost("gpt-3.5-turbo", 1000, 500)
            .unwrap();
        let expected_cost = (0.0015 * 1000.0 / 1000.0) + (0.002 * 500.0 / 1000.0);
        assert_eq!(cost, expected_cost);

        // Test unknown model (should use default costs)
        let cost = calculator
            .calculate_cost("unknown-model", 1000, 500)
            .unwrap();
        let expected_cost = (0.001 * 1000.0 / 1000.0) + (0.002 * 500.0 / 1000.0);
        assert_eq!(cost, expected_cost);

        // Test setting custom costs
        calculator
            .set_model_cost("custom-model", 0.005, 0.01)
            .unwrap();
        let cost = calculator
            .calculate_cost("custom-model", 1000, 500)
            .unwrap();
        let expected_cost = (0.005 * 1000.0 / 1000.0) + (0.01 * 500.0 / 1000.0);
        assert_eq!(cost, expected_cost);
    }

    #[tokio::test]
    async fn test_telemetry_manager() {
        let telemetry = crate::modules::telemetry::telemetry::TelemetryManager::new(
            "test-service".to_string(),
            "test".to_string(),
            "0.1.0".to_string(),
        );

        // Test LLM call metrics
        let metrics = crate::modules::telemetry::telemetry::LlmCallMetrics {
            model_id: "gpt-4".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            latency_ms: 1000,
            estimated_cost: 0.01,
            success: true,
            error_message: None,
        };

        // This should not panic
        telemetry.record_llm_call(metrics);

        // Test routing metrics
        let metrics = crate::modules::telemetry::telemetry::RoutingMetrics {
            request_id: "test-request".to_string(),
            selected_model: "gpt-4".to_string(),
            candidate_count: 3,
            decision_time_ms: 5,
            success: true,
            error_message: None,
        };

        // This should not panic
        telemetry.record_routing_decision(metrics);

        // Test request metrics
        let timer = telemetry.start_request_timer();
        tokio::time::sleep(Duration::from_millis(10)).await;

        // This should not panic
        telemetry.record_request_metrics("/test", "GET", 200, timer);
    }
}
