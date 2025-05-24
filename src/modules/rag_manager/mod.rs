//! RAG Manager Module
//!
//! This module handles Retrieval-Augmented Generation (RAG) integration.
//! It provides functionality for document indexing, retrieval, and
//! integration with LLM requests.

// Private module declarations
pub mod file_source;
pub mod manager;
pub mod source;
pub mod types;

// Re-export specific types for public API
pub use file_source::FileContextSource;
pub use manager::RagManager;
pub use source::ContextSource;
pub use types::{ContextChunk, Document as RagDocument, RAGConfig, RagError};

// Import these from the IPC module instead
pub use crate::modules::ipc::rag_manager::{
    AugmentRequestResponse, Document as IpcDocument, IndexDocumentResponse, ListDocumentsResponse,
    RAGManagerClient, RetrieveDocumentsResponse,
};

// Provide backward-compatible functions

/// Initialize the RAG system with the specified configuration
pub fn init(_config: RAGConfig) -> Result<(), String> {
    // This is just a stub for backward compatibility
    // In a real implementation, this would initialize the RAG system
    Ok(())
}

/// Index a document for retrieval
pub fn index_document(_doc: IpcDocument) -> Result<(), String> {
    // This is just a stub for backward compatibility
    // In a real implementation, this would index the document
    Ok(())
}

/// Retrieve relevant documents for a query
pub fn retrieve(_query: &str, _top_k: usize) -> Vec<IpcDocument> {
    // This is just a stub for backward compatibility
    // In a real implementation, this would retrieve documents
    Vec::new()
}

/// Augment a request with retrieved context
pub fn augment_request(request: &str) -> String {
    // This is just a stub for backward compatibility
    // In a real implementation, this would augment the request
    request.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_registry::connectors::{
        ChatCompletionRequest, ChatMessage, MessageRole,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_integration() {
        // Create a RAG manager
        let mut manager = RagManager::new();

        // Create a file context source
        let source = Arc::new(FileContextSource::new(
            "This is a test document.".to_string(),
            "test.txt".to_string(),
        ));

        // Add the source to the manager
        manager.add_source(source);

        // Retrieve context
        let chunks = manager.retrieve_context("test", 1).await.unwrap();

        // Verify the context
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "This is a test document.");
        assert_eq!(chunks[0].source, "test.txt");

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
}
