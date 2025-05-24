//! Router Core Module
//!
//! This module contains the core routing logic for IntelliRouter.
//! It determines which model or service should handle a given request
//! based on various criteria such as content, user preferences, and system load.
//! The module provides a flexible and extensible framework for implementing
//! different routing strategies.

// Tests moved to tests/unit/modules/router_core/

pub mod config;
pub mod context;
pub mod errors;
pub mod functions;
pub mod interface;
pub mod registry_integration;
pub mod request;
pub mod response;
pub mod retry;
pub mod router;
pub mod strategies;
pub mod strategy;

// Re-export types for easier access
pub use config::RouterConfig;
pub use context::RoutingContext;
pub use errors::RouterError;
pub use functions::{init, route_request};
pub use interface::Router;
pub use registry_integration::RegistryIntegration;
pub use request::RoutingRequest;
pub use response::{RoutingMetadata, RoutingResponse};
pub use retry::{CircuitBreakerConfig, DegradedServiceMode, ErrorCategory, RetryPolicy};
pub use router::RouterImpl;
pub use strategies::BaseStrategy;
pub use strategy::{RoutingStrategy, RoutingStrategyTrait};
