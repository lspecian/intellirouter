//! Plugin Registry
//!
//! This module provides a registry for managing plugins.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Error type for plugin operations
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to acquire lock")]
    LockError,

    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Invalid plugin configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Plugin loading failed: {0}")]
    LoadingFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Dynamic library error: {0}")]
    DylibError(String),
}

/// Plugin type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    RoutingStrategy,
    ModelConnector,
    TelemetryExporter,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::RoutingStrategy => write!(f, "routing_strategy"),
            PluginType::ModelConnector => write!(f, "model_connector"),
            PluginType::TelemetryExporter => write!(f, "telemetry_exporter"),
        }
    }
}

/// Base trait for all plugins
pub trait Plugin: Send + Sync {
    /// Get the name of the plugin
    fn name(&self) -> &str;

    /// Get the version of the plugin
    fn version(&self) -> &str;

    /// Get the type of the plugin
    fn plugin_type(&self) -> PluginType;

    /// Initialize the plugin with configuration
    fn initialize(&self, config: serde_json::Value) -> Result<(), PluginError>;

    /// Shutdown the plugin
    fn shutdown(&self) -> Result<(), PluginError>;
}

/// Registry for managing plugins
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().map_err(|_| PluginError::LockError)?;
        let key = format!("{}-{}", plugin.name(), plugin.version());

        if plugins.contains_key(&key) {
            return Err(PluginError::AlreadyRegistered(key));
        }

        plugins.insert(key, plugin);
        Ok(())
    }

    /// Get a plugin by name and version
    pub fn get_plugin(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<Arc<dyn Plugin>>, PluginError> {
        let plugins = self.plugins.read().map_err(|_| PluginError::LockError)?;
        let key = format!("{}-{}", name, version);
        Ok(plugins.get(&key).cloned())
    }

    /// Get the latest version of a plugin by name
    pub fn get_latest_plugin(&self, name: &str) -> Result<Option<Arc<dyn Plugin>>, PluginError> {
        let plugins = self.plugins.read().map_err(|_| PluginError::LockError)?;

        let mut latest_version = None;
        let mut latest_plugin = None;

        for (key, plugin) in plugins.iter() {
            if plugin.name() == name {
                if latest_version.is_none()
                    || semver::Version::parse(plugin.version()).map_or(false, |v| {
                        semver::Version::parse(latest_version.unwrap())
                            .map_or(true, |latest| v > latest)
                    })
                {
                    latest_version = Some(plugin.version());
                    latest_plugin = Some(plugin.clone());
                }
            }
        }

        Ok(latest_plugin)
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>, PluginError> {
        let plugins = self.plugins.read().map_err(|_| PluginError::LockError)?;
        Ok(plugins.values().cloned().collect())
    }

    /// List plugins by type
    pub fn list_plugins_by_type(
        &self,
        plugin_type: PluginType,
    ) -> Result<Vec<Arc<dyn Plugin>>, PluginError> {
        let plugins = self.plugins.read().map_err(|_| PluginError::LockError)?;
        Ok(plugins
            .values()
            .filter(|p| p.plugin_type() == plugin_type)
            .cloned()
            .collect())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&self, name: &str, version: &str) -> Result<bool, PluginError> {
        let mut plugins = self.plugins.write().map_err(|_| PluginError::LockError)?;
        let key = format!("{}-{}", name, version);
        Ok(plugins.remove(&key).is_some())
    }

    /// Initialize all plugins with configuration
    pub fn initialize_all(&self, config: serde_json::Value) -> Result<(), PluginError> {
        let plugins = self.plugins.read().map_err(|_| PluginError::LockError)?;

        for plugin in plugins.values() {
            let plugin_config = config
                .get(plugin.name())
                .and_then(|c| c.get(plugin.version()))
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            plugin.initialize(plugin_config).map_err(|e| {
                PluginError::InitializationFailed(format!("{}: {}", plugin.name(), e))
            })?;
        }

        Ok(())
    }

    /// Shutdown all plugins
    pub fn shutdown_all(&self) -> Result<(), PluginError> {
        let plugins = self.plugins.read().map_err(|_| PluginError::LockError)?;

        for plugin in plugins.values() {
            plugin.shutdown().map_err(|e| {
                PluginError::InitializationFailed(format!("{}: {}", plugin.name(), e))
            })?;
        }

        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
