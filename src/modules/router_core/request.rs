//! Routing Request Types
//!
//! This module defines the request types used for routing operations.

use std::time::Duration;

use crate::modules::model_registry::{ChatCompletionRequest, ModelFilter};
use crate::modules::router_core::context::RoutingContext;

/// Routing request wrapping a chat completion request with routing metadata
#[derive(Debug, Clone)]
pub struct RoutingRequest {
    /// Routing context
    pub context: RoutingContext,

    /// Model filter for selecting eligible models
    pub model_filter: Option<ModelFilter>,

    /// Preferred model ID (if any)
    pub preferred_model_id: Option<String>,

    /// Excluded model IDs
    pub excluded_model_ids: Vec<String>,

    /// Maximum routing attempts
    pub max_attempts: u32,

    /// Routing timeout
    pub timeout: Duration,
}

impl RoutingRequest {
    /// Create a new routing request from a chat completion request
    pub fn new(request: ChatCompletionRequest) -> Self {
        Self {
            context: RoutingContext::new(request),
            model_filter: None,
            preferred_model_id: None,
            excluded_model_ids: Vec::new(),
            max_attempts: 3,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set a model filter
    pub fn with_model_filter(mut self, filter: ModelFilter) -> Self {
        self.model_filter = Some(filter);
        self
    }

    /// Set a preferred model ID
    pub fn with_preferred_model(mut self, model_id: impl Into<String>) -> Self {
        self.preferred_model_id = Some(model_id.into());
        self
    }

    /// Add an excluded model ID
    pub fn exclude_model(mut self, model_id: impl Into<String>) -> Self {
        self.excluded_model_ids.push(model_id.into());
        self
    }

    /// Set the maximum number of routing attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set the routing timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
