# RAG Manager → Persona Layer Message Contracts

This file contains the protobuf schema definitions for communication between the RAG Manager and Persona Layer modules.

## rag_manager.proto

```protobuf
syntax = "proto3";

package intellirouter.rag_manager.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/rag_manager/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.rag_manager.v1";

// RAGManagerService provides methods for RAG operations
service RAGManagerService {
  // IndexDocument indexes a document for retrieval
  rpc IndexDocument(IndexDocumentRequest) returns (IndexDocumentResponse);
  
  // RetrieveDocuments retrieves relevant documents for a query
  rpc RetrieveDocuments(RetrieveDocumentsRequest) returns (RetrieveDocumentsResponse);
  
  // AugmentRequest augments a request with retrieved context
  rpc AugmentRequest(AugmentRequestRequest) returns (AugmentRequestResponse);
  
  // GetDocumentById retrieves a document by its ID
  rpc GetDocumentById(GetDocumentByIdRequest) returns (GetDocumentByIdResponse);
  
  // DeleteDocument deletes a document from the index
  rpc DeleteDocument(DeleteDocumentRequest) returns (DeleteDocumentResponse);
  
  // ListDocuments lists all documents in the index
  rpc ListDocuments(ListDocumentsRequest) returns (ListDocumentsResponse);
}

// Document represents a document for RAG
message Document {
  // Unique identifier for the document
  string id = 1;
  
  // Content of the document
  string content = 2;
  
  // Metadata about the document
  map<string, string> metadata = 3;
  
  // When the document was created
  google.protobuf.Timestamp created_at = 4;
  
  // When the document was last updated
  google.protobuf.Timestamp updated_at = 5;
  
  // Document embedding (if available)
  repeated float embedding = 6;
  
  // Document chunks (if chunked)
  repeated DocumentChunk chunks = 7;
}

// DocumentChunk represents a chunk of a document
message DocumentChunk {
  // Unique identifier for the chunk
  string id = 1;
  
  // Content of the chunk
  string content = 2;
  
  // Metadata about the chunk
  map<string, string> metadata = 3;
  
  // Chunk embedding (if available)
  repeated float embedding = 4;
  
  // Parent document ID
  string document_id = 5;
  
  // Chunk index within the document
  uint32 chunk_index = 6;
}

// IndexDocumentRequest is sent to index a document
message IndexDocumentRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Document to index
  Document document = 2;
  
  // Chunk size in characters (0 for no chunking)
  uint32 chunk_size = 3;
  
  // Chunk overlap in characters
  uint32 chunk_overlap = 4;
  
  // Whether to compute embeddings
  bool compute_embeddings = 5;
  
  // Model to use for embeddings (if compute_embeddings is true)
  string embedding_model = 6;
}

// IndexDocumentResponse is returned after indexing a document
message IndexDocumentResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Document ID
  string document_id = 2;
  
  // Number of chunks created
  uint32 chunk_count = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// RetrieveDocumentsRequest is sent to retrieve relevant documents
message RetrieveDocumentsRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Query text
  string query = 2;
  
  // Number of documents to retrieve
  uint32 top_k = 3;
  
  // Minimum similarity score (0.0 to 1.0)
  float min_score = 4;
  
  // Filter by metadata
  map<string, string> metadata_filter = 5;
  
  // Whether to include document content
  bool include_content = 6;
  
  // Whether to rerank results
  bool rerank = 7;
  
  // Model to use for reranking (if rerank is true)
  string rerank_model = 8;
}

// RetrieveDocumentsResponse is returned with retrieved documents
message RetrieveDocumentsResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Retrieved documents with similarity scores
  repeated ScoredDocument documents = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// ScoredDocument represents a document with a similarity score
message ScoredDocument {
  // Document
  Document document = 1;
  
  // Similarity score (0.0 to 1.0)
  float score = 2;
}

// AugmentRequestRequest is sent to augment a request with context
message AugmentRequestRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Original request text
  string request = 2;
  
  // Number of documents to retrieve
  uint32 top_k = 3;
  
  // Minimum similarity score (0.0 to 1.0)
  float min_score = 4;
  
  // Filter by metadata
  map<string, string> metadata_filter = 5;
  
  // Whether to include citations
  bool include_citations = 6;
  
  // Maximum context length in characters
  uint32 max_context_length = 7;
  
  // Context template (optional)
  string context_template = 8;
}

// AugmentRequestResponse is returned with the augmented request
message AugmentRequestResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Augmented request text
  string augmented_request = 2;
  
  // Retrieved documents with similarity scores
  repeated ScoredDocument documents = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// GetDocumentByIdRequest is sent to retrieve a document by ID
message GetDocumentByIdRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Document ID
  string document_id = 2;
  
  // Whether to include chunks
  bool include_chunks = 3;
}

// GetDocumentByIdResponse is returned with the requested document
message GetDocumentByIdResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Document
  Document document = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// DeleteDocumentRequest is sent to delete a document
message DeleteDocumentRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Document ID
  string document_id = 2;
}

// DeleteDocumentResponse is returned after deleting a document
message DeleteDocumentResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 2;
}

// ListDocumentsRequest is sent to list documents
message ListDocumentsRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Maximum number of documents to return
  uint32 limit = 2;
  
  // Offset for pagination
  uint32 offset = 3;
  
  // Filter by metadata
  map<string, string> metadata_filter = 4;
}

// ListDocumentsResponse is returned with the list of documents
message ListDocumentsResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Documents
  repeated Document documents = 2;
  
  // Total number of documents matching the filter
  uint32 total_count = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}
```

## persona_layer.proto

```protobuf
syntax = "proto3";

package intellirouter.persona_layer.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/persona_layer/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.persona_layer.v1";

// PersonaLayerService provides methods for persona management
service PersonaLayerService {
  // CreatePersona creates a new persona
  rpc CreatePersona(CreatePersonaRequest) returns (CreatePersonaResponse);
  
  // GetPersona gets a persona by ID
  rpc GetPersona(GetPersonaRequest) returns (GetPersonaResponse);
  
  // UpdatePersona updates an existing persona
  rpc UpdatePersona(UpdatePersonaRequest) returns (UpdatePersonaResponse);
  
  // DeletePersona deletes a persona
  rpc DeletePersona(DeletePersonaRequest) returns (DeletePersonaResponse);
  
  // ListPersonas lists all personas
  rpc ListPersonas(ListPersonasRequest) returns (ListPersonasResponse);
  
  // ApplyPersona applies a persona to a request
  rpc ApplyPersona(ApplyPersonaRequest) returns (ApplyPersonaResponse);
}

// Persona represents a persona configuration
message Persona {
  // Unique identifier for the persona
  string id = 1;
  
  // Name of the persona
  string name = 2;
  
  // Description of the persona
  string description = 3;
  
  // System prompt for the persona
  string system_prompt = 4;
  
  // Response format for the persona (optional)
  string response_format = 5;
  
  // When the persona was created
  google.protobuf.Timestamp created_at = 6;
  
  // When the persona was last updated
  google.protobuf.Timestamp updated_at = 7;
  
  // Additional metadata about the persona
  map<string, string> metadata = 8;
  
  // Tags for categorization
  repeated string tags = 9;
  
  // Version of the persona
  intellirouter.common.v1.VersionInfo version = 10;
}

// CreatePersonaRequest is sent to create a new persona
message CreatePersonaRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Name of the persona
  string name = 2;
  
  // Description of the persona
  string description = 3;
  
  // System prompt for the persona
  string system_prompt = 4;
  
  // Response format for the persona (optional)
  string response_format = 5;
  
  // Additional metadata about the persona
  map<string, string> metadata = 6;
  
  // Tags for categorization
  repeated string tags = 7;
}

// CreatePersonaResponse is returned after creating a persona
message CreatePersonaResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Created persona
  Persona persona = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// GetPersonaRequest is sent to get a persona by ID
message GetPersonaRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Persona ID
  string persona_id = 2;
}

// GetPersonaResponse is returned with the requested persona
message GetPersonaResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Persona
  Persona persona = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// UpdatePersonaRequest is sent to update an existing persona
message UpdatePersonaRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Persona ID
  string persona_id = 2;
  
  // Name of the persona (optional)
  optional string name = 3;
  
  // Description of the persona (optional)
  optional string description = 4;
  
  // System prompt for the persona (optional)
  optional string system_prompt = 5;
  
  // Response format for the persona (optional)
  optional string response_format = 6;
  
  // Additional metadata about the persona (optional)
  map<string, string> metadata = 7;
  
  // Tags for categorization (optional)
  repeated string tags = 8;
}

// UpdatePersonaResponse is returned after updating a persona
message UpdatePersonaResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Updated persona
  Persona persona = 2;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// DeletePersonaRequest is sent to delete a persona
message DeletePersonaRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Persona ID
  string persona_id = 2;
}

// DeletePersonaResponse is returned after deleting a persona
message DeletePersonaResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 2;
}

// ListPersonasRequest is sent to list personas
message ListPersonasRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Maximum number of personas to return
  uint32 limit = 2;
  
  // Offset for pagination
  uint32 offset = 3;
  
  // Filter by tags
  repeated string tag_filter = 4;
}

// ListPersonasResponse is returned with the list of personas
message ListPersonasResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Personas
  repeated Persona personas = 2;
  
  // Total number of personas matching the filter
  uint32 total_count = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}

// ApplyPersonaRequest is sent to apply a persona to a request
message ApplyPersonaRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Persona identifier
  oneof persona_identifier {
    // Persona ID
    string persona_id = 2;
    
    // Inline persona definition
    Persona persona = 3;
  }
  
  // Original request text
  string request = 4;
  
  // Additional context to include (optional)
  string additional_context = 5;
  
  // Whether to include the persona description in the prompt
  bool include_description = 6;
}

// ApplyPersonaResponse is returned with the personalized request
message ApplyPersonaResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Personalized request text
  string personalized_request = 2;
  
  // Applied persona
  Persona applied_persona = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
}
```

## rag_persona_integration.proto

```protobuf
syntax = "proto3";

package intellirouter.integration.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/integration/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.integration.v1";

// RAGPersonaIntegrationService provides methods for integrating RAG and Persona
service RAGPersonaIntegrationService {
  // AugmentPersonaRequest augments a persona request with RAG context
  rpc AugmentPersonaRequest(AugmentPersonaRequestRequest) returns (AugmentPersonaRequestResponse);
}

// AugmentPersonaRequestRequest is sent to augment a persona request with RAG context
message AugmentPersonaRequestRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Persona ID
  string persona_id = 2;
  
  // Original request text
  string request = 3;
  
  // Number of documents to retrieve
  uint32 top_k = 4;
  
  // Minimum similarity score (0.0 to 1.0)
  float min_score = 5;
  
  // Filter by metadata
  map<string, string> metadata_filter = 6;
  
  // Whether to include citations
  bool include_citations = 7;
  
  // Maximum context length in characters
  uint32 max_context_length = 8;
  
  // Context template (optional)
  string context_template = 9;
}

// AugmentPersonaRequestResponse is returned with the augmented persona request
message AugmentPersonaRequestResponse {
  // Status of the operation
  intellirouter.common.v1.Status status = 1;
  
  // Augmented and personalized request text
  string augmented_request = 2;
  
  // Applied persona ID
  string persona_id = 3;
  
  // Number of documents retrieved
  uint32 document_count = 4;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 5;
}
```

## rag_events.proto (for Redis pub/sub)

```protobuf
syntax = "proto3";

package intellirouter.rag_manager.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/rag_manager/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.rag_manager.v1";

// DocumentIndexedEvent is published when a document is indexed
message DocumentIndexedEvent {
  // Document ID
  string document_id = 1;
  
  // Document title or name
  string document_title = 2;
  
  // Number of chunks created
  uint32 chunk_count = 3;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 4;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 5;
}

// DocumentRetrievedEvent is published when documents are retrieved
message DocumentRetrievedEvent {
  // Query text
  string query = 1;
  
  // Number of documents retrieved
  uint32 document_count = 2;
  
  // Top document ID
  string top_document_id = 3;
  
  // Top document score
  float top_document_score = 4;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 5;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 6;
}
```

## persona_events.proto (for Redis pub/sub)

```protobuf
syntax = "proto3";

package intellirouter.persona_layer.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/persona_layer/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.persona_layer.v1";

// PersonaCreatedEvent is published when a persona is created
message PersonaCreatedEvent {
  // Persona ID
  string persona_id = 1;
  
  // Persona name
  string persona_name = 2;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 3;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 4;
}

// PersonaAppliedEvent is published when a persona is applied to a request
message PersonaAppliedEvent {
  // Persona ID
  string persona_id = 1;
  
  // Persona name
  string persona_name = 2;
  
  // Original request length in characters
  uint32 original_request_length = 3;
  
  // Personalized request length in characters
  uint32 personalized_request_length = 4;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 5;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 6;
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
   - For major changes, create a new message type with a version suffix (e.g., `PersonaV2`)
   - Use the `reserved` keyword for removed field numbers and names

## Validation Rules

- `persona_id`: Must be a non-empty string
- `document_id`: Must be a non-empty string
- `name`: Must be a non-empty string
- `system_prompt`: Must be a non-empty string
- `request`: Can be empty, but must not exceed 100,000 characters
- `top_k`: Must be greater than 0 and less than or equal to 100
- `min_score`: Must be between 0.0 and 1.0
- `chunk_size`: Must be greater than 0 if specified