//! Persona Layer IPC interface
//!
//! This module provides trait-based abstractions for the Persona Layer service,
//! ensuring a clear separation between interface and transport logic.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::rag_manager::{Document, ScoredDocument};
use crate::modules::ipc::{IpcError, IpcResult};

/// Represents a persona configuration
#[derive(Debug, Clone)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub response_format: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub version: VersionInfo,
}

/// Represents version information
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Client interface for the Persona Layer service
#[async_trait]
pub trait PersonaLayerClient: Send + Sync {
    /// Create a new persona
    async fn create_persona(
        &self,
        name: &str,
        description: &str,
        system_prompt: &str,
        response_format: Option<&str>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<Persona>;

    /// Get a persona by ID
    async fn get_persona(&self, persona_id: &str) -> IpcResult<Persona>;

    /// Update an existing persona
    async fn update_persona(
        &self,
        persona_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        system_prompt: Option<&str>,
        response_format: Option<&str>,
        metadata: Option<HashMap<String, String>>,
        tags: Option<Vec<String>>,
    ) -> IpcResult<Persona>;

    /// Delete a persona
    async fn delete_persona(&self, persona_id: &str) -> IpcResult<()>;

    /// List all personas
    async fn list_personas(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        tag_filter: Option<Vec<String>>,
    ) -> IpcResult<ListPersonasResponse>;

    /// Apply a persona to a request
    async fn apply_persona(
        &self,
        persona_id: Option<&str>,
        persona: Option<Persona>,
        request: &str,
        additional_context: Option<&str>,
        include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse>;
}

/// Server interface for the Persona Layer service
#[async_trait]
pub trait PersonaLayerService: Send + Sync {
    /// Create a new persona
    async fn create_persona(
        &self,
        name: &str,
        description: &str,
        system_prompt: &str,
        response_format: Option<&str>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<Persona>;

    /// Get a persona by ID
    async fn get_persona(&self, persona_id: &str) -> IpcResult<Persona>;

    /// Update an existing persona
    async fn update_persona(
        &self,
        persona_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        system_prompt: Option<&str>,
        response_format: Option<&str>,
        metadata: Option<HashMap<String, String>>,
        tags: Option<Vec<String>>,
    ) -> IpcResult<Persona>;

    /// Delete a persona
    async fn delete_persona(&self, persona_id: &str) -> IpcResult<()>;

    /// List all personas
    async fn list_personas(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        tag_filter: Option<Vec<String>>,
    ) -> IpcResult<ListPersonasResponse>;

    /// Apply a persona to a request
    async fn apply_persona(
        &self,
        persona_id: Option<&str>,
        persona: Option<Persona>,
        request: &str,
        additional_context: Option<&str>,
        include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse>;
}

/// Client interface for the RAG-Persona integration service
#[async_trait]
pub trait RAGPersonaIntegrationClient: Send + Sync {
    /// Augment a persona request with RAG context
    async fn augment_persona_request(
        &self,
        persona_id: &str,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<&str>,
    ) -> IpcResult<AugmentPersonaRequestResponse>;
}

/// Server interface for the RAG-Persona integration service
#[async_trait]
pub trait RAGPersonaIntegrationService: Send + Sync {
    /// Augment a persona request with RAG context
    async fn augment_persona_request(
        &self,
        persona_id: &str,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<&str>,
    ) -> IpcResult<AugmentPersonaRequestResponse>;
}

/// Response for list_personas
#[derive(Debug, Clone)]
pub struct ListPersonasResponse {
    pub personas: Vec<Persona>,
    pub total_count: u32,
}

/// Response for apply_persona
#[derive(Debug, Clone)]
pub struct ApplyPersonaResponse {
    pub personalized_request: String,
    pub applied_persona: Persona,
}

/// Response for augment_persona_request
#[derive(Debug, Clone)]
pub struct AugmentPersonaRequestResponse {
    pub augmented_request: String,
    pub persona_id: String,
    pub document_count: u32,
}

/// gRPC implementation of the Persona Layer client
pub struct GrpcPersonaLayerClient {
    // This would contain the generated gRPC client
    // client: persona_layer_client::PersonaLayerClient<tonic::transport::Channel>,
}

impl GrpcPersonaLayerClient {
    /// Create a new gRPC Persona Layer client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = persona_layer_client::PersonaLayerClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl PersonaLayerClient for GrpcPersonaLayerClient {
    async fn create_persona(
        &self,
        _name: &str,
        _description: &str,
        _system_prompt: &str,
        _response_format: Option<&str>,
        _metadata: HashMap<String, String>,
        _tags: Vec<String>,
    ) -> IpcResult<Persona> {
        // Stub implementation for now
        Ok(Persona {
            id: "stub-persona".to_string(),
            name: "Stub Persona".to_string(),
            description: "A stub persona for testing".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            response_format: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
            tags: vec!["stub".to_string()],
            version: VersionInfo {
                major: 1,
                minor: 0,
                patch: 0,
            },
        })
    }

    async fn get_persona(&self, _persona_id: &str) -> IpcResult<Persona> {
        // Stub implementation for now
        Ok(Persona {
            id: "stub-persona".to_string(),
            name: "Stub Persona".to_string(),
            description: "A stub persona for testing".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            response_format: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
            tags: vec!["stub".to_string()],
            version: VersionInfo {
                major: 1,
                minor: 0,
                patch: 0,
            },
        })
    }

    async fn update_persona(
        &self,
        _persona_id: &str,
        _name: Option<&str>,
        _description: Option<&str>,
        _system_prompt: Option<&str>,
        _response_format: Option<&str>,
        _metadata: Option<HashMap<String, String>>,
        _tags: Option<Vec<String>>,
    ) -> IpcResult<Persona> {
        // Stub implementation for now
        Ok(Persona {
            id: "stub-persona".to_string(),
            name: "Updated Stub Persona".to_string(),
            description: "An updated stub persona for testing".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            response_format: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
            tags: vec!["stub".to_string(), "updated".to_string()],
            version: VersionInfo {
                major: 1,
                minor: 0,
                patch: 0,
            },
        })
    }

    async fn delete_persona(&self, _persona_id: &str) -> IpcResult<()> {
        // Stub implementation for now
        Ok(())
    }

    async fn list_personas(
        &self,
        _limit: Option<u32>,
        _offset: Option<u32>,
        _tag_filter: Option<Vec<String>>,
    ) -> IpcResult<ListPersonasResponse> {
        // Stub implementation for now
        Ok(ListPersonasResponse {
            personas: vec![Persona {
                id: "stub-persona".to_string(),
                name: "Stub Persona".to_string(),
                description: "A stub persona for testing".to_string(),
                system_prompt: "You are a helpful assistant.".to_string(),
                response_format: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
                tags: vec!["stub".to_string()],
                version: VersionInfo {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
            }],
            total_count: 1,
        })
    }

    async fn apply_persona(
        &self,
        _persona_id: Option<&str>,
        _persona: Option<Persona>,
        _request: &str,
        _additional_context: Option<&str>,
        _include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse> {
        // Stub implementation for now
        Ok(ApplyPersonaResponse {
            personalized_request: "This is a request processed with a stub persona.".to_string(),
            applied_persona: Persona {
                id: "stub-persona".to_string(),
                name: "Stub Persona".to_string(),
                description: "A stub persona for testing".to_string(),
                system_prompt: "You are a helpful assistant.".to_string(),
                response_format: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
                tags: vec!["stub".to_string()],
                version: VersionInfo {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
            },
        })
    }
}

/// gRPC implementation of the RAG-Persona integration client
pub struct GrpcRAGPersonaIntegrationClient {
    // This would contain the generated gRPC client
    // client: rag_persona_integration_client::RAGPersonaIntegrationClient<tonic::transport::Channel>,
}

impl GrpcRAGPersonaIntegrationClient {
    /// Create a new gRPC RAG-Persona integration client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = rag_persona_integration_client::RAGPersonaIntegrationClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl RAGPersonaIntegrationClient for GrpcRAGPersonaIntegrationClient {
    async fn augment_persona_request(
        &self,
        _persona_id: &str,
        _request: &str,
        _top_k: Option<u32>,
        _min_score: Option<f32>,
        _metadata_filter: Option<HashMap<String, String>>,
        _include_citations: bool,
        _max_context_length: Option<u32>,
        _context_template: Option<&str>,
    ) -> IpcResult<AugmentPersonaRequestResponse> {
        // Stub implementation for now
        Ok(AugmentPersonaRequestResponse {
            augmented_request: "This is a request augmented with RAG content.".to_string(),
            persona_id: "stub-persona".to_string(),
            document_count: 3,
        })
    }
}

/// Mock implementation of the Persona Layer client for testing
#[cfg(test)]
pub struct MockPersonaLayerClient {
    personas: HashMap<String, Persona>,
}

#[cfg(test)]
impl MockPersonaLayerClient {
    /// Create a new mock Persona Layer client
    pub fn new() -> Self {
        Self {
            personas: HashMap::new(),
        }
    }

    /// Add a persona to the mock client
    pub fn add_persona(&mut self, persona: Persona) {
        self.personas.insert(persona.id.clone(), persona);
    }
}

#[cfg(test)]
#[async_trait]
impl PersonaLayerClient for MockPersonaLayerClient {
    async fn create_persona(
        &self,
        name: &str,
        description: &str,
        system_prompt: &str,
        response_format: Option<&str>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<Persona> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let persona = Persona {
            id,
            name: name.to_string(),
            description: description.to_string(),
            system_prompt: system_prompt.to_string(),
            response_format: response_format.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
            metadata,
            tags,
            version: VersionInfo {
                major: 1,
                minor: 0,
                patch: 0,
            },
        };

        Ok(persona)
    }

    async fn get_persona(&self, persona_id: &str) -> IpcResult<Persona> {
        self.personas
            .get(persona_id)
            .cloned()
            .ok_or_else(|| IpcError::NotFound(format!("Persona not found: {}", persona_id)))
    }

    async fn update_persona(
        &self,
        persona_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        system_prompt: Option<&str>,
        response_format: Option<&str>,
        metadata: Option<HashMap<String, String>>,
        tags: Option<Vec<String>>,
    ) -> IpcResult<Persona> {
        let mut persona = self.get_persona(persona_id).await?;

        if let Some(name) = name {
            persona.name = name.to_string();
        }

        if let Some(description) = description {
            persona.description = description.to_string();
        }

        if let Some(system_prompt) = system_prompt {
            persona.system_prompt = system_prompt.to_string();
        }

        if let Some(response_format) = response_format {
            persona.response_format = Some(response_format.to_string());
        }

        if let Some(metadata) = metadata {
            persona.metadata = metadata;
        }

        if let Some(tags) = tags {
            persona.tags = tags;
        }

        persona.updated_at = Utc::now();

        Ok(persona)
    }

    async fn delete_persona(&self, persona_id: &str) -> IpcResult<()> {
        if self.personas.contains_key(persona_id) {
            Ok(())
        } else {
            Err(IpcError::NotFound(format!(
                "Persona not found: {}",
                persona_id
            )))
        }
    }

    async fn list_personas(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        tag_filter: Option<Vec<String>>,
    ) -> IpcResult<ListPersonasResponse> {
        let mut personas = self.personas.values().cloned().collect::<Vec<_>>();

        if let Some(tag_filter) = tag_filter {
            personas.retain(|p| {
                for tag in &tag_filter {
                    if !p.tags.contains(tag) {
                        return false;
                    }
                }
                true
            });
        }

        let total_count = personas.len() as u32;

        if let Some(offset) = offset {
            let offset = offset as usize;
            if offset < personas.len() {
                personas = personas[offset..].to_vec();
            } else {
                personas = Vec::new();
            }
        }

        if let Some(limit) = limit {
            let limit = limit as usize;
            if limit < personas.len() {
                personas = personas[..limit].to_vec();
            }
        }

        Ok(ListPersonasResponse {
            personas,
            total_count,
        })
    }

    async fn apply_persona(
        &self,
        persona_id: Option<&str>,
        persona: Option<Persona>,
        request: &str,
        additional_context: Option<&str>,
        include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse> {
        let persona = if let Some(persona_id) = persona_id {
            self.get_persona(persona_id).await?
        } else if let Some(persona) = persona {
            persona
        } else {
            return Err(IpcError::InvalidArgument(
                "Either persona_id or persona must be provided".to_string(),
            ));
        };

        let mut personalized_request = String::new();

        if include_description && !persona.description.is_empty() {
            personalized_request.push_str(&format!("Description: {}\n\n", persona.description));
        }

        personalized_request.push_str(&format!("System: {}\n\n", persona.system_prompt));

        if let Some(additional_context) = additional_context {
            personalized_request.push_str(&format!("Context: {}\n\n", additional_context));
        }

        personalized_request.push_str(&format!("Request: {}", request));

        Ok(ApplyPersonaResponse {
            personalized_request,
            applied_persona: persona,
        })
    }
}

/// Mock implementation of the RAG-Persona integration client for testing
#[cfg(test)]
pub struct MockRAGPersonaIntegrationClient {
    persona_client: MockPersonaLayerClient,
}

#[cfg(test)]
impl MockRAGPersonaIntegrationClient {
    /// Create a new mock RAG-Persona integration client
    pub fn new(persona_client: MockPersonaLayerClient) -> Self {
        Self { persona_client }
    }
}

#[cfg(test)]
#[async_trait]
impl RAGPersonaIntegrationClient for MockRAGPersonaIntegrationClient {
    async fn augment_persona_request(
        &self,
        persona_id: &str,
        request: &str,
        _top_k: Option<u32>,
        _min_score: Option<f32>,
        _metadata_filter: Option<HashMap<String, String>>,
        _include_citations: bool,
        _max_context_length: Option<u32>,
        _context_template: Option<&str>,
    ) -> IpcResult<AugmentPersonaRequestResponse> {
        let persona = self.persona_client.get_persona(persona_id).await?;

        // Mock augmented request
        let augmented_request = format!(
            "Augmented with RAG context:\n\nSystem: {}\n\nRequest: {}",
            persona.system_prompt, request
        );

        Ok(AugmentPersonaRequestResponse {
            augmented_request,
            persona_id: persona_id.to_string(),
            document_count: 3, // Mock document count
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_persona_layer_client() {
        let mut client = MockPersonaLayerClient::new();

        // Test create_persona
        let persona = client
            .create_persona(
                "Test Persona",
                "A test persona",
                "You are a helpful assistant.",
                None,
                HashMap::new(),
                vec!["test".to_string()],
            )
            .await
            .unwrap();

        assert_eq!(persona.name, "Test Persona");
        assert_eq!(persona.description, "A test persona");
        assert_eq!(persona.system_prompt, "You are a helpful assistant.");

        // Add the persona to the mock client
        client.add_persona(persona.clone());

        // Test get_persona
        let result = client.get_persona(&persona.id).await.unwrap();
        assert_eq!(result.id, persona.id);

        // Test update_persona
        let updated = client
            .update_persona(
                &persona.id,
                Some("Updated Persona"),
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "Updated Persona");
        assert_eq!(updated.description, "A test persona"); // Unchanged

        // Test apply_persona
        let applied = client
            .apply_persona(Some(&persona.id), None, "Hello, world!", None, true)
            .await
            .unwrap();

        assert!(applied
            .personalized_request
            .contains("Description: A test persona"));
        assert!(applied
            .personalized_request
            .contains("System: You are a helpful assistant."));
        assert!(applied
            .personalized_request
            .contains("Request: Hello, world!"));

        // Test get_persona with non-existent ID
        let result = client.get_persona("non-existent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_rag_persona_integration_client() {
        let mut persona_client = MockPersonaLayerClient::new();

        // Create a test persona
        let persona = persona_client
            .create_persona(
                "Test Persona",
                "A test persona",
                "You are a helpful assistant.",
                None,
                HashMap::new(),
                vec!["test".to_string()],
            )
            .await
            .unwrap();

        persona_client.add_persona(persona.clone());

        let integration_client = MockRAGPersonaIntegrationClient::new(persona_client);

        // Test augment_persona_request
        let augmented = integration_client
            .augment_persona_request(
                &persona.id,
                "Hello, world!",
                Some(3),
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
            .contains("Augmented with RAG context"));
        assert!(augmented
            .augmented_request
            .contains("System: You are a helpful assistant."));
        assert!(augmented
            .augmented_request
            .contains("Request: Hello, world!"));
        assert_eq!(augmented.persona_id, persona.id);
        assert_eq!(augmented.document_count, 3);

        // Test augment_persona_request with non-existent ID
        let result = integration_client
            .augment_persona_request(
                "non-existent",
                "Hello, world!",
                None,
                None,
                None,
                false,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
    }
}
