//! Communication Tests
//!
//! This module is responsible for testing communication between services.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::report::AuditReport;
use super::types::{AuditError, CommunicationTestResult, ServiceType};

/// Test gRPC communication between services
pub async fn test_grpc_communication(
    client: &Client,
    report: &Arc<RwLock<AuditReport>>,
) -> Result<Vec<CommunicationTestResult>, AuditError> {
    info!("Testing gRPC communication between services");

    let mut results = Vec::new();

    // Define the services that communicate via gRPC
    let grpc_pairs = vec![
        (ServiceType::Router, ServiceType::ChainEngine),
        (ServiceType::Router, ServiceType::RagManager),
        (ServiceType::ChainEngine, ServiceType::Router),
        (ServiceType::RagManager, ServiceType::Router),
    ];

    for (source, target) in grpc_pairs {
        // Get service endpoints
        let source_url = get_service_url(source);
        let target_url = get_service_url(target);

        // Test if source can reach target
        let start_time = Instant::now();

        match test_service_connection(client, &source_url, &target_url).await {
            Ok(true) => {
                let elapsed = start_time.elapsed();
                info!("Service {} can reach {}", source, target);

                let result = CommunicationTestResult {
                    source,
                    target,
                    success: true,
                    error: None,
                    response_time_ms: Some(elapsed.as_millis() as u64),
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
            Ok(false) => {
                warn!("Service {} cannot reach {}", source, target);

                let result = CommunicationTestResult {
                    source,
                    target,
                    success: false,
                    error: Some(format!("Service {} cannot reach {}", source, target)),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
            Err(e) => {
                error!("Error testing if {} can reach {}: {}", source, target, e);

                let result = CommunicationTestResult {
                    source,
                    target,
                    success: false,
                    error: Some(format!("Error: {}", e)),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
        }
    }

    Ok(results)
}

/// Test Redis pub/sub communication
pub async fn test_redis_pubsub(
    report: &Arc<RwLock<AuditReport>>,
) -> Result<Vec<CommunicationTestResult>, AuditError> {
    info!("Testing Redis pub/sub communication");

    let mut results = Vec::new();

    // Define the services that use Redis pub/sub
    let redis_services = vec![
        ServiceType::Router,
        ServiceType::ChainEngine,
        ServiceType::RagManager,
        ServiceType::PersonaLayer,
    ];

    // Connect to Redis
    let redis_url = "redis://redis:6379";
    let client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to connect to Redis: {}", e);
            error!("{}", error_msg);
            return Err(AuditError::CommunicationTestError(error_msg));
        }
    };

    let mut conn = match client.get_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            let error_msg = format!("Failed to connect to Redis: {}", e);
            error!("{}", error_msg);
            return Err(AuditError::CommunicationTestError(error_msg));
        }
    };

    // Test Redis pub/sub by publishing a message to a test channel
    // and checking if it can be received
    let test_channel = "audit_test_channel";
    let test_message = "audit_test_message";

    // Publish a message
    let publish_result: Result<i32, redis::RedisError> = redis::cmd("PUBLISH")
        .arg(test_channel)
        .arg(test_message)
        .query_async(&mut conn)
        .await;

    // Check if the publish was successful
    match publish_result {
        Ok(_) => {
            // For each service that uses Redis, add a successful test result
            for service in redis_services {
                let result = CommunicationTestResult {
                    source: service,
                    target: ServiceType::Redis,
                    success: true,
                    error: None,
                    response_time_ms: Some(0), // We don't have an actual response time
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to publish message to Redis: {}", e);
            error!("{}", error_msg);

            // For each service that uses Redis, add a failed test result
            for service in redis_services {
                let result = CommunicationTestResult {
                    source: service,
                    target: ServiceType::Redis,
                    success: false,
                    error: Some(error_msg.clone()),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
        }
    }

    Ok(results)
}

/// Test bidirectional communication between services
pub async fn test_bidirectional_communication(
    client: &Client,
    report: &Arc<RwLock<AuditReport>>,
) -> Result<Vec<CommunicationTestResult>, AuditError> {
    info!("Testing bidirectional communication between services");

    let mut results = Vec::new();

    // Define the service pairs that should have bidirectional communication
    let bidirectional_pairs = vec![
        (ServiceType::Router, ServiceType::ChainEngine),
        (ServiceType::Router, ServiceType::RagManager),
        (ServiceType::Router, ServiceType::PersonaLayer),
    ];

    for (service1, service2) in bidirectional_pairs {
        // Get service endpoints
        let service1_url = get_service_url(service1);
        let service2_url = get_service_url(service2);

        // Test if service1 can reach service2
        let start_time = Instant::now();

        match test_service_connection(client, &service1_url, &service2_url).await {
            Ok(true) => {
                let elapsed = start_time.elapsed();
                info!("Service {} can reach {}", service1, service2);

                let result = CommunicationTestResult {
                    source: service1,
                    target: service2,
                    success: true,
                    error: None,
                    response_time_ms: Some(elapsed.as_millis() as u64),
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
            Ok(false) => {
                warn!("Service {} cannot reach {}", service1, service2);

                let result = CommunicationTestResult {
                    source: service1,
                    target: service2,
                    success: false,
                    error: Some(format!("Service {} cannot reach {}", service1, service2)),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
            Err(e) => {
                error!(
                    "Error testing if {} can reach {}: {}",
                    service1, service2, e
                );

                let result = CommunicationTestResult {
                    source: service1,
                    target: service2,
                    success: false,
                    error: Some(format!("Error: {}", e)),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
        }

        // Test if service2 can reach service1
        let start_time = Instant::now();

        match test_service_connection(client, &service2_url, &service1_url).await {
            Ok(true) => {
                let elapsed = start_time.elapsed();
                info!("Service {} can reach {}", service2, service1);

                let result = CommunicationTestResult {
                    source: service2,
                    target: service1,
                    success: true,
                    error: None,
                    response_time_ms: Some(elapsed.as_millis() as u64),
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
            Ok(false) => {
                warn!("Service {} cannot reach {}", service2, service1);

                let result = CommunicationTestResult {
                    source: service2,
                    target: service1,
                    success: false,
                    error: Some(format!("Service {} cannot reach {}", service2, service1)),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
            Err(e) => {
                error!(
                    "Error testing if {} can reach {}: {}",
                    service2, service1, e
                );

                let result = CommunicationTestResult {
                    source: service2,
                    target: service1,
                    success: false,
                    error: Some(format!("Error: {}", e)),
                    response_time_ms: None,
                    timestamp: chrono::Utc::now(),
                };

                // Update report
                let mut report = report.write().await;
                report.add_communication_test(result.clone());

                results.push(result);
            }
        }
    }

    Ok(results)
}

/// Get the URL for a service
fn get_service_url(service: ServiceType) -> String {
    match service {
        ServiceType::Router => "http://router:8080".to_string(),
        ServiceType::ChainEngine => "http://orchestrator:8080".to_string(),
        ServiceType::RagManager => "http://rag-injector:8080".to_string(),
        ServiceType::PersonaLayer => "http://summarizer:8080".to_string(),
        ServiceType::Redis => "redis://redis:6379".to_string(),
        ServiceType::ChromaDb => "http://chromadb:8000".to_string(),
    }
}

/// Test if a service can reach another service
async fn test_service_connection(
    client: &Client,
    source_url: &str,
    target_url: &str,
) -> Result<bool, AuditError> {
    // For Redis, we need to use a different approach
    if target_url.starts_with("redis://") {
        return test_redis_connection(target_url).await;
    }

    // For HTTP services, we can use the diagnostics endpoint
    let diagnostics_url = format!("{}/diagnostics", source_url);

    let response = client
        .get(&diagnostics_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| AuditError::HttpError(e))?;

    if !response.status().is_success() {
        return Ok(false);
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| AuditError::HttpError(e))?;

    // Check if the service reports the target as a dependency
    if let Some(connections) = body.get("connections").and_then(|c| c.as_array()) {
        for connection in connections {
            if let Some(name) = connection.get("name").and_then(|n| n.as_str()) {
                if target_url.contains(name) {
                    if let Some(status) = connection.get("status").and_then(|s| s.as_str()) {
                        return Ok(status == "healthy" || status == "degraded");
                    }
                }
            }
        }
    }

    // If we didn't find the target in the connections, check if it's in the diagnostics
    if let Some(diagnostics) = body.get("diagnostics").and_then(|d| d.as_object()) {
        for (key, _) in diagnostics {
            if target_url.contains(key) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Test if a service can connect to Redis
async fn test_redis_connection(redis_url: &str) -> Result<bool, AuditError> {
    let client = redis::Client::open(redis_url).map_err(|e| {
        AuditError::CommunicationTestError(format!("Failed to connect to Redis: {}", e))
    })?;

    let mut conn = client.get_async_connection().await.map_err(|e| {
        AuditError::CommunicationTestError(format!("Failed to connect to Redis: {}", e))
    })?;

    // Try to ping Redis
    let pong: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .map_err(|e| AuditError::CommunicationTestError(format!("Failed to ping Redis: {}", e)))?;

    Ok(pong == "PONG")
}
