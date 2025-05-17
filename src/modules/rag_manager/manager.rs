use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::model_registry::connectors::{ChatCompletionRequest, ChatMessage, MessageRole};
use crate::modules::rag_manager::source::ContextSource;
use crate::modules::rag_manager::types::{ContextChunk, RagError};

/// The RAG Manager
///
/// This struct manages context sources and provides methods for retrieving
/// and injecting context into LLM requests.
pub struct RagManager {
    /// The context sources, keyed by name
    sources: HashMap<String, Arc<dyn ContextSource>>,
}

impl RagManager {
    /// Create a new RAG manager
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Add a context source
    ///
    /// # Arguments
    ///
    /// * `source` - The context source to add
    pub fn add_source(&mut self, source: Arc<dyn ContextSource>) {
        let name = source.get_name();
        self.sources.insert(name, source);
    }

    /// Remove a context source
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the source to remove
    ///
    /// # Returns
    ///
    /// The removed source, if it existed
    pub fn remove_source(&mut self, name: &str) -> Option<Arc<dyn ContextSource>> {
        self.sources.remove(name)
    }

    /// Get a context source by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the source to get
    ///
    /// # Returns
    ///
    /// The source, if it exists
    pub fn get_source(&self, name: &str) -> Option<&Arc<dyn ContextSource>> {
        self.sources.get(name)
    }

    /// Get all context sources
    ///
    /// # Returns
    ///
    /// A reference to the map of context sources
    pub fn get_sources(&self) -> &HashMap<String, Arc<dyn ContextSource>> {
        &self.sources
    }

    /// Retrieve context from all sources
    ///
    /// # Arguments
    ///
    /// * `query` - The query to retrieve context for
    /// * `max_chunks` - The maximum number of chunks to retrieve per source
    ///
    /// # Returns
    ///
    /// A vector of context chunks, sorted by relevance score
    pub async fn retrieve_context(
        &self,
        query: &str,
        max_chunks: usize,
    ) -> Result<Vec<ContextChunk>, RagError> {
        if self.sources.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_chunks = Vec::new();

        for source in self.sources.values() {
            match source.get_context(query, max_chunks).await {
                Ok(chunks) => all_chunks.extend(chunks),
                Err(e) => {
                    // Log the error but continue with other sources
                    eprintln!("Error retrieving context from {}: {}", source.get_name(), e);
                }
            }
        }

        // Sort by relevance score (highest first)
        all_chunks.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to max_chunks
        if all_chunks.len() > max_chunks {
            all_chunks.truncate(max_chunks);
        }

        Ok(all_chunks)
    }

    /// Inject context into a chat completion request
    ///
    /// This method retrieves context based on the query and injects it
    /// as a system message at the beginning of the request.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request to inject context into
    /// * `query` - The query to retrieve context for
    /// * `max_chunks` - The maximum number of chunks to retrieve
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error if context retrieval fails
    pub async fn inject_context(
        &self,
        request: &mut ChatCompletionRequest,
        query: &str,
        max_chunks: usize,
    ) -> Result<(), RagError> {
        let chunks = self.retrieve_context(query, max_chunks).await?;

        if chunks.is_empty() {
            return Ok(());
        }

        // Format the context as a system message
        let context_text = chunks
            .iter()
            .map(|chunk| format!("Source: {}\n\n{}", chunk.source, chunk.content))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        // Insert as a system message at the beginning
        request.messages.insert(
            0,
            ChatMessage {
                role: MessageRole::System,
                content: format!(
                    "Use the following information to answer the user's question:\n\n{}",
                    context_text
                ),
                name: None,
                function_call: None,
                tool_calls: None,
            },
        );

        Ok(())
    }

    /// Fuse multiple context chunks into a single string
    ///
    /// # Arguments
    ///
    /// * `chunks` - The chunks to fuse
    /// * `max_length` - The maximum length of the fused context
    ///
    /// # Returns
    ///
    /// The fused context, truncated to max_length if necessary
    pub async fn fuse_context(
        &self,
        chunks: &[ContextChunk],
        max_length: usize,
    ) -> Result<String, RagError> {
        if chunks.is_empty() {
            return Ok(String::new());
        }

        // For MVP, just concatenate the chunks with separators
        // In a more sophisticated implementation, we would:
        // 1. Use an LLM to summarize or fuse the chunks
        // 2. Ensure the most relevant information is preserved
        // 3. Handle token limits more intelligently

        let fused_context = chunks
            .iter()
            .map(|chunk| format!("Source: {}\n\n{}", chunk.source, chunk.content))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        // Truncate if needed
        if fused_context.len() > max_length {
            Ok(fused_context[..max_length].to_string())
        } else {
            Ok(fused_context)
        }
    }

    /// Summarize multiple context chunks
    ///
    /// # Arguments
    ///
    /// * `chunks` - The chunks to summarize
    /// * `max_length` - The maximum length of the summary
    ///
    /// # Returns
    ///
    /// The summary, truncated to max_length if necessary
    pub async fn summarize_context(
        &self,
        chunks: &[ContextChunk],
        max_length: usize,
    ) -> Result<String, RagError> {
        // For MVP, just return the fused context
        // In a real implementation, you would use an LLM to summarize the context
        self.fuse_context(chunks, max_length).await
    }
}

impl Default for RagManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::rag_manager::file_source::FileContextSource;

    #[tokio::test]
    async fn test_rag_manager_add_remove_source() {
        let mut manager = RagManager::new();

        // Create a source
        let source = Arc::new(FileContextSource::new(
            "This is a test document.".to_string(),
            "test.txt".to_string(),
        ));

        // Add the source
        manager.add_source(source.clone());

        // Check that the source was added
        assert_eq!(manager.get_sources().len(), 1);
        assert!(manager.get_source("test.txt").is_some());

        // Remove the source
        let removed = manager.remove_source("test.txt");
        assert!(removed.is_some());

        // Check that the source was removed
        assert_eq!(manager.get_sources().len(), 0);
        assert!(manager.get_source("test.txt").is_none());
    }

    #[tokio::test]
    async fn test_rag_manager_retrieve_context() {
        let mut manager = RagManager::new();

        // Create a source
        let source = Arc::new(FileContextSource::new(
            "This is a test document.".to_string(),
            "test.txt".to_string(),
        ));

        // Add the source
        manager.add_source(source);

        // Retrieve context
        let chunks = manager.retrieve_context("test", 1).await.unwrap();

        // Verify the context
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "This is a test document.");
        assert_eq!(chunks[0].source, "test.txt");
    }

    #[tokio::test]
    async fn test_rag_manager_inject_context() {
        let mut manager = RagManager::new();

        // Create a source
        let source = Arc::new(FileContextSource::new(
            "This is a test document.".to_string(),
            "test.txt".to_string(),
        ));

        // Add the source
        manager.add_source(source);

        // Create a chat completion request
        let mut request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "What is in the test document?".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            }],
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            functions: None,
            tools: None,
            additional_params: None,
        };

        // Inject context
        manager
            .inject_context(&mut request, "test", 1)
            .await
            .unwrap();

        // Verify the request
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert!(request.messages[0]
            .content
            .contains("This is a test document."));
    }

    #[tokio::test]
    async fn test_rag_manager_fuse_context() {
        let manager = RagManager::new();

        // Create some chunks
        let chunks = vec![
            ContextChunk {
                content: "This is chunk 1.".to_string(),
                source: "source1".to_string(),
                relevance_score: 0.9,
                metadata: HashMap::new(),
            },
            ContextChunk {
                content: "This is chunk 2.".to_string(),
                source: "source2".to_string(),
                relevance_score: 0.8,
                metadata: HashMap::new(),
            },
        ];

        // Fuse the chunks
        let fused = manager.fuse_context(&chunks, 1000).await.unwrap();

        // Verify the fused context
        assert!(fused.contains("This is chunk 1."));
        assert!(fused.contains("This is chunk 2."));
        assert!(fused.contains("Source: source1"));
        assert!(fused.contains("Source: source2"));
    }
}
