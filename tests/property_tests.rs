//! Property-based tests for IntelliRouter
//!
//! This file contains property-based tests that verify invariants and properties
//! of the IntelliRouter components using proptest and quickcheck.

#[cfg(test)]
mod tests {
    use intellirouter::{
        modules::{
            llm_proxy, model_registry,
            router_core::{self, RouterConfig, RoutingStrategy},
        },
        test_utils::{self, init_test_logging, TestConfig},
    };
    use proptest::prelude::*;
    use quickcheck::{quickcheck, TestResult};

    // Proptest: Router should handle any valid string input
    proptest! {
        #[test]
        fn router_handles_any_string(s in "\\PC*") {
            // Initialize router with content-based strategy
            let router_config = RouterConfig::default();
            let init_result = router_core::init(router_config);
            assert!(init_result.is_ok());

            // Route the request
            let routing_result = router_core::route_request(&s);

            // The router should not panic and should return a result
            assert!(routing_result.is_ok());
        }
    }

    // Proptest: Router should maintain consistent routing for identical inputs
    proptest! {
        #[test]
        fn router_is_deterministic(s in "\\PC*") {
            // Initialize router with content-based strategy
            let router_config = RouterConfig::default();
            let init_result = router_core::init(router_config);
            assert!(init_result.is_ok());

            // Route the same request twice
            let first_result = router_core::route_request(&s);
            let second_result = router_core::route_request(&s);

            // The results should be the same
            assert_eq!(first_result, second_result);
        }
    }

    // Quickcheck: Router should handle empty strings
    quickcheck! {
        fn router_handles_empty_string() -> TestResult {
            // Initialize router with content-based strategy
            let router_config = RouterConfig::default();
            let init_result = router_core::init(router_config);
            if init_result.is_err() {
                return TestResult::discard();
            }

            // Route an empty string
            let routing_result = router_core::route_request("");

            // The router should not panic and should return a result
            TestResult::from_bool(routing_result.is_ok())
        }
    }

    // Quickcheck: Router should handle very long strings
    quickcheck! {
        fn router_handles_long_strings(s: String) -> TestResult {
            // Skip empty strings
            if s.is_empty() {
                return TestResult::discard();
            }

            // Create a very long string by repeating the input
            let long_string = s.repeat(1000);

            // Initialize router with content-based strategy
            let router_config = RouterConfig::default();
            let init_result = router_core::init(router_config);
            if init_result.is_err() {
                return TestResult::discard();
            }

            // Route the long string
            let routing_result = router_core::route_request(&long_string);

            // The router should not panic and should return a result
            TestResult::from_bool(routing_result.is_ok())
        }
    }

    // Property: Different routing strategies should produce valid results
    proptest! {
        #[test]
        fn all_routing_strategies_work(s in "\\PC*") {
            let strategies = vec![
                RoutingStrategy::RoundRobin,
                RoutingStrategy::LoadBalanced,
                RoutingStrategy::ContentBased,
                RoutingStrategy::CostOptimized,
                RoutingStrategy::LatencyOptimized,
            ];

            for strategy in strategies {
                // Initialize router with the current strategy
                let mut router_config = RouterConfig::default();
                router_config.strategy = strategy;
                let init_result = router_core::init(router_config);
                assert!(init_result.is_ok());

                // Route the request
                let routing_result = router_core::route_request(&s);

                // The router should not panic and should return a result
                assert!(routing_result.is_ok());
            }
        }
    }
}
