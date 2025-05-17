//! Model Registry Health Check
//!
//! This module implements a health check mechanism for the Model Registry.
//! It provides functionality to check the health status of models and
//! periodically update their status based on health check results.

// We need rand for random number generation in the health check simulation
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, error, info, warn};

use super::api::ModelRegistryApi;
use super::types::{ModelMetadata, ModelStatus, RegistryError};

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks in seconds
    pub check_interval_seconds: u64,
    /// Timeout for health check requests in seconds
    pub request_timeout_seconds: u64,
    /// Maximum number of consecutive failures before marking a model as unavailable
    pub max_consecutive_failures: u32,
    /// Whether to automatically update model status based on health checks
    pub auto_update_status: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            request_timeout_seconds: 10,
            max_consecutive_failures: 3,
            auto_update_status: true,
        }
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Model ID
    pub model_id: String,
    /// Whether the health check was successful
    pub success: bool,
    /// Error message if the health check failed
    pub error_message: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Timestamp of the health check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check manager
#[derive(Debug)]
pub struct HealthCheckManager {
    /// Model registry API
    registry_api: Arc<ModelRegistryApi>,
    /// Health check configuration
    config: HealthCheckConfig,
    /// Health check task handle
    health_check_task: Option<JoinHandle<()>>,
    /// Failure counters for models
    failure_counters: Arc<dashmap::DashMap<String, u32>>,
}

impl HealthCheckManager {
    /// Create a new health check manager
    pub fn new(registry_api: Arc<ModelRegistryApi>, config: HealthCheckConfig) -> Self {
        Self {
            registry_api,
            config,
            health_check_task: None,
            failure_counters: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Start periodic health checks
    pub fn start_health_checks(&mut self) {
        if self.health_check_task.is_some() {
            warn!("Health check task already running");
            return;
        }

        let registry_api = self.registry_api.clone();
        let config = self.config.clone();
        let failure_counters = self.failure_counters.clone();

        self.health_check_task = Some(tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(config.check_interval_seconds));
            loop {
                interval.tick().await;
                debug!("Running periodic health check");

                // Get all models
                let models = registry_api.list_models();

                // Check each model
                for model in models {
                    match check_model_health(&model, config.request_timeout_seconds).await {
                        Ok(result) => {
                            debug!(
                                "Health check for model {} completed: success={}",
                                model.id, result.success
                            );

                            // Update failure counter
                            if !result.success {
                                let mut current_failures =
                                    failure_counters.entry(model.id.clone()).or_insert(0);
                                *current_failures += 1;

                                debug!(
                                    "Model {} consecutive failures: {}",
                                    model.id, *current_failures
                                );

                                // Check if max failures exceeded
                                if *current_failures >= config.max_consecutive_failures {
                                    if config.auto_update_status {
                                        if let Err(e) = registry_api.update_model_status(
                                            &model.id,
                                            ModelStatus::Unavailable,
                                        ) {
                                            error!(
                                                "Failed to update model status to Unavailable: {}",
                                                e
                                            );
                                        } else {
                                            info!("Model {} marked as Unavailable after {} consecutive failures", 
                                                model.id, *current_failures);
                                        }
                                    }
                                }
                            } else {
                                // Reset failure counter on success
                                failure_counters.remove(&model.id);

                                // Update model status if it was previously unavailable
                                if config.auto_update_status
                                    && model.status == ModelStatus::Unavailable
                                {
                                    if let Err(e) = registry_api
                                        .update_model_status(&model.id, ModelStatus::Available)
                                    {
                                        error!("Failed to update model status to Available: {}", e);
                                    } else {
                                        info!("Model {} marked as Available after successful health check", model.id);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Health check for model {} failed: {}", model.id, e);
                        }
                    }
                }
            }
        }));

        info!(
            "Health check task started with interval of {} seconds",
            self.config.check_interval_seconds
        );
    }

    /// Stop periodic health checks
    pub fn stop_health_checks(&mut self) {
        if let Some(task) = self.health_check_task.take() {
            task.abort();
            info!("Health check task stopped");
        }
    }

    /// Check health of a specific model
    pub async fn check_model(&self, model_id: &str) -> Result<HealthCheckResult, RegistryError> {
        // Get the model
        let model = self.registry_api.get_model(model_id)?;

        // Check model health
        let result = check_model_health(&model, self.config.request_timeout_seconds).await?;

        // Update failure counter and model status if needed
        if !result.success {
            let mut current_failures = self
                .failure_counters
                .entry(model_id.to_string())
                .or_insert(0);
            *current_failures += 1;

            if *current_failures >= self.config.max_consecutive_failures
                && self.config.auto_update_status
            {
                if let Err(e) = self
                    .registry_api
                    .update_model_status(model_id, ModelStatus::Unavailable)
                {
                    error!("Failed to update model status to Unavailable: {}", e);
                }
            }
        } else {
            // Reset failure counter on success
            self.failure_counters.remove(model_id);

            // Update model status if it was previously unavailable and auto-update is enabled
            if self.config.auto_update_status {
                let current_model = self.registry_api.get_model(model_id)?;
                if current_model.status == ModelStatus::Unavailable {
                    if let Err(e) = self
                        .registry_api
                        .update_model_status(model_id, ModelStatus::Available)
                    {
                        error!("Failed to update model status to Available: {}", e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Check health of all models
    pub async fn check_all_models(&self) -> Vec<Result<HealthCheckResult, RegistryError>> {
        let models = self.registry_api.list_models();
        let mut results = Vec::with_capacity(models.len());

        for model in models {
            results.push(self.check_model(&model.id).await);
        }

        results
    }

    /// Get the current health status of a model
    pub fn get_model_health_status(&self, model_id: &str) -> Result<ModelStatus, RegistryError> {
        let model = self.registry_api.get_model(model_id)?;
        Ok(model.status)
    }

    /// Get the current failure count for a model
    pub fn get_failure_count(&self, model_id: &str) -> u32 {
        self.failure_counters
            .get(model_id)
            .map(|count| *count)
            .unwrap_or(0)
    }

    /// Reset the failure counter for a model
    pub fn reset_failure_counter(&self, model_id: &str) {
        self.failure_counters.remove(model_id);
        debug!("Reset failure counter for model {}", model_id);
    }

    /// Update the health check configuration
    pub fn update_config(&mut self, config: HealthCheckConfig) {
        // Stop existing health check task if running
        if self.health_check_task.is_some() {
            self.stop_health_checks();
        }

        // Update config
        self.config = config;

        // Restart health checks if needed
        self.start_health_checks();

        info!("Health check configuration updated");
    }
}

/// Check health of a model
pub async fn check_model_health(
    model: &ModelMetadata,
    timeout_seconds: u64,
) -> Result<HealthCheckResult, RegistryError> {
    let start_time = std::time::Instant::now();
    let model_id = model.id.clone();

    // Create a timeout for the health check
    let timeout = Duration::from_secs(timeout_seconds);

    // Perform the health check with timeout
    let result = tokio::time::timeout(timeout, perform_health_check(model)).await;

    match result {
        Ok(Ok(_)) => {
            // Health check succeeded
            let elapsed = start_time.elapsed();
            let response_time_ms = elapsed.as_millis() as u64;

            Ok(HealthCheckResult {
                model_id,
                success: true,
                error_message: None,
                response_time_ms: Some(response_time_ms),
                timestamp: chrono::Utc::now(),
            })
        }
        Ok(Err(e)) => {
            // Health check failed
            Ok(HealthCheckResult {
                model_id,
                success: false,
                error_message: Some(e.to_string()),
                response_time_ms: None,
                timestamp: chrono::Utc::now(),
            })
        }
        Err(_) => {
            // Health check timed out
            Ok(HealthCheckResult {
                model_id,
                success: false,
                error_message: Some(format!(
                    "Health check timed out after {} seconds",
                    timeout_seconds
                )),
                response_time_ms: None,
                timestamp: chrono::Utc::now(),
            })
        }
    }
}

/// Perform a health check for a model
async fn perform_health_check(model: &ModelMetadata) -> Result<(), RegistryError> {
    // In a real implementation, this would make a request to the model's endpoint
    // to check if it's available. For now, we'll just simulate a health check.

    // Simulate a health check based on the model's endpoint
    // In a real implementation, this would make an actual HTTP request

    // For demonstration purposes, we'll consider models with "unavailable" in their endpoint as unhealthy
    if model.endpoint.contains("unavailable") {
        return Err(RegistryError::CommunicationError(format!(
            "Model {} is unavailable",
            model.id
        )));
    }

    // For demonstration purposes, we'll randomly fail some health checks
    let random_number = rand::random::<u8>();
    if random_number < 10 {
        // ~4% chance of failure
        return Err(RegistryError::CommunicationError(format!(
            "Random health check failure for model {}",
            model.id
        )));
    }

    // Health check succeeded
    Ok(())
}

/// Create a health check manager with default configuration
pub fn create_health_check_manager(registry_api: Arc<ModelRegistryApi>) -> HealthCheckManager {
    HealthCheckManager::new(registry_api, HealthCheckConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::api::ModelRegistryApi;
    use crate::modules::model_registry::types::{ModelMetadata, ModelStatus};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    fn create_test_model(id: &str, endpoint: &str) -> ModelMetadata {
        ModelMetadata::new(
            id.to_string(),
            format!("{} Model", id),
            "test".to_string(),
            "1.0".to_string(),
            endpoint.to_string(),
        )
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let api = Arc::new(ModelRegistryApi::new());
        let model = create_test_model("test-model", "https://api.example.com");
        api.register_model(model.clone()).unwrap();

        let result = check_model_health(&model, 5).await.unwrap();
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert!(result.response_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let api = Arc::new(ModelRegistryApi::new());
        let model = create_test_model("test-model", "https://unavailable.example.com");
        api.register_model(model.clone()).unwrap();

        let result = check_model_health(&model, 5).await.unwrap();
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert!(result.error_message.unwrap().contains("unavailable"));
    }

    #[tokio::test]
    async fn test_health_check_manager() {
        let api = Arc::new(ModelRegistryApi::new());

        // Register some models
        api.register_model(create_test_model("model1", "https://api.example.com/1"))
            .unwrap();
        api.register_model(create_test_model("model2", "https://api.example.com/2"))
            .unwrap();
        api.register_model(create_test_model(
            "model3",
            "https://unavailable.example.com",
        ))
        .unwrap();

        // Create health check manager with custom config
        let config = HealthCheckConfig {
            check_interval_seconds: 1,
            request_timeout_seconds: 2,
            max_consecutive_failures: 1,
            auto_update_status: true,
        };
        let mut manager = HealthCheckManager::new(api.clone(), config);

        // Check a specific model
        let result = manager.check_model("model1").await.unwrap();
        assert!(result.success);

        // Check all models
        let results = manager.check_all_models().await;
        assert_eq!(results.len(), 3);

        // Start periodic health checks
        manager.start_health_checks();

        // Wait for a health check cycle to complete
        sleep(Duration::from_secs(2)).await;

        // Check model statuses
        let _model1 = api.get_model("model1").unwrap();
        let model3 = api.get_model("model3").unwrap();

        // model1 should be available, model3 should be unavailable
        // Note: This test might be flaky due to the random failure simulation
        // but model3 should definitely be unavailable due to its endpoint
        assert_eq!(model3.status, ModelStatus::Unavailable);

        // Stop health checks
        manager.stop_health_checks();
    }

    #[tokio::test]
    async fn test_failure_counter() {
        let api = Arc::new(ModelRegistryApi::new());
        let model = create_test_model("test-model", "https://api.example.com");
        api.register_model(model.clone()).unwrap();

        let config = HealthCheckConfig {
            check_interval_seconds: 1,
            request_timeout_seconds: 2,
            max_consecutive_failures: 3,
            auto_update_status: true,
        };
        let manager = HealthCheckManager::new(api.clone(), config);

        // Initially, failure count should be 0
        assert_eq!(manager.get_failure_count("test-model"), 0);

        // Mock a failure by directly manipulating the failure counter
        manager.failure_counters.insert("test-model".to_string(), 2);
        assert_eq!(manager.get_failure_count("test-model"), 2);

        // Reset the counter
        manager.reset_failure_counter("test-model");
        assert_eq!(manager.get_failure_count("test-model"), 0);
    }

    #[tokio::test]
    async fn test_config_update() {
        let api = Arc::new(ModelRegistryApi::new());
        let initial_config = HealthCheckConfig {
            check_interval_seconds: 60,
            request_timeout_seconds: 5,
            max_consecutive_failures: 3,
            auto_update_status: true,
        };

        let mut manager = HealthCheckManager::new(api.clone(), initial_config);

        // Start health checks
        manager.start_health_checks();

        // Update config
        let new_config = HealthCheckConfig {
            check_interval_seconds: 30,
            request_timeout_seconds: 10,
            max_consecutive_failures: 5,
            auto_update_status: false,
        };

        manager.update_config(new_config.clone());

        // Verify config was updated
        assert_eq!(manager.config.check_interval_seconds, 30);
        assert_eq!(manager.config.request_timeout_seconds, 10);
        assert_eq!(manager.config.max_consecutive_failures, 5);
        assert_eq!(manager.config.auto_update_status, false);

        // Stop health checks
        manager.stop_health_checks();
    }
}
