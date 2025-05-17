//! Plugin Traits
//!
//! This module defines the traits for different plugin types.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::modules::model_registry::ModelConnector;
use crate::modules::router_core::RoutingStrategyTrait;

use super::registry::{Plugin, PluginError, PluginType};

/// Telemetry Exporter trait
///
/// This trait defines the interface for telemetry exporters.
/// Telemetry exporters are responsible for exporting telemetry data
/// to external systems such as monitoring platforms, logging services,
/// or analytics tools.
#[async_trait]
pub trait TelemetryExporter: Send + Sync {
    /// Get the name of the exporter
    fn name(&self) -> &str;

    /// Initialize the exporter with configuration
    async fn initialize(&self, config: Value) -> Result<(), String>;

    /// Export metrics data
    async fn export_metrics(&self, metrics: Value) -> Result<(), String>;

    /// Export logs data
    async fn export_logs(&self, logs: Value) -> Result<(), String>;

    /// Export traces data
    async fn export_traces(&self, traces: Value) -> Result<(), String>;

    /// Flush any buffered data
    async fn flush(&self) -> Result<(), String>;

    /// Shutdown the exporter
    async fn shutdown(&self) -> Result<(), String>;
}

/// Trait for routing strategy plugins
pub trait RoutingStrategyPlugin: Plugin {
    /// Create a new routing strategy instance
    fn create_strategy(&self, config: Value) -> Result<Box<dyn RoutingStrategyTrait>, PluginError>;
}

/// Trait for model connector plugins
pub trait ModelConnectorPlugin: Plugin {
    /// Create a new model connector instance
    fn create_connector(&self, config: Value) -> Result<Box<dyn ModelConnector>, PluginError>;
}

/// Trait for telemetry exporter plugins
pub trait TelemetryExporterPlugin: Plugin {
    /// Create a new telemetry exporter instance
    fn create_exporter(&self, config: Value) -> Result<Box<dyn TelemetryExporter>, PluginError>;
}

/// Helper function to get a routing strategy plugin
pub fn get_routing_strategy_plugin(
    registry: &Arc<super::registry::PluginRegistry>,
    name: &str,
    version: Option<&str>,
) -> Result<Arc<dyn RoutingStrategyPlugin>, PluginError> {
    let plugin = if let Some(version) = version {
        registry
            .get_plugin(name, version)?
            .ok_or_else(|| PluginError::NotFound(format!("{}-{}", name, version)))?
    } else {
        registry
            .get_latest_plugin(name)?
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?
    };

    if plugin.plugin_type() != PluginType::RoutingStrategy {
        return Err(PluginError::InvalidConfiguration(format!(
            "Plugin {} is not a routing strategy plugin",
            name
        )));
    }

    // For now, just return a stub implementation
    // TODO: Implement proper downcasting when Plugin trait is updated
    Err(PluginError::NotFound(format!(
        "Plugin {} not found or not a routing strategy plugin",
        name
    )))
}

/// Helper function to get a model connector plugin
pub fn get_model_connector_plugin(
    registry: &Arc<super::registry::PluginRegistry>,
    name: &str,
    version: Option<&str>,
) -> Result<Arc<dyn ModelConnectorPlugin>, PluginError> {
    let plugin = if let Some(version) = version {
        registry
            .get_plugin(name, version)?
            .ok_or_else(|| PluginError::NotFound(format!("{}-{}", name, version)))?
    } else {
        registry
            .get_latest_plugin(name)?
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?
    };

    if plugin.plugin_type() != PluginType::ModelConnector {
        return Err(PluginError::InvalidConfiguration(format!(
            "Plugin {} is not a model connector plugin",
            name
        )));
    }

    // For now, just return a stub implementation
    // TODO: Implement proper downcasting when Plugin trait is updated
    Err(PluginError::NotFound(format!(
        "Plugin {} not found or not a model connector plugin",
        name
    )))
}

/// Helper function to get a telemetry exporter plugin
pub fn get_telemetry_exporter_plugin(
    registry: &Arc<super::registry::PluginRegistry>,
    name: &str,
    version: Option<&str>,
) -> Result<Arc<dyn TelemetryExporterPlugin>, PluginError> {
    let plugin = if let Some(version) = version {
        registry
            .get_plugin(name, version)?
            .ok_or_else(|| PluginError::NotFound(format!("{}-{}", name, version)))?
    } else {
        registry
            .get_latest_plugin(name)?
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?
    };

    if plugin.plugin_type() != PluginType::TelemetryExporter {
        return Err(PluginError::InvalidConfiguration(format!(
            "Plugin {} is not a telemetry exporter plugin",
            name
        )));
    }

    // For now, just return a stub implementation
    // TODO: Implement proper downcasting when Plugin trait is updated
    Err(PluginError::NotFound(format!(
        "Plugin {} not found or not a telemetry exporter plugin",
        name
    )))
}
