//! Docker Environment Implementation
//!
//! This module provides a Docker environment implementation for testing.

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

use super::config::{DockerConfig, EnvironmentConfig, EnvironmentType, ResourceRequirements};
use super::{Environment, ResourceUsage, ServiceResourceUsage};
use crate::modules::test_harness::types::TestHarnessError;

/// Docker environment implementation
pub struct DockerEnvironment {
    /// Environment configuration
    config: EnvironmentConfig,
    /// Docker configuration
    docker_config: DockerConfig,
    /// Temporary directory for test artifacts
    temp_dir: Option<TempDir>,
    /// Artifacts directory
    artifacts_dir: PathBuf,
    /// Logs directory
    logs_dir: PathBuf,
    /// Environment variables
    env_vars: HashMap<String, String>,
    /// Running containers
    running_containers: HashMap<String, String>,
    /// Shared environment state
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Docker network
    network: Option<String>,
    /// Docker compose project name
    project_name: Option<String>,
}

impl DockerEnvironment {
    /// Create a new Docker environment
    pub async fn new(
        config: EnvironmentConfig,
        docker_config: DockerConfig,
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
        env_vars.extend(docker_config.env_vars.clone());

        // Apply environment variables
        for (key, value) in &env_vars {
            std::env::set_var(key, value);
        }

        Ok(Self {
            config,
            docker_config,
            temp_dir,
            artifacts_dir,
            logs_dir,
            env_vars,
            running_containers: HashMap::new(),
            state: Arc::new(RwLock::new(HashMap::new())),
            network: docker_config.network.clone(),
            project_name: docker_config.project_name.clone(),
        })
    }

    /// Check if Docker is available
    async fn check_docker_available(&self) -> Result<(), TestHarnessError> {
        let output = TokioCommand::new("docker")
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                TestHarnessError::EnvironmentError(format!("Docker is not available: {}", e))
            })?;

        if !output.status.success() {
            return Err(TestHarnessError::EnvironmentError(
                "Docker is not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if Docker Compose is available
    async fn check_docker_compose_available(&self) -> Result<(), TestHarnessError> {
        let output = TokioCommand::new("docker-compose")
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                TestHarnessError::EnvironmentError(format!(
                    "Docker Compose is not available: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(TestHarnessError::EnvironmentError(
                "Docker Compose is not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Start Docker Compose services
    async fn start_docker_compose(&mut self) -> Result<(), TestHarnessError> {
        if let Some(compose_file) = &self.docker_config.compose_file {
            info!(
                "Starting Docker Compose services from {}",
                compose_file.display()
            );

            // Check if Docker and Docker Compose are available
            self.check_docker_available().await?;
            self.check_docker_compose_available().await?;

            // Build the command
            let mut command = TokioCommand::new("docker-compose");
            command.arg("-f").arg(compose_file);

            // Add project name if specified
            if let Some(project_name) = &self.project_name {
                command.arg("-p").arg(project_name);
            }

            // Add environment file if specified
            if let Some(env_file) = &self.docker_config.env_file {
                command.arg("--env-file").arg(env_file);
            }

            // Add up command
            command.arg("up").arg("-d");

            // Add specific services if specified
            if !self.docker_config.services.is_empty() {
                command.args(&self.docker_config.services);
            }

            // Execute the command
            let output = command.output().await.map_err(|e| {
                TestHarnessError::EnvironmentError(format!(
                    "Failed to start Docker Compose services: {}",
                    e
                ))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Failed to start Docker Compose services: {}",
                    stderr
                )));
            }

            // Get the list of running containers
            self.update_running_containers().await?;

            info!("Docker Compose services started");
        } else {
            warn!("No Docker Compose file specified");
        }

        Ok(())
    }

    /// Stop Docker Compose services
    async fn stop_docker_compose(&mut self) -> Result<(), TestHarnessError> {
        if let Some(compose_file) = &self.docker_config.compose_file {
            info!("Stopping Docker Compose services");

            // Build the command
            let mut command = TokioCommand::new("docker-compose");
            command.arg("-f").arg(compose_file);

            // Add project name if specified
            if let Some(project_name) = &self.project_name {
                command.arg("-p").arg(project_name);
            }

            // Add down command
            command.arg("down");

            // Execute the command
            let output = command.output().await.map_err(|e| {
                TestHarnessError::EnvironmentError(format!(
                    "Failed to stop Docker Compose services: {}",
                    e
                ))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Failed to stop Docker Compose services: {}",
                    stderr
                )));
            }

            // Clear the list of running containers
            self.running_containers.clear();

            info!("Docker Compose services stopped");
        }

        Ok(())
    }

    /// Update the list of running containers
    async fn update_running_containers(&mut self) -> Result<(), TestHarnessError> {
        if let Some(compose_file) = &self.docker_config.compose_file {
            // Build the command
            let mut command = TokioCommand::new("docker-compose");
            command.arg("-f").arg(compose_file);

            // Add project name if specified
            if let Some(project_name) = &self.project_name {
                command.arg("-p").arg(project_name);
            }

            // Add ps command
            command.arg("ps").arg("-q");

            // Execute the command
            let output = command.output().await.map_err(|e| {
                TestHarnessError::EnvironmentError(format!(
                    "Failed to get Docker Compose container IDs: {}",
                    e
                ))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(TestHarnessError::EnvironmentError(format!(
                    "Failed to get Docker Compose container IDs: {}",
                    stderr
                )));
            }

            // Parse the output
            let container_ids = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| line.trim().to_string())
                .collect::<Vec<_>>();

            // Get container names
            self.running_containers.clear();
            for container_id in container_ids {
                let output = TokioCommand::new("docker")
                    .arg("inspect")
                    .arg("--format")
                    .arg("{{.Name}}")
                    .arg(&container_id)
                    .output()
                    .await
                    .map_err(|e| {
                        TestHarnessError::EnvironmentError(format!(
                            "Failed to get container name: {}",
                            e
                        ))
                    })?;

                if output.status.success() {
                    let name = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .trim_start_matches('/')
                        .to_string();
                    self.running_containers.insert(name, container_id);
                }
            }
        }

        Ok(())
    }

    /// Wait for a container to be ready
    async fn wait_for_container(
        &self,
        container_id: &str,
        timeout_secs: u64,
    ) -> Result<(), TestHarnessError> {
        info!("Waiting for container {} to be ready", container_id);

        let mut retries = timeout_secs as usize;
        let mut ready = false;

        while retries > 0 && !ready {
            let output = TokioCommand::new("docker")
                .arg("inspect")
                .arg("--format")
                .arg("{{.State.Running}}")
                .arg(container_id)
                .output()
                .await
                .map_err(|e| {
                    TestHarnessError::EnvironmentError(format!(
                        "Failed to check container status: {}",
                        e
                    ))
                })?;

            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout).trim();
                if status == "true" {
                    ready = true;
                    break;
                }
            }

            retries -= 1;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        if !ready {
            return Err(TestHarnessError::EnvironmentError(format!(
                "Container {} did not become ready in time",
                container_id
            )));
        }

        info!("Container {} is ready", container_id);
        Ok(())
    }
}

#[async_trait]
impl Environment for DockerEnvironment {
    fn env_type(&self) -> EnvironmentType {
        EnvironmentType::Docker
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
        info!("Setting up Docker environment: {}", self.name());

        // Create artifacts directory
        tokio::fs::create_dir_all(&self.artifacts_dir)
            .await
            .map_err(TestHarnessError::IoError)?;

        // Create logs directory
        tokio::fs::create_dir_all(&self.logs_dir)
            .await
            .map_err(TestHarnessError::IoError)?;

        // Start Docker Compose services
        if !self.docker_config.use_existing {
            self.start_docker_compose().await?;
        } else {
            // Update the list of running containers
            self.update_running_containers().await?;
        }

        // Wait for all containers to be ready
        for (_, container_id) in &self.running_containers {
            self.wait_for_container(container_id, 60).await?;
        }

        Ok(())
    }

    async fn teardown(&mut self) -> Result<(), TestHarnessError> {
        info!("Tearing down Docker environment: {}", self.name());

        // Stop Docker Compose services
        if !self.docker_config.use_existing && self.config.cleanup {
            self.stop_docker_compose().await?;
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
        if let Some(container_id) = self.running_containers.get(service) {
            self.wait_for_container(container_id, timeout_secs).await?;
        } else {
            return Err(TestHarnessError::EnvironmentError(format!(
                "Service {} not found",
                service
            )));
        }

        Ok(())
    }

    async fn wait_for_all_services(&self, timeout_secs: u64) -> Result<(), TestHarnessError> {
        for (_, container_id) in &self.running_containers {
            self.wait_for_container(container_id, timeout_secs).await?;
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
        self.config.database_url.as_deref()
    }

    fn redis_url(&self) -> Option<&str> {
        self.config.redis_url.as_deref()
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
        if let Some(container_id) = self.running_containers.get(service) {
            let output = TokioCommand::new("docker")
                .arg("inspect")
                .arg("--format")
                .arg("{{.State.Health.Status}}")
                .arg(container_id)
                .output()
                .await
                .map_err(|e| {
                    TestHarnessError::EnvironmentError(format!(
                        "Failed to check container health: {}",
                        e
                    ))
                })?;

            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout).trim();
                return Ok(status == "healthy");
            }

            // If the container doesn't have a health check, check if it's running
            let output = TokioCommand::new("docker")
                .arg("inspect")
                .arg("--format")
                .arg("{{.State.Running}}")
                .arg(container_id)
                .output()
                .await
                .map_err(|e| {
                    TestHarnessError::EnvironmentError(format!(
                        "Failed to check container status: {}",
                        e
                    ))
                })?;

            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout).trim();
                return Ok(status == "true");
            }
        }

        Ok(false)
    }

    async fn get_service_logs(
        &self,
        service: &str,
        lines: Option<usize>,
    ) -> Result<String, TestHarnessError> {
        if let Some(container_id) = self.running_containers.get(service) {
            let mut command = TokioCommand::new("docker");
            command.arg("logs").arg(container_id);

            if let Some(lines) = lines {
                command.arg("--tail").arg(lines.to_string());
            }

            let output = command.output().await.map_err(|e| {
                TestHarnessError::EnvironmentError(format!("Failed to get container logs: {}", e))
            })?;

            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        Ok(String::new())
    }

    async fn get_resource_usage(&self) -> Result<ResourceUsage, TestHarnessError> {
        let mut services = HashMap::new();

        for (name, container_id) in &self.running_containers {
            // Get CPU usage
            let cpu_output = TokioCommand::new("docker")
                .arg("stats")
                .arg("--no-stream")
                .arg("--format")
                .arg("{{.CPUPerc}}")
                .arg(container_id)
                .output()
                .await
                .map_err(|e| {
                    TestHarnessError::EnvironmentError(format!(
                        "Failed to get CPU usage for container {}: {}",
                        container_id, e
                    ))
                })?;

            let cpu_usage = if cpu_output.status.success() {
                let cpu_str = String::from_utf8_lossy(&cpu_output.stdout)
                    .trim()
                    .to_string();
                cpu_str.trim_end_matches('%').parse::<f64>().unwrap_or(0.0)
            } else {
                0.0
            };

            // Get memory usage
            let memory_output = TokioCommand::new("docker")
                .arg("stats")
                .arg("--no-stream")
                .arg("--format")
                .arg("{{.MemUsage}}")
                .arg(container_id)
                .output()
                .await
                .map_err(|e| {
                    TestHarnessError::EnvironmentError(format!(
                        "Failed to get memory usage for container {}: {}",
                        container_id, e
                    ))
                })?;

            let memory_usage = if memory_output.status.success() {
                let memory_str = String::from_utf8_lossy(&memory_output.stdout)
                    .trim()
                    .to_string();
                // Parse memory usage (e.g., "1.5GiB / 15.5GiB")
                let memory_parts: Vec<&str> = memory_str.split('/').collect();
                if !memory_parts.is_empty() {
                    let memory_value = memory_parts[0].trim();
                    if memory_value.ends_with("GiB") {
                        memory_value
                            .trim_end_matches("GiB")
                            .parse::<f64>()
                            .unwrap_or(0.0)
                            * 1024.0
                            * 1024.0
                            * 1024.0
                    } else if memory_value.ends_with("MiB") {
                        memory_value
                            .trim_end_matches("MiB")
                            .parse::<f64>()
                            .unwrap_or(0.0)
                            * 1024.0
                            * 1024.0
                    } else if memory_value.ends_with("KiB") {
                        memory_value
                            .trim_end_matches("KiB")
                            .parse::<f64>()
                            .unwrap_or(0.0)
                            * 1024.0
                    } else {
                        memory_value.parse::<f64>().unwrap_or(0.0)
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            } as u64;

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

        Ok(ResourceUsage {
            cpu_usage: services.values().map(|s| s.cpu_usage).sum(),
            memory_usage: services.values().map(|s| s.memory_usage).sum(),
            disk_usage: 0,
            network_rx: 0,
            network_tx: 0,
            services,
        })
    }

    async fn take_snapshot(&self, _name: &str) -> Result<(), TestHarnessError> {
        // Docker environments don't support snapshots yet
        Err(TestHarnessError::EnvironmentError(
            "Snapshots are not supported for Docker environments".to_string(),
        ))
    }

    async fn restore_snapshot(&self, _name: &str) -> Result<(), TestHarnessError> {
        // Docker environments don't support snapshots yet
        Err(TestHarnessError::EnvironmentError(
            "Snapshots are not supported for Docker environments".to_string(),
        ))
    }

    async fn list_snapshots(&self) -> Result<Vec<String>, TestHarnessError> {
        // Docker environments don't support snapshots yet
        Ok(Vec::new())
    }
}
