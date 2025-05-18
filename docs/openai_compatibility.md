# OpenAI API Compatibility

This document describes the OpenAI API compatibility layer in IntelliRouter, which allows it to handle requests in the OpenAI API format, including multimodal content.

## Overview

IntelliRouter provides an OpenAI-compatible API interface that can route requests to various LLM providers (like Anthropic, Mistral, etc.) while maintaining a consistent API format. This allows applications built for the OpenAI API to work seamlessly with IntelliRouter.

## API Endpoints

The following OpenAI-compatible endpoints are supported:

- `POST /v1/chat/completions` - For regular chat completions
- `POST /v1/chat/completions/stream` - For streaming chat completions

## Message Format

IntelliRouter supports both the simple string content format and the newer multimodal content format:

### Simple String Content

```json
{
  "model": "claude-3-sonnet",
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ]
}
```

### Multimodal Content

```json
{
  "model": "claude-3-sonnet",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "What's in this image?"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "https://example.com/image.jpg",
            "detail": "auto"
          }
        }
      ]
    }
  ]
}
```

## Supported Content Types

The following content types are supported:

- `text` - Text content
- `image_url` - Image content (URL or base64 data)
- `input_audio` - Audio content (base64 data)
- `file` - File content (base64 data or file ID)

## Implementation Details

The implementation follows clean architecture principles, with a clear separation of concerns:

1. **Domain Layer** - Core business logic and entities
   - `Message` - Represents a chat message with role and content
   - `MessageContent` - Can be either a string or an array of content parts
   - `ContentPart` - Represents a specific type of content (text, image, etc.)

2. **DTO Layer** - Data Transfer Objects for the API interface
   - `ChatCompletionRequest` - Request format for chat completions
   - `ChatCompletionResponse` - Response format for chat completions
   - `ChatCompletionChunk` - Response format for streaming chat completions

3. **Service Layer** - Business logic for processing requests
   - `ChatCompletionService` - Handles processing of chat completion requests

4. **Controller Layer** - API endpoints and request handling
   - `chat_completions` - Handler for regular chat completions
   - `chat_completions_stream` - Handler for streaming chat completions

5. **Validation Layer** - Request validation
   - `validate_chat_completion_request` - Validates chat completion requests

## Testing

The implementation includes comprehensive tests:

1. **Unit Tests** - Test individual components in isolation
   - Domain model tests
   - DTO tests
   - Service tests
   - Validation tests

2. **Integration Tests** - Test the API endpoints
   - Test with string content
   - Test with array content
   - Test with multimodal content
   - Test error handling

## Usage

To use IntelliRouter as an OpenAI API-compatible service:

```python
from openai import OpenAI

client = OpenAI(
    api_key="not-needed-for-local",  # API key isn't validated for local use
    base_url="http://localhost:9000/v1"  # Point to your local IntelliRouter
)

response = client.chat.completions.create(
    model="claude-3-sonnet",  # Use this model name
    messages=[
        {"role": "user", "content": "Hello, how are you?"}
    ]
)
print(response.choices[0].message.content)
```

## Future Improvements

- Support for function calling
- Support for tool use
- Support for more content types
- Support for more OpenAI API endpoints