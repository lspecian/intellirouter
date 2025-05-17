//! System Topology
//!
//! This module defines the data structures for representing the system topology.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::modules::audit::types::{ServiceStatus, ServiceType};

/// System topology node
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopologyNode {
    /// Service type
    pub service_type: ServiceType,
    /// Service status
    pub status: ServiceStatus,
    /// Node position in visualization (x, y)
    pub position: Option<(f32, f32)>,
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

impl TopologyNode {
    /// Create a new topology node
    pub fn new(service_type: ServiceType, status: ServiceStatus) -> Self {
        Self {
            service_type,
            status,
            position: None,
            metadata: HashMap::new(),
        }
    }

    /// Set node position
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// System topology edge
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopologyEdge {
    /// Source service
    pub source: ServiceType,
    /// Target service
    pub target: ServiceType,
    /// Connection status
    pub connected: bool,
    /// Edge metadata
    pub metadata: HashMap<String, String>,
}

impl TopologyEdge {
    /// Create a new topology edge
    pub fn new(source: ServiceType, target: ServiceType, connected: bool) -> Self {
        Self {
            source,
            target,
            connected,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// System topology
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemTopology {
    /// Nodes
    pub nodes: Vec<TopologyNode>,
    /// Edges
    pub edges: Vec<TopologyEdge>,
}

impl SystemTopology {
    /// Create a new system topology
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node
    pub fn add_node(&mut self, node: TopologyNode) {
        // Check if node already exists
        if !self
            .nodes
            .iter()
            .any(|n| n.service_type == node.service_type)
        {
            self.nodes.push(node);
        }
    }

    /// Add an edge
    pub fn add_edge(&mut self, edge: TopologyEdge) {
        // Check if edge already exists
        if !self
            .edges
            .iter()
            .any(|e| e.source == edge.source && e.target == edge.target)
        {
            self.edges.push(edge);
        }
    }

    /// Get all service types
    pub fn get_service_types(&self) -> HashSet<ServiceType> {
        self.nodes.iter().map(|n| n.service_type).collect()
    }

    /// Get node by service type
    pub fn get_node(&self, service_type: ServiceType) -> Option<&TopologyNode> {
        self.nodes.iter().find(|n| n.service_type == service_type)
    }

    /// Get edges for a service
    pub fn get_edges_for_service(&self, service_type: ServiceType) -> Vec<&TopologyEdge> {
        self.edges
            .iter()
            .filter(|e| e.source == service_type || e.target == service_type)
            .collect()
    }

    /// Get outgoing edges for a service
    pub fn get_outgoing_edges(&self, service_type: ServiceType) -> Vec<&TopologyEdge> {
        self.edges
            .iter()
            .filter(|e| e.source == service_type)
            .collect()
    }

    /// Get incoming edges for a service
    pub fn get_incoming_edges(&self, service_type: ServiceType) -> Vec<&TopologyEdge> {
        self.edges
            .iter()
            .filter(|e| e.target == service_type)
            .collect()
    }

    /// Calculate connectivity status
    pub fn calculate_connectivity_status(&self) -> f32 {
        if self.edges.is_empty() {
            return 0.0;
        }

        let connected_edges = self.edges.iter().filter(|e| e.connected).count();
        connected_edges as f32 / self.edges.len() as f32
    }
}

impl Default for SystemTopology {
    fn default() -> Self {
        Self::new()
    }
}
