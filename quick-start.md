# IntelliRouter Quick Start Guide

## What IntelliRouter Actually Does

IntelliRouter is an LLM gateway that routes requests to different AI model providers (OpenAI, Anthropic, Ollama, etc.) through a single OpenAI-compatible API endpoint.

## Minimal Setup to Get It Working

### 1. Prerequisites

- Rust installed
- Ollama installed and running (if you want to use local models)
- OpenAI API key (if you want to use OpenAI)

### 2. Create a Minimal Config File

Create `config/minimal.toml`:

```toml
# Minimal configuration for IntelliRouter
environment = "development"

[server]
host = "127.0.0.1"
port = 8080

[model_registry]
default_provider = "ollama"

# Local Ollama models
[[model_registry.providers]]
name = "ollama"
endpoint = "http://localhost:11434/api"
default_model = "llama3"
available_models = ["llama3", "mistral", "codellama"]
timeout_secs = 120
max_retries = 3

# OpenAI (optional - comment out if not using)
[[model_registry.providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
endpoint = "https://api.openai.com/v1"
default_model = "gpt-3.5-turbo"
available_models = ["gpt-3.5-turbo", "gpt-4"]
timeout_secs = 60
max_retries = 3

[router]
default_strategy = "round-robin"

[telemetry]
log_level = "info"

# Disable all the complex features for now
[auth]
auth_enabled = false

[rag]
enabled = false

[chain_engine]
enable_caching = false

[persona_layer]
enabled = false

[plugin_sdk]
enabled = false
```

### 3. Build and Run

```bash
# Build the project
cargo build --release

# Set environment variables (if using OpenAI)
export OPENAI_API_KEY="your-key-here"

# Run with minimal config
./target/release/intellirouter run --role router --config config/minimal.toml
```

### 4. Test It

```bash
# Test with Ollama (local)
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3",
    "messages": [{"role": "user", "content": "Hello! Can you see this?"}]
  }'

# Test with OpenAI (if configured)
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [{"role": "user", "content": "Hello! Can you see this?"}]
  }'

# Test streaming
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3",
    "messages": [{"role": "user", "content": "Tell me a short story"}],
    "stream": true
  }'
```

## What's Actually Working

Based on the code analysis:

1. **✓ OpenAI-compatible API** - The `/v1/chat/completions` endpoint exists
2. **✓ Ollama support** - Full connector implementation for local models
3. **✓ OpenAI support** - Connector for OpenAI API
4. **✓ Basic routing** - Can route between different providers
5. **✓ Streaming support** - Both SSE streaming and non-streaming responses

## Common Issues & Solutions

### Issue: "Service is at maximum capacity"
- The default max_connections might be too low
- Add to config: `max_connections = 1000` under `[server]`

### Issue: Can't connect to Ollama
- Make sure Ollama is running: `ollama serve`
- Check if Ollama is on the default port: `curl http://localhost:11434/api/tags`
- Pull a model if needed: `ollama pull llama3`

### Issue: Routing doesn't work as expected
- Check logs for errors
- Try specifying the provider explicitly in the model name: `ollama/llama3`
- Ensure the model is listed in `available_models` in the config

## Simplified Architecture

For your use case, you really only need:
- `llm_proxy` - The API endpoint
- `model_registry` - Tracks available models
- `router_core` - Routes requests to the right backend
- Model connectors (Ollama, OpenAI)

Everything else (RAG, personas, chain engine, auth, plugins, etc.) can be disabled.

## Next Steps

Once basic routing works:
1. Enable simple persona injection if needed
2. Add basic authentication with API keys
3. Configure routing strategies (cost-based, latency-based)
4. Add more model providers as needed

## Python SDK Quick Test

```python
import openai

# Point to your IntelliRouter instance
client = openai.OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"  # Since auth is disabled
)

# Use Ollama model
response = client.chat.completions.create(
    model="llama3",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response.choices[0].message.content)

# Use OpenAI model (if configured)
response = client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response.choices[0].message.content)