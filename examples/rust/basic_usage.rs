// Basic Usage Example for IntelliRouter
//
// This example demonstrates how to use the IntelliRouter library
// for basic LLM request routing.

use intellirouter::config::Config;
use intellirouter::modules::llm_proxy::{self, Provider};
use intellirouter::modules::router_core::{self, RouterConfig, RoutingStrategy};

fn main() {
    println!("IntelliRouter Basic Usage Example");

    // Load configuration
    let config = Config::new();

    // Initialize LLM Proxy (without starting the server)
    if let Err(e) = llm_proxy::init_without_server(Provider::OpenAI) {
        eprintln!("Failed to initialize LLM Proxy: {}", e);
        return;
    }

    // Initialize Router
    let router_config = RouterConfig {
        strategy: RoutingStrategy::ContentBased,
    };

    if let Err(e) = router_core::init(router_config) {
        eprintln!("Failed to initialize Router: {}", e);
        return;
    }

    // Example request
    let request = "What is the capital of France?";
    println!("Sending request: {}", request);

    // Route the request
    match router_core::route_request(request) {
        Ok(response) => {
            println!("Response: {}", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("Example completed");
}
