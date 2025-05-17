//! Memory IPC interface
//!
//! This module provides trait-based abstractions for the Memory service,
//! ensuring a clear separation between interface and transport logic.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::{IpcError, IpcResult};

/// Represents a message in a conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub id: String,
    pub parent_id: Option<String>,
    pub token_count: Option<u32>,
}

/// Represents a conversation
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}

/// Represents a message search result
#[derive(Debug, Clone)]
pub struct MessageSearchResult {
    pub message: Message,
    pub conversation_id: String,
    pub conversation_title: Option<String>,
    pub score: f32,
    pub highlighted_content: String,
}

/// Represents a step result for chain integration
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub step_name: String,
    pub input: String,
    pub output: String,
    pub model: String,
}

/// Client interface for the Memory service
#[async_trait]
pub trait MemoryClient: Send + Sync {
    /// Create a new conversation
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<Conversation>;

    /// Get a conversation by ID
    async fn get_conversation(&self, conversation_id: &str) -> IpcResult<Conversation>;

    /// Add a message to a conversation
    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<&str>,
    ) -> IpcResult<Message>;

    /// Get the conversation history formatted for an LLM request
    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
    ) -> IpcResult<GetHistoryResponse>;

    /// Save a conversation to persistent storage
    async fn save_conversation(&self, conversation_id: &str) -> IpcResult<()>;

    /// Load a conversation from persistent storage
    async fn load_conversation(&self, conversation_id: &str) -> IpcResult<Conversation>;

    /// Delete a conversation
    async fn delete_conversation(&self, conversation_id: &str) -> IpcResult<()>;

    /// List all conversations
    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<&str>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse>;

    /// Search for messages across conversations
    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<&str>,
        user_id: Option<&str>,
        role: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse>;
}

/// Server interface for the Memory service
#[async_trait]
pub trait MemoryService: Send + Sync {
    /// Create a new conversation
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<Conversation>;

    /// Get a conversation by ID
    async fn get_conversation(&self, conversation_id: &str) -> IpcResult<Conversation>;

    /// Add a message to a conversation
    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<&str>,
    ) -> IpcResult<Message>;

    /// Get the conversation history formatted for an LLM request
    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
    ) -> IpcResult<GetHistoryResponse>;

    /// Save a conversation to persistent storage
    async fn save_conversation(&self, conversation_id: &str) -> IpcResult<()>;

    /// Load a conversation from persistent storage
    async fn load_conversation(&self, conversation_id: &str) -> IpcResult<Conversation>;

    /// Delete a conversation
    async fn delete_conversation(&self, conversation_id: &str) -> IpcResult<()>;

    /// List all conversations
    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<&str>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse>;

    /// Search for messages across conversations
    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<&str>,
        user_id: Option<&str>,
        role: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse>;
}

/// Client interface for the Memory-Chain integration service
#[async_trait]
pub trait MemoryChainIntegrationClient: Send + Sync {
    /// Get conversation history formatted for a chain
    async fn get_conversation_history_for_chain(
        &self,
        conversation_id: &str,
        chain_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
        additional_context: Option<&str>,
    ) -> IpcResult<GetConversationHistoryForChainResponse>;

    /// Store a chain result in a conversation
    async fn store_chain_result_in_conversation(
        &self,
        conversation_id: &str,
        chain_id: &str,
        execution_id: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        metadata: HashMap<String, String>,
    ) -> IpcResult<StoreChainResultInConversationResponse>;

    /// Create a new conversation from a chain execution
    async fn create_conversation_from_chain_execution(
        &self,
        chain_id: &str,
        execution_id: &str,
        input: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> IpcResult<CreateConversationFromChainExecutionResponse>;
}

/// Server interface for the Memory-Chain integration service
#[async_trait]
pub trait MemoryChainIntegrationService: Send + Sync {
    /// Get conversation history formatted for a chain
    async fn get_conversation_history_for_chain(
        &self,
        conversation_id: &str,
        chain_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
        additional_context: Option<&str>,
    ) -> IpcResult<GetConversationHistoryForChainResponse>;

    /// Store a chain result in a conversation
    async fn store_chain_result_in_conversation(
        &self,
        conversation_id: &str,
        chain_id: &str,
        execution_id: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        metadata: HashMap<String, String>,
    ) -> IpcResult<StoreChainResultInConversationResponse>;

    /// Create a new conversation from a chain execution
    async fn create_conversation_from_chain_execution(
        &self,
        chain_id: &str,
        execution_id: &str,
        input: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> IpcResult<CreateConversationFromChainExecutionResponse>;
}

/// Response for get_history
#[derive(Debug, Clone)]
pub struct GetHistoryResponse {
    pub messages: Vec<Message>,
    pub total_tokens: u32,
    pub formatted_history: Option<String>,
}

/// Response for list_conversations
#[derive(Debug, Clone)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
    pub total_count: u32,
}

/// Response for search_messages
#[derive(Debug, Clone)]
pub struct SearchMessagesResponse {
    pub results: Vec<MessageSearchResult>,
    pub total_count: u32,
}

/// Response for get_conversation_history_for_chain
#[derive(Debug, Clone)]
pub struct GetConversationHistoryForChainResponse {
    pub formatted_history: String,
    pub messages: Vec<Message>,
    pub total_tokens: u32,
}

/// Response for store_chain_result_in_conversation
#[derive(Debug, Clone)]
pub struct StoreChainResultInConversationResponse {
    pub message_id: String,
    pub step_message_ids: Vec<String>,
}

/// Response for create_conversation_from_chain_execution
#[derive(Debug, Clone)]
pub struct CreateConversationFromChainExecutionResponse {
    pub conversation_id: String,
}

/// gRPC implementation of the Memory client
pub struct GrpcMemoryClient {
    // This would contain the generated gRPC client
    // client: memory_client::MemoryClient<tonic::transport::Channel>,
}

impl GrpcMemoryClient {
    /// Create a new gRPC Memory client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = memory_client::MemoryClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl MemoryClient for GrpcMemoryClient {
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<Conversation> {
        // Implementation would use the generated gRPC client
        todo!("Implement create_conversation using gRPC client")
    }

    async fn get_conversation(&self, conversation_id: &str) -> IpcResult<Conversation> {
        // Implementation would use the generated gRPC client
        todo!("Implement get_conversation using gRPC client")
    }

    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<&str>,
    ) -> IpcResult<Message> {
        // Implementation would use the generated gRPC client
        todo!("Implement add_message using gRPC client")
    }

    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
    ) -> IpcResult<GetHistoryResponse> {
        // Implementation would use the generated gRPC client
        todo!("Implement get_history using gRPC client")
    }

    async fn save_conversation(&self, conversation_id: &str) -> IpcResult<()> {
        // Implementation would use the generated gRPC client
        todo!("Implement save_conversation using gRPC client")
    }

    async fn load_conversation(&self, conversation_id: &str) -> IpcResult<Conversation> {
        // Implementation would use the generated gRPC client
        todo!("Implement load_conversation using gRPC client")
    }

    async fn delete_conversation(&self, conversation_id: &str) -> IpcResult<()> {
        // Implementation would use the generated gRPC client
        todo!("Implement delete_conversation using gRPC client")
    }

    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<&str>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse> {
        // Implementation would use the generated gRPC client
        todo!("Implement list_conversations using gRPC client")
    }

    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<&str>,
        user_id: Option<&str>,
        role: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse> {
        // Implementation would use the generated gRPC client
        todo!("Implement search_messages using gRPC client")
    }
}

/// gRPC implementation of the Memory-Chain integration client
pub struct GrpcMemoryChainIntegrationClient {
    // This would contain the generated gRPC client
    // client: memory_chain_integration_client::MemoryChainIntegrationClient<tonic::transport::Channel>,
}

impl GrpcMemoryChainIntegrationClient {
    /// Create a new gRPC Memory-Chain integration client
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        // This would create the gRPC client
        // let client = memory_chain_integration_client::MemoryChainIntegrationClient::connect(addr).await?;
        Ok(Self {
            // client,
        })
    }
}

#[async_trait]
impl MemoryChainIntegrationClient for GrpcMemoryChainIntegrationClient {
    async fn get_conversation_history_for_chain(
        &self,
        conversation_id: &str,
        chain_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
        additional_context: Option<&str>,
    ) -> IpcResult<GetConversationHistoryForChainResponse> {
        // Implementation would use the generated gRPC client
        todo!("Implement get_conversation_history_for_chain using gRPC client")
    }

    async fn store_chain_result_in_conversation(
        &self,
        conversation_id: &str,
        chain_id: &str,
        execution_id: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        metadata: HashMap<String, String>,
    ) -> IpcResult<StoreChainResultInConversationResponse> {
        // Implementation would use the generated gRPC client
        todo!("Implement store_chain_result_in_conversation using gRPC client")
    }

    async fn create_conversation_from_chain_execution(
        &self,
        chain_id: &str,
        execution_id: &str,
        input: &str,
        result: &str,
        step_results: Vec<StepResult>,
        store_step_results: bool,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> IpcResult<CreateConversationFromChainExecutionResponse> {
        // Implementation would use the generated gRPC client
        todo!("Implement create_conversation_from_chain_execution using gRPC client")
    }
}

/// Mock implementation of the Memory client for testing
#[cfg(test)]
pub struct MockMemoryClient {
    conversations: HashMap<String, Conversation>,
}

#[cfg(test)]
impl MockMemoryClient {
    /// Create a new mock Memory client
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
        }
    }

    /// Add a conversation to the mock client
    pub fn add_conversation(&mut self, conversation: Conversation) {
        self.conversations
            .insert(conversation.id.clone(), conversation);
    }
}

#[cfg(test)]
#[async_trait]
impl MemoryClient for MockMemoryClient {
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<Conversation> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let conversation = Conversation {
            id,
            messages: initial_messages,
            metadata,
            created_at: now,
            updated_at: now,
            user_id: user_id.to_string(),
            title: title.map(|t| t.to_string()),
            tags,
        };

        Ok(conversation)
    }

    async fn get_conversation(&self, conversation_id: &str) -> IpcResult<Conversation> {
        self.conversations
            .get(conversation_id)
            .cloned()
            .ok_or_else(|| {
                IpcError::NotFound(format!("Conversation not found: {}", conversation_id))
            })
    }

    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<&str>,
    ) -> IpcResult<Message> {
        if !self.conversations.contains_key(conversation_id) {
            return Err(IpcError::NotFound(format!(
                "Conversation not found: {}",
                conversation_id
            )));
        }

        let message = Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata,
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: parent_id.map(|id| id.to_string()),
            token_count: None,
        };

        Ok(message)
    }

    async fn get_history(
        &self,
        conversation_id: &str,
        _max_tokens: Option<u32>,
        _max_messages: Option<u32>,
        _include_system_messages: bool,
        _format: Option<&str>,
    ) -> IpcResult<GetHistoryResponse> {
        if let Some(conversation) = self.conversations.get(conversation_id) {
            Ok(GetHistoryResponse {
                messages: conversation.messages.clone(),
                total_tokens: 0,
                formatted_history: None,
            })
        } else {
            Err(IpcError::NotFound(format!(
                "Conversation not found: {}",
                conversation_id
            )))
        }
    }

    async fn save_conversation(&self, conversation_id: &str) -> IpcResult<()> {
        if self.conversations.contains_key(conversation_id) {
            Ok(())
        } else {
            Err(IpcError::NotFound(format!(
                "Conversation not found: {}",
                conversation_id
            )))
        }
    }

    async fn load_conversation(&self, conversation_id: &str) -> IpcResult<Conversation> {
        self.conversations
            .get(conversation_id)
            .cloned()
            .ok_or_else(|| {
                IpcError::NotFound(format!("Conversation not found: {}", conversation_id))
            })
    }

    async fn delete_conversation(&self, conversation_id: &str) -> IpcResult<()> {
        if self.conversations.contains_key(conversation_id) {
            Ok(())
        } else {
            Err(IpcError::NotFound(format!(
                "Conversation not found: {}",
                conversation_id
            )))
        }
    }

    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<&str>,
        _tag_filter: Vec<String>,
        _start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse> {
        let mut conversations = self.conversations.values().cloned().collect::<Vec<_>>();

        if let Some(user_id) = user_id {
            conversations.retain(|c| c.user_id == user_id);
        }

        let total_count = conversations.len() as u32;

        if let Some(offset) = offset {
            let offset = offset as usize;
            if offset < conversations.len() {
                conversations = conversations[offset..].to_vec();
            } else {
                conversations = Vec::new();
            }
        }

        if let Some(limit) = limit {
            let limit = limit as usize;
            if limit < conversations.len() {
                conversations = conversations[..limit].to_vec();
            }
        }

        Ok(ListConversationsResponse {
            conversations,
            total_count,
        })
    }

    async fn search_messages(
        &self,
        query: &str,
        _limit: Option<u32>,
        _offset: Option<u32>,
        conversation_id: Option<&str>,
        _user_id: Option<&str>,
        _role: Option<&str>,
        _start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse> {
        let mut results = Vec::new();

        for (id, conversation) in &self.conversations {
            if let Some(conv_id) = conversation_id {
                if id != conv_id {
                    continue;
                }
            }

            for message in &conversation.messages {
                if message.content.contains(query) {
                    results.push(MessageSearchResult {
                        message: message.clone(),
                        conversation_id: id.clone(),
                        conversation_title: conversation.title.clone(),
                        score: 1.0,
                        highlighted_content: message.content.clone(),
                    });
                }
            }
        }

        Ok(SearchMessagesResponse {
            results,
            total_count: results.len() as u32,
        })
    }
}

/// Mock implementation of the Memory-Chain integration client for testing
#[cfg(test)]
pub struct MockMemoryChainIntegrationClient {
    memory_client: MockMemoryClient,
}

#[cfg(test)]
impl MockMemoryChainIntegrationClient {
    /// Create a new mock Memory-Chain integration client
    pub fn new(memory_client: MockMemoryClient) -> Self {
        Self { memory_client }
    }
}

#[cfg(test)]
#[async_trait]
impl MemoryChainIntegrationClient for MockMemoryChainIntegrationClient {
    async fn get_conversation_history_for_chain(
        &self,
        conversation_id: &str,
        _chain_id: &str,
        _max_tokens: Option<u32>,
        _max_messages: Option<u32>,
        _include_system_messages: bool,
        _format: Option<&str>,
        _additional_context: Option<&str>,
    ) -> IpcResult<GetConversationHistoryForChainResponse> {
        let history = self
            .memory_client
            .get_history(conversation_id, None, None, false, None)
            .await?;

        Ok(GetConversationHistoryForChainResponse {
            formatted_history: "Mock formatted history".to_string(),
            messages: history.messages,
            total_tokens: 0,
        })
    }

    async fn store_chain_result_in_conversation(
        &self,
        conversation_id: &str,
        _chain_id: &str,
        _execution_id: &str,
        result: &str,
        _step_results: Vec<StepResult>,
        _store_step_results: bool,
        metadata: HashMap<String, String>,
    ) -> IpcResult<StoreChainResultInConversationResponse> {
        let message = self
            .memory_client
            .add_message(conversation_id, "assistant", result, metadata, None)
            .await?;

        Ok(StoreChainResultInConversationResponse {
            message_id: message.id,
            step_message_ids: Vec::new(),
        })
    }

    async fn create_conversation_from_chain_execution(
        &self,
        _chain_id: &str,
        _execution_id: &str,
        input: &str,
        result: &str,
        _step_results: Vec<StepResult>,
        _store_step_results: bool,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> IpcResult<CreateConversationFromChainExecutionResponse> {
        let now = Utc::now();

        let user_message = Message {
            role: "user".to_string(),
            content: input.to_string(),
            timestamp: now,
            metadata: HashMap::new(),
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            token_count: None,
        };

        let assistant_message = Message {
            role: "assistant".to_string(),
            content: result.to_string(),
            timestamp: now,
            metadata: HashMap::new(),
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            token_count: None,
        };

        let conversation = self
            .memory_client
            .create_conversation(
                metadata,
                user_id,
                title,
                tags,
                vec![user_message, assistant_message],
            )
            .await?;

        Ok(CreateConversationFromChainExecutionResponse {
            conversation_id: conversation.id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_memory_client() {
        let mut client = MockMemoryClient::new();

        // Test create_conversation
        let conversation = client
            .create_conversation(
                HashMap::new(),
                "test-user",
                Some("Test Conversation"),
                vec!["test".to_string()],
                Vec::new(),
            )
            .await
            .unwrap();

        assert_eq!(conversation.user_id, "test-user");
        assert_eq!(conversation.title, Some("Test Conversation".to_string()));

        // Add the conversation to the mock client
        client.add_conversation(conversation.clone());

        // Test get_conversation
        let result = client.get_conversation(&conversation.id).await.unwrap();
        assert_eq!(result.id, conversation.id);

        // Test add_message
        let message = client
            .add_message(
                &conversation.id,
                "user",
                "Hello, world!",
                HashMap::new(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello, world!");

        // Test get_conversation with non-existent ID
        let result = client.get_conversation("non-existent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_memory_chain_integration_client() {
        let mut memory_client = MockMemoryClient::new();

        // Create a test conversation
        let conversation = memory_client
            .create_conversation(
                HashMap::new(),
                "test-user",
                Some("Test Conversation"),
                vec!["test".to_string()],
                Vec::new(),
            )
            .await
            .unwrap();

        memory_client.add_conversation(conversation.clone());

        let integration_client = MockMemoryChainIntegrationClient::new(memory_client);

        // Test store_chain_result_in_conversation
        let result = integration_client
            .store_chain_result_in_conversation(
                &conversation.id,
                "test-chain",
                "test-execution",
                "Test result",
                Vec::new(),
                false,
                HashMap::new(),
            )
            .await
            .unwrap();

        assert!(!result.message_id.is_empty());

        // Test create_conversation_from_chain_execution
        let result = integration_client
            .create_conversation_from_chain_execution(
                "test-chain",
                "test-execution",
                "Test input",
                "Test result",
                Vec::new(),
                false,
                "test-user",
                Some("Chain Execution"),
                vec!["test".to_string()],
                HashMap::new(),
            )
            .await
            .unwrap();

        assert!(!result.conversation_id.is_empty());
    }
}
