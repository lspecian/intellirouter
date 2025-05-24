//! Environment Configuration Module
//!
//! This module provides configuration structures for different environment types.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Environment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentType {
    /// Local environment
    Local,
    /// Docker environment
    Docker,
    /// Kubernetes environment
    Kubernetes,
    /// Remote environment
    Remote,
}

impl Default for EnvironmentType {
    fn default() -> Self {
        Self::Local
    }
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment type
    pub env_type: EnvironmentType,
    /// Environment ID
    pub id: String,
    /// Environment name
    pub name: String,
    /// Environment description
    pub description: Option<String>,
    /// Base directory for test artifacts
    pub base_dir: PathBuf,
    /// Whether to use a temporary directory for test artifacts
    pub use_temp_dir: bool,
    /// Environment variables to set for tests
    pub env_vars: HashMap<String, String>,
    /// Test data directory
    pub test_data_dir: Option<PathBuf>,
    /// Whether to clean up after tests
    pub cleanup: bool,
    /// Docker configuration
    pub docker_config: Option<DockerConfig>,
    /// Kubernetes configuration
    pub kubernetes_config: Option<KubernetesConfig>,
    /// Local configuration
    pub local_config: Option<LocalConfig>,
    /// Remote configuration
    pub remote_config: Option<RemoteConfig>,
    /// Database connection string for tests
    pub database_url: Option<String>,
    /// Redis connection string for tests
    pub redis_url: Option<String>,
    /// Kafka connection string for tests
    pub kafka_url: Option<String>,
    /// Elasticsearch connection string for tests
    pub elasticsearch_url: Option<String>,
    /// S3 connection string for tests
    pub s3_url: Option<String>,
    /// Service endpoints
    pub services: HashMap<String, String>,
    /// Custom environment properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Resource requirements
    pub resource_requirements: Option<ResourceRequirements>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            env_type: EnvironmentType::default(),
            id: uuid::Uuid::new_v4().to_string(),
            name: "test-environment".to_string(),
            description: None,
            base_dir: PathBuf::from("test_artifacts"),
            use_temp_dir: true,
            env_vars: HashMap::new(),
            test_data_dir: None,
            cleanup: true,
            docker_config: None,
            kubernetes_config: None,
            local_config: None,
            remote_config: None,
            database_url: None,
            redis_url: None,
            kafka_url: None,
            elasticsearch_url: None,
            s3_url: None,
            services: HashMap::new(),
            properties: HashMap::new(),
            resource_requirements: None,
        }
    }
}

/// Docker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Docker compose file
    pub compose_file: Option<PathBuf>,
    /// Docker compose project name
    pub project_name: Option<String>,
    /// Docker compose environment file
    pub env_file: Option<PathBuf>,
    /// Docker compose services to start
    pub services: Vec<String>,
    /// Whether to use existing Docker services
    pub use_existing: bool,
    /// Docker network
    pub network: Option<String>,
    /// Docker volume mounts
    pub volumes: HashMap<String, String>,
    /// Docker ports to expose
    pub ports: HashMap<u16, u16>,
    /// Docker environment variables
    pub env_vars: HashMap<String, String>,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            compose_file: None,
            project_name: None,
            env_file: None,
            services: Vec::new(),
            use_existing: false,
            network: None,
            volumes: HashMap::new(),
            ports: HashMap::new(),
            env_vars: HashMap::new(),
        }
    }
}

/// Kubernetes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Kubernetes context
    pub context: Option<String>,
    /// Kubernetes namespace
    pub namespace: Option<String>,
    /// Kubernetes config file
    pub config_file: Option<PathBuf>,
    /// Kubernetes in-cluster config
    pub in_cluster: bool,
    /// Kubernetes service account
    pub service_account: Option<String>,
    /// Kubernetes pod name
    pub pod_name: Option<String>,
    /// Kubernetes pod namespace
    pub pod_namespace: Option<String>,
    /// Kubernetes volume mounts
    pub volume_mounts: HashMap<String, String>,
    /// Kubernetes environment variables
    pub env_vars: HashMap<String, String>,
    /// Kubernetes labels
    pub labels: HashMap<String, String>,
    /// Kubernetes annotations
    pub annotations: HashMap<String, String>,
    /// Kubernetes resource limits
    pub resource_limits: Option<ResourceRequirements>,
    /// Kubernetes resource requests
    pub resource_requests: Option<ResourceRequirements>,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            context: None,
            namespace: None,
            config_file: None,
            in_cluster: false,
            service_account: None,
            pod_name: None,
            pod_namespace: None,
            volume_mounts: HashMap::new(),
            env_vars: HashMap::new(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            resource_limits: None,
            resource_requests: None,
        }
    }
}

/// Local configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Local working directory
    pub working_dir: Option<PathBuf>,
    /// Local environment variables
    pub env_vars: HashMap<String, String>,
    /// Local ports to use
    pub ports: HashMap<u16, u16>,
    /// Local database URL
    pub database_url: Option<String>,
    /// Local Redis URL
    pub redis_url: Option<String>,
    /// Local services
    pub services: HashMap<String, String>,
    /// Local service commands
    pub service_commands: HashMap<String, String>,
    /// Local service ports
    pub service_ports: HashMap<String, u16>,
    /// Local service environment variables
    pub service_env_vars: HashMap<String, HashMap<String, String>>,
    /// Local service working directories
    pub service_working_dirs: HashMap<String, PathBuf>,
    /// Local service dependencies
    pub service_dependencies: HashMap<String, Vec<String>>,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            working_dir: None,
            env_vars: HashMap::new(),
            ports: HashMap::new(),
            database_url: None,
            redis_url: None,
            services: HashMap::new(),
            service_commands: HashMap::new(),
            service_ports: HashMap::new(),
            service_env_vars: HashMap::new(),
            service_working_dirs: HashMap::new(),
            service_dependencies: HashMap::new(),
        }
    }
}

/// Remote configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// Remote host
    pub host: String,
    /// Remote port
    pub port: u16,
    /// Remote username
    pub username: Option<String>,
    /// Remote password
    pub password: Option<String>,
    /// Remote private key
    pub private_key: Option<String>,
    /// Remote private key file
    pub private_key_file: Option<PathBuf>,
    /// Remote passphrase
    pub passphrase: Option<String>,
    /// Remote working directory
    pub working_dir: Option<PathBuf>,
    /// Remote environment variables
    pub env_vars: HashMap<String, String>,
    /// Remote ports to use
    pub ports: HashMap<u16, u16>,
    /// Remote services
    pub services: HashMap<String, String>,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 22,
            username: None,
            password: None,
            private_key: None,
            private_key_file: None,
            passphrase: None,
            working_dir: None,
            env_vars: HashMap::new(),
            ports: HashMap::new(),
            services: HashMap::new(),
        }
    }
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirements in millicores
    pub cpu: Option<u32>,
    /// Memory requirements in bytes
    pub memory: Option<u64>,
    /// Disk requirements in bytes
    pub disk: Option<u64>,
    /// GPU requirements
    pub gpu: Option<u32>,
    /// Network requirements in bytes per second
    pub network: Option<u64>,
    /// Custom requirements
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu: None,
            memory: None,
            disk: None,
            gpu: None,
            network: None,
            custom: HashMap::new(),
        }
    }
}
