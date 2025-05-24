//! Environment Management Module
//!
//! This module provides enhanced functionality for managing test environments,
//! including provisioning, configuration, and teardown.

mod config;
mod docker;
mod environment_trait;
mod kubernetes;
mod local;
mod remote;
mod template;

pub use config::{EnvironmentConfig, EnvironmentType, ResourceRequirements};
pub use docker::{DockerConfig, DockerEnvironment};
pub use environment_trait::{Environment, EnvironmentExt};
pub use kubernetes::{KubernetesConfig, KubernetesEnvironment};
pub use local::{LocalConfig, LocalEnvironment};
pub use remote::{RemoteConfig, RemoteEnvironment};
pub use template::{
    EnvironmentCacheStats, EnvironmentTemplate, EnvironmentTemplateBuilder,
    EnvironmentTemplateManager, TemplateHooks, TemplateParameter, TemplateParameterType,
    TemplateParameterValidation,
};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::types::TestHarnessError;

// Environment trait is now defined in environment_trait.rs

/// Environment factory for creating environments
pub struct EnvironmentFactory;

impl EnvironmentFactory {
    /// Create a new environment
    pub async fn create_environment(
        config: EnvironmentConfig,
    ) -> Result<Box<dyn Environment>, TestHarnessError> {
        match config.env_type {
            EnvironmentType::Local => {
                let local_config = config.local_config.clone().unwrap_or_default();
                let env = LocalEnvironment::new(config.clone(), local_config).await?;
                Ok(Box::new(env))
            }
            EnvironmentType::Docker => {
                let docker_config = config.docker_config.clone().unwrap_or_default();
                let env = DockerEnvironment::new(config.clone(), docker_config).await?;
                Ok(Box::new(env))
            }
            EnvironmentType::Kubernetes => {
                let k8s_config = config.kubernetes_config.clone().unwrap_or_default();
                let env = KubernetesEnvironment::new(config.clone(), k8s_config).await?;
                Ok(Box::new(env))
            }
            EnvironmentType::Remote => {
                let remote_config = config.remote_config.clone().unwrap_or_default();
                let env = RemoteEnvironment::new(config.clone(), remote_config).await?;
                Ok(Box::new(env))
            }
        }
    }
}

/// Environment builder for creating environments
pub struct EnvironmentBuilder {
    /// Environment configuration
    config: EnvironmentConfig,
}

impl EnvironmentBuilder {
    /// Create a new environment builder
    pub fn new() -> Self {
        Self {
            config: EnvironmentConfig::default(),
        }
    }

    /// Set the environment type
    pub fn with_env_type(mut self, env_type: EnvironmentType) -> Self {
        self.config.env_type = env_type;
        self
    }

    /// Set the environment ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.config.id = id.into();
        self
    }

    /// Set the environment name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the environment description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    /// Set the base directory
    pub fn with_base_dir(mut self, base_dir: impl Into<PathBuf>) -> Self {
        self.config.base_dir = base_dir.into();
        self
    }

    /// Set whether to use a temporary directory
    pub fn with_temp_dir(mut self, use_temp_dir: bool) -> Self {
        self.config.use_temp_dir = use_temp_dir;
        self
    }

    /// Add an environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn with_env_vars(
        mut self,
        vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in vars {
            self.config.env_vars.insert(key.into(), value.into());
        }
        self
    }

    /// Set the test data directory
    pub fn with_test_data_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.test_data_dir = Some(dir.into());
        self
    }

    /// Set whether to clean up after tests
    pub fn with_cleanup(mut self, cleanup: bool) -> Self {
        self.config.cleanup = cleanup;
        self
    }

    /// Set the Docker configuration
    pub fn with_docker_config(mut self, docker_config: DockerConfig) -> Self {
        self.config.docker_config = Some(docker_config);
        self
    }

    /// Set the Kubernetes configuration
    pub fn with_kubernetes_config(mut self, kubernetes_config: KubernetesConfig) -> Self {
        self.config.kubernetes_config = Some(kubernetes_config);
        self
    }

    /// Set the local configuration
    pub fn with_local_config(mut self, local_config: LocalConfig) -> Self {
        self.config.local_config = Some(local_config);
        self
    }

    /// Set the remote configuration
    pub fn with_remote_config(mut self, remote_config: RemoteConfig) -> Self {
        self.config.remote_config = Some(remote_config);
        self
    }

    /// Set the database URL
    pub fn with_database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = Some(url.into());
        self
    }

    /// Set the Redis URL
    pub fn with_redis_url(mut self, url: impl Into<String>) -> Self {
        self.config.redis_url = Some(url.into());
        self
    }

    /// Set the Kafka URL
    pub fn with_kafka_url(mut self, url: impl Into<String>) -> Self {
        self.config.kafka_url = Some(url.into());
        self
    }

    /// Set the Elasticsearch URL
    pub fn with_elasticsearch_url(mut self, url: impl Into<String>) -> Self {
        self.config.elasticsearch_url = Some(url.into());
        self
    }

    /// Set the S3 URL
    pub fn with_s3_url(mut self, url: impl Into<String>) -> Self {
        self.config.s3_url = Some(url.into());
        self
    }

    /// Add a service
    pub fn with_service(mut self, name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        self.config.services.insert(name.into(), endpoint.into());
        self
    }

    /// Add multiple services
    pub fn with_services(
        mut self,
        services: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (name, endpoint) in services {
            self.config.services.insert(name.into(), endpoint.into());
        }
        self
    }

    /// Add a property
    pub fn with_property(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.config.properties.insert(key.into(), value);
        Ok(self)
    }

    /// Set resource requirements
    pub fn with_resource_requirements(mut self, requirements: ResourceRequirements) -> Self {
        self.config.resource_requirements = Some(requirements);
        self
    }

    /// Build the environment
    pub async fn build(self) -> Result<Box<dyn Environment>, TestHarnessError> {
        EnvironmentFactory::create_environment(self.config).await
    }
}

impl Default for EnvironmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage in percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Network usage in bytes
    pub network_rx: u64,
    /// Network usage in bytes
    pub network_tx: u64,
    /// Per-service resource usage
    pub services: HashMap<String, ServiceResourceUsage>,
}

/// Service resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResourceUsage {
    /// CPU usage in percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Network usage in bytes
    pub network_rx: u64,
    /// Network usage in bytes
    pub network_tx: u64,
}
