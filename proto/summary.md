# IntelliRouter Protobuf Schema Design Summary

This document provides an overview of the protobuf schemas and message contracts designed for the IntelliRouter project's IPC infrastructure.

## Overview

We've designed protobuf schemas for the following module pairs:

1. **chain_engine → router_core**: For multi-step orchestration and routing
2. **rag_manager → persona_layer**: For RAG integration and persona management
3. **memory → chain_engine**: For conversation history and memory management
4. **router_core → model_registry**: For model tracking and routing

Each schema set includes:
- Service definitions for synchronous gRPC communication
- Message types for request/response patterns
- Event types for asynchronous Redis pub/sub communication
- Validation rules and schema evolution guidelines

## Directory Structure

```
proto/
├── README.md                       # Overview of the protobuf schemas
├── common.md                       # Common message types used across services
├── chain_engine_router_core.md     # Chain Engine → Router Core communication
├── rag_manager_persona_layer.md    # RAG Manager → Persona Layer communication
├── memory_chain_engine.md          # Memory → Chain Engine communication
├── router_core_model_registry.md   # Router Core → Model Registry communication
└── summary.md                      # This summary document
```

## Message Types Overview

### Common Message Types

- **Status**: Represents the status of an operation (SUCCESS, ERROR, etc.)
- **ErrorDetails**: Provides additional information about an error
- **Metadata**: Contains key-value pairs for additional information
- **Message**: Represents a chat message
- **ModelInfo**: Contains information about a model
- **RequestContext**: Contains information about the request context
- **VersionInfo**: Contains version information for schema evolution

### Chain Engine → Router Core

- **ChainEngineService**: Service for chain execution and management
- **Chain**: Represents a chain configuration
- **ChainStep**: Represents a step in a chain
- **ChainExecutionRequest/Response**: For executing chains
- **ChainStatusRequest/Response**: For checking chain status
- **ChainExecutionEvent**: For streaming chain execution events
- **ChainExecutionStatusEvent**: For publishing chain execution status changes (pub/sub)

### RAG Manager → Persona Layer

- **RAGManagerService**: Service for RAG operations
- **PersonaLayerService**: Service for persona management
- **Document**: Represents a document for RAG
- **Persona**: Represents a persona configuration
- **AugmentRequestRequest/Response**: For augmenting requests with RAG context
- **ApplyPersonaRequest/Response**: For applying personas to requests
- **RAGPersonaIntegrationService**: Service for integrating RAG and Persona
- **DocumentIndexedEvent**: For publishing document indexing events (pub/sub)
- **PersonaAppliedEvent**: For publishing persona application events (pub/sub)

### Memory → Chain Engine

- **MemoryService**: Service for conversation history and memory management
- **Conversation**: Represents a conversation
- **Message**: Represents a message in a conversation
- **GetHistoryRequest/Response**: For getting conversation history
- **MemoryChainIntegrationService**: Service for integrating Memory and Chain Engine
- **ConversationCreatedEvent**: For publishing conversation creation events (pub/sub)
- **ChainResultStoredEvent**: For publishing chain result storage events (pub/sub)

### Router Core → Model Registry

- **ModelRegistryService**: Service for model registry operations
- **RouterService**: Service for routing operations
- **ModelMetadata**: Represents metadata for a model
- **ModelFilter**: Represents a filter for finding models
- **RoutingRequest**: Represents a request to route
- **RoutingMetadata**: Represents metadata about a routing decision
- **ModelRegisteredEvent**: For publishing model registration events (pub/sub)
- **RequestRoutedEvent**: For publishing request routing events (pub/sub)

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
   - For major changes, create a new message type with a version suffix (e.g., `ModelMetadataV2`)
   - Use the `reserved` keyword for removed field numbers and names

## Implementation Recommendations

1. **Build Process**: Use the `tonic-build` crate to compile the protobuf schemas to Rust code during the build process.
2. **Directory Structure**: Organize the protobuf schemas in a logical directory structure, with separate files for each module pair or functional area.
3. **Versioning**: Include version information in the package names (e.g., `intellirouter.model_registry.v1`) to support future versioning.
4. **Documentation**: Include comprehensive comments for all fields and messages to document their purpose.
5. **Validation**: Implement validation for all fields according to the validation rules specified in each schema file.
6. **Error Handling**: Use the `ErrorDetails` message to provide detailed error information.
7. **Testing**: Create comprehensive tests for all message types and services.

## Next Steps

1. Extract the protobuf schemas from the Markdown files and create actual `.proto` files.
2. Set up the build process to compile the protobuf schemas to Rust code.
3. Implement the services defined in the schemas.
4. Create integration tests to verify the communication between modules.
5. Implement the Redis pub/sub infrastructure for asynchronous communication.