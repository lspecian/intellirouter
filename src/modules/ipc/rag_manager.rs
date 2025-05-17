//! RAG Manager IPC interface
//!
//! This module provides trait-based abstractions for the RAG Manager service,
//! ensuring a clear separation between interface and transport logic.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::{IpcError, IpcResult};

/// Represents a document for RAG
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub embedding: Option<Vec<f32>>,
    pub chunks: Vec<DocumentChunk>,
}

/// Represents a chunk of a document
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub embedding: Option<Vec<f32>>,
    pub document_id: String,
    pub chunk_index: u32,
}

/// Represents a document with a similarity score
#[derive(Debug, Clone)]
pub struct ScoredDocument {
    pub document: Document,
    pub score: f32,
}

/// Client interface for the RAG Manager service
#[async_trait]
pub trait RAGManagerClient: Send + Sync {
    /// Index a document for retrieval
    async fn index_document(
        &self,
        document: Document,
        chunk_size: Option<u32>,
        chunk_overlap: Option<u32>,
        compute_embeddings: bool,
        embedding_model: Option<&str>,
    ) -> IpcResult<IndexDocumentResponse>;

    /// Retrieve relevant documents for a query
    async fn retrieve_documents(
        &self,
        query: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_content: bool,
        rerank: bool,
        rerank_model: Option<&str>,
    ) -> IpcResult<RetrieveDocumentsResponse>;

    /// Augment a request with retrieved context
    async fn augment_request(
        &self,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<&str>,
    ) -> IpcResult<AugmentRequestResponse>;

    /// Get a document by ID
    async fn get_document_by_id(
        &self,
        document_id: &str,
        include_chunks: bool,
    ) -> IpcResult<Document>;

    /// Delete a document from the index
    async fn delete_document(&self, document_id: &str) -> IpcResult<()>;

    /// List all documents in the index
    async fn list_documents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        metadata_filter: Option<HashMap<String, String>>,
    ) -> IpcResult<ListDocumentsResponse>;
}

/// Server interface for the RAG Manager service
#[async_trait]
pub trait RAGManagerService: Send + Sync {
    /// Index a document for retrieval
    async fn index_document(
        &self,
        document: Document,
        chunk_size: Option<u32>,
        chunk_overlap: Option<u32>,
        compute_embeddings: bool,
        embedding_model: Option<&str>,
    ) -> IpcResult<IndexDocumentResponse>;

    /// Retrieve relevant documents for a query
    async fn retrieve_documents(
        &self,
        query: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_content: bool,
        rerank: bool,
        rerank_model: Option<&str>,
    ) -> IpcResult<RetrieveDocumentsResponse>;

    /// Augment a request with retrieved context
    async fn augment_request(
        &self,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<&str>,
    ) -> IpcResult<AugmentRequestResponse>;

    /// Get a document by ID
    async fn get_document_by_id(
        &self,
        document_id: &str,
        include_chunks: bool,
    ) -> IpcResult<Document>;

    /// Delete a document from the index
    async fn delete_document(&self, document_id: &str) -> IpcResult<()>;

    /// List all documents in the index
    async fn list_documents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        metadata_filter: Option<HashMap<String, String>>,
    ) -> IpcResult<ListDocumentsResponse>;
}

/// Response for index_document
#[derive(Debug, Clone)]
pub struct IndexDocumentResponse {
    pub document_id: String,
    pub chunk_count: u32,
}

/// Response for retrieve_documents
#[derive(Debug, Clone)]
pub struct RetrieveDocumentsResponse {
    pub documents: Vec<ScoredDocument>,
}

/// Response for augment_request
#[derive(Debug, Clone)]
pub struct AugmentRequestResponse {
    pub augmented_request: String,
    pub documents: Vec<ScoredDocument>,
}

/// Response for list_documents
#[derive(Debug, Clone)]
pub struct ListDocumentsResponse {
    pub documents: Vec<Document>,
    pub total_count: u32,
}

/// gRPC implementation of the RAG Manager client
pub struct GrpcRAGManagerClient {
    // This would contain the generated gRPC client
    // client: rag_manager_client::RAGManagerClient<tonic::transport::Channel>,
}

impl GrpcRAGManagerClient {
    /// Create a new gRPC RAG Manager client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = rag_manager_client::RAGManagerClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl RAGManagerClient for GrpcRAGManagerClient {
    async fn index_document(
        &self,
        _document: Document,
        _chunk_size: Option<u32>,
        _chunk_overlap: Option<u32>,
        _compute_embeddings: bool,
        _embedding_model: Option<&str>,
    ) -> IpcResult<IndexDocumentResponse> {
        // Stub implementation for now
        Ok(IndexDocumentResponse {
            document_id: "stub-document-id".to_string(),
            chunk_count: 5,
        })
    }

    async fn retrieve_documents(
        &self,
        _query: &str,
        _top_k: Option<u32>,
        _min_score: Option<f32>,
        _metadata_filter: Option<HashMap<String, String>>,
        _include_content: bool,
        _rerank: bool,
        _rerank_model: Option<&str>,
    ) -> IpcResult<RetrieveDocumentsResponse> {
        // Stub implementation for now
        Ok(RetrieveDocumentsResponse {
            documents: vec![ScoredDocument {
                document: Document {
                    id: "stub-document-1".to_string(),
                    content: "This is a stub document for testing.".to_string(),
                    metadata: HashMap::new(),
                    chunks: vec![],
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    embedding: None,
                },
                score: 0.95,
            }],
        })
    }

    async fn augment_request(
        &self,
        _request: &str,
        _top_k: Option<u32>,
        _min_score: Option<f32>,
        _metadata_filter: Option<HashMap<String, String>>,
        _include_citations: bool,
        _max_context_length: Option<u32>,
        _context_template: Option<&str>,
    ) -> IpcResult<AugmentRequestResponse> {
        // Stub implementation for now
        Ok(AugmentRequestResponse {
            augmented_request: "This is a request augmented with RAG content.".to_string(),
            documents: vec![ScoredDocument {
                document: Document {
                    id: "stub-document-1".to_string(),
                    content: "This is a stub document for testing.".to_string(),
                    metadata: HashMap::new(),
                    chunks: vec![],
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    embedding: None,
                },
                score: 0.95,
            }],
        })
    }

    async fn get_document_by_id(
        &self,
        _document_id: &str,
        _include_chunks: bool,
    ) -> IpcResult<Document> {
        // Stub implementation for now
        Ok(Document {
            id: "stub-document-1".to_string(),
            content: "This is a stub document for testing.".to_string(),
            metadata: HashMap::new(),
            chunks: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            embedding: None,
        })
    }

    async fn delete_document(&self, _document_id: &str) -> IpcResult<()> {
        // Stub implementation for now
        Ok(())
    }

    async fn list_documents(
        &self,
        _limit: Option<u32>,
        _offset: Option<u32>,
        _metadata_filter: Option<HashMap<String, String>>,
    ) -> IpcResult<ListDocumentsResponse> {
        // Stub implementation for now
        Ok(ListDocumentsResponse {
            documents: vec![Document {
                id: "stub-document-1".to_string(),
                content: "This is a stub document for testing.".to_string(),
                metadata: HashMap::new(),
                chunks: vec![],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                embedding: None,
            }],
            total_count: 1,
        })
    }
}

/// Mock implementation of the RAG Manager client for testing
#[cfg(test)]
pub struct MockRAGManagerClient {
    documents: HashMap<String, Document>,
}

#[cfg(test)]
impl MockRAGManagerClient {
    /// Create a new mock RAG Manager client
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Add a document to the mock client
    pub fn add_document(&mut self, document: Document) {
        self.documents.insert(document.id.clone(), document);
    }
}

#[cfg(test)]
#[async_trait]
impl RAGManagerClient for MockRAGManagerClient {
    async fn index_document(
        &self,
        document: Document,
        _chunk_size: Option<u32>,
        _chunk_overlap: Option<u32>,
        _compute_embeddings: bool,
        _embedding_model: Option<&str>,
    ) -> IpcResult<IndexDocumentResponse> {
        Ok(IndexDocumentResponse {
            document_id: document.id,
            chunk_count: document.chunks.len() as u32,
        })
    }

    async fn retrieve_documents(
        &self,
        query: &str,
        top_k: Option<u32>,
        _min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        _include_content: bool,
        _rerank: bool,
        _rerank_model: Option<&str>,
    ) -> IpcResult<RetrieveDocumentsResponse> {
        let mut documents = Vec::new();

        for document in self.documents.values() {
            // Apply metadata filter if provided
            if let Some(filter) = &metadata_filter {
                let mut match_filter = true;
                for (key, value) in filter {
                    if !document.metadata.contains_key(key)
                        || document.metadata.get(key).unwrap() != value
                    {
                        match_filter = false;
                        break;
                    }
                }
                if !match_filter {
                    continue;
                }
            }

            // Simple mock scoring based on whether the query appears in the content
            if document.content.contains(query) {
                documents.push(ScoredDocument {
                    document: document.clone(),
                    score: 0.8,
                });
            }
        }

        // Sort by score (descending)
        documents.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Apply top_k if provided
        if let Some(top_k) = top_k {
            if documents.len() > top_k as usize {
                documents.truncate(top_k as usize);
            }
        }

        Ok(RetrieveDocumentsResponse { documents })
    }

    async fn augment_request(
        &self,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        _include_citations: bool,
        _max_context_length: Option<u32>,
        context_template: Option<&str>,
    ) -> IpcResult<AugmentRequestResponse> {
        let retrieved = self
            .retrieve_documents(
                request,
                top_k,
                min_score,
                metadata_filter,
                true,
                false,
                None,
            )
            .await?;

        let mut context = String::new();
        for doc in &retrieved.documents {
            context.push_str(&doc.document.content);
            context.push_str("\n\n");
        }

        let template = context_template.unwrap_or("Context: {context}\n\nRequest: {request}");
        let augmented_request = template
            .replace("{context}", &context)
            .replace("{request}", request);

        Ok(AugmentRequestResponse {
            augmented_request,
            documents: retrieved.documents,
        })
    }

    async fn get_document_by_id(
        &self,
        document_id: &str,
        _include_chunks: bool,
    ) -> IpcResult<Document> {
        self.documents
            .get(document_id)
            .cloned()
            .ok_or_else(|| IpcError::NotFound(format!("Document not found: {}", document_id)))
    }

    async fn delete_document(&self, document_id: &str) -> IpcResult<()> {
        if self.documents.contains_key(document_id) {
            Ok(())
        } else {
            Err(IpcError::NotFound(format!(
                "Document not found: {}",
                document_id
            )))
        }
    }

    async fn list_documents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        metadata_filter: Option<HashMap<String, String>>,
    ) -> IpcResult<ListDocumentsResponse> {
        let mut documents = Vec::new();

        for document in self.documents.values() {
            // Apply metadata filter if provided
            if let Some(filter) = &metadata_filter {
                let mut match_filter = true;
                for (key, value) in filter {
                    if !document.metadata.contains_key(key)
                        || document.metadata.get(key).unwrap() != value
                    {
                        match_filter = false;
                        break;
                    }
                }
                if !match_filter {
                    continue;
                }
            }

            documents.push(document.clone());
        }

        let total_count = documents.len() as u32;

        // Apply offset if provided
        if let Some(offset) = offset {
            let offset = offset as usize;
            if offset < documents.len() {
                documents = documents[offset..].to_vec();
            } else {
                documents = Vec::new();
            }
        }

        // Apply limit if provided
        if let Some(limit) = limit {
            let limit = limit as usize;
            if limit < documents.len() {
                documents = documents[..limit].to_vec();
            }
        }

        Ok(ListDocumentsResponse {
            documents,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_rag_manager_client() {
        let mut client = MockRAGManagerClient::new();

        // Create a test document
        let document = Document {
            id: "test-doc".to_string(),
            content: "This is a test document about artificial intelligence.".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map.insert("category".to_string(), "ai".to_string());
                map
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            embedding: None,
            chunks: Vec::new(),
        };

        // Add the document to the mock client
        client.add_document(document.clone());

        // Test get_document_by_id
        let result = client
            .get_document_by_id(&document.id, false)
            .await
            .unwrap();
        assert_eq!(result.id, document.id);

        // Test retrieve_documents
        let retrieved = client
            .retrieve_documents(
                "artificial intelligence",
                Some(10),
                None,
                None,
                true,
                false,
                None,
            )
            .await
            .unwrap();

        assert_eq!(retrieved.documents.len(), 1);
        assert_eq!(retrieved.documents[0].document.id, document.id);

        // Test retrieve_documents with metadata filter
        let mut filter = HashMap::new();
        filter.insert("category".to_string(), "ai".to_string());

        let retrieved = client
            .retrieve_documents(
                "artificial intelligence",
                Some(10),
                None,
                Some(filter),
                true,
                false,
                None,
            )
            .await
            .unwrap();

        assert_eq!(retrieved.documents.len(), 1);

        // Test retrieve_documents with non-matching metadata filter
        let mut filter = HashMap::new();
        filter.insert("category".to_string(), "non-existent".to_string());

        let retrieved = client
            .retrieve_documents(
                "artificial intelligence",
                Some(10),
                None,
                Some(filter),
                true,
                false,
                None,
            )
            .await
            .unwrap();

        assert_eq!(retrieved.documents.len(), 0);

        // Test augment_request
        let augmented = client
            .augment_request(
                "Tell me about artificial intelligence",
                Some(10),
                None,
                None,
                true,
                None,
                None,
            )
            .await
            .unwrap();

        assert!(augmented
            .augmented_request
            .contains("This is a test document about artificial intelligence"));
        assert!(augmented
            .augmented_request
            .contains("Tell me about artificial intelligence"));

        // Test get_document_by_id with non-existent ID
        let result = client.get_document_by_id("non-existent", false).await;
        assert!(result.is_err());
    }
}
