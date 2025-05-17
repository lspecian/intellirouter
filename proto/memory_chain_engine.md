# Memory → Chain Engine Message Contracts

This file contains the protobuf schema definitions for communication between the Memory and Chain Engine modules.

## memory.proto

```protobuf
syntax = "proto3";

package intellirouter.memory.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/memory/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.memory.v1";

// MemoryService provides methods for conversation history and memory management
service MemoryService {
  // CreateConversation creates a new conversation
  rpc CreateConversation(CreateConversationRequest) returns (CreateConversationResponse);
  
  // GetConversation gets a conversation by ID
  rpc GetConversation(GetConversationRequest) returns (GetConversationResponse);
  
  // AddMessage adds a message to a conversation
  rpc AddMessage(AddMessageRequest) returns (AddMessageResponse);
  
  // GetHistory gets the conversation history formatted for an LLM request
  rpc GetHistory(GetHistoryRequest) returns (GetHistoryResponse);
  
  // SaveConversation saves a conversation to persistent storage
  rpc SaveConversation(SaveConversationRequest) returns (SaveConversationResponse);
  
  // LoadConversation loads a conversation from persistent storage
  rpc LoadConversation(LoadConversationRequest) returns (LoadConversationResponse);
  
  // DeleteConversation deletes a conversation
  rpc DeleteConversation(DeleteConversationRequest) returns (DeleteConversationResponse);
  
  // ListConversations lists all conversations
  rpc ListConversations(ListConversationsRequest) returns (ListConversationsResponse);
  
  // SearchMessages searches for messages across conversations
  rpc SearchMessages(SearchMessagesRequest) returns (SearchMessagesResponse);
}

// Message represents a message in a conversation
message Message {
  // Role of the message sender (e.g., "user", "assistant", "system")
  string role = 1;
  
  // Content of the message
  string content = 2;
  
  // When the message was created
  google.protobuf.Timestamp timestamp = 3;
  
  // Additional metadata about the message
  map<string, string> metadata = 4;
  
  // Message ID
  string id = 5;
  
  // Parent message ID (for threaded conversations)
  string parent_id = 6;
  
  // Token count (if available)
  uint32 token_count = 7;
}

// Conversation represents a conversation
message Conversation {
  // Unique identifier for the conversation
  string id = 1;
  
  // Messages in the conversation
  repeated Message messages = 2;
  
  // Additional metadata about the conversation
  map<string, string> metadata = 3;
  
  // When the conversation was created
  google.protobuf.Timestamp created_at = 4;
  
  // When the conversation was last updated
  google.protobuf.Timestamp updated_at = 5;
  
  // User ID associated with the conversation
  string user_id = 6;
  
  // Title of the conversation (if available)
  string title = 7;
  
  // Tags for categorization
  repeated string tags = 8;
}

// CreateConversationRequest is sent to create a new conversation
message CreateConversationRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Initial metadata for the conversation
  map<string, string> metadata = 2;
  
  // User ID to associate with the conversation
  string user_id = 3;
  
  // Initial title for the conversation (optional)
  string title = 4;
  
  // Initial tags for the conversation
  repeated string tags = 5;
  
  // Initial messages for the conversation
  repeated Message initial_messages = 6;
}

// CreateConversationResponse is returned after creating a conversation
message CreateConversationResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Created conversation
  Conversation conversation = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// GetConversationRequest is sent to get a conversation by ID
message GetConversationRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
}

// GetConversationResponse is returned with the requested conversation
message GetConversationResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Conversation
  Conversation conversation = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// AddMessageRequest is sent to add a message to a conversation
message AddMessageRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
  
  // Role of the message sender
  string role = 3;
  
  // Content of the message
  string content = 4;
  
  // Additional metadata about the message
  map<string, string> metadata = 5;
  
  // Parent message ID (for threaded conversations)
  string parent_id = 6;
}

// AddMessageResponse is returned after adding a message
message AddMessageResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Added message
  Message message = 2;
  
  // Updated conversation
  Conversation conversation = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// GetHistoryRequest is sent to get the conversation history
message GetHistoryRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
  
  // Maximum number of tokens to include
  uint32 max_tokens = 3;
  
  // Maximum number of messages to include
  uint32 max_messages = 4;
  
  // Whether to include system messages
  bool include_system_messages = 5;
  
  // Format for the history (e.g., "openai", "anthropic", "raw")
  string format = 6;
}

// GetHistoryResponse is returned with the conversation history
message GetHistoryResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Messages in the conversation history
  repeated Message messages = 2;
  
  // Total token count
  uint32 total_tokens = 3;
  
  // Formatted history (if format was specified)
  string formatted_history = 4;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 5;
}

// SaveConversationRequest is sent to save a conversation
message SaveConversationRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
}

// SaveConversationResponse is returned after saving a conversation
message SaveConversationResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 2;
}

// LoadConversationRequest is sent to load a conversation
message LoadConversationRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
}

// LoadConversationResponse is returned with the loaded conversation
message LoadConversationResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Loaded conversation
  Conversation conversation = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// DeleteConversationRequest is sent to delete a conversation
message DeleteConversationRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
}

// DeleteConversationResponse is returned after deleting a conversation
message DeleteConversationResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 2;
}

// ListConversationsRequest is sent to list conversations
message ListConversationsRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Maximum number of conversations to return
  uint32 limit = 2;
  
  // Offset for pagination
  uint32 offset = 3;
  
  // Filter by user ID
  string user_id = 4;
  
  // Filter by tags
  repeated string tag_filter = 5;
  
  // Filter by date range (start)
  google.protobuf.Timestamp start_date = 6;
  
  // Filter by date range (end)
  google.protobuf.Timestamp end_date = 7;
}

// ListConversationsResponse is returned with the list of conversations
message ListConversationsResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Conversations
  repeated Conversation conversations = 2;
  
  // Total number of conversations matching the filter
  uint32 total_count = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// SearchMessagesRequest is sent to search for messages
message SearchMessagesRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Search query
  string query = 2;
  
  // Maximum number of results to return
  uint32 limit = 3;
  
  // Offset for pagination
  uint32 offset = 4;
  
  // Filter by conversation ID
  string conversation_id = 5;
  
  // Filter by user ID
  string user_id = 6;
  
  // Filter by role
  string role = 7;
  
  // Filter by date range (start)
  google.protobuf.Timestamp start_date = 8;
  
  // Filter by date range (end)
  google.protobuf.Timestamp end_date = 9;
}

// SearchMessagesResponse is returned with the search results
message SearchMessagesResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Messages matching the search query
  repeated MessageSearchResult results = 2;
  
  // Total number of messages matching the search query
  uint32 total_count = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// MessageSearchResult represents a message search result
message MessageSearchResult {
  // Message
  Message message = 1;
  
  // Conversation ID
  string conversation_id = 2;
  
  // Conversation title
  string conversation_title = 3;
  
  // Relevance score (0.0 to 1.0)
  float score = 4;
  
  // Highlighted content with search matches
  string highlighted_content = 5;
}
```

## memory_chain_integration.proto

```protobuf
syntax = "proto3";

package intellirouter.integration.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/integration/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.integration.v1";

// MemoryChainIntegrationService provides methods for integrating Memory and Chain Engine
service MemoryChainIntegrationService {
  // GetConversationHistoryForChain gets conversation history formatted for a chain
  rpc GetConversationHistoryForChain(GetConversationHistoryForChainRequest) returns (GetConversationHistoryForChainResponse);
  
  // StoreChainResultInConversation stores a chain result in a conversation
  rpc StoreChainResultInConversation(StoreChainResultInConversationRequest) returns (StoreChainResultInConversationResponse);
  
  // CreateConversationFromChainExecution creates a new conversation from a chain execution
  rpc CreateConversationFromChainExecution(CreateConversationFromChainExecutionRequest) returns (CreateConversationFromChainExecutionResponse);
}

// GetConversationHistoryForChainRequest is sent to get conversation history for a chain
message GetConversationHistoryForChainRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
  
  // Chain ID
  string chain_id = 3;
  
  // Maximum number of tokens to include
  uint32 max_tokens = 4;
  
  // Maximum number of messages to include
  uint32 max_messages = 5;
  
  // Whether to include system messages
  bool include_system_messages = 6;
  
  // Format for the history (e.g., "openai", "anthropic", "raw")
  string format = 7;
  
  // Additional context to include
  string additional_context = 8;
}

// GetConversationHistoryForChainResponse is returned with the conversation history for a chain
message GetConversationHistoryForChainResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Formatted history for the chain
  string formatted_history = 2;
  
  // Messages included in the history
  repeated intellirouter.common.v1.Message messages = 3;
  
  // Total token count
  uint32 total_tokens = 4;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 5;
}

// StoreChainResultInConversationRequest is sent to store a chain result in a conversation
message StoreChainResultInConversationRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Conversation ID
  string conversation_id = 2;
  
  // Chain ID
  string chain_id = 3;
  
  // Chain execution ID
  string execution_id = 4;
  
  // Chain result
  string result = 5;
  
  // Step results to store as separate messages
  repeated StepResult step_results = 6;
  
  // Whether to store step results as separate messages
  bool store_step_results = 7;
  
  // Additional metadata about the chain execution
  map<string, string> metadata = 8;
}

// StepResult represents the result of a chain step
message StepResult {
  // Step ID
  string step_id = 1;
  
  // Step name
  string step_name = 2;
  
  // Input to the step
  string input = 3;
  
  // Output from the step
  string output = 4;
  
  // Model used for this step
  string model = 5;
}

// StoreChainResultInConversationResponse is returned after storing a chain result
message StoreChainResultInConversationResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Added message ID
  string message_id = 2;
  
  // Added step message IDs (if store_step_results was true)
  repeated string step_message_ids = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// CreateConversationFromChainExecutionRequest is sent to create a conversation from a chain execution
message CreateConversationFromChainExecutionRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Chain ID
  string chain_id = 2;
  
  // Chain execution ID
  string execution_id = 3;
  
  // Chain input
  string input = 4;
  
  // Chain result
  string result = 5;
  
  // Step results to store as separate messages
  repeated StepResult step_results = 6;
  
  // Whether to store step results as separate messages
  bool store_step_results = 7;
  
  // User ID to associate with the conversation
  string user_id = 8;
  
  // Title for the conversation
  string title = 9;
  
  // Tags for the conversation
  repeated string tags = 10;
  
  // Additional metadata about the conversation
  map<string, string> metadata = 11;
}

// CreateConversationFromChainExecutionResponse is returned after creating a conversation
message CreateConversationFromChainExecutionResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Created conversation ID
  string conversation_id = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}
```

## memory_events.proto (for Redis pub/sub)

```protobuf
syntax = "proto3";

package intellirouter.memory.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/memory/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.memory.v1";

// ConversationCreatedEvent is published when a conversation is created
message ConversationCreatedEvent {
  // Conversation ID
  string conversation_id = 1;
  
  // User ID associated with the conversation
  string user_id = 2;
  
  // Initial message count
  uint32 initial_message_count = 3;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 4;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 5;
}

// MessageAddedEvent is published when a message is added to a conversation
message MessageAddedEvent {
  // Conversation ID
  string conversation_id = 1;
  
  // Message ID
  string message_id = 2;
  
  // Message role
  string role = 3;
  
  // Message content length
  uint32 content_length = 4;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 5;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 6;
}

// ConversationHistoryRequestedEvent is published when conversation history is requested
message ConversationHistoryRequestedEvent {
  // Conversation ID
  string conversation_id = 1;
  
  // Maximum tokens requested
  uint32 max_tokens = 2;
  
  // Number of messages returned
  uint32 message_count = 3;
  
  // Total tokens returned
  uint32 total_tokens = 4;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 5;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 6;
}
```

## chain_memory_events.proto (for Redis pub/sub)

```protobuf
syntax = "proto3";

package intellirouter.integration.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/integration/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.integration.v1";

// ChainResultStoredEvent is published when a chain result is stored in a conversation
message ChainResultStoredEvent {
  // Conversation ID
  string conversation_id = 1;
  
  // Chain ID
  string chain_id = 2;
  
  // Chain execution ID
  string execution_id = 3;
  
  // Message ID
  string message_id = 4;
  
  // Result length
  uint32 result_length = 5;
  
  // Number of step results stored
  uint32 step_result_count = 6;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 7;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 8;
}

// ConversationCreatedFromChainEvent is published when a conversation is created from a chain execution
message ConversationCreatedFromChainEvent {
  // Conversation ID
  string conversation_id = 1;
  
  // Chain ID
  string chain_id = 2;
  
  // Chain execution ID
  string execution_id = 3;
  
  // User ID
  string user_id = 4;
  
  // Number of messages created
  uint32 message_count = 5;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 6;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 7;
}
```

## Schema Evolution Guidelines

1. **Field Numbering**: Reserve field numbers for removed fields to prevent accidental reuse.
2. **Versioning**: Use the `VersionInfo` message to track schema versions.
3. **Backward Compatibility**: 
   - Never remove fields unless they've been deprecated for at least one version
   - Never change the meaning of existing fields
   - Always add new fields as optional
4. **Forward Compatibility**:
   - Clients should ignore unknown fields
   - Use default values for missing fields
5. **Message Evolution**:
   - For major changes, create a new message type with a version suffix (e.g., `ConversationV2`)
   - Use the `reserved` keyword for removed field numbers and names

## Validation Rules

- `conversation_id`: Must be a non-empty string
- `message_id`: Must be a non-empty string
- `role`: Must be one of "user", "assistant", "system", or "function"
- `content`: Can be empty, but must not exceed 100,000 characters
- `max_tokens`: Must be greater than 0 if specified
- `max_messages`: Must be greater than 0 if specified
- `limit`: Must be greater than 0 and less than or equal to 100
- `offset`: Must be greater than or equal to 0