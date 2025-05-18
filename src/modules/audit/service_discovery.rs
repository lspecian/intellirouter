//! Service Discovery Validation
//!
//! This module is responsible for validating that all services can discover each other
//! and that the service registry functionality is working correctly.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::communication_tests;
use super::report::AuditReport;
use super::types::{
    AuditError, CommunicationTestResult, DiscoveryConfig, ServiceInfo, ServiceStatus, ServiceType,
};

/// Service Discovery Validator
#[derive(Debug, Clone)]
pub struct ServiceDiscovery {
    /// Discovery configuration
    config: DiscoveryConfig,
    /// HTTP client for service checks
    client: Client,
    /// Service information
    services: HashMap<ServiceType, ServiceInfo>,
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
}

impl ServiceDiscovery {
    /// Create a new service discovery validator
    pub fn new(config: DiscoveryConfig, report: Arc<RwLock<AuditReport>>) -> Self {
        // Initialize service information
        let mut services = HashMap::new();

        // Router service
        services.insert(
            ServiceType::Router,
            ServiceInfo::new(ServiceType::Router, "router", 8080),
        );

        // Chain Engine service
        services.insert(
            ServiceType::ChainEngine,
            ServiceInfo::new(ServiceType::ChainEngine, "orchestrator", 8080),
        );

        // RAG Manager service
        services.insert(
            ServiceType::RagManager,
            ServiceInfo::new(ServiceType::RagManager, "rag-injector", 8080),
        );

        // Persona Layer service
        services.insert(
            ServiceType::PersonaLayer,
            ServiceInfo::new(ServiceType::PersonaLayer, "summarizer", 8080),
        );

        // Redis service
        services.insert(
            ServiceType::Redis,
            ServiceInfo::new(ServiceType::Redis, "redis", 6379),
        );

        // ChromaDB service
        services.insert(
            ServiceType::ChromaDb,
            ServiceInfo::new(ServiceType::ChromaDb, "chromadb", 8000),
        );

        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            services,
            report,
        }
    }

    /// Validate service discovery
    pub async fn validate_service_discovery(&self) -> Result<(), AuditError> {
        info!("Starting service discovery validation");

        let start_time = Instant::now();
        let discovery_timeout = Duration::from_secs(self.config.discovery_timeout_secs);

        // First, check that all services are running
        for (service_type, service) in &self.services {
            match self.check_service_health(service).await {
                Ok(true) => {
                    info!("Service {} is healthy", service_type);

                    // Update report with service health
                    let mut report = self.report.write().await;
                    report.add_success(format!("Service {} is healthy", service_type));
                }
                Ok(false) => {
                    let error_msg = format!("Service {} is not healthy", service_type);
                    warn!("{}", error_msg);

                    // Update report with service health warning
                    let mut report = self.report.write().await;
                    report.add_warning(format!("Service discovery warning: {}", error_msg));
                }
                Err(e) => {
                    let error_msg =
                        format!("Failed to check health of service {}: {}", service_type, e);
                    error!("{}", error_msg);

                    // Update report with service health error
                    let mut report = self.report.write().await;
                    report.add_error(format!("Service discovery error: {}", error_msg));

                    return Err(AuditError::ServiceDiscoveryError(error_msg));
                }
            }
        }

        // Check service registry functionality
        // For each service, check that it can discover its dependencies
        for (service_type, service) in &self.services {
            for dependency in &service.dependencies {
                if let Some(dep_service) = self.services.get(dependency) {
                    match self.check_service_can_discover(service, dep_service).await {
                        Ok(true) => {
                            info!("Service {} can discover {}", service_type, dependency);

                            // Update report with discovery success
                            let mut report = self.report.write().await;
                            report.add_success(format!(
                                "Service {} can discover {}",
                                service_type, dependency
                            ));
                        }
                        Ok(false) => {
                            let error_msg =
                                format!("Service {} cannot discover {}", service_type, dependency);
                            warn!("{}", error_msg);

                            // Update report with discovery warning
                            let mut report = self.report.write().await;
                            report.add_warning(format!("Service discovery warning: {}", error_msg));

                            if self.config.validate_all_connections {
                                return Err(AuditError::ServiceDiscoveryError(error_msg));
                            }
                        }
                        Err(e) => {
                            let error_msg = format!(
                                "Failed to check if service {} can discover {}: {}",
                                service_type, dependency, e
                            );
                            error!("{}", error_msg);

                            // Update report with discovery error
                            let mut report = self.report.write().await;
                            report.add_error(format!("Service discovery error: {}", error_msg));

                            if self.config.validate_all_connections {
                                return Err(AuditError::ServiceDiscoveryError(error_msg));
                            }
                        }
                    }
                }
            }
        }

        // If validate_all_connections is true, check that all services can discover all other services
        if self.config.validate_all_connections {
            let service_types: Vec<ServiceType> = self.services.keys().cloned().collect();

            for &source in &service_types {
                for &target in &service_types {
                    if source != target {
                        let source_service = self.services.get(&source).unwrap();
                        let target_service = self.services.get(&target).unwrap();

                        match self
                            .check_service_can_discover(source_service, target_service)
                            .await
                        {
                            Ok(true) => {
                                info!("Service {} can discover {}", source, target);

                                // Update report with discovery success
                                let mut report = self.report.write().await;
                                report.add_success(format!(
                                    "Service {} can discover {}",
                                    source, target
                                ));
                            }
                            Ok(false) => {
                                let error_msg =
                                    format!("Service {} cannot discover {}", source, target);
                                warn!("{}", error_msg);

                                // Update report with discovery warning
                                let mut report = self.report.write().await;
                                report.add_warning(format!(
                                    "Service discovery warning: {}",
                                    error_msg
                                ));
                            }
                            Err(e) => {
                                let error_msg = format!(
                                    "Failed to check if service {} can discover {}: {}",
                                    source, target, e
                                );
                                error!("{}", error_msg);

                                // Update report with discovery error
                                let mut report = self.report.write().await;
                                report.add_error(format!("Service discovery error: {}", error_msg));
                            }
                        }
                    }
                }
            }
        }

        info!("Service discovery validation completed successfully");

        // Update report with discovery success
        let mut report = self.report.write().await;
        report.add_success("Service discovery validation completed successfully");

        Ok(())
    }

    /// Validate communication between services
    pub async fn validate_communication(&self) -> Result<(), AuditError> {
        info!("Starting communication validation");

        let mut test_results = Vec::new();

        // Test gRPC communication between services
        info!("Testing gRPC communication");
        let grpc_results =
            communication_tests::test_grpc_communication(&self.client, &self.report).await?;
        test_results.extend(grpc_results);

        // Test Redis pub/sub functionality
        info!("Testing Redis pub/sub functionality");
        let redis_results = communication_tests::test_redis_pubsub(&self.report).await?;
        test_results.extend(redis_results);

        // Test bidirectional communication
        info!("Testing bidirectional communication");
        let bidirectional_results =
            communication_tests::test_bidirectional_communication(&self.client, &self.report)
                .await?;
        test_results.extend(bidirectional_results);

        // Update report with communication test results
        let mut report = self.report.write().await;
        for result in &test_results {
            if result.success {
                report.add_success(format!(
                    "Communication test from {} to {} successful",
                    result.source, result.target
                ));
            } else if let Some(error) = &result.error {
                report.add_error(format!(
                    "Communication test from {} to {} failed: {}",
                    result.source, result.target, error
                ));
            }
        }

        // Check if all tests were successful
        let all_successful = test_results.iter().all(|r| r.success);

        if all_successful {
            info!("All communication tests passed");
            report.add_success("All communication tests passed");
            Ok(())
        } else {
            let error_msg = "Some communication tests failed";
            error!("{}", error_msg);
            report.add_error(format!("Communication test error: {}", error_msg));
            Err(AuditError::CommunicationTestError(error_msg.to_string()))
        }
    }

    /// Check if a service is healthy
    async fn check_service_health(&self, service: &ServiceInfo) -> Result<bool, AuditError> {
        // For Redis, use a different check since it doesn't have HTTP endpoints
        if service.service_type == ServiceType::Redis {
            return self.check_redis_health(service).await;
        }

        // For ChromaDB, use its specific health endpoint
        if service.service_type == ServiceType::ChromaDb {
            return self.check_chromadb_health(service).await;
        }

        // For other services, use the health endpoint
        let response = self
            .client
            .get(&service.health_endpoint)
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

        // Check if the service reports itself as healthy
        if let Some(status) = body.get("status").and_then(|s| s.as_str()) {
            Ok(status == "healthy" || status == "degraded")
        } else {
            Ok(false)
        }
    }

    /// Check if Redis is healthy
    async fn check_redis_health(&self, service: &ServiceInfo) -> Result<bool, AuditError> {
        // Use redis crate to check if Redis is healthy
        let redis_url = format!("redis://{}:{}", service.host, service.port);
        let client = redis::Client::open(redis_url).map_err(|e| {
            AuditError::ServiceDiscoveryError(format!("Failed to connect to Redis: {}", e))
        })?;

        let mut conn = client.get_async_connection().await.map_err(|e| {
            AuditError::ServiceDiscoveryError(format!("Failed to connect to Redis: {}", e))
        })?;

        // Try to ping Redis
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                AuditError::ServiceDiscoveryError(format!("Failed to ping Redis: {}", e))
            })?;

        Ok(pong == "PONG")
    }

    /// Check if ChromaDB is healthy
    async fn check_chromadb_health(&self, service: &ServiceInfo) -> Result<bool, AuditError> {
        // ChromaDB has a specific health endpoint
        let health_url = format!("http://{}:{}/api/v1/heartbeat", service.host, service.port);

        let response = self
            .client
            .get(&health_url)
            .send()
            .await
            .map_err(|e| AuditError::HttpError(e))?;

        Ok(response.status().is_success())
    }

    /// Check if a service can discover another service
    async fn check_service_can_discover(
        &self,
        source: &ServiceInfo,
        target: &ServiceInfo,
    ) -> Result<bool, AuditError> {
        // For Redis and ChromaDB, we can't check if they can discover other services
        if source.service_type == ServiceType::Redis || source.service_type == ServiceType::ChromaDb
        {
            return Ok(true);
        }

        // For other services, check if they can reach the target's health endpoint
        // We do this by calling the source service's diagnostics endpoint and checking
        // if it reports the target service as a dependency
        let response = self
            .client
            .get(&source.diagnostics_endpoint)
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
                    if name.contains(&target.service_type.to_string()) {
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
                if key.contains(&target.service_type.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_discovery_creation() {
        let config = DiscoveryConfig::default();
        let report = Arc::new(RwLock::new(AuditReport::new()));
        let discovery = ServiceDiscovery::new(config, report);

        assert_eq!(discovery.services.len(), 6);
        assert!(discovery.services.contains_key(&ServiceType::Router));
        assert!(discovery.services.contains_key(&ServiceType::ChainEngine));
        assert!(discovery.services.contains_key(&ServiceType::RagManager));
        assert!(discovery.services.contains_key(&ServiceType::PersonaLayer));
        assert!(discovery.services.contains_key(&ServiceType::Redis));
        assert!(discovery.services.contains_key(&ServiceType::ChromaDb));
    }
}
