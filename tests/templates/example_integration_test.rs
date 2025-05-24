//! Example integration test demonstrating best practices
//!
//! This file shows how to write integration tests following IntelliRouter's test-first approach.

use intellirouter::modules::model_registry::ModelRegistry;
use intellirouter::modules::orchestrator::Orchestrator;
use intellirouter::modules::rag_manager::RagManager;
use intellirouter::modules::router_core::Router;
use intellirouter_test_utils::fixtures::{create_test_model, create_test_request};
use intellirouter_test_utils::helpers::{spawn_test_server, wait_for_condition};
use std::time::Duration;

/// Test the integration between Router and ModelRegistry
#[test]
fn test_router_model_registry_integration() {
    // Arrange
    let mut registry = ModelRegistry::new();
    registry
        .register_model("test-model", create_test_model())
        .expect("Failed to register model");

    let router = Router::new(registry);
    let request = create_test_request("Test content for test-model");

    // Act
    let result = router.route(request);

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.model_id(), "test-model");
}

/// Test the integration between Router and Orchestrator
#[test]
fn test_router_orchestrator_integration() {
    // Arrange
    let registry = ModelRegistry::new();
    let orchestrator = Orchestrator::new();
    let router = Router::with_orchestrator(registry, orchestrator);

    let request = create_test_request("Test content for chain execution");

    // Act
    let result = router.route_with_chain(request, "test-chain");

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.content().contains("chain execution"));
}

/// Test the integration between Router, Orchestrator, and RagManager
#[test]
fn test_router_orchestrator_rag_integration() {
    // Arrange
    let registry = ModelRegistry::new();
    let orchestrator = Orchestrator::new();
    let rag_manager = RagManager::new();

    let router = Router::with_orchestrator_and_rag(registry, orchestrator, rag_manager);

    let request = create_test_request("Test content with RAG");

    // Act
    let result = router.route_with_rag(request);

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.content().contains("augmented with RAG"));
}

/// Test the HTTP API integration
#[tokio::test]
async fn test_http_api_integration() {
    // Arrange
    let server = spawn_test_server()
        .await
        .expect("Failed to spawn test server");
    let client = reqwest::Client::new();

    // Wait for server to be ready
    wait_for_condition(
        || async {
            let response = client.get(&format!("{}/health", server.url())).send().await;
            response.is_ok() && response.unwrap().status() == 200
        },
        Duration::from_secs(5),
    )
    .await
    .expect("Server did not become ready");

    // Act
    let response = client
        .post(&format!("{}/v1/chat/completions", server.url()))
        .json(&serde_json::json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}]
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert_eq!(response.status(), 200);
    let json = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON");
    assert!(json.get("choices").is_some());
}

/// Test error propagation between components
#[test]
fn test_error_propagation() {
    // Arrange
    let mut registry = ModelRegistry::new();
    registry
        .register_model("error-model", create_test_model())
        .expect("Failed to register model");

    let router = Router::new(registry);
    let request = create_test_request("ERROR");

    // Act
    let result = router.route(request);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Routing failed: ERROR content detected");
}

/// Test with a real database connection
#[tokio::test]
async fn test_with_database_integration() {
    // Arrange
    let db_config = intellirouter_test_utils::fixtures::create_test_db_config();
    let db_pool = sqlx::PgPool::connect(&db_config.connection_string)
        .await
        .expect("Failed to connect to database");

    let registry = ModelRegistry::with_database(db_pool.clone());
    let router = Router::new(registry);

    // Create a test model in the database
    sqlx::query("INSERT INTO models (id, name, provider) VALUES ($1, $2, $3)")
        .bind("test-db-model")
        .bind("Test DB Model")
        .bind("test-provider")
        .execute(&db_pool)
        .await
        .expect("Failed to insert test model");

    let request = create_test_request("Test content for database model");

    // Act
    let result = router.route(request);

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.model_id(), "test-db-model");

    // Clean up
    sqlx::query("DELETE FROM models WHERE id = $1")
        .bind("test-db-model")
        .execute(&db_pool)
        .await
        .expect("Failed to delete test model");
}

/// Test with Redis for caching
#[tokio::test]
async fn test_with_redis_integration() {
    // Arrange
    let redis_config = intellirouter_test_utils::fixtures::create_test_redis_config();
    let redis_client =
        redis::Client::open(redis_config.connection_string).expect("Failed to create Redis client");
    let mut redis_conn = redis_client
        .get_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Set up a cached model
    redis::cmd("SET")
        .arg("model:cached-model")
        .arg(serde_json::to_string(&create_test_model()).unwrap())
        .query_async::<_, ()>(&mut redis_conn)
        .await
        .expect("Failed to set cached model");

    let registry = ModelRegistry::with_redis(redis_client);
    let router = Router::new(registry);

    let request = create_test_request("Test content for cached model");

    // Act
    let result = router.route(request);

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.model_id(), "cached-model");

    // Clean up
    redis::cmd("DEL")
        .arg("model:cached-model")
        .query_async::<_, ()>(&mut redis_conn)
        .await
        .expect("Failed to delete cached model");
}

/// Test with Docker services
#[tokio::test]
#[ignore] // This test requires Docker to be running
async fn test_with_docker_services() {
    // Arrange
    let docker_config = intellirouter_test_utils::fixtures::create_test_docker_config();
    let docker_services = intellirouter_test_utils::helpers::DockerServices::new(docker_config)
        .await
        .expect("Failed to create Docker services");

    // Start Redis service
    docker_services
        .start_service("redis")
        .await
        .expect("Failed to start Redis service");

    // Get Redis connection string
    let redis_url = docker_services
        .get_connection_string("redis")
        .await
        .expect("Failed to get Redis connection string");

    // Create Redis client
    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");

    // Create registry with Redis
    let registry = ModelRegistry::with_redis(redis_client);
    let router = Router::new(registry);

    let request = create_test_request("Test content for Docker test");

    // Act
    let result = router.route(request);

    // Assert
    assert!(result.is_ok());

    // Clean up
    docker_services
        .stop_service("redis")
        .await
        .expect("Failed to stop Redis service");
}
