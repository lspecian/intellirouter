//! Environment Template Module
//!
//! This module provides functionality for creating and managing environment templates
//! to enable faster provisioning of test environments.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::config::{
    DockerConfig, EnvironmentConfig, EnvironmentType, KubernetesConfig, LocalConfig, RemoteConfig,
    ResourceRequirements,
};
use super::{Environment, EnvironmentFactory};
use crate::modules::test_harness::types::TestHarnessError;

/// Environment template for creating environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: Option<String>,
    /// Base environment configuration
    pub base_config: EnvironmentConfig,
    /// Template variables
    pub variables: HashMap<String, String>,
    /// Template parameters
    pub parameters: HashMap<String, TemplateParameter>,
    /// Template hooks
    pub hooks: TemplateHooks,
    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Template tags
    pub tags: Vec<String>,
    /// Template version
    pub version: String,
    /// Template creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Template last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Template parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: Option<String>,
    /// Parameter type
    pub param_type: TemplateParameterType,
    /// Default value
    pub default_value: Option<serde_json::Value>,
    /// Required parameter
    pub required: bool,
    /// Parameter validation
    pub validation: Option<TemplateParameterValidation>,
}

/// Template parameter type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateParameterType {
    /// String parameter
    String,
    /// Integer parameter
    Integer,
    /// Float parameter
    Float,
    /// Boolean parameter
    Boolean,
    /// Array parameter
    Array,
    /// Object parameter
    Object,
    /// Enum parameter
    Enum(Vec<String>),
}

/// Template parameter validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameterValidation {
    /// Minimum value for numeric parameters
    pub min: Option<f64>,
    /// Maximum value for numeric parameters
    pub max: Option<f64>,
    /// Minimum length for string parameters
    pub min_length: Option<usize>,
    /// Maximum length for string parameters
    pub max_length: Option<usize>,
    /// Pattern for string parameters
    pub pattern: Option<String>,
    /// Allowed values for enum parameters
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

/// Template hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateHooks {
    /// Pre-provision hook
    pub pre_provision: Option<String>,
    /// Post-provision hook
    pub post_provision: Option<String>,
    /// Pre-teardown hook
    pub pre_teardown: Option<String>,
    /// Post-teardown hook
    pub post_teardown: Option<String>,
    /// Health check hook
    pub health_check: Option<String>,
    /// Self-healing hook
    pub self_healing: Option<String>,
}

/// Environment template manager
pub struct EnvironmentTemplateManager {
    /// Templates directory
    templates_dir: PathBuf,
    /// Templates cache
    templates: Arc<RwLock<HashMap<String, EnvironmentTemplate>>>,
    /// Environment cache
    environment_cache: Arc<RwLock<HashMap<String, CachedEnvironment>>>,
}

/// Cached environment
struct CachedEnvironment {
    /// Environment ID
    id: String,
    /// Environment template ID
    template_id: String,
    /// Environment parameters
    parameters: HashMap<String, serde_json::Value>,
    /// Environment instance
    environment: Box<dyn Environment>,
    /// Last used timestamp
    last_used: chrono::DateTime<chrono::Utc>,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Cache expiration timestamp
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Environment cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentCacheStats {
    /// Total number of cached environments
    pub total: usize,
    /// Number of active cached environments
    pub active: usize,
    /// Number of expired cached environments
    pub expired: usize,
    /// Average age of cached environments in seconds
    pub avg_age_seconds: f64,
}

impl EnvironmentTemplateManager {
    /// Create a new environment template manager
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
            templates: Arc::new(RwLock::new(HashMap::new())),
            environment_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the template manager
    pub async fn initialize(&self) -> Result<(), TestHarnessError> {
        // Create templates directory if it doesn't exist
        if !self.templates_dir.exists() {
            fs::create_dir_all(&self.templates_dir)
                .await
                .map_err(TestHarnessError::IoError)?;
        }

        // Load templates
        self.load_templates().await?;

        Ok(())
    }

    /// Load templates from the templates directory
    async fn load_templates(&self) -> Result<(), TestHarnessError> {
        let mut templates = self.templates.write().await;
        templates.clear();

        // Read template files
        let mut entries = fs::read_dir(&self.templates_dir)
            .await
            .map_err(TestHarnessError::IoError)?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(TestHarnessError::IoError)?
        {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let template_json = fs::read_to_string(&path)
                    .await
                    .map_err(TestHarnessError::IoError)?;

                let template: EnvironmentTemplate =
                    serde_json::from_str(&template_json).map_err(|e| {
                        TestHarnessError::ConfigurationError(format!(
                            "Failed to parse template {}: {}",
                            path.display(),
                            e
                        ))
                    })?;

                templates.insert(template.id.clone(), template);
            }
        }

        info!("Loaded {} environment templates", templates.len());

        Ok(())
    }

    /// Save a template
    pub async fn save_template(
        &self,
        template: &EnvironmentTemplate,
    ) -> Result<(), TestHarnessError> {
        let template_path = self.templates_dir.join(format!("{}.json", template.id));
        let template_json =
            serde_json::to_string_pretty(template).map_err(TestHarnessError::SerializationError)?;

        fs::write(&template_path, template_json)
            .await
            .map_err(TestHarnessError::IoError)?;

        // Update cache
        let mut templates = self.templates.write().await;
        templates.insert(template.id.clone(), template.clone());

        Ok(())
    }

    /// Get a template by ID
    pub async fn get_template(&self, id: &str) -> Option<EnvironmentTemplate> {
        let templates = self.templates.read().await;
        templates.get(id).cloned()
    }

    /// List all templates
    pub async fn list_templates(&self) -> Vec<EnvironmentTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }

    /// Delete a template
    pub async fn delete_template(&self, id: &str) -> Result<(), TestHarnessError> {
        let template_path = self.templates_dir.join(format!("{}.json", id));

        if template_path.exists() {
            fs::remove_file(&template_path)
                .await
                .map_err(TestHarnessError::IoError)?;
        }

        // Update cache
        let mut templates = self.templates.write().await;
        templates.remove(id);

        Ok(())
    }

    /// Create an environment from a template
    pub async fn create_environment_from_template(
        &self,
        template_id: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<Box<dyn Environment>, TestHarnessError> {
        // Check if we have a cached environment
        let cache_key = self.generate_cache_key(template_id, &parameters);

        {
            let mut env_cache = self.environment_cache.write().await;
            if let Some(cached_env) = env_cache.get_mut(&cache_key) {
                // Check if the cached environment is still valid
                if cached_env
                    .expires_at
                    .map_or(true, |exp| exp > chrono::Utc::now())
                {
                    // Update last used timestamp
                    cached_env.last_used = chrono::Utc::now();

                    info!("Using cached environment for template {}", template_id);

                    // Return a clone of the cached environment
                    return Ok(cached_env.environment.clone());
                } else {
                    // Remove expired environment
                    env_cache.remove(&cache_key);
                }
            }
        }

        // Get the template
        let template = self.get_template(template_id).await.ok_or_else(|| {
            TestHarnessError::ConfigurationError(format!("Template {} not found", template_id))
        })?;

        // Validate parameters
        self.validate_parameters(&template, &parameters)?;

        // Apply parameters to the template
        let config = self.apply_parameters_to_config(&template, &parameters)?;

        // Create the environment
        let environment = EnvironmentFactory::create_environment(config).await?;

        // Execute pre-provision hook if defined
        if let Some(hook) = &template.hooks.pre_provision {
            self.execute_hook(hook, &environment).await?;
        }

        // Set up the environment
        let mut env = environment.clone();
        env.setup().await?;

        // Execute post-provision hook if defined
        if let Some(hook) = &template.hooks.post_provision {
            self.execute_hook(hook, &env).await?;
        }

        // Cache the environment
        let now = chrono::Utc::now();
        let cached_env = CachedEnvironment {
            id: env.id().to_string(),
            template_id: template_id.to_string(),
            parameters: parameters.clone(),
            environment: env.clone(),
            last_used: now,
            created_at: now,
            expires_at: Some(now + chrono::Duration::hours(1)), // Cache for 1 hour by default
        };

        let mut env_cache = self.environment_cache.write().await;
        env_cache.insert(cache_key, cached_env);

        Ok(env)
    }

    /// Generate a cache key for a template and parameters
    fn generate_cache_key(
        &self,
        template_id: &str,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> String {
        let mut params_str = parameters
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();
        params_str.sort(); // Sort for consistent keys

        format!("{}:{}", template_id, params_str.join(","))
    }

    /// Validate parameters against a template
    fn validate_parameters(
        &self,
        template: &EnvironmentTemplate,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<(), TestHarnessError> {
        for (name, param) in &template.parameters {
            if param.required && !parameters.contains_key(name) && param.default_value.is_none() {
                return Err(TestHarnessError::ConfigurationError(format!(
                    "Required parameter {} is missing",
                    name
                )));
            }

            if let Some(value) = parameters.get(name) {
                if let Some(validation) = &param.validation {
                    match param.param_type {
                        TemplateParameterType::String => {
                            if let Some(s) = value.as_str() {
                                if let Some(min_len) = validation.min_length {
                                    if s.len() < min_len {
                                        return Err(TestHarnessError::ConfigurationError(format!(
                                            "Parameter {} must be at least {} characters long",
                                            name, min_len
                                        )));
                                    }
                                }

                                if let Some(max_len) = validation.max_length {
                                    if s.len() > max_len {
                                        return Err(TestHarnessError::ConfigurationError(format!(
                                            "Parameter {} must be at most {} characters long",
                                            name, max_len
                                        )));
                                    }
                                }

                                if let Some(pattern) = &validation.pattern {
                                    let regex = regex::Regex::new(pattern).map_err(|e| {
                                        TestHarnessError::ConfigurationError(format!(
                                            "Invalid regex pattern for parameter {}: {}",
                                            name, e
                                        ))
                                    })?;

                                    if !regex.is_match(s) {
                                        return Err(TestHarnessError::ConfigurationError(format!(
                                            "Parameter {} must match pattern {}",
                                            name, pattern
                                        )));
                                    }
                                }
                            } else {
                                return Err(TestHarnessError::ConfigurationError(format!(
                                    "Parameter {} must be a string",
                                    name
                                )));
                            }
                        }
                        TemplateParameterType::Integer | TemplateParameterType::Float => {
                            let num = if let Some(n) = value.as_i64() {
                                n as f64
                            } else if let Some(n) = value.as_f64() {
                                n
                            } else {
                                return Err(TestHarnessError::ConfigurationError(format!(
                                    "Parameter {} must be a number",
                                    name
                                )));
                            };

                            if let Some(min) = validation.min {
                                if num < min {
                                    return Err(TestHarnessError::ConfigurationError(format!(
                                        "Parameter {} must be at least {}",
                                        name, min
                                    )));
                                }
                            }

                            if let Some(max) = validation.max {
                                if num > max {
                                    return Err(TestHarnessError::ConfigurationError(format!(
                                        "Parameter {} must be at most {}",
                                        name, max
                                    )));
                                }
                            }
                        }
                        TemplateParameterType::Enum(ref allowed) => {
                            if let Some(s) = value.as_str() {
                                if !allowed.contains(&s.to_string()) {
                                    return Err(TestHarnessError::ConfigurationError(format!(
                                        "Parameter {} must be one of: {}",
                                        name,
                                        allowed.join(", ")
                                    )));
                                }
                            } else {
                                return Err(TestHarnessError::ConfigurationError(format!(
                                    "Parameter {} must be a string",
                                    name
                                )));
                            }
                        }
                        _ => {} // Other types don't have specific validations yet
                    }

                    if let Some(allowed_values) = &validation.allowed_values {
                        if !allowed_values.contains(value) {
                            return Err(TestHarnessError::ConfigurationError(format!(
                                "Parameter {} must be one of the allowed values",
                                name
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply parameters to a template configuration
    fn apply_parameters_to_config(
        &self,
        template: &EnvironmentTemplate,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<EnvironmentConfig, TestHarnessError> {
        let mut config = template.base_config.clone();

        // Apply parameters
        for (name, param) in &template.parameters {
            let value = if let Some(value) = parameters.get(name) {
                value.clone()
            } else if let Some(default) = &param.default_value {
                default.clone()
            } else {
                continue;
            };

            // Apply parameter to config based on name
            match name.as_str() {
                "env_type" => {
                    if let Some(env_type) = value.as_str() {
                        config.env_type = match env_type {
                            "local" => EnvironmentType::Local,
                            "docker" => EnvironmentType::Docker,
                            "kubernetes" => EnvironmentType::Kubernetes,
                            "remote" => EnvironmentType::Remote,
                            _ => {
                                return Err(TestHarnessError::ConfigurationError(format!(
                                    "Invalid environment type: {}",
                                    env_type
                                )))
                            }
                        };
                    }
                }
                "name" => {
                    if let Some(name) = value.as_str() {
                        config.name = name.to_string();
                    }
                }
                "description" => {
                    if let Some(description) = value.as_str() {
                        config.description = Some(description.to_string());
                    }
                }
                "base_dir" => {
                    if let Some(base_dir) = value.as_str() {
                        config.base_dir = PathBuf::from(base_dir);
                    }
                }
                "use_temp_dir" => {
                    if let Some(use_temp_dir) = value.as_bool() {
                        config.use_temp_dir = use_temp_dir;
                    }
                }
                "cleanup" => {
                    if let Some(cleanup) = value.as_bool() {
                        config.cleanup = cleanup;
                    }
                }
                "database_url" => {
                    if let Some(url) = value.as_str() {
                        config.database_url = Some(url.to_string());
                    }
                }
                "redis_url" => {
                    if let Some(url) = value.as_str() {
                        config.redis_url = Some(url.to_string());
                    }
                }
                "kafka_url" => {
                    if let Some(url) = value.as_str() {
                        config.kafka_url = Some(url.to_string());
                    }
                }
                "elasticsearch_url" => {
                    if let Some(url) = value.as_str() {
                        config.elasticsearch_url = Some(url.to_string());
                    }
                }
                "s3_url" => {
                    if let Some(url) = value.as_str() {
                        config.s3_url = Some(url.to_string());
                    }
                }
                _ => {
                    // Handle custom properties
                    if name.starts_with("property.") {
                        let property_name = name.strip_prefix("property.").unwrap();
                        config.properties.insert(property_name.to_string(), value);
                    }
                    // Handle environment variables
                    else if name.starts_with("env.") {
                        let env_name = name.strip_prefix("env.").unwrap();
                        if let Some(env_value) = value.as_str() {
                            config
                                .env_vars
                                .insert(env_name.to_string(), env_value.to_string());
                        }
                    }
                    // Handle services
                    else if name.starts_with("service.") {
                        let service_name = name.strip_prefix("service.").unwrap();
                        if let Some(service_url) = value.as_str() {
                            config
                                .services
                                .insert(service_name.to_string(), service_url.to_string());
                        }
                    }
                }
            }
        }

        // Generate a unique ID if not already set
        if config.id.is_empty() {
            config.id = uuid::Uuid::new_v4().to_string();
        }

        Ok(config)
    }

    /// Execute a template hook
    async fn execute_hook(
        &self,
        hook: &str,
        environment: &Box<dyn Environment>,
    ) -> Result<(), TestHarnessError> {
        // Execute the hook as a shell command
        let (stdout, stderr) = environment.execute_command("sh", &["-c", hook]).await?;

        if !stderr.is_empty() {
            warn!("Hook execution stderr: {}", stderr);
        }

        debug!("Hook execution stdout: {}", stdout);

        Ok(())
    }

    /// Clean up expired environments
    pub async fn cleanup_expired_environments(&self) -> Result<usize, TestHarnessError> {
        let now = chrono::Utc::now();
        let mut count = 0;

        let mut env_cache = self.environment_cache.write().await;
        let expired_keys: Vec<String> = env_cache
            .iter()
            .filter(|(_, cached_env)| cached_env.expires_at.map_or(false, |exp| exp <= now))
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(cached_env) = env_cache.remove(&key) {
                // Execute pre-teardown hook if defined
                let template = self.get_template(&cached_env.template_id).await;

                if let Some(template) = template {
                    if let Some(hook) = &template.hooks.pre_teardown {
                        let _ = self.execute_hook(hook, &cached_env.environment).await;
                    }
                }

                // Tear down the environment
                let mut env = cached_env.environment;
                let _ = env.teardown().await;

                // Execute post-teardown hook if defined
                if let Some(template) = template {
                    if let Some(hook) = &template.hooks.post_teardown {
                        let _ = self.execute_hook(hook, &env).await;
                    }
                }

                count += 1;
            }
        }

        Ok(count)
    }

    /// Get environment cache statistics
    pub async fn get_cache_stats(&self) -> EnvironmentCacheStats {
        let env_cache = self.environment_cache.read().await;
        let now = chrono::Utc::now();

        let total = env_cache.len();
        let expired = env_cache
            .values()
            .filter(|cached_env| cached_env.expires_at.map_or(false, |exp| exp <= now))
            .count();

        let active = total - expired;

        let avg_age = if total > 0 {
            let total_age: i64 = env_cache
                .values()
                .map(|cached_env| (now - cached_env.created_at).num_seconds())
                .sum();

            total_age as f64 / total as f64
        } else {
            0.0
        };

        EnvironmentCacheStats {
            total,
            active,
            expired,
            avg_age_seconds: avg_age,
        }
    }

    /// Check environment health and perform self-healing if needed
    pub async fn check_environment_health(
        &self,
        environment_id: &str,
    ) -> Result<bool, TestHarnessError> {
        let mut env_cache = self.environment_cache.write().await;

        // Find the environment in the cache
        let cached_env = env_cache
            .values_mut()
            .find(|cached_env| cached_env.id == environment_id);

        if let Some(cached_env) = cached_env {
            // Get the template
            let template = self.get_template(&cached_env.template_id).await;

            if let Some(template) = template {
                // Check if the environment is healthy
                let is_healthy = if let Some(hook) = &template.hooks.health_check {
                    // Execute health check hook
                    match self.execute_hook(hook, &cached_env.environment).await {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                } else {
                    // Default health check: check if all services are healthy
                    let mut all_healthy = true;

                    for service in cached_env.environment.config().services.keys() {
                        match cached_env.environment.is_service_healthy(service).await {
                            Ok(healthy) => {
                                if !healthy {
                                    all_healthy = false;
                                    break;
                                }
                            }
                            Err(_) => {
                                all_healthy = false;
                                break;
                            }
                        }
                    }

                    all_healthy
                };

                // If not healthy, try to heal
                if !is_healthy {
                    info!(
                        "Environment {} is not healthy, attempting self-healing",
                        environment_id
                    );

                    if let Some(hook) = &template.hooks.self_healing {
                        // Execute self-healing hook
                        match self.execute_hook(hook, &cached_env.environment).await {
                            Ok(_) => {
                                info!("Self-healing successful for environment {}", environment_id);
                                return Ok(true);
                            }
                            Err(e) => {
                                error!(
                                    "Self-healing failed for environment {}: {}",
                                    environment_id, e
                                );
                                return Ok(false);
                            }
                        }
                    } else {
                        // Default self-healing: restart services
                        let mut env = cached_env.environment.clone();

                        // Tear down and set up again
                        if let Err(e) = env.teardown().await {
                            error!("Failed to tear down environment {}: {}", environment_id, e);
                            return Ok(false);
                        }

                        if let Err(e) = env.setup().await {
                            error!("Failed to set up environment {}: {}", environment_id, e);
                            return Ok(false);
                        }

                        // Update the cached environment
                        cached_env.environment = env;

                        info!(
                            "Default self-healing successful for environment {}",
                            environment_id
                        );
                        return Ok(true);
                    }
                }

                return Ok(is_healthy);
            }
        }

        Err(TestHarnessError::EnvironmentError(format!(
            "Environment {} not found in cache",
            environment_id
        )))
    }
}

/// Environment template builder
pub struct EnvironmentTemplateBuilder {
    /// Template ID
    id: Option<String>,
    /// Template name
    name: String,
    /// Template description
    description: Option<String>,
    /// Base environment configuration
    base_config: EnvironmentConfig,
    /// Template variables
    variables: HashMap<String, String>,
    /// Template parameters
    parameters: HashMap<String, TemplateParameter>,
    /// Template hooks
    hooks: TemplateHooks,
    /// Template metadata
    metadata: HashMap<String, serde_json::Value>,
    /// Template tags
    tags: Vec<String>,
    /// Template version
    version: String,
}

impl EnvironmentTemplateBuilder {
    /// Create a new environment template builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: None,
            base_config: EnvironmentConfig::default(),
            variables: HashMap::new(),
            parameters: HashMap::new(),
            hooks: TemplateHooks {
                pre_provision: None,
                post_provision: None,
                pre_teardown: None,
                post_teardown: None,
                health_check: None,
                self_healing: None,
            },
            metadata: HashMap::new(),
            tags: Vec::new(),
            version: "1.0.0".to_string(),
        }
    }

    /// Set the template ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the template description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the base environment configuration
    pub fn with_base_config(mut self, config: EnvironmentConfig) -> Self {
        self.base_config = config;
        self
    }

    /// Add a template variable
    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add a template parameter
    pub fn with_parameter(mut self, parameter: TemplateParameter) -> Self {
        self.parameters.insert(parameter.name.clone(), parameter);
        self
    }

    /// Set the pre-provision hook
    pub fn with_pre_provision_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.pre_provision = Some(hook.into());
        self
    }

    /// Set the post-provision hook
    pub fn with_post_provision_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.post_provision = Some(hook.into());
        self
    }

    /// Set the pre-teardown hook
    pub fn with_pre_teardown_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.pre_teardown = Some(hook.into());
        self
    }

    /// Set the post-teardown hook
    pub fn with_post_teardown_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.post_teardown = Some(hook.into());
        self
    }

    /// Set the health check hook
    pub fn with_health_check_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.health_check = Some(hook.into());
        self
    }

    /// Set the self-healing hook
    pub fn with_self_healing_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.self_healing = Some(hook.into());
        self
    }

    /// Add a metadata entry
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, TestHarnessError> {
        let value = serde_json::to_value(value).map_err(TestHarnessError::SerializationError)?;
        self.metadata.insert(key.into(), value);
        Ok(self)
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set the template version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Build the environment template
    pub fn build(self) -> EnvironmentTemplate {
        let now = chrono::Utc::now();

        EnvironmentTemplate {
            id: self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: self.name,
            description: self.description,
            base_config: self.base_config,
            variables: self.variables,
            parameters: self.parameters,
            hooks: self.hooks,
            metadata: self.metadata,
            tags: self.tags,
            version: self.version,
            created_at: now,
            updated_at: now,
        }
    }
}

impl EnvironmentTemplateManager {
    /// Create a template from an environment configuration
    pub fn create_template_from_config(
        config: EnvironmentConfig,
        name: impl Into<String>,
        description: Option<String>,
    ) -> EnvironmentTemplate {
        let now = chrono::Utc::now();

        EnvironmentTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description,
            base_config: config,
            variables: HashMap::new(),
            parameters: HashMap::new(),
            hooks: TemplateHooks {
                pre_provision: None,
                post_provision: None,
                pre_teardown: None,
                post_teardown: None,
                health_check: None,
                self_healing: None,
            },
            metadata: HashMap::new(),
            tags: Vec::new(),
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}
