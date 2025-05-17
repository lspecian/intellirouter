//! Plugin Manager
//!
//! This module provides a high-level manager for plugins.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use super::loader::{PluginLoader, PluginMetadata};
use super::registry::{Plugin, PluginError, PluginRegistry, PluginType};
use super::traits::{
    get_model_connector_plugin, get_routing_strategy_plugin, get_telemetry_exporter_plugin,
    ModelConnectorPlugin, RoutingStrategyPlugin, TelemetryExporterPlugin,
};

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Directory containing plugin libraries
    pub plugin_dir: Option<String>,
    /// Plugin metadata
    pub plugins: Vec<PluginMetadata>,
    /// Plugin configuration
    pub config: serde_json::Value,
}

/// Plugin manager
pub struct PluginManager {
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    /// Plugin loader
    loader: PluginLoader,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let registry = Arc::new(PluginRegistry::new());
        let loader = PluginLoader::new(registry.clone());

        Self { registry, loader }
    }

    /// Get the plugin registry
    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }

    /// Load plugins from configuration
    pub fn load_plugins(&mut self, config: &PluginConfig) -> Result<Vec<String>, PluginError> {
        let mut loaded_plugins = Vec::new();

        // Load plugins from directory
        if let Some(plugin_dir) = &config.plugin_dir {
            let dir_plugins = self.loader.load_plugins_from_directory(plugin_dir)?;
            loaded_plugins.extend(dir_plugins);
        }

        // Load plugins from metadata
        let meta_plugins = self.loader.load_plugins_from_metadata(&config.plugins)?;
        loaded_plugins.extend(meta_plugins);

        // Initialize all plugins
        self.registry.initialize_all(config.config.clone())?;

        Ok(loaded_plugins)
    }

    /// Load a single plugin
    pub fn load_plugin(&mut self, path: impl AsRef<Path>) -> Result<(), PluginError> {
        self.loader.load_plugin(path)
    }

    /// Get a routing strategy plugin
    pub fn get_routing_strategy_plugin(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Arc<dyn RoutingStrategyPlugin>, PluginError> {
        get_routing_strategy_plugin(&self.registry, name, version)
    }

    /// Get a model connector plugin
    pub fn get_model_connector_plugin(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Arc<dyn ModelConnectorPlugin>, PluginError> {
        get_model_connector_plugin(&self.registry, name, version)
    }

    /// Get a telemetry exporter plugin
    pub fn get_telemetry_exporter_plugin(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Arc<dyn TelemetryExporterPlugin>, PluginError> {
        get_telemetry_exporter_plugin(&self.registry, name, version)
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>, PluginError> {
        self.registry.list_plugins()
    }

    /// List plugins by type
    pub fn list_plugins_by_type(
        &self,
        plugin_type: PluginType,
    ) -> Result<Vec<Arc<dyn Plugin>>, PluginError> {
        self.registry.list_plugins_by_type(plugin_type)
    }

    /// Shutdown all plugins
    pub fn shutdown(&self) -> Result<(), PluginError> {
        self.registry.shutdown_all()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
