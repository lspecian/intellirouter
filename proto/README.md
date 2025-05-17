# IntelliRouter Protobuf Schemas

This directory contains the Protocol Buffer (protobuf) schema definitions for the IntelliRouter project's IPC infrastructure. These schemas define the message contracts for communication between different modules of the system.

## Directory Structure

- `common/`: Common message types used across multiple services
- `chain_engine/`: Messages related to the Chain Engine module
- `router_core/`: Messages related to the Router Core module
- `rag_manager/`: Messages related to the RAG Manager module
- `persona_layer/`: Messages related to the Persona Layer module
- `memory/`: Messages related to the Memory module
- `model_registry/`: Messages related to the Model Registry module

## Module Pairs

The following module pairs communicate with each other:

1. chain_engine → router_core
2. rag_manager → persona_layer
3. memory → chain_engine
4. router_core → model_registry

## Schema Evolution Guidelines

1. **Never** change the meaning of an existing field
2. **Never** remove a field unless it's marked as deprecated for at least one version
3. **Always** add new fields as optional
4. **Always** use the `reserved` keyword for removed field numbers and names
5. Use versioned message types for major changes (e.g., `ModelInfoV1`, `ModelInfoV2`)
6. Include comments for all fields and messages to document their purpose
7. Use enums for fields with a fixed set of values
8. Use `oneof` for fields that can have different types
9. Include validation rules in comments

## Build Process

The protobuf schemas are compiled to Rust code using the `tonic-build` crate during the build process. The generated code is placed in the `src/proto` directory.

## Usage

To use these message types in your code, import the generated modules:

```rust
use intellirouter::proto::chain_engine::v1::ChainExecutionRequest;