use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur in the RAG system
#[derive(Error, Debug)]
pub enum RagError {
    /// Source not found
    #[error("Source not found: {0}")]
    SourceNotFound(String),

    /// Retrieval error
    #[error("Retrieval error: {0}")]
    RetrievalError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Document structure for RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

/// A chunk of context retrieved from a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    /// The content of the chunk
    pub content: String,

    /// The source identifier
    pub source: String,

    /// The relevance score (0.0 to 1.0)
    pub relevance_score: f32,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// RAG configuration
#[derive(Debug, Clone)]
pub struct RAGConfig {
    pub index_path: String,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_chunks_per_source: usize,
    pub max_total_chunks: usize,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            index_path: "rag_index".to_string(),
            chunk_size: 1000,
            chunk_overlap: 200,
            max_chunks_per_source: 5,
            max_total_chunks: 10,
        }
    }
}
