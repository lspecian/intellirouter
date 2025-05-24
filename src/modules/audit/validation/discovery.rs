//! Service Discovery Validation
//!
//! This module provides functionality for validating service discovery.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::time::timeout;
use tracing::{error, info};

use crate::modules::audit::service_discovery::{ServiceDiscovery, ServiceStatus};
// Import ServiceType from types for other uses
use crate::modules::audit::types::ServiceType;

use super::types::{ValidationResult, ValidationType};

/// Validate service discovery
pub async fn validate_service_discovery(
    service_discovery: &ServiceDiscovery,
    timeout_secs: u64,
) -> ValidationResult {
    info!("Validating service discovery");
    let start_time = Instant::now();
    let validation_type = ValidationType::ServiceDiscovery;

    let mut details = HashMap::new();

    // Run service discovery with timeout
    match timeout(
        Duration::from_secs(timeout_secs),
        service_discovery.discover_services(),
    )
    .await
    {
        Ok(discovery_result) => match discovery_result {
            Ok(services) => {
                // Check if any services were discovered
                if services.is_empty() {
                    let error_msg = "No services discovered".to_string();
                    error!("{}", error_msg);

                    details.insert("error".to_string(), json!(error_msg));
                    details.insert("services_found".to_string(), json!(0));

                    ValidationResult::failure(
                        validation_type,
                        error_msg,
                        start_time.elapsed().as_millis() as u64,
                        details,
                    )
                } else {
                    // Count services by status
                    let mut active_count = 0;
                    let mut inactive_count = 0;
                    let mut degraded_count = 0;

                    for service in &services {
                        match service.status {
                            ServiceStatus::Active => active_count += 1,
                            ServiceStatus::Inactive => inactive_count += 1,
                            ServiceStatus::Unknown => inactive_count += 1, // Count Unknown as Inactive
                        }
                    }

                    // Add service details to result
                    details.insert("services_found".to_string(), json!(services.len()));
                    details.insert("active_services".to_string(), json!(active_count));
                    details.insert("inactive_services".to_string(), json!(inactive_count));
                    details.insert("degraded_services".to_string(), json!(degraded_count));

                    // Add service list to details
                    let service_list = services
                        .iter()
                        .map(|s| {
                            json!({
                                "type": format!("{:?}", s.service_type),
                                "host": s.host,
                                "port": s.port,
                                "status": format!("{:?}", s.status),
                            })
                        })
                        .collect::<Vec<_>>();

                    details.insert("services".to_string(), json!(service_list));

                    info!(
                        "Service discovery validation successful: found {} services ({} active, {} inactive, {} degraded)",
                        services.len(),
                        active_count,
                        inactive_count,
                        degraded_count
                    );

                    ValidationResult::success(
                        validation_type,
                        start_time.elapsed().as_millis() as u64,
                        details,
                    )
                }
            }
            Err(e) => {
                let error_msg = format!("Service discovery failed: {}", e);
                error!("{}", error_msg);

                details.insert("error".to_string(), json!(error_msg));

                ValidationResult::failure(
                    validation_type,
                    error_msg,
                    start_time.elapsed().as_millis() as u64,
                    details,
                )
            }
        },
        Err(_) => {
            let error_msg = format!("Service discovery timed out after {} seconds", timeout_secs);
            error!("{}", error_msg);

            details.insert("error".to_string(), json!(error_msg));
            details.insert("timeout_secs".to_string(), json!(timeout_secs));

            ValidationResult::failure(
                validation_type,
                error_msg,
                start_time.elapsed().as_millis() as u64,
                details,
            )
        }
    }
}
