//! IntelliRouter Modules
//!
//! This module contains all the modules that make up the IntelliRouter system.

pub mod audit;
pub mod authz;
pub mod chain_engine;
pub mod common;
pub mod health;
pub mod ipc;
pub mod llm_proxy;
pub mod memory;
pub mod model_registry;
pub mod monitoring;
pub mod orchestrator;
pub mod persona_layer;
pub mod rag_manager;
pub mod router_core;
pub mod telemetry;

// Re-enable the test harness module
#[cfg(feature = "test-harness")]
pub mod test_harness;
