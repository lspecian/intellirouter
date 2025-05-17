// Core modules for IntelliRouter
//
// This file exports the core modules of the IntelliRouter project,
// each handling a specific aspect of the system's functionality.

// LLM Proxy - OpenAI-compatible API
pub mod llm_proxy;

// Model Registry - Model tracking and metadata
pub mod model_registry;

// Router Core - Routing logic
pub mod router_core;

// Persona Layer - System prompt injection
pub mod persona_layer;

// Chain Engine - Multi-step orchestration
pub mod chain_engine;

// RAG Manager - RAG integration
pub mod rag_manager;

// Memory - Conversation history
pub mod memory;

// AuthZ - Authentication and authorization
pub mod authz;

// Telemetry - Logging and metrics
pub mod telemetry;

// Plugin SDK - Plugin interfaces
pub mod plugin_sdk;

// IPC - Inter-process communication infrastructure
pub mod ipc;
