//! Data Loader Module
//!
//! This module provides functionality for loading test data from various sources.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Loaded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedData {
    /// Data
    pub data: serde_json::Value,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Data loader configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLoaderConfig {
    /// Loader name
    pub name: String,
    /// Loader description
    pub description: Option<String>,
    /// Supported file extensions
    pub extensions: Vec<String>,
    /// Base directory
    pub base_dir: Option<PathBuf>,
    /// Custom configuration
    pub custom: Option<serde_json::Value>,
}

impl Default for DataLoaderConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            extensions: vec!["json".to_string()],
            base_dir: None,
            custom: None,
        }
    }
}

/// Data loader trait
#[async_trait]
pub trait DataLoader: Send + Sync {
    /// Get the loader name
    fn name(&self) -> &str;

    /// Get the loader description
    fn description(&self) -> Option<&str>;

    /// Get the supported file extensions
    fn extensions(&self) -> &[String];

    /// Load data from a file
    async fn load(&self, path: &Path) -> Result<LoadedData, String>;

    /// Save data to a file
    async fn save(&self, path: &Path, data: &LoadedData) -> Result<(), String>;

    /// Check if the loader supports a file
    fn supports_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            self.extensions().iter().any(|e| e == ext)
        } else {
            false
        }
    }
}

/// JSON data loader
pub struct JsonDataLoader {
    /// Loader configuration
    config: DataLoaderConfig,
}

impl JsonDataLoader {
    /// Create a new JSON data loader
    pub fn new() -> Self {
        Self {
            config: DataLoaderConfig {
                name: "json".to_string(),
                description: Some("JSON data loader".to_string()),
                extensions: vec!["json".to_string()],
                base_dir: None,
                custom: None,
            },
        }
    }

    /// Create a new JSON data loader with a custom configuration
    pub fn with_config(config: DataLoaderConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl DataLoader for JsonDataLoader {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn extensions(&self) -> &[String] {
        &self.config.extensions
    }

    async fn load(&self, path: &Path) -> Result<LoadedData, String> {
        // Read the file
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse the JSON
        let data =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), path.to_string_lossy().to_string());
        metadata.insert("loader".to_string(), self.name().to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(LoadedData { data, metadata })
    }

    async fn save(&self, path: &Path, data: &LoadedData) -> Result<(), String> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        // Serialize the JSON
        let content = serde_json::to_string_pretty(&data.data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        // Write the file
        fs::write(path, content)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }
}

/// YAML data loader
pub struct YamlDataLoader {
    /// Loader configuration
    config: DataLoaderConfig,
}

impl YamlDataLoader {
    /// Create a new YAML data loader
    pub fn new() -> Self {
        Self {
            config: DataLoaderConfig {
                name: "yaml".to_string(),
                description: Some("YAML data loader".to_string()),
                extensions: vec!["yaml".to_string(), "yml".to_string()],
                base_dir: None,
                custom: None,
            },
        }
    }

    /// Create a new YAML data loader with a custom configuration
    pub fn with_config(config: DataLoaderConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl DataLoader for YamlDataLoader {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn extensions(&self) -> &[String] {
        &self.config.extensions
    }

    async fn load(&self, path: &Path) -> Result<LoadedData, String> {
        // Read the file
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse the YAML
        let data =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), path.to_string_lossy().to_string());
        metadata.insert("loader".to_string(), self.name().to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(LoadedData { data, metadata })
    }

    async fn save(&self, path: &Path, data: &LoadedData) -> Result<(), String> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        // Serialize the YAML
        let content = serde_yaml::to_string(&data.data)
            .map_err(|e| format!("Failed to serialize YAML: {}", e))?;

        // Write the file
        fs::write(path, content)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }
}

/// CSV data loader
pub struct CsvDataLoader {
    /// Loader configuration
    config: DataLoaderConfig,
}

impl CsvDataLoader {
    /// Create a new CSV data loader
    pub fn new() -> Self {
        Self {
            config: DataLoaderConfig {
                name: "csv".to_string(),
                description: Some("CSV data loader".to_string()),
                extensions: vec!["csv".to_string()],
                base_dir: None,
                custom: None,
            },
        }
    }

    /// Create a new CSV data loader with a custom configuration
    pub fn with_config(config: DataLoaderConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl DataLoader for CsvDataLoader {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn extensions(&self) -> &[String] {
        &self.config.extensions
    }

    async fn load(&self, path: &Path) -> Result<LoadedData, String> {
        // Read the file
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse the CSV
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let headers = reader
            .headers()
            .map_err(|e| format!("Failed to read CSV headers: {}", e))?
            .clone();

        let mut records = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| format!("Failed to read CSV record: {}", e))?;
            let mut row = serde_json::Map::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    let header = headers.get(i).unwrap_or_default();
                    row.insert(
                        header.to_string(),
                        serde_json::Value::String(field.to_string()),
                    );
                }
            }
            records.push(serde_json::Value::Object(row));
        }

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), path.to_string_lossy().to_string());
        metadata.insert("loader".to_string(), self.name().to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(LoadedData {
            data: serde_json::Value::Array(records),
            metadata,
        })
    }

    async fn save(&self, path: &Path, data: &LoadedData) -> Result<(), String> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        // Check if the data is an array
        let records = match &data.data {
            serde_json::Value::Array(records) => records,
            _ => return Err("CSV data must be an array".to_string()),
        };

        // Create a CSV writer
        let file = tokio::fs::File::create(path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;
        let file = tokio::io::BufWriter::new(file);
        let file = tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(file);
        let mut writer = csv::Writer::from_writer(file);

        // Get all headers
        let mut headers = Vec::new();
        for record in records {
            if let serde_json::Value::Object(obj) = record {
                for key in obj.keys() {
                    if !headers.contains(key) {
                        headers.push(key.clone());
                    }
                }
            }
        }

        // Write headers
        writer
            .write_record(&headers)
            .map_err(|e| format!("Failed to write CSV headers: {}", e))?;

        // Write records
        for record in records {
            if let serde_json::Value::Object(obj) = record {
                let mut row = Vec::new();
                for header in &headers {
                    let value = obj.get(header).unwrap_or(&serde_json::Value::Null);
                    let value_str = match value {
                        serde_json::Value::Null => "".to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        _ => serde_json::to_string(value).unwrap_or_else(|_| "".to_string()),
                    };
                    row.push(value_str);
                }
                writer
                    .write_record(&row)
                    .map_err(|e| format!("Failed to write CSV record: {}", e))?;
            }
        }

        // Flush the writer
        writer
            .flush()
            .map_err(|e| format!("Failed to flush CSV writer: {}", e))?;

        Ok(())
    }
}

/// HTTP data loader
pub struct HttpDataLoader {
    /// Loader configuration
    config: DataLoaderConfig,
    /// HTTP client
    client: reqwest::Client,
}

impl HttpDataLoader {
    /// Create a new HTTP data loader
    pub fn new() -> Self {
        Self {
            config: DataLoaderConfig {
                name: "http".to_string(),
                description: Some("HTTP data loader".to_string()),
                extensions: vec!["http".to_string(), "https".to_string()],
                base_dir: None,
                custom: None,
            },
            client: reqwest::Client::new(),
        }
    }

    /// Create a new HTTP data loader with a custom configuration
    pub fn with_config(config: DataLoaderConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DataLoader for HttpDataLoader {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }

    fn extensions(&self) -> &[String] {
        &self.config.extensions
    }

    async fn load(&self, path: &Path) -> Result<LoadedData, String> {
        // Convert path to URL
        let url = path.to_string_lossy().to_string();

        // Make the request
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to make HTTP request: {}", e))?;

        // Check the status
        if !response.status().is_success() {
            return Err(format!(
                "HTTP request failed with status: {}",
                response.status()
            ));
        }

        // Parse the response
        let data = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse response as JSON: {}", e))?;

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), url);
        metadata.insert("loader".to_string(), self.name().to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(LoadedData { data, metadata })
    }

    async fn save(&self, _path: &Path, _data: &LoadedData) -> Result<(), String> {
        Err("HTTP data loader does not support saving".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_json_data_loader() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Create test data
        let data = serde_json::json!({
            "name": "test",
            "value": 42
        });

        let loaded_data = LoadedData {
            data: data.clone(),
            metadata: HashMap::new(),
        };

        // Save the data
        let loader = JsonDataLoader::new();
        loader.save(&file_path, &loaded_data).await.unwrap();

        // Load the data
        let loaded = loader.load(&file_path).await.unwrap();

        assert_eq!(loaded.data, data);
    }
}
