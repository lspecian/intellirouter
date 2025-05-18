//! Routing Response Types
//!
//! This module defines the response types used for routing operations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::modules::model_registry::ChatCompletionResponse;

/// Metadata about a routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetadata {
    /// ID of the selected model
    pub selected_model_id: String,

    /// Name of the strategy that made the selection
    pub strategy_name: String,

    /// Timestamp when routing started
    pub routing_start_time: chrono::DateTime<chrono::Utc>,

    /// Timestamp when routing completed
    pub routing_end_time: chrono::DateTime<chrono::Utc>,

    /// Total routing time in milliseconds
    pub routing_time_ms: u64,

    /// Number of models considered during routing
    pub models_considered: u32,

    /// Number of routing attempts made
    pub attempts: u32,

    /// Whether this was a fallback selection
    pub is_fallback: bool,

    /// Selection criteria used (e.g., "lowest_cost", "lowest_latency")
    pub selection_criteria: Option<String>,

    /// Additional metadata about the routing decision
    pub additional_metadata: HashMap<String, String>,
}

/// Routing response containing the chat completion response and routing metadata
#[derive(Debug, Clone)]
pub struct RoutingResponse {
    /// Original chat completion response
    pub response: ChatCompletionResponse,

    /// Metadata about the routing decision
    pub metadata: RoutingMetadata,
}
