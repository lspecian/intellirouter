// IntelliRouter Core Library
//
// This file exports the core modules of the IntelliRouter project,
// which can assume different functional roles at runtime.

// Core modules
pub mod cli;
pub mod config;
pub mod modules;

// Make test_utils available when the test-utils feature is enabled
#[cfg(feature = "test-utils")]
pub mod test_utils;

// Test templates are only available during tests
#[cfg(test)]
pub mod test_templates;

// Re-exports of commonly used items
pub use cli::{Cli, Commands, Role};
pub use config::Config;
