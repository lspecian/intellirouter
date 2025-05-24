//! Model Registry Types
//!
//! This module defines the core data structures for the Model Registry.
//! These types are used to store and manage metadata about LLM models,
//! their capabilities, status, and other relevant information.

pub mod capabilities;
pub mod errors;
pub mod filters;
pub mod formats;
pub mod health;
pub mod model;
pub mod performance;
pub mod status;
pub mod version;

// Re-export types for easier access
pub use capabilities::{FineTuningCapabilities, ModelCapabilities, RateLimits};
pub use errors::RegistryError;
pub use filters::ModelFilter;
pub use formats::{InputFormat, OutputFormat};
pub use health::ModelHealthStatus;
pub use model::{ModelMetadata, ModelType};
pub use performance::ModelPerformance;
pub use status::ModelStatus;
pub use version::ModelVersionInfo;
