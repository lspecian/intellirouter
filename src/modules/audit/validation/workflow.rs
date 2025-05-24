//! Validation Workflow
//!
//! This module provides the main validation workflow implementation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::modules::audit::report::AuditReport;
use crate::modules::audit::service_discovery::ServiceDiscovery;
use crate::modules::audit::types::AuditError;
// Import ServiceInfo and ServiceType from types for other uses
use crate::modules::audit::types::{
    ServiceInfo as TypesServiceInfo, ServiceType as TypesServiceType,
};

// Import from communication module
use super::communication::{validate_direct_communication, ServiceInfo, ServiceType};
use super::config::ValidationConfig;
use super::data_integrity::validate_data_integrity;
use super::discovery::validate_service_discovery;
use super::error_handling::validate_error_handling;
use super::flows::validate_end_to_end_flows;
use super::reporting::generate_validation_report;
use super::security::validate_security;
use super::types::{TestFlow, ValidationResult};

/// Validation Workflow
#[derive(Debug)]
pub struct ValidationWorkflow {
    /// Validation configuration
    config: ValidationConfig,
    /// HTTP client for API requests
    _client: Client,
    /// Service discovery validator
    service_discovery: ServiceDiscovery,
    /// Shared audit report
    report: Arc<RwLock<AuditReport>>,
    /// Service information
    services: HashMap<TypesServiceType, TypesServiceInfo>,
}

impl ValidationWorkflow {
    /// Create a new validation workflow
    pub fn new(
        config: ValidationConfig,
        service_discovery: ServiceDiscovery,
        report: Arc<RwLock<AuditReport>>,
        services: HashMap<TypesServiceType, TypesServiceInfo>,
    ) -> Self {
        Self {
            config,
            _client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            service_discovery,
            report,
            services,
        }
    }

    /// Run the complete validation workflow
    pub async fn run_validation(&self) -> Result<Vec<ValidationResult>, AuditError> {
        info!("Starting validation workflow");

        let mut results = Vec::new();

        // Step 1: Service discovery validation
        if self.config.validate_service_discovery {
            let result = validate_service_discovery(
                &self.service_discovery,
                self.config.validation_timeout_secs,
            )
            .await;

            results.push(result.clone());

            // Check if we should fail fast
            if self.config.fail_fast && !result.success {
                error!("Service discovery validation failed, stopping workflow due to fail-fast configuration");
                return Ok(results);
            }
        }

        // Step 2: Direct communication validation
        if self.config.validate_direct_communication {
            // Convert services to the format expected by validate_direct_communication
            let converted_services: HashMap<ServiceType, ServiceInfo> = self
                .services
                .iter()
                .map(|(k, v)| {
                    (
                        match k {
                            TypesServiceType::Router => ServiceType::Router,
                            TypesServiceType::ChainEngine => ServiceType::ChainEngine,
                            TypesServiceType::RagManager => ServiceType::RagManager,
                            TypesServiceType::PersonaLayer => ServiceType::PersonaLayer,
                            TypesServiceType::Redis => ServiceType::Redis,
                            TypesServiceType::ChromaDb => ServiceType::ChromaDb,
                            TypesServiceType::ModelRegistry => ServiceType::ModelRegistry,
                            TypesServiceType::Memory => ServiceType::Memory,
                            TypesServiceType::Orchestrator => ServiceType::Orchestrator,
                        },
                        ServiceInfo {
                            service_type: match v.service_type {
                                TypesServiceType::Router => ServiceType::Router,
                                TypesServiceType::ChainEngine => ServiceType::ChainEngine,
                                TypesServiceType::RagManager => ServiceType::RagManager,
                                TypesServiceType::PersonaLayer => ServiceType::PersonaLayer,
                                TypesServiceType::Redis => ServiceType::Redis,
                                TypesServiceType::ChromaDb => ServiceType::ChromaDb,
                                TypesServiceType::ModelRegistry => ServiceType::ModelRegistry,
                                TypesServiceType::Memory => ServiceType::Memory,
                                TypesServiceType::Orchestrator => ServiceType::Orchestrator,
                            },
                            host: v.host.clone(),
                            port: v.port,
                        },
                    )
                })
                .collect();

            let result = validate_direct_communication(
                &converted_services,
                self.config.validation_timeout_secs,
            )
            .await;

            results.push(result.clone());

            // Check if we should fail fast
            if self.config.fail_fast && !result.success {
                error!("Direct communication validation failed, stopping workflow due to fail-fast configuration");
                return Ok(results);
            }
        }

        // Step 3: End-to-end flow validation
        if self.config.validate_end_to_end_flows {
            // Get test flows
            let flows = self.get_test_flows().await?;

            let result = validate_end_to_end_flows(
                &self.services,
                &flows,
                self.config.validation_timeout_secs,
            )
            .await;

            results.push(result.clone());

            // Check if we should fail fast
            if self.config.fail_fast && !result.success {
                error!("End-to-end flow validation failed, stopping workflow due to fail-fast configuration");
                return Ok(results);
            }
        }

        // Step 4: Data integrity validation
        if self.config.validate_data_integrity {
            let result =
                validate_data_integrity(&self.services, self.config.validation_timeout_secs).await;

            results.push(result.clone());

            // Check if we should fail fast
            if self.config.fail_fast && !result.success {
                error!("Data integrity validation failed, stopping workflow due to fail-fast configuration");
                return Ok(results);
            }
        }

        // Step 5: Error handling validation
        if self.config.validate_error_handling {
            let result =
                validate_error_handling(&self.services, self.config.validation_timeout_secs).await;

            results.push(result.clone());

            // Check if we should fail fast
            if self.config.fail_fast && !result.success {
                error!("Error handling validation failed, stopping workflow due to fail-fast configuration");
                return Ok(results);
            }
        }

        // Step 6: Security validation
        if self.config.validate_security {
            let result =
                validate_security(&self.services, self.config.validation_timeout_secs).await;

            results.push(result.clone());

            // Check if we should fail fast
            if self.config.fail_fast && !result.success {
                error!(
                    "Security validation failed, stopping workflow due to fail-fast configuration"
                );
                return Ok(results);
            }
        }

        // Generate validation report
        let mut report = self.report.write().await;
        generate_validation_report(&results, &mut report).await?;

        info!(
            "Validation workflow completed with {} results",
            results.len()
        );
        Ok(results)
    }

    /// Get test flows for end-to-end validation
    async fn get_test_flows(&self) -> Result<Vec<TestFlow>, AuditError> {
        // This is a placeholder implementation
        // In a real implementation, this would load test flows from configuration or database

        let mut flows = Vec::new();

        // Example flow 1: Model registration and retrieval
        let mut steps1 = Vec::new();
        steps1.push(super::types::Step::Step {
            name: "register_model".to_string(),
            service_type: TypesServiceType::ModelRegistry,
            endpoint: "/models".to_string(),
            method: "POST".to_string(),
            payload: Some(serde_json::json!({
                "name": "test-model",
                "version": "1.0.0",
                "provider": "openai",
                "capabilities": ["text-generation", "embeddings"]
            })),
            expected_status: 201,
        });
        steps1.push(super::types::Step::Step {
            name: "get_model".to_string(),
            service_type: TypesServiceType::ModelRegistry,
            endpoint: "/models/test-model".to_string(),
            method: "GET".to_string(),
            payload: None,
            expected_status: 200,
        });

        flows.push(TestFlow {
            name: "model_registration_flow".to_string(),
            description: "Test model registration and retrieval".to_string(),
            steps: steps1,
        });

        // Example flow 2: Chain execution
        let mut steps2 = Vec::new();
        steps2.push(super::types::Step::Step {
            name: "register_chain".to_string(),
            service_type: TypesServiceType::ChainEngine,
            endpoint: "/chains".to_string(),
            method: "POST".to_string(),
            payload: Some(serde_json::json!({
                "name": "test-chain",
                "steps": [
                    {
                        "id": "step1",
                        "type": "llm",
                        "model": "test-model",
                        "prompt": "Hello, world!"
                    }
                ]
            })),
            expected_status: 201,
        });
        steps2.push(super::types::Step::Step {
            name: "execute_chain".to_string(),
            service_type: TypesServiceType::ChainEngine,
            endpoint: "/chains/test-chain/execute".to_string(),
            method: "POST".to_string(),
            payload: Some(serde_json::json!({
                "inputs": {}
            })),
            expected_status: 200,
        });

        flows.push(TestFlow {
            name: "chain_execution_flow".to_string(),
            description: "Test chain registration and execution".to_string(),
            steps: steps2,
        });

        Ok(flows)
    }
}
