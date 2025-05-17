// IntelliRouter Core Library
//
// This file exports the core modules of the IntelliRouter project,
// which can assume different functional roles at runtime.

// Core modules
pub mod cli;
pub mod config;
pub mod modules;

#[cfg(test)]
pub mod test_utils;

// Re-exports of commonly used items
pub use cli::{Cli, Commands, Role};
pub use config::Config;
