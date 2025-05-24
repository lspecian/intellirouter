//! Test Harness Plugins
//!
//! This module provides plugin support for the test harness.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::types::{TestCase, TestContext, TestHarnessError, TestResult, TestSuite};

/// Plugin trait for extending the test harness
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Initialize the plugin
    async fn initialize(&self) -> Result<(), TestHarnessError>;

    /// Cleanup the plugin
    async fn cleanup(&self) -> Result<(), TestHarnessError>;

    /// Hook called before a test suite is run
    async fn before_suite(&self, suite: &TestSuite) -> Result<(), TestHarnessError>;

    /// Hook called after a test suite is run
    async fn after_suite(&self, suite: &TestSuite) -> Result<(), TestHarnessError>;

    /// Hook called before a test case is run
    async fn before_test(&self, test_case: &TestCase) -> Result<(), TestHarnessError>;

    /// Hook called after a test case is run
    async fn after_test(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError>;

    /// Hook called when a test case fails
    async fn on_test_failure(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError>;

    /// Hook called when a test case passes
    async fn on_test_success(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError>;

    /// Hook called when a test case is skipped
    async fn on_test_skip(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError>;
}

/// Plugin manager for managing test harness plugins
pub struct PluginManager {
    /// Plugins
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<(), TestHarnessError> {
        let name = plugin.name().to_string();
        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&name) {
            return Err(TestHarnessError::PluginError(format!(
                "Plugin with name '{}' already registered",
                name
            )));
        }
        plugins.insert(name, plugin);
        Ok(())
    }

    /// Unregister a plugin
    pub async fn unregister_plugin(&self, name: &str) -> Result<(), TestHarnessError> {
        let mut plugins = self.plugins.write().await;
        if !plugins.contains_key(name) {
            return Err(TestHarnessError::PluginError(format!(
                "Plugin with name '{}' not registered",
                name
            )));
        }
        plugins.remove(name);
        Ok(())
    }

    /// Get a plugin by name
    pub async fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(name).cloned()
    }

    /// Get all plugins
    pub async fn get_all_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Initialize all plugins
    pub async fn initialize_all(&self) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.initialize().await?;
        }
        Ok(())
    }

    /// Cleanup all plugins
    pub async fn cleanup_all(&self) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.cleanup().await?;
        }
        Ok(())
    }

    /// Call before_suite hook on all plugins
    pub async fn before_suite(&self, suite: &TestSuite) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.before_suite(suite).await?;
        }
        Ok(())
    }

    /// Call after_suite hook on all plugins
    pub async fn after_suite(&self, suite: &TestSuite) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.after_suite(suite).await?;
        }
        Ok(())
    }

    /// Call before_test hook on all plugins
    pub async fn before_test(&self, test_case: &TestCase) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.before_test(test_case).await?;
        }
        Ok(())
    }

    /// Call after_test hook on all plugins
    pub async fn after_test(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.after_test(test_case, result).await?;
        }
        Ok(())
    }

    /// Call on_test_failure hook on all plugins
    pub async fn on_test_failure(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.on_test_failure(test_case, result).await?;
        }
        Ok(())
    }

    /// Call on_test_success hook on all plugins
    pub async fn on_test_success(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.on_test_success(test_case, result).await?;
        }
        Ok(())
    }

    /// Call on_test_skip hook on all plugins
    pub async fn on_test_skip(
        &self,
        test_case: &TestCase,
        result: &TestResult,
    ) -> Result<(), TestHarnessError> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.on_test_skip(test_case, result).await?;
        }
        Ok(())
    }
}
