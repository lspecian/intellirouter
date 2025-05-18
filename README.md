# IntelliRouter

IntelliRouter is a programmable LLM gateway that provides an OpenAI-compatible API endpoint for chat completions, supporting both streaming and non-streaming responses. It's designed to be highly extensible and configurable, with support for various deployment scenarios.

## Features

- **Programmable Routing**: Route requests to different LLM backends based on customizable strategies
- **Extensibility**: Plugin system for custom routing strategies, model connectors, and telemetry exporters
- **Multi-Role Deployment**: Support for deploying as separate services with secure IPC
- **Client SDKs**: Python, TypeScript, and Rust libraries for easy integration
- **Deployment Options**: Configurations for various environments from edge to cloud

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Docker (for containerized deployment)
- Kubernetes (for production deployment)

### Installation

#### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/intellirouter.git
   cd intellirouter
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the router:
   ```bash
   ./target/release/intellirouter --role router
   ```

#### Using Docker

1. Build the Docker image:
   ```bash
   docker build -t intellirouter .
   ```

2. Run the container:
   ```bash
   docker run -p 8000:8000 -e INTELLIROUTER_ROLE=router intellirouter
   ```

#### Using Docker Compose

1. Start all services:
   ```bash
   docker-compose up -d
   ```

### Configuration

IntelliRouter can be configured using a TOML file. Create a `config.toml` file in the `config` directory:

```toml
[server]
host = "0.0.0.0"
port = 8000

[logging]
level = "info"

[redis]
host = "localhost"
port = 6379

[chromadb]
host = "localhost"
port = 8001
```

### Deployment Options

#### Local Development

For local development, you can use Docker Compose:

```bash
docker-compose -f docker-compose.dev.yml up -d
```

#### Edge Deployment

For edge deployment, use the edge-specific Docker Compose file:

```bash
cd deployment/edge
docker-compose up -d
```

#### Kubernetes Deployment

For Kubernetes deployment, use Helm:

```bash
# MicroK8s
cd deployment/microk8s
helm install intellirouter ../../helm/intellirouter -f values.yaml

# EKS
cd deployment/eks
helm install intellirouter ../../helm/intellirouter -f values.yaml

# GKE
cd deployment/gke
helm install intellirouter ../../helm/intellirouter -f values.yaml
```

## Architecture

IntelliRouter consists of several modules:

- **LLM Proxy**: OpenAI-compatible API endpoint
- **Model Registry**: Tracks available LLM backends
- **Router Core**: Routes requests to appropriate model backends
- **Persona Layer**: Injects system prompts and guardrails
- **Chain Engine**: Orchestrates multi-step inference flows
- **Memory**: Provides short-term and long-term memory capabilities
- **RAG Manager**: Manages Retrieval Augmented Generation
- **Authentication**: Handles API key validation and RBAC
- **Telemetry**: Collects logs, costs, and usage metrics
- **Plugin SDK**: Provides extensibility for custom components

## SDKs

IntelliRouter provides SDKs for easy integration:

- [Python SDK](sdk/python/README.md)
- [TypeScript SDK](sdk/typescript/README.md)
- [Rust SDK](sdk/rust/README.md)

## HTTP API

IntelliRouter provides an OpenAI-compatible HTTP API:

### Chat Completions

```
POST /v1/chat/completions
```

Request:
```json
{
  "model": "gpt-3.5-turbo",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ]
}
```

Response:
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-3.5-turbo",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Hello! How can I assist you today?"
    },
    "finish_reason": "stop"
  }]
}
```

## Testing

### Router Role Testing

The router role can be tested using the provided integration tests and CLI script:

#### Running Integration Tests

```bash
# Run all tests
cargo test

# Run router-specific integration tests
cargo test --test router_integration_tests
```

#### Manual Testing with CLI Script

A CLI script is provided to test the router role manually:

```bash
# Start the router in one terminal
cargo run -- --role router

# In another terminal, run the test script
./scripts/test-router.sh

# You can specify a different host, port, or model
./scripts/test-router.sh localhost 9000 mock-llama
```

The test script will:
1. Check if the router is running
2. Send a test request to the `/v1/chat/completions` endpoint
3. Validate the response structure
4. Display the results

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.