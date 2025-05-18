//! Model version information

use crate::modules::model_registry::types::errors::RegistryError;
use serde::{Deserialize, Serialize};

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelVersionInfo {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Release date
    pub release_date: Option<chrono::DateTime<chrono::Utc>>,
    /// End of life date
    pub end_of_life_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Is this a preview/beta version
    pub is_preview: bool,
}

impl ModelVersionInfo {
    /// Create a new model version info from a version string (e.g., "1.0.0")
    pub fn from_version_string(version: &str) -> Result<Self, RegistryError> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(RegistryError::InvalidMetadata(format!(
                "Invalid version string: {}",
                version
            )));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            RegistryError::InvalidMetadata(format!("Invalid major version: {}", parts[0]))
        })?;

        let minor = parts[1].parse::<u32>().map_err(|_| {
            RegistryError::InvalidMetadata(format!("Invalid minor version: {}", parts[1]))
        })?;

        let patch = parts[2].parse::<u32>().map_err(|_| {
            RegistryError::InvalidMetadata(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(Self {
            major,
            minor,
            patch,
            release_date: None,
            end_of_life_date: None,
            is_preview: false,
        })
    }

    /// Convert to a version string (e.g., "1.0.0")
    pub fn to_version_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Check if this version is newer than another version
    pub fn is_newer_than(&self, other: &Self) -> bool {
        if self.major > other.major {
            return true;
        }
        if self.major < other.major {
            return false;
        }
        if self.minor > other.minor {
            return true;
        }
        if self.minor < other.minor {
            return false;
        }
        self.patch > other.patch
    }
}
