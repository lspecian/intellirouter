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
}
