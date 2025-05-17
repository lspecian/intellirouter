//! Plugin SDK Module
//!
//! This module provides a system for extending IntelliRouter's functionality with
//! custom routing strategies, model connectors, and telemetry exporters.
//! It defines interfaces and mechanisms for loading and managing plugins.

pub mod examples;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod traits;

#[cfg(test)]
mod tests;

// Re-export types for easier access
pub use examples::register_example_plugins;
pub use loader::{PluginLoader, PluginMetadata};
pub use manager::{PluginConfig, PluginManager};
pub use registry::{Plugin, PluginError, PluginRegistry, PluginType};
pub use traits::{
    get_model_connector_plugin, get_routing_strategy_plugin, get_telemetry_exporter_plugin,
    ModelConnectorPlugin, RoutingStrategyPlugin, TelemetryExporterPlugin,
};

/// Initialize the plugin system with the specified configuration
pub fn init_plugin_system(config: PluginConfig) -> Result<PluginManager, PluginError> {
    let mut manager = PluginManager::new();
    manager.load_plugins(&config)?;
    Ok(manager)
}

/// Get a plugin manager instance
pub fn create_plugin_manager() -> PluginManager {
    PluginManager::new()
}
