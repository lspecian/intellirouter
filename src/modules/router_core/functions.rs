//! Module Functions
//!
//! This module provides global functions for the router core.

use crate::modules::model_registry;
use crate::modules::router_core::config::RouterConfig;
use crate::modules::router_core::errors::RouterError;
use crate::modules::router_core::interface::Router;
use crate::modules::router_core::request::RoutingRequest;
use crate::modules::router_core::router::RouterImpl;

/// Initialize the router with the specified configuration
pub fn init(config: RouterConfig) -> Result<(), RouterError> {
    // Get the global registry
    let registry = model_registry::global_registry().registry();
    let mut router = RouterImpl::new(config.clone(), registry)?;
    router.init(config)
}

/// Route a request to the appropriate model or service
pub async fn route_request(request: &str) -> Result<String, RouterError> {
    // Parse the request
    let chat_request: model_registry::connectors::ChatCompletionRequest =
        serde_json::from_str(request)
            .map_err(|e| RouterError::InvalidRequest(format!("Invalid request: {}", e)))?;

    // Create a routing request
    let routing_request = RoutingRequest::new(chat_request);

    // Get the router
    let registry = model_registry::global_registry().registry();
    let router = RouterImpl::new(RouterConfig::default(), registry)?;

    // Route the request
    let response = router.route(routing_request).await?;

    // Serialize the response
    let response_str = serde_json::to_string(&response.response)
        .map_err(|e| RouterError::Other(format!("Failed to serialize response: {}", e)))?;

    Ok(response_str)
}
