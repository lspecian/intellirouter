# Getting Started with IntelliRouter

This guide will walk you through the process of installing, configuring, and using IntelliRouter. By the end, you'll have a working IntelliRouter instance and understand how to use its key features.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [From Source](#from-source)
  - [Using Docker](#using-docker)
  - [Using Docker Compose](#using-docker-compose)
- [Configuration](#configuration)
  - [Basic Configuration](#basic-configuration)
  - [Provider Configuration](#provider-configuration)
  - [Advanced Configuration](#advanced-configuration)
- [Running IntelliRouter](#running-intellirouter)
  - [Running Specific Roles](#running-specific-roles)
  - [Verifying Installation](#verifying-installation)
- [Basic Usage](#basic-usage)
  - [Sending Chat Completion Requests](#sending-chat-completion-requests)
  - [Streaming Responses](#streaming-responses)
- [Using the SDKs](#using-the-sdks)
  - [Python SDK](#python-sdk)
  - [TypeScript SDK](#typescript-sdk)
  - [Rust SDK](#rust-sdk)
- [Advanced Features](#advanced-features)
  - [Custom Routing Strategies](#custom-routing-strategies)
  - [Retrieval Augmented Generation (RAG)](#retrieval-augmented-generation-rag)
  - [Chain Engine](#chain-engine)
  - [Persona Layer](#persona-layer)
- [Deployment Options](#deployment-options)
  - [Local Development](#local-development)
  - [Edge Deployment](#edge-deployment)
  - [Kubernetes Deployment](#kubernetes-deployment)
- [Troubleshooting](#troubleshooting)
  - [Common Issues](#common-issues)
  - [Logs and Debugging](#logs-and-debugging)
- [Next Steps](#next-steps)

## Overview

IntelliRouter is a programmable LLM gateway that provides an OpenAI-compatible API endpoint for chat completions, supporting both streaming and non-streaming responses. It's designed to be highly extensible and configurable, with support for various deployment scenarios.

Key features include:
- **Programmable Routing**: Route requests to different LLM backends based on customizable strategies
- **Extensibility**: Plugin system for custom routing strategies, model connectors, and telemetry exporters
- **Multi-Role Deployment**: Support for deploying as separate services with secure IPC
- **Client SDKs**: Python, TypeScript, and Rust libraries for easy integration
- **Deployment Options**: Configurations for various environments from edge to cloud

## Prerequisites

Before installing IntelliRouter, ensure you have:

- **Rust 1.70 or later**: Required for building from source
- **Docker** (optional): For containerized deployment
- **Kubernetes** (optional): For production deployment
- **API Keys**: For LLM providers you plan to use (e.g., OpenAI, Anthropic)

## Installation

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/intellirouter.git
   cd intellirouter
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

   This will create the executable at `./target/release/intellirouter`.

### Using Docker

1. Build the Docker image:
   ```bash
   docker build -t intellirouter .
   ```

2. Run the container:
   ```bash
   docker run -p 8000:8000 intellirouter run
   ```

### Using Docker Compose

1. Start all services:
   ```bash
   docker-compose up -d
   ```

   This will start all IntelliRouter services as defined in the `docker-compose.yml` file.

## Configuration

IntelliRouter uses TOML files for configuration. The default configuration files are located in the `config` directory.

### Basic Configuration

Create a `config/local.toml` file (or copy from `config/local.toml.example`):

```toml
# Environment setting (development, testing, production)
environment = "development"

# Server configuration
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
request_timeout_secs = 30
cors_enabled = true
cors_allowed_origins = ["http://localhost:3000"]

# Logging configuration
[telemetry]
log_level = "info"
metrics_enabled = true
tracing_enabled = true
```

### Provider Configuration

Add LLM provider configurations to your TOML file:

```toml
# Model registry configuration
[model_registry]
default_provider = "openai"
cache_ttl_secs = 3600

# OpenAI provider
[[model_registry.providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
endpoint = "https://api.openai.com/v1"
default_model = "gpt-4o"
available_models = ["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"]
timeout_secs = 60
max_retries = 3

# Anthropic provider
[[model_registry.providers]]
name = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
endpoint = "https://api.anthropic.com/v1"
default_model = "claude-3-opus-20240229"
available_models = ["claude-3-opus-20240229", "claude-3-sonnet-20240229", "claude-3-haiku-20240307"]
timeout_secs = 60
max_retries = 3
```

Make sure to set the corresponding environment variables for your API keys:

```bash
export OPENAI_API_KEY="your-openai-api-key"
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

### Advanced Configuration

For more advanced configurations, see the example configuration file at `examples/config/simple.toml`, which includes settings for:

- Router strategies
- Memory backends
- Authentication
- RAG (Retrieval Augmented Generation)
- Chain engine
- Persona layer
- Plugin SDK

## Running IntelliRouter

To run IntelliRouter with all services:

```bash
./target/release/intellirouter run
```

This will start all IntelliRouter services using the default configuration.

### Running Specific Roles

IntelliRouter can be run in different roles:

```bash
# Run only the router service
./target/release/intellirouter run --role router

# Run only the orchestrator service
./target/release/intellirouter run --role orchestrator

# Run only the RAG Manager service
./target/release/intellirouter run --role rag-injector

# Run only the Persona Layer service
./target/release/intellirouter run --role summarizer
```

Available roles:
- `router`: Runs the Router service
- `orchestrator`: Runs the Orchestrator service
- `rag-injector`: Runs the RAG Manager service
- `summarizer`: Runs the Persona Layer service
- `all`: Runs all services (default)

### Verifying Installation

To verify that IntelliRouter is running correctly, you can send a simple request to the API:

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ]
  }'
```

If everything is working correctly, you should receive a response similar to:

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
      "content": "Hello! I'm doing well, thank you for asking. How can I assist you today?"
    },
    "finish_reason": "stop"
  }]
}
```

## Basic Usage

### Sending Chat Completion Requests

You can send chat completion requests to IntelliRouter using the OpenAI-compatible API:

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "What is the capital of France?"}
    ]
  }'
```

### Streaming Responses

To receive streaming responses, add the `stream` parameter:

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Tell me about Paris."}
    ],
    "stream": true
  }'
```

The response will be a stream of server-sent events (SSE), with each event containing a chunk of the response.

## Using the SDKs

IntelliRouter provides SDKs for Python, TypeScript, and Rust to make integration easier.

### Python SDK

Install the Python SDK:

```bash
pip install intellirouter
```

Basic usage:

```python
from intellirouter import IntelliRouter

# Initialize the client
client = IntelliRouter(api_key="your-api-key", base_url="http://localhost:8080")

# Create a chat completion
completion = client.chat.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello, how are you?"}
    ]
)

# Print the response
print(completion.choices[0].message.content)
```

For streaming:

```python
# Create a streaming chat completion
for chunk in client.chat.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello, how are you?"}
    ],
    stream=True
):
    content = chunk.choices[0].delta.content
    if content:
        print(content, end="", flush=True)
```

### TypeScript SDK

Install the TypeScript SDK:

```bash
npm install intellirouter
```

Basic usage:

```typescript
import { IntelliRouter } from 'intellirouter';

// Initialize the client
const client = new IntelliRouter({
  apiKey: 'your-api-key',
  baseURL: 'http://localhost:8080'
});

// Create a chat completion
async function main() {
  const completion = await client.chat.create({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello, how are you?' }
    ]
  });

  console.log(completion.choices[0].message.content);
}

main();
```

For streaming:

```typescript
// Create a streaming chat completion
const stream = await client.chat.create({
  model: 'gpt-3.5-turbo',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Hello, how are you?' }
  ],
  stream: true
});

for await (const chunk of stream) {
  const content = chunk.choices[0]?.delta?.content || '';
  process.stdout.write(content);
}
```

### Rust SDK

Add the Rust SDK to your `Cargo.toml`:

```toml
[dependencies]
intellirouter = "0.1.0"
```

Basic usage:

```rust
use intellirouter::IntelliRouter;
use intellirouter::types::{ChatCompletionRequest, ChatMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client
    let client = IntelliRouter::new("your-api-key", "http://localhost:8080");

    // Create a chat completion
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
            },
        ],
        stream: false,
    };

    let completion = client.chat.create(request).await?;
    println!("{}", completion.choices[0].message.content);

    Ok(())
}
```

## Advanced Features

### Custom Routing Strategies

IntelliRouter supports custom routing strategies to determine which LLM provider and model to use for a given request. You can configure these in your TOML file:

```toml
[router]
default_strategy = "cost-optimized"
available_strategies = ["cost-optimized", "performance-optimized", "round-robin", "fallback"]
```

The available built-in strategies are:

- `cost-optimized`: Routes requests to the cheapest model that meets the requirements
- `performance-optimized`: Routes requests to the fastest model that meets the requirements
- `round-robin`: Distributes requests evenly across all available models
- `fallback`: Tries models in order, falling back to the next one if a request fails

You can also implement custom routing strategies using the Plugin SDK.

### Retrieval Augmented Generation (RAG)

IntelliRouter includes a RAG Manager service that can enhance LLM responses with information from your own data sources. To enable RAG:

1. Configure RAG in your TOML file:
   ```toml
   [rag]
   enabled = true
   default_embedding_model = "text-embedding-3-small"
   chunk_size = 1000
   chunk_overlap = 200
   ```

2. Start IntelliRouter with the RAG Manager service:
   ```bash
   ./target/release/intellirouter run --role rag-injector
   ```

3. Add documents to your RAG system using the API or SDK.

4. Use RAG in your chat completion requests by adding the `rag` parameter:
   ```json
   {
     "model": "gpt-3.5-turbo",
     "messages": [
       {"role": "user", "content": "What does our company policy say about remote work?"}
     ],
     "rag": {
       "enabled": true,
       "collection": "company_policies"
     }
   }
   ```

### Chain Engine

The Chain Engine allows you to create multi-step inference flows. To use it:

1. Configure the Chain Engine in your TOML file:
   ```toml
   [chain_engine]
   max_chain_length = 10
   max_execution_time_secs = 300
   enable_caching = true
   cache_ttl_secs = 3600
   ```

2. Create a chain using the API or SDK:
   ```json
   {
     "name": "Simple Chain",
     "description": "A simple chain that generates a response",
     "steps": {
       "step1": {
         "id": "step1",
         "type": "llm",
         "name": "Generate Response",
         "description": "Generate a response to the input",
         "inputs": {"prompt": "string"},
         "outputs": {"response": "string"}
       }
     }
   }
   ```

3. Execute the chain using the API or SDK:
   ```json
   {
     "chain_id": "chain-123",
     "inputs": {"prompt": "Hello, world!"}
   }
   ```

### Persona Layer

The Persona Layer allows you to inject system prompts and guardrails into your chat completion requests. To use it:

1. Configure the Persona Layer in your TOML file:
   ```toml
   [persona_layer]
   enabled = true
   default_persona = "default"
   personas_dir = "config/personas"
   ```

2. Create persona definitions in the `config/personas` directory.

3. Use personas in your chat completion requests by adding the `persona` parameter:
   ```json
   {
     "model": "gpt-3.5-turbo",
     "messages": [
       {"role": "user", "content": "Hello, how are you?"}
     ],
     "persona": "customer_support"
   }
   ```

## Deployment Options

### Local Development

For local development, you can use Docker Compose:

```bash
docker-compose -f docker-compose.dev.yml up -d
```

This will start all IntelliRouter services in development mode.

### Edge Deployment

For edge deployment, use the edge-specific Docker Compose file:

```bash
cd deployment/edge
docker-compose up -d
```

### Kubernetes Deployment

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

## Troubleshooting

### Common Issues

1. **IntelliRouter not starting:**
   - Check that you have the correct command: `intellirouter run`
   - Verify that the configuration file exists and is valid
   - Check for any error messages in the output

2. **Connection refused errors:**
   - Make sure IntelliRouter is running
   - Check that you're using the correct host and port
   - Verify that there are no firewall issues

3. **Authentication errors:**
   - Ensure that you've set the required API keys in your environment variables
   - Check that the API keys are valid and have not expired

4. **Missing dependencies for Rust examples:**
   - Run `cargo check` to see what dependencies are missing
   - Add the required dependencies to your Cargo.toml file

### Logs and Debugging

To enable more detailed logging, set the log level in your configuration file:

```toml
[telemetry]
log_level = "debug"  # Options: debug, info, warn, error
```

You can also check the logs for specific services:

```bash
# For Docker Compose deployments
docker-compose logs router
docker-compose logs orchestrator
docker-compose logs rag-injector
docker-compose logs summarizer

# For Kubernetes deployments
kubectl logs -l app=intellirouter -c router
kubectl logs -l app=intellirouter -c orchestrator
kubectl logs -l app=intellirouter -c rag-injector
kubectl logs -l app=intellirouter -c summarizer
```

## Next Steps

Now that you have IntelliRouter up and running, you can:

- Explore the [examples](../examples/README.md) directory for more advanced usage patterns
- Check out the [documentation](../docs/source/index.rst) for detailed information about each component
- Contribute to the project by submitting pull requests or reporting issues
- Join the community to share your experiences and get help

For more information, see the [README.md](../README.md) file.