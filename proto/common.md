# Common Message Types

This file contains common message types used across multiple services in the IntelliRouter project.

## common.proto

```protobuf
syntax = "proto3";

package intellirouter.common.v1;

// Import Google's well-known type definitions
import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

option go_package = "github.com/intellirouter/intellirouter/proto/common/v1";
option java_multiple_files = true;
option java_package = "com.intellirouter.proto.common.v1";

// Status represents the status of an operation
enum Status {
  // Default value, should not be used
  STATUS_UNSPECIFIED = 0;
  // Operation succeeded
  STATUS_SUCCESS = 1;
  // Operation failed
  STATUS_ERROR = 2;
  // Operation is in progress
  STATUS_IN_PROGRESS = 3;
  // Operation timed out
  STATUS_TIMEOUT = 4;
  // Operation was cancelled
  STATUS_CANCELLED = 5;
}

// ErrorDetails provides additional information about an error
message ErrorDetails {
  // Error code
  string code = 1;
  // Error message
  string message = 2;
  // Additional details about the error
  google.protobuf.Struct details = 3;
  // Stack trace (for debugging)
  string stack_trace = 4;
}

// Metadata contains key-value pairs for additional information
message Metadata {
  // Map of metadata key-value pairs
  map<string, string> values = 1;
}

// Message represents a chat message
message Message {
  // Role of the message sender (e.g., "user", "assistant", "system")
  string role = 1;
  // Content of the message
  string content = 2;
  // When the message was created
  google.protobuf.Timestamp timestamp = 3;
  // Additional metadata about the message
  Metadata metadata = 4;
}

// ModelInfo contains information about a model
message ModelInfo {
  // Unique identifier for the model
  string id = 1;
  // Name of the model
  string name = 2;
  // Provider of the model (e.g., "openai", "anthropic")
  string provider = 3;
  // Version of the model
  string version = 4;
  // Maximum context window size in tokens
  uint32 context_window = 5;
  // Model capabilities
  repeated string capabilities = 6;
  // Model type (e.g., "chat", "embedding", "image")
  string type = 7;
  // Additional metadata about the model
  Metadata metadata = 8;
}

// RequestContext contains information about the request context
message RequestContext {
  // Unique identifier for the request
  string request_id = 1;
  // User identifier
  string user_id = 2;
  // Organization identifier
  string org_id = 3;
  // When the request was created
  google.protobuf.Timestamp timestamp = 4;
  // Request priority (higher values indicate higher priority)
  uint32 priority = 5;
  // Request tags for categorization
  repeated string tags = 6;
  // Additional metadata about the request
  Metadata metadata = 7;
}

// VersionInfo contains version information for schema evolution
message VersionInfo {
  // Major version number
  uint32 major = 1;
  // Minor version number
  uint32 minor = 2;
  // Patch version number
  uint32 patch = 3;
}