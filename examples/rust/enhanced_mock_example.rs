//! Enhanced Mock Example
//!
//! This example demonstrates how to use the enhanced mocking framework
//! with HTTP interaction recording/replaying, intelligent stub generation,
//! configurable response behaviors, and network condition simulation.

use std::sync::Arc;

use intellirouter::modules::test_harness::mock::{
    create_enhanced_http_mock, create_mock_manager, EnhancedMockConfig, HttpRequest, HttpResponse,
    InteractionRecorderConfig, LatencyConfig, LatencyDistribution, NetworkConditionType,
    NetworkSimulatorConfig, RecordingMode, StubGenerationConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enhanced Mock Example");
    println!("=====================");

    // Create a mock manager
    let manager = Arc::new(create_mock_manager());

    // Create an enhanced HTTP mock with custom configuration
    let mock = create_enhanced_http_mock("api-mock", manager.clone())
        .with_description("API Mock with enhanced capabilities")
        .with_recorder_config(InteractionRecorderConfig {
            mode: RecordingMode::Auto,
            storage_dir: "interactions".into(),
            simulate_latency: true,
            default_latency_ms: 50,
            latency_multiplier: 1.0,
            ..Default::default()
        })
        .with_network_simulator_config(NetworkSimulatorConfig {
            enabled: true,
            condition_type: NetworkConditionType::Good,
            ..Default::default()
        })
        .build()
        .await?;

    println!("Created enhanced HTTP mock: {}", mock.mock().name());

    // Create a request
    let request = HttpRequest::new("GET", "/api/users")
        .with_header("Content-Type", "application/json")
        .with_header("Accept", "application/json");

    println!("\nSending request to /api/users...");

    // Handle the request
    let response = mock.handle_request(&request).await?;

    println!("Received response: Status {}", response.status);
    if let Some(body) = &response.body {
        println!("Response body: {}", serde_json::to_string_pretty(body)?);
    }

    // Generate stubs from recorded interactions
    println!("\nGenerating stubs from recorded interactions...");
    let stubs = mock.generate_stubs().await;
    println!("Generated {} stubs", stubs.len());

    // Save recorded interactions to a file
    println!("\nSaving recorded interactions...");
    mock.save_interactions("interactions/api-mock.json").await?;
    println!("Saved interactions to interactions/api-mock.json");

    // Change network conditions
    println!("\nChanging network conditions to simulate poor network...");
    mock.network_simulator()
        .set_condition_type(NetworkConditionType::Poor)
        .await;

    // Create another request
    let request2 = HttpRequest::new("GET", "/api/products")
        .with_header("Content-Type", "application/json")
        .with_header("Accept", "application/json");

    println!("Sending request to /api/products with poor network conditions...");

    // Handle the request with poor network conditions
    let start = std::time::Instant::now();
    let response2 = mock.handle_request(&request2).await?;
    let elapsed = start.elapsed();

    println!(
        "Received response after {:?}: Status {}",
        elapsed, response2.status
    );
    if let Some(body) = &response2.body {
        println!("Response body: {}", serde_json::to_string_pretty(body)?);
    }

    // Demonstrate recording and replaying
    println!("\nDemonstrating recording and replaying...");

    // Switch to record mode
    mock.recorder().set_mode(RecordingMode::Record).await;
    println!("Switched to RECORD mode");

    // Create a request that will be recorded
    let request3 = HttpRequest::new("POST", "/api/orders")
        .with_header("Content-Type", "application/json")
        .with_header("Accept", "application/json")
        .with_body(serde_json::json!({
            "product_id": "123",
            "quantity": 2,
            "customer_id": "456"
        }))?;

    println!("Sending POST request to /api/orders...");
    let response3 = mock.handle_request(&request3).await?;
    println!("Recorded response: Status {}", response3.status);

    // Switch to replay mode
    mock.recorder().set_mode(RecordingMode::Replay).await;
    println!("Switched to REPLAY mode");

    // Send the same request again
    println!("Sending the same POST request to /api/orders...");
    let response4 = mock.handle_request(&request3).await?;
    println!("Replayed response: Status {}", response4.status);

    // Check that the responses are the same
    if response3.status == response4.status {
        println!("Success! The recorded and replayed responses have the same status code.");
    } else {
        println!("Error! The recorded and replayed responses have different status codes.");
    }

    println!("\nEnhanced mock example completed successfully!");

    Ok(())
}
