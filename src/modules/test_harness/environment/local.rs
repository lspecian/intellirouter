//! Local Environment Implementation
//!
//! This module provides a local environment implementation for testing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command as TokioCommand;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::config::{EnvironmentConfig, EnvironmentType, LocalConfig, ResourceRequirements};
use super::{Environment, ResourceUsage, ServiceResourceUsage};
use crate::modules::test_harness::types::TestHarnessError;

/// Local environment implementation
pub struct LocalEnvironment {
    /// Environment configuration
    config: EnvironmentConfig,
    /// Local configuration
    local_config: LocalConfig,
    /// Temporary directory for test artifacts
    temp_dir: Option<TempDir>,
    /// Artifacts directory
    artifacts_dir: PathBuf,
    /// Logs directory
    logs_dir: PathBuf,
    /// Environment variables
    env_vars: HashMap<String, String>,
    /// Running services
    running_services: HashMap<String, LocalService>,
    /// Shared environment state
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Snapshots
    snapshots: HashMap<String, LocalSnapshot>,
}

/// Local service
#[derive(Debug)]
struct LocalService {
    /// Service name
    name: String,
    /// Service command
    command: String,
    /// Service working directory
    working_dir: Option<PathBuf>,
    /// Service environment variables
    env_vars: HashMap<String, String>,
    /// Service port
    port: Option<u16>,
    /// Service process ID
    pid: Option<u32>,
    /// Service log file
    log_file: Option<PathBuf>,
    /// Service health check command
    health_check_command: Option<String>,
    /// Service health check URL
    health_check_url: Option<String>,
    /// Service dependencies
    dependencies: Vec<String>,
}

/// Local snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalSnapshot {
    /// Snapshot name
    name: String,
    /// Snapshot timestamp
    timestamp: chrono::DateTime<chrono::Utc>,
    /// Snapshot state
    state: HashMap<String, serde_json::Value>,
    /// Snapshot environment variables
    env_vars: HashMap<String, String>,
    /// Snapshot services
    services: HashMap<String, LocalServiceSnapshot>,
}

/// Local service snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalServiceSnapshot {
    /// Service name
    name: String,
    /// Service command
    command: String,
    /// Service working directory
    working_dir: Option<PathBuf>,
    /// Service environment variables
    env_vars: HashMap<String, String>,
    /// Service port
    port: Option<u16>,
    /// Service log file
    log_file: Option<PathBuf>,
}

impl LocalEnvironment {
    /// Create a new local environment
    pub async fn new(
        config: EnvironmentConfig,
        local_config: LocalConfig,
    ) -> Result<Self, TestHarnessError> {
        // Create temporary directory if needed
        let (temp_dir, artifacts_dir) = if config.use_temp_dir {
            let temp_dir = TempDir::new().map_err(TestHarnessError::IoError)?;
            let artifacts_dir = temp_dir.path().to_path_buf();
            (Some(temp_dir), artifacts_dir)
        } else {
            let artifacts_dir = config.base_dir.clone().join("artifacts");
            tokio::fs::create_dir_all(&artifacts_dir)
                .await
                .map_err(TestHarnessError::IoError)?;
            (None, artifacts_dir)
        };

        // Create logs directory
        let logs_dir = artifacts_dir.join("logs");
        tokio::fs::create_dir_all(&logs_dir)
            .await
            .map_err(TestHarnessError::IoError)?;

        // Set environment variables
        let mut env_vars = config.env_vars.clone();
        env_vars.extend(local_config.env_vars.clone());

        // Apply environment variables
        for (key, value) in &env_vars {
            std::env::set_var(key, value);
        }

        // Create running services map
        let mut running_services = HashMap::new();
        for (name, command) in &local_config.service_commands {
            let working_dir = local_config.service_working_dirs.get(name).cloned();
            let env_vars = local_config
                .service_env_vars
                .get(name)
                .cloned()
                .unwrap_or_default();
            let port = local_config.service_ports.get(name).cloned();
            let log_file = Some(logs_dir.join(format!("{}.log", name)));
            let health_check_command = local_config
                .service_health_check_commands
                .get(name)
                .cloned();
            let health_check_url = local_config.service_health_check_urls.get(name).cloned();
            let dependencies = local_config
                .service_dependencies
                .get(name)
                .cloned()
                .unwrap_or_default();

            let service = LocalService {
                name: name.clone(),
                command: command.clone(),
                working_dir,
                env_vars,
                port,
                pid: None,
                log_file,
                health_check_command,
                health_check_url,
                dependencies,
            };

            running_services.insert(name.clone(), service);
        }

        Ok(Self {
            config,
            local_config,
            temp_dir,
            artifacts_dir,
            logs_dir,
            env_vars,
            running_services,
            state: Arc::new(RwLock::new(HashMap::new())),
            snapshots: HashMap::new(),
        })
    }

    /// Start a service
    async fn start_service(&mut self, name: &str) -> Result<(), TestHarnessError> {
        let service = self.running_services.get_mut(name).ok_or_else(|| {
            TestHarnessError::EnvironmentError(format!("Service {} not found", name))
        })?;

        // Check if service is already running
        if service.pid.is_some() {
            return Ok(());
        }

        // Start service dependencies
        for dependency in service.dependencies.clone() {
            self.start_service(&dependency).await?;
        }

        info!("Starting service: {}", name);

        // Prepare command
        let mut command = TokioCommand::new("sh");
        command.arg("-c").arg(&service.command);

        // Set working directory
        if let Some(working_dir) = &service.working_dir {
            command.current_dir(working_dir);
        }

        // Set environment variables
        for (key, value) in &service.env_vars {
            command.env(key, value);
        }

        // Redirect output to log file
        if let Some(log_file) = &service.log_file {
            let log_dir = log_file.parent().unwrap();
            tokio::fs::create_dir_all(log_dir)
                .await
                .map_err(TestHarnessError::IoError)?;

            let file = tokio::fs::File::create(log_file)
                .await
                .map_err(TestHarnessError::IoError)?;

            command.stdout(file);
        }

        // Start the process
        let child = command.spawn().map_err(|e| {
            TestHarnessError::EnvironmentError(format!("Failed to start service {}: {}", name, e))
        })?;

        // Store the process ID
        service.pid = Some(child.id().unwrap_or(0));

        // Wait for service to be ready
        if let Some(health_check_command) = &service.health_check_command {
            info!("Waiting for service {} to be ready", name);

            let mut retries = 30;
            let mut ready = false;

            while retries > 0 && !ready {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(health_check_command)
                    .output();

                match output {
                    Ok(output) => {
                        if output.status.success() {
                            ready = true;
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Health check command failed: {}", e);
                    }
                }

                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            if !ready {
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Service {} did not become ready in time",
                    name
                )));
            }
        } else if let Some(health_check_url) = &service.health_check_url {
            info!("Waiting for service {} to be ready", name);

            let mut retries = 30;
            let mut ready = false;

            while retries > 0 && !ready {
                let response = reqwest::get(health_check_url).await;

                match response {
                    Ok(response) => {
                        if response.status().is_success() {
                            ready = true;
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Health check URL failed: {}", e);
                    }
                }

                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            if !ready {
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Service {} did not become ready in time",
                    name
                )));
            }
        }

        info!("Service {} started", name);

        Ok(())
    }

    /// Stop a service
    async fn stop_service(&mut self, name: &str) -> Result<(), TestHarnessError> {
        let service = self.running_services.get_mut(name).ok_or_else(|| {
            TestHarnessError::EnvironmentError(format!("Service {} not found", name))
        })?;

        // Check if service is running
        if let Some(pid) = service.pid {
            info!("Stopping service: {}", name);

            // Kill the process
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();

            // Clear the process ID
            service.pid = None;

            info!("Service {} stopped", name);
        }

        Ok(())
    }
}

#[async_trait]
impl Environment for LocalEnvironment {
    fn env_type(&self) -> EnvironmentType {
        EnvironmentType::Local
    }

    fn id(&self) -> &str {
        &self.config.id
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn base_dir(&self) -> &Path {
        &self.config.base_dir
    }

    fn artifacts_dir(&self) -> &Path {
        &self.artifacts_dir
    }

    fn logs_dir(&self) -> &Path {
        &self.logs_dir
    }

    fn config(&self) -> &EnvironmentConfig {
        &self.config
    }

    fn get_env_var(&self, key: &str) -> Option<&str> {
        self.env_vars.get(key).map(|s| s.as_str())
    }

    fn set_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        std::env::set_var(&key, &value);
        self.env_vars.insert(key, value);
    }

    fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.properties.get(key)
    }

    fn get_property_as<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        if let Some(value) = self.config.properties.get(key) {
            let typed_value = serde_json::from_value(value.clone())
                .map_err(TestHarnessError::SerializationError)?;
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }

    async fn setup(&mut self) -> Result<(), TestHarnessError> {
        info!("Setting up local environment: {}", self.name());

        // Create artifacts directory
        tokio::fs::create_dir_all(&self.artifacts_dir)
            .await
            .map_err(TestHarnessError::IoError)?;

        // Create logs directory
        tokio::fs::create_dir_all(&self.logs_dir)
            .await
            .map_err(TestHarnessError::IoError)?;

        // Start services
        for name in self.running_services.keys().cloned().collect::<Vec<_>>() {
            self.start_service(&name).await?;
        }

        Ok(())
    }

    async fn teardown(&mut self) -> Result<(), TestHarnessError> {
        info!("Tearing down local environment: {}", self.name());

        // Stop services in reverse dependency order
        let mut services = self.running_services.keys().cloned().collect::<Vec<_>>();
        services.reverse();

        for name in services {
            self.stop_service(&name).await?;
        }

        // Clean up temporary directory
        if self.config.cleanup {
            if let Some(temp_dir) = self.temp_dir.take() {
                drop(temp_dir);
            }
        }

        Ok(())
    }

    fn get_service_endpoint(&self, service: &str) -> Option<String> {
        self.config.services.get(service).cloned()
    }

    async fn wait_for_service(
        &self,
        service: &str,
        timeout_secs: u64,
    ) -> Result<(), TestHarnessError> {
        let service_info = self.running_services.get(service).ok_or_else(|| {
            TestHarnessError::EnvironmentError(format!("Service {} not found", service))
        })?;

        if let Some(health_check_url) = &service_info.health_check_url {
            info!("Waiting for service {} to be ready", service);

            let mut retries = timeout_secs as usize;
            let mut ready = false;

            while retries > 0 && !ready {
                let response = reqwest::get(health_check_url).await;

                match response {
                    Ok(response) => {
                        if response.status().is_success() {
                            ready = true;
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Health check URL failed: {}", e);
                    }
                }

                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            if !ready {
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Service {} did not become ready in time",
                    service
                )));
            }
        } else if let Some(health_check_command) = &service_info.health_check_command {
            info!("Waiting for service {} to be ready", service);

            let mut retries = timeout_secs as usize;
            let mut ready = false;

            while retries > 0 && !ready {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(health_check_command)
                    .output();

                match output {
                    Ok(output) => {
                        if output.status.success() {
                            ready = true;
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Health check command failed: {}", e);
                    }
                }

                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            if !ready {
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Service {} did not become ready in time",
                    service
                )));
            }
        }

        Ok(())
    }

    async fn wait_for_all_services(&self, timeout_secs: u64) -> Result<(), TestHarnessError> {
        for service in self.running_services.keys() {
            self.wait_for_service(service, timeout_secs).await?;
        }

        Ok(())
    }

    fn create_artifacts_subdir(
        &self,
        subdir: impl AsRef<Path>,
    ) -> Result<PathBuf, TestHarnessError> {
        let path = self.artifacts_dir.join(subdir);
        std::fs::create_dir_all(&path).map_err(TestHarnessError::IoError)?;
        Ok(path)
    }

    fn database_url(&self) -> Option<&str> {
        self.local_config
            .database_url
            .as_deref()
            .or_else(|| self.config.database_url.as_deref())
    }

    fn redis_url(&self) -> Option<&str> {
        self.local_config
            .redis_url
            .as_deref()
            .or_else(|| self.config.redis_url.as_deref())
    }

    fn kafka_url(&self) -> Option<&str> {
        self.config.kafka_url.as_deref()
    }

    fn elasticsearch_url(&self) -> Option<&str> {
        self.config.elasticsearch_url.as_deref()
    }

    fn s3_url(&self) -> Option<&str> {
        self.config.s3_url.as_deref()
    }

    async fn get_state(&self, key: &str) -> Option<serde_json::Value> {
        let state = self.state.read().await;
        state.get(key).cloned()
    }

    async fn set_state<T: Serialize>(
        &self,
        key: impl Into<String>,
        value: T,
    ) -> Result<(), TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        let mut state = self.state.write().await;
        state.insert(key.into(), value);
        Ok(())
    }

    async fn get_state_as<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TestHarnessError> {
        if let Some(value) = self.get_state(key).await {
            let typed_value =
                serde_json::from_value(value).map_err(TestHarnessError::SerializationError)?;
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }

    async fn clear_state(&self) -> Result<(), TestHarnessError> {
        let mut state = self.state.write().await;
        state.clear();
        Ok(())
    }

    async fn execute_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<(String, String), TestHarnessError> {
        let output = TokioCommand::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| {
                TestHarnessError::EnvironmentError(format!("Failed to execute command: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            warn!("Command failed with status: {}", output.status);
            warn!("Stderr: {}", stderr);
        }

        Ok((stdout, stderr))
    }

    async fn copy_file_to(
        &self,
        local_path: impl AsRef<Path>,
        remote_path: impl AsRef<Path>,
    ) -> Result<(), TestHarnessError> {
        let local_path = local_path.as_ref();
        let remote_path = remote_path.as_ref();

        fs::copy(local_path, remote_path).await.map_err(|e| {
            TestHarnessError::EnvironmentError(format!(
                "Failed to copy file from {} to {}: {}",
                local_path.display(),
                remote_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    async fn copy_file_from(
        &self,
        remote_path: impl AsRef<Path>,
        local_path: impl AsRef<Path>,
    ) -> Result<(), TestHarnessError> {
        let remote_path = remote_path.as_ref();
        let local_path = local_path.as_ref();

        fs::copy(remote_path, local_path).await.map_err(|e| {
            TestHarnessError::EnvironmentError(format!(
                "Failed to copy file from {} to {}: {}",
                remote_path.display(),
                local_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    async fn is_service_healthy(&self, service: &str) -> Result<bool, TestHarnessError> {
        let service_info = self.running_services.get(service).ok_or_else(|| {
            TestHarnessError::EnvironmentError(format!("Service {} not found", service))
        })?;

        if let Some(health_check_url) = &service_info.health_check_url {
            let response = reqwest::get(health_check_url).await;

            match response {
                Ok(response) => Ok(response.status().is_success()),
                Err(_) => Ok(false),
            }
        } else if let Some(health_check_command) = &service_info.health_check_command {
            let output = Command::new("sh")
                .arg("-c")
                .arg(health_check_command)
                .output();

            match output {
                Ok(output) => Ok(output.status.success()),
                Err(_) => Ok(false),
            }
        } else {
            // If no health check is defined, assume the service is healthy if it's running
            Ok(service_info.pid.is_some())
        }
    }

    async fn get_service_logs(
        &self,
        service: &str,
        lines: Option<usize>,
    ) -> Result<String, TestHarnessError> {
        let service_info = self.running_services.get(service).ok_or_else(|| {
            TestHarnessError::EnvironmentError(format!("Service {} not found", service))
        })?;

        if let Some(log_file) = &service_info.log_file {
            if !log_file.exists() {
                return Ok(String::new());
            }

            let content = fs::read_to_string(log_file).await.map_err(|e| {
                TestHarnessError::EnvironmentError(format!(
                    "Failed to read log file {}: {}",
                    log_file.display(),
                    e
                ))
            })?;

            if let Some(lines) = lines {
                let lines = content.lines().rev().take(lines).collect::<Vec<_>>();
                Ok(lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
            } else {
                Ok(content)
            }
        } else {
            Ok(String::new())
        }
    }

    async fn get_resource_usage(&self) -> Result<ResourceUsage, TestHarnessError> {
        let mut services = HashMap::new();

        for (name, service) in &self.running_services {
            if let Some(pid) = service.pid {
                // Get CPU usage
                let cpu_usage = Command::new("ps")
                    .arg("-p")
                    .arg(pid.to_string())
                    .arg("-o")
                    .arg("%cpu")
                    .arg("--no-headers")
                    .output()
                    .map_err(|e| {
                        TestHarnessError::EnvironmentError(format!(
                            "Failed to get CPU usage for service {}: {}",
                            name, e
                        ))
                    })?;

                let cpu_usage = String::from_utf8_lossy(&cpu_usage.stdout)
                    .trim()
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Get memory usage
                let memory_usage = Command::new("ps")
                    .arg("-p")
                    .arg(pid.to_string())
                    .arg("-o")
                    .arg("rss")
                    .arg("--no-headers")
                    .output()
                    .map_err(|e| {
                        TestHarnessError::EnvironmentError(format!(
                            "Failed to get memory usage for service {}: {}",
                            name, e
                        ))
                    })?;

                let memory_usage = String::from_utf8_lossy(&memory_usage.stdout)
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(0)
                    * 1024; // Convert from KB to bytes

                services.insert(
                    name.clone(),
                    ServiceResourceUsage {
                        cpu_usage,
                        memory_usage,
                        disk_usage: 0,
                        network_rx: 0,
                        network_tx: 0,
                    },
                );
            }
        }

        Ok(ResourceUsage {
            cpu_usage: services.values().map(|s| s.cpu_usage).sum(),
            memory_usage: services.values().map(|s| s.memory_usage).sum(),
            disk_usage: 0,
            network_rx: 0,
            network_tx: 0,
            services,
        })
    }

    async fn take_snapshot(&self, name: &str) -> Result<(), TestHarnessError> {
        let state = self.state.read().await.clone();

        let mut services = HashMap::new();
        for (name, service) in &self.running_services {
            let snapshot = LocalServiceSnapshot {
                name: name.clone(),
                command: service.command.clone(),
                working_dir: service.working_dir.clone(),
                env_vars: service.env_vars.clone(),
                port: service.port,
                log_file: service.log_file.clone(),
            };

            services.insert(name.clone(), snapshot);
        }

        let snapshot = LocalSnapshot {
            name: name.to_string(),
            timestamp: chrono::Utc::now(),
            state,
            env_vars: self.env_vars.clone(),
            services,
        };

        let mut snapshots = self.snapshots.clone();
        snapshots.insert(name.to_string(), snapshot);
        self.snapshots = snapshots;

        Ok(())
    }

    async fn restore_snapshot(&self, name: &str) -> Result<(), TestHarnessError> {
        let snapshot = self
            .snapshots
            .get(name)
            .ok_or_else(|| {
                TestHarnessError::EnvironmentError(format!("Snapshot {} not found", name))
            })?
            .clone();

        // Restore state
        let mut state = self.state.write().await;
        *state = snapshot.state;

        // Restore environment variables
        for (key, value) in &snapshot.env_vars {
            std::env::set_var(key, value);
        }

        Ok(())
    }

    async fn list_snapshots(&self) -> Result<Vec<String>, TestHarnessError> {
        Ok(self.snapshots.keys().cloned().collect())
    }
}
