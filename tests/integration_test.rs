// Integration Tests for IntelliRouter
//
// This file contains integration tests for the IntelliRouter application.
// These tests verify that the different components work together correctly.

use intellirouter::{
    config::Config,
    modules::{
        chain_engine, llm_proxy, model_registry, rag_manager,
        router_core::{self, RouterConfig, RoutingStrategy},
    },
    test_utils::{self, init_test_logging, TestConfig},
};
use std::path::PathBuf;
use tokio;

// Setup function to initialize test environment
fn setup() -> TestConfig {
    init_test_logging();
    TestConfig::new()
}

#[test]
fn test_config_loading() {
    let test_config = setup();

    // Create a test config file
    let config_path = test_config.path().join("test_config.toml");
    std::fs::write(
        &config_path,
        r#"
        [server]
        host = "127.0.0.1"
        port = 8080
        
        [router]
        strategy = "content_based"
        
        [models]
        default = "test-model"
    "#,
    )
    .unwrap();

    // TODO: Implement actual config loading test when Config::from_file is available
    let config = Config::new();
    assert!(true); // Placeholder assertion
}

#[test]
fn test_router_initialization() {
    let router_config = RouterConfig {
        strategy: RoutingStrategy::ContentBased,
    };

    let result = router_core::init(router_config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_end_to_end_request_flow() {
    // This test will verify the full request flow through the system
    // For now, it's a placeholder until more implementation is available

    // 1. Create a test request
    let request = test_utils::create_test_request("Test request content");

    // 2. Route the request (placeholder)
    let routing_result = router_core::route_request(&request);
    assert!(routing_result.is_ok());

    // 3. Assert the expected outcome
    assert!(true); // Placeholder assertion
}

// Property-based test for the integration
#[cfg(feature = "proptest")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_router_with_various_inputs(s in "\\PC*") {
            let result = router_core::route_request(&s);
            assert!(result.is_ok());
        }
    }
}

// Test fixtures for different module combinations
mod test_fixtures {
    use super::*;

    // Test fixture for router + model registry
    pub struct RouterWithRegistry {
        pub config: RouterConfig,
    }

    impl RouterWithRegistry {
        pub fn new() -> Self {
            Self {
                config: RouterConfig {
                    strategy: RoutingStrategy::ContentBased,
                },
            }
        }

        pub fn init(&self) -> Result<(), String> {
            router_core::init(self.config.clone())
        }
    }
}
