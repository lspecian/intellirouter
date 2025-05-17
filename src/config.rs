// Configuration loading and validation
//
// This module handles loading configuration from various sources
// (environment variables, config files, command-line arguments)
// and validating the configuration values.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use config::{Config as ConfigFile, Environment as ConfigEnvironment, File};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use toml;

use crate::modules::telemetry::LogLevel;

/// Environment type for configuration profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AppEnvironment {
    Development,
    Testing,
    Production,
}

impl Default for AppEnvironment {
    fn default() -> Self {
        AppEnvironment::Development
    }
}

impl FromStr for AppEnvironment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(AppEnvironment::Development),
            "testing" | "test" => Ok(AppEnvironment::Testing),
            "production" | "prod" => Ok(AppEnvironment::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: IpAddr,
    /// Port to listen on
    pub port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Enable CORS
    pub cors_enabled: bool,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 8080,
            max_connections: 1000,
            request_timeout_secs: 30,
            cors_enabled: false,
            cors_allowed_origins: vec!["*".to_string()],
        }
    }
}

impl ServerConfig {
    /// Get the socket address for the server
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmProviderConfig {
    /// Provider name
    pub name: String,
    /// API key environment variable name
    pub api_key_env: String,
    /// API endpoint
    pub endpoint: String,
    /// Default model to use
    pub default_model: String,
    /// Available models
    pub available_models: Vec<String>,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Additional provider-specific settings
    pub settings: HashMap<String, String>,
}

/// Model registry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelRegistryConfig {
    /// Default provider to use
    pub default_provider: String,
    /// Available providers
    pub providers: Vec<LlmProviderConfig>,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for ModelRegistryConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            providers: vec![
                LlmProviderConfig {
                    name: "openai".to_string(),
                    api_key_env: "OPENAI_API_KEY".to_string(),
                    endpoint: "https://api.openai.com/v1".to_string(),
                    default_model: "gpt-4o".to_string(),
                    available_models: vec![
                        "gpt-4o".to_string(),
                        "gpt-4-turbo".to_string(),
                        "gpt-3.5-turbo".to_string(),
                    ],
                    timeout_secs: 60,
                    max_retries: 3,
                    settings: HashMap::new(),
                },
                LlmProviderConfig {
                    name: "anthropic".to_string(),
                    api_key_env: "ANTHROPIC_API_KEY".to_string(),
                    endpoint: "https://api.anthropic.com/v1".to_string(),
                    default_model: "claude-3-opus-20240229".to_string(),
                    available_models: vec![
                        "claude-3-opus-20240229".to_string(),
                        "claude-3-sonnet-20240229".to_string(),
                        "claude-3-haiku-20240307".to_string(),
                    ],
                    timeout_secs: 60,
                    max_retries: 3,
                    settings: HashMap::new(),
                },
            ],
            cache_ttl_secs: 3600,
        }
    }
}

/// Router configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouterConfig {
    /// Default routing strategy
    pub default_strategy: String,
    /// Available routing strategies
    pub available_strategies: Vec<String>,
    /// Routing rules
    pub rules: HashMap<String, String>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_strategy: "cost-optimized".to_string(),
            available_strategies: vec![
                "cost-optimized".to_string(),
                "performance-optimized".to_string(),
                "round-robin".to_string(),
                "fallback".to_string(),
            ],
            rules: HashMap::new(),
        }
    }
}

/// Memory backend configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Memory backend type
    pub backend_type: String,
    /// Redis connection string (if using Redis backend)
    pub redis_url: Option<String>,
    /// File storage path (if using file backend)
    pub file_path: Option<String>,
    /// Maximum conversation history length
    pub max_history_length: usize,
    /// TTL for conversation history in seconds
    pub history_ttl_secs: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            backend_type: "memory".to_string(),
            redis_url: None,
            file_path: None,
            max_history_length: 100,
            history_ttl_secs: 86400, // 24 hours
        }
    }
}

/// Telemetry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    /// Log level
    pub log_level: String,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    /// Metrics endpoint
    pub metrics_endpoint: Option<String>,
    /// Tracing endpoint
    pub tracing_endpoint: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            metrics_enabled: true,
            tracing_enabled: true,
            metrics_endpoint: None,
            tracing_endpoint: None,
        }
    }
}

impl TelemetryConfig {
    /// Convert string log level to LogLevel enum
    pub fn log_level(&self) -> Result<LogLevel, String> {
        match self.log_level.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warning" | "warn" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("Invalid log level: {}", self.log_level)),
        }
    }
}

/// Authentication and authorization configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub auth_enabled: bool,
    /// Authentication method
    pub auth_method: String,
    /// JWT secret (if using JWT authentication)
    pub jwt_secret: Option<String>,
    /// JWT expiration time in seconds
    pub jwt_expiration_secs: u64,
    /// API key header name (if using API key authentication)
    pub api_key_header: String,
    /// Valid API keys
    pub api_keys: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_enabled: false,
            auth_method: "api_key".to_string(),
            jwt_secret: None,
            jwt_expiration_secs: 3600, // 1 hour
            api_key_header: "X-API-Key".to_string(),
            api_keys: vec![],
        }
    }
}

/// RAG configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RagConfig {
    /// Enable RAG
    pub enabled: bool,
    /// Vector database connection string
    pub vector_db_url: Option<String>,
    /// Default embedding model
    pub default_embedding_model: String,
    /// Chunk size for text splitting
    pub chunk_size: usize,
    /// Chunk overlap
    pub chunk_overlap: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            vector_db_url: None,
            default_embedding_model: "text-embedding-3-small".to_string(),
            chunk_size: 1000,
            chunk_overlap: 200,
        }
    }
}

/// Chain engine configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainEngineConfig {
    /// Maximum chain length
    pub max_chain_length: usize,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: u64,
    /// Enable chain caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for ChainEngineConfig {
    fn default() -> Self {
        Self {
            max_chain_length: 10,
            max_execution_time_secs: 300, // 5 minutes
            enable_caching: true,
            cache_ttl_secs: 3600, // 1 hour
        }
    }
}

/// Persona layer configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersonaLayerConfig {
    /// Enable persona layer
    pub enabled: bool,
    /// Default persona
    pub default_persona: String,
    /// Persona definitions directory
    pub personas_dir: String,
}

impl Default for PersonaLayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_persona: "default".to_string(),
            personas_dir: "config/personas".to_string(),
        }
    }
}

/// Plugin SDK configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginSdkConfig {
    /// Enable plugins
    pub enabled: bool,
    /// Plugin directory
    pub plugins_dir: String,
    /// Allowed plugin hosts
    pub allowed_hosts: Vec<String>,
    /// Plugin timeout in seconds
    pub timeout_secs: u64,
}

impl Default for PluginSdkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            plugins_dir: "plugins".to_string(),
            allowed_hosts: vec![],
            timeout_secs: 30,
        }
    }
}

/// Main configuration structure for IntelliRouter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Environment (development, testing, production)
    pub environment: AppEnvironment,
    /// Server configuration
    pub server: ServerConfig,
    /// Model registry configuration
    pub model_registry: ModelRegistryConfig,
    /// Router configuration
    pub router: RouterConfig,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Telemetry configuration
    pub telemetry: TelemetryConfig,
    /// Authentication and authorization configuration
    pub auth: AuthConfig,
    /// RAG configuration
    pub rag: RagConfig,
    /// Chain engine configuration
    pub chain_engine: ChainEngineConfig,
    /// Persona layer configuration
    pub persona_layer: PersonaLayerConfig,
    /// Plugin SDK configuration
    pub plugin_sdk: PluginSdkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            environment: AppEnvironment::default(),
            server: ServerConfig::default(),
            model_registry: ModelRegistryConfig::default(),
            router: RouterConfig::default(),
            memory: MemoryConfig::default(),
            telemetry: TelemetryConfig::default(),
            auth: AuthConfig::default(),
            rag: RagConfig::default(),
            chain_engine: ChainEngineConfig::default(),
            persona_layer: PersonaLayerConfig::default(),
            plugin_sdk: PluginSdkConfig::default(),
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        // Load .env file if it exists
        let _ = dotenv();

        // Create a default configuration
        let mut config = Self::default();

        // Update configuration from environment variables
        if let Ok(env) = env::var("INTELLIROUTER_ENVIRONMENT") {
            config.environment = AppEnvironment::from_str(&env)
                .map_err(|e| format!("Invalid environment: {}", e))?;
        }

        // Try to load from config file based on environment
        let config_file = match config.environment {
            AppEnvironment::Development => "config/development.toml",
            AppEnvironment::Testing => "config/testing.toml",
            AppEnvironment::Production => "config/production.toml",
        };

        if Path::new(config_file).exists() {
            match Self::from_file(config_file) {
                Ok(file_config) => config = file_config,
                Err(e) => return Err(format!("Failed to load config file: {}", e)),
            }
        }

        // Override with environment variables
        Self::update_from_env_vars(&mut config)?;

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a file
    pub fn from_file(path: &str) -> Result<Self, String> {
        let config_file = ConfigFile::builder()
            .add_source(File::with_name(path))
            .build()
            .map_err(|e| format!("Failed to load config file: {}", e))?;

        let config: Self = config_file
            .try_deserialize()
            .map_err(|e| format!("Failed to deserialize config: {}", e))?;

        Ok(config)
    }

    /// Load configuration from multiple sources with proper precedence
    pub fn load() -> Result<Self, String> {
        // Load .env file if it exists
        let _ = dotenv();

        // Determine environment
        let environment = env::var("INTELLIROUTER_ENVIRONMENT")
            .map(|env| AppEnvironment::from_str(&env))
            .unwrap_or(Ok(AppEnvironment::default()))
            .map_err(|e| format!("Invalid environment: {}", e))?;

        // Create config builder
        let mut builder = ConfigFile::builder();

        // Add default config
        let default_config_path = "config/default.toml";
        if Path::new(default_config_path).exists() {
            builder = builder.add_source(File::with_name(default_config_path));
        }

        // Add environment-specific config
        let env_config_path = match environment {
            AppEnvironment::Development => "config/development.toml",
            AppEnvironment::Testing => "config/testing.toml",
            AppEnvironment::Production => "config/production.toml",
        };

        if Path::new(env_config_path).exists() {
            builder = builder.add_source(File::with_name(env_config_path));
        }

        // Add local config (not version controlled)
        let local_config_path = "config/local.toml";
        if Path::new(local_config_path).exists() {
            builder = builder.add_source(File::with_name(local_config_path));
        }

        // Add environment variables with prefix
        builder = builder.add_source(
            ConfigEnvironment::with_prefix("INTELLIROUTER")
                .separator("__")
                .try_parsing(true),
        );

        // Build config
        let config_file = builder
            .build()
            .map_err(|e| format!("Failed to build config: {}", e))?;

        // Deserialize
        let mut config: Self = config_file
            .try_deserialize()
            .map_err(|e| format!("Failed to deserialize config: {}", e))?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Update configuration from environment variables
    fn update_from_env_vars(config: &mut Self) -> Result<(), String> {
        // Server config
        if let Ok(host) = env::var("INTELLIROUTER__SERVER__HOST") {
            config.server.host =
                IpAddr::from_str(&host).map_err(|e| format!("Invalid host address: {}", e))?;
        }

        if let Ok(port) = env::var("INTELLIROUTER__SERVER__PORT") {
            config.server.port = port
                .parse::<u16>()
                .map_err(|e| format!("Invalid port: {}", e))?;
        }

        // Telemetry config
        if let Ok(log_level) = env::var("INTELLIROUTER__TELEMETRY__LOG_LEVEL") {
            config.telemetry.log_level = log_level;
        }

        if let Ok(metrics_enabled) = env::var("INTELLIROUTER__TELEMETRY__METRICS_ENABLED") {
            config.telemetry.metrics_enabled = metrics_enabled.to_lowercase() == "true";
        }

        // Memory config
        if let Ok(backend_type) = env::var("INTELLIROUTER__MEMORY__BACKEND_TYPE") {
            config.memory.backend_type = backend_type;
        }

        if let Ok(redis_url) = env::var("INTELLIROUTER__MEMORY__REDIS_URL") {
            config.memory.redis_url = Some(redis_url);
        }

        // Model registry config
        if let Ok(default_provider) = env::var("INTELLIROUTER__MODEL_REGISTRY__DEFAULT_PROVIDER") {
            config.model_registry.default_provider = default_provider;
        }

        // Auth config
        if let Ok(auth_enabled) = env::var("INTELLIROUTER__AUTH__AUTH_ENABLED") {
            config.auth.auth_enabled = auth_enabled.to_lowercase() == "true";
        }

        if let Ok(jwt_secret) = env::var("INTELLIROUTER__AUTH__JWT_SECRET") {
            config.auth.jwt_secret = Some(jwt_secret);
        }

        // RAG config
        if let Ok(rag_enabled) = env::var("INTELLIROUTER__RAG__ENABLED") {
            config.rag.enabled = rag_enabled.to_lowercase() == "true";
        }

        if let Ok(vector_db_url) = env::var("INTELLIROUTER__RAG__VECTOR_DB_URL") {
            config.rag.vector_db_url = Some(vector_db_url);
        }

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server config
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        // Validate telemetry config
        self.telemetry.log_level().map_err(|e| e)?;

        // Validate model registry config
        if self.model_registry.providers.is_empty() {
            return Err("At least one LLM provider must be configured".to_string());
        }

        let default_provider_exists = self
            .model_registry
            .providers
            .iter()
            .any(|p| p.name == self.model_registry.default_provider);

        if !default_provider_exists {
            return Err(format!(
                "Default provider '{}' not found in configured providers",
                self.model_registry.default_provider
            ));
        }

        // Validate memory config
        match self.memory.backend_type.as_str() {
            "memory" => {
                // No additional validation needed for memory backend
            }
            "redis" => {
                if self.memory.redis_url.is_none() {
                    return Err("Redis URL must be provided for Redis memory backend".to_string());
                }
            }
            "file" => {
                if self.memory.file_path.is_none() {
                    return Err("File path must be provided for file memory backend".to_string());
                }
            }
            _ => {
                return Err(format!(
                    "Unknown memory backend type: {}",
                    self.memory.backend_type
                ));
            }
        }

        // Validate auth config
        if self.auth.auth_enabled {
            match self.auth.auth_method.as_str() {
                "jwt" => {
                    if self.auth.jwt_secret.is_none()
                        || self.auth.jwt_secret.as_ref().unwrap().is_empty()
                    {
                        return Err(
                            "JWT secret must be provided for JWT authentication".to_string()
                        );
                    }
                }
                "api_key" => {
                    if self.auth.api_keys.is_empty() {
                        return Err(
                            "At least one API key must be provided for API key authentication"
                                .to_string(),
                        );
                    }
                }
                _ => {
                    return Err(format!(
                        "Unknown authentication method: {}",
                        self.auth.auth_method
                    ));
                }
            }
        }

        // Validate RAG config
        if self.rag.enabled && self.rag.vector_db_url.is_none() {
            return Err("Vector database URL must be provided when RAG is enabled".to_string());
        }

        Ok(())
    }

    /// Get the configuration for a specific environment
    pub fn for_environment(environment: AppEnvironment) -> Result<Self, String> {
        let mut config = Self::default();
        config.environment = environment;

        // Try to load from config file based on environment
        let config_file = match environment {
            AppEnvironment::Development => "config/development.toml",
            AppEnvironment::Testing => "config/testing.toml",
            AppEnvironment::Production => "config/production.toml",
        };

        if Path::new(config_file).exists() {
            match Self::from_file(config_file) {
                Ok(file_config) => config = file_config,
                Err(e) => return Err(format!("Failed to load config file: {}", e)),
            }
        }

        // Override with environment variables
        Self::update_from_env_vars(&mut config)?;

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Save the configuration to a file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let toml = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, toml).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Create default configuration files
    pub fn create_default_configs() -> Result<(), String> {
        // Create config directory if it doesn't exist
        if !Path::new("config").exists() {
            fs::create_dir("config")
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Create default config
        let default_config = Self::default();
        default_config.save_to_file("config/default.toml")?;

        // Create environment-specific configs
        let mut dev_config = Self::default();
        dev_config.environment = AppEnvironment::Development;
        dev_config.telemetry.log_level = "debug".to_string();
        dev_config.save_to_file("config/development.toml")?;

        let mut test_config = Self::default();
        test_config.environment = AppEnvironment::Testing;
        test_config.save_to_file("config/testing.toml")?;

        let mut prod_config = Self::default();
        prod_config.environment = AppEnvironment::Production;
        prod_config.server.host = IpAddr::from_str("0.0.0.0").unwrap();
        prod_config.telemetry.log_level = "info".to_string();
        prod_config.auth.auth_enabled = true;
        prod_config.save_to_file("config/production.toml")?;

        Ok(())
    }
}
