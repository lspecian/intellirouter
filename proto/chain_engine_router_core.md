# Chain Engine → Router Core Message Contracts

This file contains the protobuf schema definitions for communication between the Chain Engine and Router Core modules.

## chain_engine.proto

```protobuf
syntax = "proto3";

package intellirouter.chain_engine.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/chain_engine/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.chain_engine.v1";

// ChainEngineService provides methods for chain execution and management
service ChainEngineService {
  // ExecuteChain executes a chain with the given input
  rpc ExecuteChain(ChainExecutionRequest) returns (ChainExecutionResponse);
  
  // GetChainStatus gets the status of a chain execution
  rpc GetChainStatus(ChainStatusRequest) returns (ChainStatusResponse);
  
  // CancelChainExecution cancels a running chain execution
  rpc CancelChainExecution(CancelChainRequest) returns (CancelChainResponse);
  
  // StreamChainExecution streams the results of a chain execution
  rpc StreamChainExecution(ChainExecutionRequest) returns (stream ChainExecutionEvent);
}

// ChainStep represents a step in a chain
message ChainStep {
  // Unique identifier for the step
  string id = 1;
  
  // Description of the step
  string description = 2;
  
  // Model to use for this step (optional)
  string model = 3;
  
  // System prompt for this step (optional)
  string system_prompt = 4;
  
  // Input template for this step
  string input_template = 5;
  
  // Output format for this step (optional)
  string output_format = 6;
  
  // Maximum tokens to generate (optional)
  uint32 max_tokens = 7;
  
  // Temperature for generation (optional)
  float temperature = 8;
  
  // Additional parameters for this step
  map<string, string> parameters = 9;
}

// Chain represents a chain configuration
message Chain {
  // Unique identifier for the chain
  string id = 1;
  
  // Name of the chain
  string name = 2;
  
  // Description of the chain
  string description = 3;
  
  // Steps in the chain
  repeated ChainStep steps = 4;
  
  // Version of the chain
  intellirouter.common.v1.VersionInfo version = 5;
  
  // Additional metadata about the chain
  intellirouter.common.v1.Metadata metadata = 6;
}

// ChainExecutionRequest is sent to execute a chain
message ChainExecutionRequest {
  // Request context
  intellirouter.common.v1.RequestContext context = 1;
  
  // Chain to execute
  oneof chain_identifier {
    // Chain ID to execute
    string chain_id = 2;
    
    // Inline chain definition
    Chain chain = 3;
  }
  
  // Input for the chain
  string input = 4;
  
  // Variables to use during execution
  map<string, string> variables = 5;
  
  // Whether to stream the results
  bool stream = 6;
  
  // Timeout for the execution in seconds
  uint32 timeout_seconds = 7;
}

// ChainExecutionResponse is returned after executing a chain
message ChainExecutionResponse {
  // Execution ID
  string execution_id = 1;
  
  // Status of the execution
  intellirouter.common.v1.Status status = 2;
  
  // Output of the chain
  string output = 3;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 4;
  
  // Execution start time
  google.protobuf.Timestamp start_time = 5;
  
  // Execution end time
  google.protobuf.Timestamp end_time = 6;
  
  // Step results
  repeated StepResult step_results = 7;
  
  // Total tokens used
  uint32 total_tokens = 8;
  
  // Additional metadata about the execution
  intellirouter.common.v1.Metadata metadata = 9;
}

// StepResult represents the result of executing a step
message StepResult {
  // Step ID
  string step_id = 1;
  
  // Status of the step
  intellirouter.common.v1.Status status = 2;
  
  // Input to the step
  string input = 3;
  
  // Output from the step
  string output = 4;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 5;
  
  // Step start time
  google.protobuf.Timestamp start_time = 6;
  
  // Step end time
  google.protobuf.Timestamp end_time = 7;
  
  // Tokens used by this step
  uint32 tokens = 8;
  
  // Model used for this step
  string model = 9;
  
  // Additional metadata about the step execution
  intellirouter.common.v1.Metadata metadata = 10;
}

// ChainStatusRequest is sent to get the status of a chain execution
message ChainStatusRequest {
  // Execution ID
  string execution_id = 1;
}

// ChainStatusResponse is returned with the status of a chain execution
message ChainStatusResponse {
  // Execution ID
  string execution_id = 1;
  
  // Status of the execution
  intellirouter.common.v1.Status status = 2;
  
  // Current step ID (if in progress)
  string current_step_id = 3;
  
  // Completed steps
  uint32 completed_steps = 4;
  
  // Total steps
  uint32 total_steps = 5;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 6;
  
  // Execution start time
  google.protobuf.Timestamp start_time = 7;
  
  // Execution update time
  google.protobuf.Timestamp update_time = 8;
}

// CancelChainRequest is sent to cancel a chain execution
message CancelChainRequest {
  // Execution ID
  string execution_id = 1;
}

// CancelChainResponse is returned after cancelling a chain execution
message CancelChainResponse {
  // Execution ID
  string execution_id = 1;
  
  // Whether the cancellation was successful
  bool success = 2;
  
  // Error details (if not successful)
  intellirouter.common.v1.ErrorDetails error = 3;
}

// ChainExecutionEvent is streamed during chain execution
message ChainExecutionEvent {
  // Event type
  EventType type = 1;
  
  // Execution ID
  string execution_id = 2;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 3;
  
  // Event data
  oneof data {
    // Step started event
    StepStartedEvent step_started = 4;
    
    // Step completed event
    StepCompletedEvent step_completed = 5;
    
    // Step failed event
    StepFailedEvent step_failed = 6;
    
    // Chain completed event
    ChainCompletedEvent chain_completed = 7;
    
    // Chain failed event
    ChainFailedEvent chain_failed = 8;
    
    // Token generated event
    TokenGeneratedEvent token_generated = 9;
  }
}

// EventType represents the type of chain execution event
enum EventType {
  // Default value, should not be used
  EVENT_TYPE_UNSPECIFIED = 0;
  
  // Step started
  EVENT_TYPE_STEP_STARTED = 1;
  
  // Step completed
  EVENT_TYPE_STEP_COMPLETED = 2;
  
  // Step failed
  EVENT_TYPE_STEP_FAILED = 3;
  
  // Chain completed
  EVENT_TYPE_CHAIN_COMPLETED = 4;
  
  // Chain failed
  EVENT_TYPE_CHAIN_FAILED = 5;
  
  // Token generated
  EVENT_TYPE_TOKEN_GENERATED = 6;
}

// StepStartedEvent is sent when a step starts
message StepStartedEvent {
  // Step ID
  string step_id = 1;
  
  // Step index
  uint32 step_index = 2;
  
  // Input to the step
  string input = 3;
}

// StepCompletedEvent is sent when a step completes
message StepCompletedEvent {
  // Step ID
  string step_id = 1;
  
  // Step index
  uint32 step_index = 2;
  
  // Output from the step
  string output = 3;
  
  // Tokens used by this step
  uint32 tokens = 4;
}

// StepFailedEvent is sent when a step fails
message StepFailedEvent {
  // Step ID
  string step_id = 1;
  
  // Step index
  uint32 step_index = 2;
  
  // Error details
  intellirouter.common.v1.ErrorDetails error = 3;
}

// ChainCompletedEvent is sent when a chain completes
message ChainCompletedEvent {
  // Output of the chain
  string output = 1;
  
  // Total tokens used
  uint32 total_tokens = 2;
  
  // Execution time in milliseconds
  uint64 execution_time_ms = 3;
}

// ChainFailedEvent is sent when a chain fails
message ChainFailedEvent {
  // Error details
  intellirouter.common.v1.ErrorDetails error = 1;
  
  // Execution time in milliseconds
  uint64 execution_time_ms = 2;
}

// TokenGeneratedEvent is sent when a token is generated
message TokenGeneratedEvent {
  // Step ID
  string step_id = 1;
  
  // Token
  string token = 2;
}
```

## chain_engine_events.proto (for Redis pub/sub)

```protobuf
syntax = "proto3";

package intellirouter.chain_engine.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/chain_engine/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.chain_engine.v1";

// ChainExecutionStatusEvent is published when a chain execution status changes
message ChainExecutionStatusEvent {
  // Execution ID
  string execution_id = 1;
  
  // Chain ID
  string chain_id = 2;
  
  // Status of the execution
  intellirouter.common.v1.Status status = 3;
  
  // Current step ID (if in progress)
  string current_step_id = 4;
  
  // Completed steps
  uint32 completed_steps = 5;
  
  // Total steps
  uint32 total_steps = 6;
  
  // Error details (if status is ERROR)
  intellirouter.common.v1.ErrorDetails error = 7;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 8;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 9;
}

// ChainExecutionCompletedEvent is published when a chain execution completes
message ChainExecutionCompletedEvent {
  // Execution ID
  string execution_id = 1;
  
  // Chain ID
  string chain_id = 2;
  
  // Output of the chain
  string output = 3;
  
  // Total tokens used
  uint32 total_tokens = 4;
  
  // Execution time in milliseconds
  uint64 execution_time_ms = 5;
  
  // Event timestamp
  google.protobuf.Timestamp timestamp = 6;
  
  // Request context
  intellirouter.common.v1.RequestContext context = 7;
  
  // Step results
  repeated StepResult step_results = 8;
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
   - For major changes, create a new message type with a version suffix (e.g., `ChainV2`)
   - Use the `reserved` keyword for removed field numbers and names

## Validation Rules

- `chain_id`: Must be a non-empty string
- `execution_id`: Must be a non-empty string
- `timeout_seconds`: Must be greater than 0
- `step_id`: Must be a non-empty string
- `input`: Can be empty, but must not exceed 100,000 characters