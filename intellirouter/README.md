# IntelliRouter Python SDK

A Python SDK for interacting with the IntelliRouter API.

## Installation

```bash
pip install intellirouter
```

## Features

- **Chat Completions API**: Create chat completions with support for streaming responses
- **Chain Execution Framework**: Build and execute complex chains of operations
- **Model Management**: List, select, and manage available models
- **Asynchronous API**: Full async support for all operations
- **Streaming Support**: Stream responses for both chat completions and chain executions
- **Comprehensive Error Handling**: Detailed exception hierarchy for better error handling
- **Flexible Configuration**: Configure the SDK through constructor arguments, environment variables, or a configuration file

## Quick Start

```python
import os
from intellirouter import IntelliRouter

# Initialize the client
client = IntelliRouter(api_key="your-api-key")

# Create a chat completion
completion = client.chat.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello, how are you?"}
    ]
)

print(completion.choices[0].message.content)
```

## Documentation

### Chat Completions

The Chat Completions API allows you to create chat completions with various models.

```python
# Create a chat completion
completion = client.chat.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello, how are you?"}
    ]
)

print(completion.choices[0].message.content)

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

### Chain Execution

The Chain Execution Framework allows you to build and execute complex chains of operations.

```python
# Create a chain
chain = client.chains.create(
    name="Simple Text Processing Chain",
    description="A chain that processes text through multiple steps",
    steps={
        "tokenize": {
            "id": "tokenize",
            "type": "text_processor",
            "name": "Tokenize Text",
            "description": "Split text into tokens",
            "inputs": {"text": "string"},
            "outputs": {"tokens": "tokens"},
            "config": {"lowercase": True},
        },
        "filter": {
            "id": "filter",
            "type": "text_processor",
            "name": "Filter Tokens",
            "description": "Filter out stopwords",
            "inputs": {"tokens": "tokens"},
            "outputs": {"filtered_tokens": "tokens"},
            "config": {"stopwords": ["the", "a", "an"]},
        },
        "join": {
            "id": "join",
            "type": "text_processor",
            "name": "Join Tokens",
            "description": "Join tokens back into text",
            "inputs": {"tokens": "tokens"},
            "outputs": {"processed_text": "string"},
            "config": {"separator": " "},
        },
    },
    dependencies=[
        {
            "dependent_step": "filter",
            "required_step": "tokenize",
        },
        {
            "dependent_step": "join",
            "required_step": "filter",
        },
    ],
)

# Execute the chain
result = client.chains.run(
    chain_id=chain.id,
    inputs={"text": "The quick brown fox jumps over the lazy dog"},
)

print(f"Chain execution status: {result.status}")
print(f"Chain outputs: {result.outputs}")

# Execute the chain with streaming
for event in client.chains.stream(
    chain_id=chain.id,
    inputs={"text": "The quick brown fox jumps over the lazy dog"},
):
    print(f"Event: {event.event_type}, Step: {event.step_id}")
```

### Asynchronous API

The SDK provides full async support for all operations.

```python
import asyncio
from intellirouter import IntelliRouter

client = IntelliRouter(api_key="your-api-key")

async def main():
    # Create a chat completion
    completion = await client.chat.acreate(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello, how are you?"}
        ]
    )
    
    print(completion.choices[0].message.content)
    
    # Create a streaming chat completion
    async for chunk in client.chat.acreate(
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

asyncio.run(main())
```

## Configuration

The SDK can be configured using:

1. Constructor arguments
2. Environment variables
3. Configuration file

### Environment Variables

- `INTELLIROUTER_API_KEY`: API key for authentication
- `INTELLIROUTER_BASE_URL`: Base URL for the API (default: http://localhost:8000)
- `INTELLIROUTER_TIMEOUT`: Timeout for API requests in seconds (default: 60)
- `INTELLIROUTER_MAX_RETRIES`: Maximum number of retries for failed requests (default: 3)
- `INTELLIROUTER_CONFIG_FILE`: Path to configuration file

### Configuration File

The default location for the configuration file is `~/.intellirouter/config.json`. You can specify a different location using the `INTELLIROUTER_CONFIG_FILE` environment variable.

Example configuration file:

```json
{
  "api_key": "your-api-key",
  "base_url": "http://localhost:8000",
  "timeout": 60,
  "max_retries": 3
}
```

## Error Handling

The SDK provides a comprehensive exception hierarchy for error handling:

- `IntelliRouterError`: Base class for all exceptions
- `APIError`: Base class for API errors
- `AuthenticationError`: Authentication failed
- `RateLimitError`: Rate limit exceeded
- `ServerError`: Server error
- `ValidationError`: Validation failed
- `ConfigurationError`: Configuration error

Example:

```python
from intellirouter import IntelliRouter
from intellirouter.exceptions import AuthenticationError, RateLimitError, ServerError

client = IntelliRouter(api_key="your-api-key")

try:
    completion = client.chat.create(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello, how are you?"}
        ]
    )
    print(completion.choices[0].message.content)
except AuthenticationError:
    print("Authentication failed. Check your API key.")
except RateLimitError:
    print("Rate limit exceeded. Please try again later.")
except ServerError:
    print("Server error. Please try again later.")
except Exception as e:
    print(f"An error occurred: {str(e)}")
```

## License

MIT