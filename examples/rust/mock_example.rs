//! Mock Framework Example
//!
//! This example demonstrates how to use the IntelliRouter mock framework.

use intellirouter::modules::test_harness::{
    mock::{
        Behavior, BehaviorBuilder, HttpMock, HttpMockBuilder, HttpRequest, HttpResponse, HttpStub,
        Mock, MockFactory, MockManager, Response, ResponseBuilder,
    },
    types::TestHarnessError,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Mock Framework Example");
    println!("====================================");

    // Create a mock manager
    let manager = Arc::new(MockManager::new());
    println!("Created mock manager");

    // Create a mock factory
    let factory = MockFactory::new(Arc::clone(&manager));
    println!("Created mock factory");

    // Create an HTTP mock using the builder
    println!("\nCreating HTTP mock using builder...");
    let http_mock = factory
        .create_http_mock("api-mock")
        .with_description("API mock for testing")
        .with_server("localhost", 8080)
        .with_behavior(
            BehaviorBuilder::new("get-users")
                .with_description("GET /api/users behavior")
                .with_matcher(|interaction| {
                    let path = interaction
                        .request_field::<String>("path")
                        .unwrap_or_default();
                    let method = interaction
                        .request_field::<String>("method")
                        .unwrap_or_default();
                    path == "/api/users" && method.eq_ignore_ascii_case("GET")
                })
                .with_responder(|_| {
                    ResponseBuilder::new()
                        .with_data(serde_json::json!([
                            {"id": 1, "name": "Alice"},
                            {"id": 2, "name": "Bob"},
                            {"id": 3, "name": "Charlie"}
                        ]))
                        .unwrap()
                        .with_status(200)
                        .with_header("Content-Type", "application/json")
                        .build()
                })
                .build()
                .unwrap(),
        )
        .with_behavior(
            BehaviorBuilder::new("get-user")
                .with_description("GET /api/users/{id} behavior")
                .with_matcher(|interaction| {
                    let path = interaction
                        .request_field::<String>("path")
                        .unwrap_or_default();
                    let method = interaction
                        .request_field::<String>("method")
                        .unwrap_or_default();
                    path.starts_with("/api/users/") && method.eq_ignore_ascii_case("GET")
                })
                .with_responder(|interaction| {
                    let path = interaction
                        .request_field::<String>("path")
                        .unwrap_or_default();
                    let parts: Vec<&str> = path.split('/').collect();
                    let id = parts.last().unwrap_or(&"0").parse::<u64>().unwrap_or(0);

                    match id {
                        1 => ResponseBuilder::new()
                            .with_data(serde_json::json!({"id": 1, "name": "Alice"}))
                            .unwrap()
                            .with_status(200)
                            .with_header("Content-Type", "application/json")
                            .build(),
                        2 => ResponseBuilder::new()
                            .with_data(serde_json::json!({"id": 2, "name": "Bob"}))
                            .unwrap()
                            .with_status(200)
                            .with_header("Content-Type", "application/json")
                            .build(),
                        3 => ResponseBuilder::new()
                            .with_data(serde_json::json!({"id": 3, "name": "Charlie"}))
                            .unwrap()
                            .with_status(200)
                            .with_header("Content-Type", "application/json")
                            .build(),
                        _ => ResponseBuilder::not_found("User not found"),
                    }
                })
                .build()
                .unwrap(),
        )
        .with_stub(HttpStub::for_path_and_method("/api/health", "GET", |_| {
            HttpResponse::new(200)
                .with_body(serde_json::json!({"status": "healthy"}))
                .unwrap()
                .with_header("Content-Type", "application/json")
        }))
        .build()
        .await?;

    println!("Created HTTP mock: {}", http_mock.name());
    if let Some(url) = http_mock.url() {
        println!("Mock URL: {}", url);
    }

    // Set up the mock
    println!("\nSetting up mock...");
    http_mock.setup().await?;
    println!("Mock set up successfully");

    // Create some test requests
    println!("\nSending test requests...");

    // Request 1: GET /api/users
    let request1 = HttpRequest::new("GET", "/api/users").with_header("Accept", "application/json");

    println!("Request 1: GET /api/users");
    let response1 = http_mock.handle_request(&request1).await;
    println!("Response 1: Status {}", response1.status);
    println!(
        "Response 1 Body: {}",
        serde_json::to_string_pretty(&response1.body).unwrap()
    );

    // Request 2: GET /api/users/1
    let request2 =
        HttpRequest::new("GET", "/api/users/1").with_header("Accept", "application/json");

    println!("\nRequest 2: GET /api/users/1");
    let response2 = http_mock.handle_request(&request2).await;
    println!("Response 2: Status {}", response2.status);
    println!(
        "Response 2 Body: {}",
        serde_json::to_string_pretty(&response2.body).unwrap()
    );

    // Request 3: GET /api/users/999 (not found)
    let request3 =
        HttpRequest::new("GET", "/api/users/999").with_header("Accept", "application/json");

    println!("\nRequest 3: GET /api/users/999");
    let response3 = http_mock.handle_request(&request3).await;
    println!("Response 3: Status {}", response3.status);
    println!(
        "Response 3 Body: {}",
        serde_json::to_string_pretty(&response3.body).unwrap()
    );

    // Request 4: GET /api/health (stub)
    let request4 = HttpRequest::new("GET", "/api/health").with_header("Accept", "application/json");

    println!("\nRequest 4: GET /api/health");
    let response4 = http_mock.handle_request(&request4).await;
    println!("Response 4: Status {}", response4.status);
    println!(
        "Response 4 Body: {}",
        serde_json::to_string_pretty(&response4.body).unwrap()
    );

    // Request 5: GET /api/unknown (not found)
    let request5 =
        HttpRequest::new("GET", "/api/unknown").with_header("Accept", "application/json");

    println!("\nRequest 5: GET /api/unknown");
    let response5 = http_mock.handle_request(&request5).await;
    println!("Response 5: Status {}", response5.status);
    println!(
        "Response 5 Body: {}",
        serde_json::to_string_pretty(&response5.body).unwrap()
    );

    // Check the recorded interactions
    println!("\nRecorded interactions:");
    let interactions = http_mock.recorder().get_interactions().await;
    println!("Total interactions: {}", interactions.len());

    for (i, interaction) in interactions.iter().enumerate() {
        println!("Interaction {}: {}", i + 1, interaction.interaction.id);
        if let Some(behavior) = &interaction.behavior {
            println!("  Behavior: {}", behavior);
        } else {
            println!("  Behavior: None");
        }
        println!("  Verified: {}", interaction.verified);
    }

    // Verify the mock
    println!("\nVerifying mock...");
    match http_mock.verify().await {
        Ok(_) => println!("Mock verification successful"),
        Err(e) => println!("Mock verification failed: {}", e),
    }

    // Reset the mock
    println!("\nResetting mock...");
    http_mock.reset().await?;
    println!("Mock reset successful");

    // Check the interactions after reset
    let interactions = http_mock.recorder().get_interactions().await;
    println!("Interactions after reset: {}", interactions.len());

    // Tear down the mock
    println!("\nTearing down mock...");
    http_mock.teardown().await?;
    println!("Mock teardown successful");

    println!("\nMock framework example completed successfully!");
    Ok(())
}
