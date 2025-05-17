//! Tests for the Plugin SDK
//!
//! This module contains tests for the plugin SDK.

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::Arc;

    use super::super::examples::{
        register_example_plugins, ExampleModelConnectorPlugin, ExampleRoutingStrategyPlugin,
        ExampleTelemetryExporterPlugin,
    };
    use super::super::manager::PluginManager;
    use super::super::registry::{Plugin, PluginRegistry, PluginType};

    #[test]
    fn test_plugin_registry() {
        let registry = PluginRegistry::new();

        // Register plugins
        registry
            .register_plugin(Arc::new(ExampleRoutingStrategyPlugin::new()))
            .unwrap();
        registry
            .register_plugin(Arc::new(ExampleModelConnectorPlugin::new()))
            .unwrap();
        registry
            .register_plugin(Arc::new(ExampleTelemetryExporterPlugin::new()))
            .unwrap();

        // Get plugins
        let routing_plugin = registry
            .get_plugin("example_routing_strategy", "0.1.0")
            .unwrap()
            .unwrap();
        assert_eq!(routing_plugin.name(), "example_routing_strategy");
        assert_eq!(routing_plugin.version(), "0.1.0");
        assert_eq!(routing_plugin.plugin_type(), PluginType::RoutingStrategy);

        let connector_plugin = registry
            .get_plugin("example_model_connector", "0.1.0")
            .unwrap()
            .unwrap();
        assert_eq!(connector_plugin.name(), "example_model_connector");
        assert_eq!(connector_plugin.version(), "0.1.0");
        assert_eq!(connector_plugin.plugin_type(), PluginType::ModelConnector);

        let exporter_plugin = registry
            .get_plugin("example_telemetry_exporter", "0.1.0")
            .unwrap()
            .unwrap();
        assert_eq!(exporter_plugin.name(), "example_telemetry_exporter");
        assert_eq!(exporter_plugin.version(), "0.1.0");
        assert_eq!(exporter_plugin.plugin_type(), PluginType::TelemetryExporter);

        // List plugins
        let all_plugins = registry.list_plugins().unwrap();
        assert_eq!(all_plugins.len(), 3);

        let routing_plugins = registry
            .list_plugins_by_type(PluginType::RoutingStrategy)
            .unwrap();
        assert_eq!(routing_plugins.len(), 1);
        assert_eq!(routing_plugins[0].name(), "example_routing_strategy");

        let connector_plugins = registry
            .list_plugins_by_type(PluginType::ModelConnector)
            .unwrap();
        assert_eq!(connector_plugins.len(), 1);
        assert_eq!(connector_plugins[0].name(), "example_model_connector");

        let exporter_plugins = registry
            .list_plugins_by_type(PluginType::TelemetryExporter)
            .unwrap();
        assert_eq!(exporter_plugins.len(), 1);
        assert_eq!(exporter_plugins[0].name(), "example_telemetry_exporter");

        // Unregister plugins
        assert!(registry
            .unregister_plugin("example_routing_strategy", "0.1.0")
            .unwrap());
        assert!(registry
            .get_plugin("example_routing_strategy", "0.1.0")
            .unwrap()
            .is_none());

        // Initialize and shutdown
        registry
            .initialize_all(json!({
                "example_model_connector": {
                    "0.1.0": {
                        "key": "value"
                    }
                }
            }))
            .unwrap();

        registry.shutdown_all().unwrap();
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();

        // Register example plugins
        register_example_plugins(&manager.registry()).unwrap();

        // Get plugins
        let routing_plugin = manager
            .get_routing_strategy_plugin("example_routing_strategy", None)
            .unwrap();
        assert_eq!(routing_plugin.name(), "example_routing_strategy");
        assert_eq!(routing_plugin.version(), "0.1.0");
        assert_eq!(routing_plugin.plugin_type(), PluginType::RoutingStrategy);

        let connector_plugin = manager
            .get_model_connector_plugin("example_model_connector", None)
            .unwrap();
        assert_eq!(connector_plugin.name(), "example_model_connector");
        assert_eq!(connector_plugin.version(), "0.1.0");
        assert_eq!(connector_plugin.plugin_type(), PluginType::ModelConnector);

        let exporter_plugin = manager
            .get_telemetry_exporter_plugin("example_telemetry_exporter", None)
            .unwrap();
        assert_eq!(exporter_plugin.name(), "example_telemetry_exporter");
        assert_eq!(exporter_plugin.version(), "0.1.0");
        assert_eq!(exporter_plugin.plugin_type(), PluginType::TelemetryExporter);

        // List plugins
        let all_plugins = manager.list_plugins().unwrap();
        assert_eq!(all_plugins.len(), 3);

        let routing_plugins = manager
            .list_plugins_by_type(PluginType::RoutingStrategy)
            .unwrap();
        assert_eq!(routing_plugins.len(), 1);

        let connector_plugins = manager
            .list_plugins_by_type(PluginType::ModelConnector)
            .unwrap();
        assert_eq!(connector_plugins.len(), 1);

        let exporter_plugins = manager
            .list_plugins_by_type(PluginType::TelemetryExporter)
            .unwrap();
        assert_eq!(exporter_plugins.len(), 1);

        // Shutdown
        manager.shutdown().unwrap();
    }

    #[test]
    fn test_plugin_error() {
        use super::super::registry::PluginError;

        let error = PluginError::NotFound("test".to_string());
        assert_eq!(error.to_string(), "Plugin not found: test");

        let error = PluginError::AlreadyRegistered("test".to_string());
        assert_eq!(error.to_string(), "Plugin already registered: test");

        let error = PluginError::InitializationFailed("test".to_string());
        assert_eq!(error.to_string(), "Plugin initialization failed: test");
    }
}
