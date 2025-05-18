//! Routing Context
//!
//! This module defines the routing context used during routing operations.

use std::collections::HashMap;

use crate::modules::model_registry::ChatCompletionRequest;

/// Routing context containing information used during routing
#[derive(Debug, Clone)]
pub struct RoutingContext {
    /// Original chat completion request
    pub request: ChatCompletionRequest,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Organization ID (if available)
    pub org_id: Option<String>,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Request priority (higher values indicate higher priority)
    pub priority: u8,

    /// Request tags for categorization
    pub tags: Vec<String>,

    /// Additional context parameters
    pub parameters: HashMap<String, String>,
}

impl RoutingContext {
    /// Create a new routing context from a chat completion request
    pub fn new(request: ChatCompletionRequest) -> Self {
        Self {
            request,
            user_id: None,
            org_id: None,
            timestamp: chrono::Utc::now(),
            priority: 0,
            tags: Vec::new(),
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter to the routing context
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the organization ID
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}
