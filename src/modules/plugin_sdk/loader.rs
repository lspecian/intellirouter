//! Plugin Loader
//!
//! This module provides functionality for loading plugins from dynamic libraries.

use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::registry::{Plugin, PluginError, PluginRegistry};

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Name of the plugin
    pub name: String,
    /// Version of the plugin
    pub version: String,
    /// Type of the plugin
    pub plugin_type: String,
    /// Path to the plugin library
    pub library_path: PathBuf,
    /// Configuration for the plugin
    pub config: serde_json::Value,
}

/// Plugin loader
pub struct PluginLoader {
    /// Registry to register plugins
    registry: Arc<PluginRegistry>,
    /// Loaded libraries
    libraries: Vec<Library>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            registry,
            libraries: Vec::new(),
        }
    }

    /// Load a plugin from a dynamic library
    pub fn load_plugin(&mut self, path: impl AsRef<Path>) -> Result<(), PluginError> {
        let path = path.as_ref();

        // Load the library
        let library =
            unsafe { Library::new(path).map_err(|e| PluginError::LoadingFailed(e.to_string()))? };

        // Get the plugin creation function
        let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = unsafe {
            library
                .get(b"create_plugin")
                .map_err(|e| PluginError::LoadingFailed(e.to_string()))?
        };

        // Create the plugin
        let plugin = unsafe {
            let raw_plugin = create_plugin();
            Arc::from_raw(raw_plugin)
        };

        // Register the plugin
        self.registry.register_plugin(plugin)?;

        // Keep the library alive
        self.libraries.push(library);

        Ok(())
    }

    /// Load plugins from a directory
    pub fn load_plugins_from_directory(
        &mut self,
        dir: impl AsRef<Path>,
    ) -> Result<Vec<String>, PluginError> {
        let dir = dir.as_ref();

        if !dir.exists() || !dir.is_dir() {
            return Err(PluginError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Directory not found: {}", dir.display()),
            )));
        }

        let mut loaded_plugins = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let extension = path.extension().and_then(|e| e.to_str());

                #[cfg(target_os = "linux")]
                let is_plugin = extension == Some("so");

                #[cfg(target_os = "macos")]
                let is_plugin = extension == Some("dylib");

                #[cfg(target_os = "windows")]
                let is_plugin = extension == Some("dll");

                if is_plugin {
                    self.load_plugin(&path)?;
                    loaded_plugins.push(path.display().to_string());
                }
            }
        }

        Ok(loaded_plugins)
    }

    /// Load plugins from metadata
    pub fn load_plugins_from_metadata(
        &mut self,
        metadata: &[PluginMetadata],
    ) -> Result<Vec<String>, PluginError> {
        let mut loaded_plugins = Vec::new();

        for meta in metadata {
            self.load_plugin(&meta.library_path)?;
            loaded_plugins.push(meta.library_path.display().to_string());
        }

        Ok(loaded_plugins)
    }
}
