//! Memory IPC module
//!
//! This module provides trait-based abstractions for the Memory service,
//! ensuring a clear separation between interface and transport logic.

mod client;
mod grpc;
mod responses;
mod service;
mod types;

// Re-export types
pub use client::*;
pub use grpc::*;
pub use responses::*;
pub use service::*;
pub use types::*;

// TODO: Add in-memory implementation of the Memory service
// TODO: Add Redis implementation of the Memory service
// TODO: Add tests for the Memory service implementations
