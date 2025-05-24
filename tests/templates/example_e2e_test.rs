//! Example end-to-end test demonstrating best practices
//!
//! This file shows how to write end-to-end tests following IntelliRouter's test-first approach.

use intellirouter_test_utils::helpers::{
    spawn_test_environment, wait_for_condition, TestClient, TestEnvironment,
};
use serde_json::json;
use std::time::Duration;

/// Test the complete chat completions workflow
#[tokio::test]
async fn test_chat_completions_workflow() {
    // Arrange
    let env = spawn_test_environment()
        .await
        .expect("Failed to spawn test environment");

    let client = TestClient::new(&env);

    // Wait for all services to be ready
    wait_for_condition(
        || async {
            let router_health = client.get("/health").send().await;
            let orchestrator_health = client.get_orchestrator("/health").send().await;
            let rag_health = client.get_rag("/health").send().await;

            router_health.is_ok() && orchestrator_health.is_ok() && rag_health.is_ok()
        },
        Duration::from_secs(10),
    )
    .await
    .expect("Services did not become ready");

    // Act
    let response = client
        .post("/v1/chat/completions")
        .json(&json!({
            "model": "test-model",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "Hello, how are you?"}
            ],
            "temperature": 0.7,
            "max_tokens": 100
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert_eq!(response.status(), 200);

    let json_response = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    assert!(json_response.get("choices").is_some());
    assert!(json_response["choices"].is_array());
    assert!(json_response["choices"].as_array().unwrap().len() > 0);
    assert!(json_response["choices"][0].get("message").is_some());
    assert!(json_response["choices"][0]["message"]
        .get("content")
        .is_some());
    assert!(json_response["choices"][0]["message"]["content"].is_string());
}

/// Test the RAG workflow
#[tokio::test]
async fn test_rag_workflow() {
    // Arrange
    let env = spawn_test_environment()
        .await
        .expect("Failed to spawn test environment");

    let client = TestClient::new(&env);

    // Wait for all services to be ready
    wait_for_condition(
        || async {
            let router_health = client.get("/health").send().await;
            let orchestrator_health = client.get_orchestrator("/health").send().await;
            let rag_health = client.get_rag("/health").send().await;

            router_health.is_ok() && orchestrator_health.is_ok() && rag_health.is_ok()
        },
        Duration::from_secs(10),
    )
    .await
    .expect("Services did not become ready");

    // Upload a document to the RAG system
    let upload_response = client
        .post_rag("/documents")
        .body("This is a test document about IntelliRouter.")
        .header("Content-Type", "text/plain")
        .send()
        .await
        .expect("Failed to upload document");

    assert_eq!(upload_response.status(), 200);

    let document_id = upload_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response")["document_id"]
        .as_str()
        .expect("Failed to get document ID")
        .to_string();

    // Act - Send a query that should use the uploaded document
    let response = client
        .post("/v1/chat/completions")
        .json(&json!({
            "model": "test-model",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is IntelliRouter?"}
            ],
            "temperature": 0.7,
            "max_tokens": 100,
            "rag_enabled": true
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert_eq!(response.status(), 200);

    let json_response = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    assert!(json_response.get("choices").is_some());
    assert!(json_response["choices"].is_array());
    assert!(json_response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .contains("IntelliRouter"));

    // Clean up
    let delete_response = client
        .delete_rag(&format!("/documents/{}", document_id))
        .send()
        .await
        .expect("Failed to delete document");

    assert_eq!(delete_response.status(), 200);
}

/// Test the chain execution workflow
#[tokio::test]
async fn test_chain_execution_workflow() {
    // Arrange
    let env = spawn_test_environment()
        .await
        .expect("Failed to spawn test environment");

    let client = TestClient::new(&env);

    // Wait for all services to be ready
    wait_for_condition(
        || async {
            let router_health = client.get("/health").send().await;
            let orchestrator_health = client.get_orchestrator("/health").send().await;

            router_health.is_ok() && orchestrator_health.is_ok()
        },
        Duration::from_secs(10),
    )
    .await
    .expect("Services did not become ready");

    // Create a chain
    let create_chain_response = client
        .post_orchestrator("/chains")
        .json(&json!({
            "name": "test-chain",
            "steps": [
                {
                    "name": "step1",
                    "model": "test-model",
                    "prompt": "Summarize the following: {{input}}"
                },
                {
                    "name": "step2",
                    "model": "test-model",
                    "prompt": "Translate the following to French: {{step1.output}}"
                }
            ]
        }))
        .send()
        .await
        .expect("Failed to create chain");

    assert_eq!(create_chain_response.status(), 200);

    // Act - Execute the chain
    let response = client
        .post("/v1/chains/execute")
        .json(&json!({
            "chain_name": "test-chain",
            "input": "IntelliRouter is a system for routing requests to different language models."
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert_eq!(response.status(), 200);

    let json_response = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    assert!(json_response.get("result").is_some());
    assert!(json_response["result"].is_string());
    assert!(json_response["steps"].is_array());
    assert_eq!(json_response["steps"].as_array().unwrap().len(), 2);

    // Clean up
    let delete_response = client
        .delete_orchestrator("/chains/test-chain")
        .send()
        .await
        .expect("Failed to delete chain");

    assert_eq!(delete_response.status(), 200);
}

/// Test the performance of the system
#[tokio::test]
#[ignore] // This test is ignored by default because it's slow
async fn test_system_performance() {
    // Arrange
    let env = spawn_test_environment()
        .await
        .expect("Failed to spawn test environment");

    let client = TestClient::new(&env);

    // Wait for all services to be ready
    wait_for_condition(
        || async {
            let router_health = client.get("/health").send().await;
            router_health.is_ok()
        },
        Duration::from_secs(10),
    )
    .await
    .expect("Services did not become ready");

    const NUM_REQUESTS: usize = 10;
    let mut response_times = Vec::with_capacity(NUM_REQUESTS);

    // Act - Send multiple requests and measure response times
    for i in 0..NUM_REQUESTS {
        let start_time = std::time::Instant::now();

        let response = client
            .post("/v1/chat/completions")
            .json(&json!({
                "model": "test-model",
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant."},
                    {"role": "user", "content": format!("Request {}", i)}
                ],
                "temperature": 0.7,
                "max_tokens": 100
            }))
            .send()
            .await
            .expect("Failed to send request");

        let elapsed = start_time.elapsed();
        response_times.push(elapsed);

        assert_eq!(response.status(), 200);
    }

    // Assert
    let total_time: Duration = response_times.iter().sum();
    let average_time = total_time / NUM_REQUESTS as u32;

    println!("Average response time: {:?}", average_time);

    // Check that the average response time is below the threshold
    assert!(
        average_time < Duration::from_millis(500),
        "Average response time ({:?}) exceeded threshold (500ms)",
        average_time
    );
}

/// Test the system's error handling
#[tokio::test]
async fn test_error_handling() {
    // Arrange
    let env = spawn_test_environment()
        .await
        .expect("Failed to spawn test environment");

    let client = TestClient::new(&env);

    // Wait for all services to be ready
    wait_for_condition(
        || async {
            let router_health = client.get("/health").send().await;
            router_health.is_ok()
        },
        Duration::from_secs(10),
    )
    .await
    .expect("Services did not become ready");

    // Test cases with expected error responses
    let test_cases = vec![
        // Missing model
        (
            json!({
                "messages": [{"role": "user", "content": "Hello"}],
                "temperature": 0.7,
                "max_tokens": 100
            }),
            400,
            "model is required",
        ),
        // Invalid model
        (
            json!({
                "model": "non-existent-model",
                "messages": [{"role": "user", "content": "Hello"}],
                "temperature": 0.7,
                "max_tokens": 100
            }),
            404,
            "model not found",
        ),
        // Missing messages
        (
            json!({
                "model": "test-model",
                "temperature": 0.7,
                "max_tokens": 100
            }),
            400,
            "messages is required",
        ),
        // Invalid temperature
        (
            json!({
                "model": "test-model",
                "messages": [{"role": "user", "content": "Hello"}],
                "temperature": 2.0,
                "max_tokens": 100
            }),
            400,
            "temperature must be between 0 and 1",
        ),
    ];

    // Act and Assert
    for (request_body, expected_status, expected_error) in test_cases {
        let response = client
            .post("/v1/chat/completions")
            .json(&request_body)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            expected_status,
            "Expected status {} for request {:?}, got {}",
            expected_status,
            request_body,
            response.status()
        );

        let json_response = response
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse JSON response");

        assert!(
            json_response.get("error").is_some(),
            "Expected error field in response for request {:?}",
            request_body
        );

        let error_message = json_response["error"]["message"].as_str().unwrap_or("");
        assert!(
            error_message.contains(expected_error),
            "Expected error message to contain '{}' for request {:?}, got '{}'",
            expected_error,
            request_body,
            error_message
        );
    }
}

/// Test the system's resilience to failures
#[tokio::test]
async fn test_system_resilience() {
    // Arrange
    let env = spawn_test_environment()
        .await
        .expect("Failed to spawn test environment");

    let client = TestClient::new(&env);

    // Wait for all services to be ready
    wait_for_condition(
        || async {
            let router_health = client.get("/health").send().await;
            let orchestrator_health = client.get_orchestrator("/health").send().await;
            let rag_health = client.get_rag("/health").send().await;

            router_health.is_ok() && orchestrator_health.is_ok() && rag_health.is_ok()
        },
        Duration::from_secs(10),
    )
    .await
    .expect("Services did not become ready");

    // Stop the RAG service to simulate a failure
    env.stop_service("rag-manager")
        .await
        .expect("Failed to stop RAG service");

    // Act - Send a request that would normally use RAG
    let response = client
        .post("/v1/chat/completions")
        .json(&json!({
            "model": "test-model",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is IntelliRouter?"}
            ],
            "temperature": 0.7,
            "max_tokens": 100,
            "rag_enabled": true
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Assert - The system should still work, but without RAG
    assert_eq!(response.status(), 200);

    let json_response = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    assert!(json_response.get("choices").is_some());
    assert!(json_response["choices"].is_array());

    // There should be a warning about RAG being unavailable
    assert!(json_response.get("warnings").is_some());
    assert!(json_response["warnings"].is_array());
    assert!(json_response["warnings"][0]
        .as_str()
        .unwrap()
        .contains("RAG"));

    // Restart the RAG service
    env.start_service("rag-manager")
        .await
        .expect("Failed to restart RAG service");
}
