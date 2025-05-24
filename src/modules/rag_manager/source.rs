use std::collections::HashMap;

use crate::modules::rag_manager::types::{ContextChunk, RagError};
use async_trait::async_trait;

/// Trait for context sources
///
/// A context source provides context chunks based on a query.
/// Different implementations can retrieve context from different sources,
/// such as files, databases, APIs, etc.
#[async_trait]
pub trait ContextSource: Send + Sync {
    /// Get context chunks based on a query
    ///
    /// # Arguments
    ///
    /// * `query` - The query to retrieve context for
    /// * `max_chunks` - The maximum number of chunks to retrieve
    ///
    /// # Returns
    ///
    /// A vector of context chunks, or an error if retrieval fails
    async fn get_context(
        &self,
        query: &str,
        max_chunks: usize,
    ) -> Result<Vec<ContextChunk>, RagError>;

    /// Get the name of this context source
    fn get_name(&self) -> String;

    /// Get the type of this context source (e.g., "file", "database", "api")
    fn get_type(&self) -> String {
        "generic".to_string()
    }

    /// Get metadata about this context source
    fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get vector database statistics for this source
    async fn get_vector_stats(&self) -> Option<HashMap<String, usize>> {
        None
    }

    /// Get collections information for this source
    async fn get_collections(&self) -> Option<Vec<HashMap<String, serde_json::Value>>> {
        None
    }
}
