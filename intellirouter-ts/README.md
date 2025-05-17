# IntelliRouter TypeScript SDK

A TypeScript SDK for interacting with the IntelliRouter API.

## Installation

```bash
npm install intellirouter
```

## Features

- Chat completions (both Promise-based and streaming)
- Chain execution for complex workflows
- Configuration management
- Type-safe API
- Comprehensive error handling

## Quick Start

```typescript
import { IntelliRouter } from 'intellirouter';

// Initialize the client
const client = new IntelliRouter({
  apiKey: 'your-api-key',
  baseUrl: 'http://localhost:8000', // Optional
});

// Create a chat completion
async function main() {
  const completion = await client.chat.createCompletion({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello, how are you?' },
    ],
  });

  console.log(completion.choices[0].message.content);
}

main().catch(console.error);
```

## Streaming Example

```typescript
import { IntelliRouter } from 'intellirouter';

// Initialize the client
const client = new IntelliRouter({
  apiKey: 'your-api-key',
});

// Create a streaming chat completion
async function main() {
  const stream = await client.chat.createCompletionStream({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello, how are you?' },
    ],
  });

  for await (const chunk of stream) {
    const content = chunk.choices[0].delta.content;
    if (content) {
      process.stdout.write(content);
    }
  }
}

main().catch(console.error);
```

## API Reference

### Client Configuration

```typescript
interface IntelliRouterConfig {
  baseUrl?: string;        // Default: 'http://localhost:8000'
  apiKey?: string;         // Required for authentication
  timeout?: number;        // Default: 30000 (30 seconds)
  maxRetries?: number;     // Default: 3
  defaultHeaders?: Record<string, string>;
}
```

### Chat Completions

#### Creating a Chat Completion

```typescript
// Create a chat completion
client.chat.createCompletion(options: ChatCompletionOptions): Promise<ChatCompletionResponse>
```

Example:

```typescript
const completion = await client.chat.createCompletion({
  model: 'gpt-3.5-turbo',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Hello, how are you?' },
  ],
  temperature: 0.7,
  max_tokens: 100,
});

console.log(completion.choices[0].message.content);
console.log(`Usage: ${completion.usage.total_tokens} tokens`);
```

#### Creating a Streaming Chat Completion

```typescript
// Create a streaming chat completion
client.chat.createCompletionStream(options: ChatCompletionOptions): Promise<ReadableStream<ChatCompletionChunk>>
```

Example:

```typescript
const stream = await client.chat.createCompletionStream({
  model: 'gpt-3.5-turbo',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Write a short poem about programming.' },
  ],
});

// Process the stream
for await (const chunk of stream) {
  const content = chunk.choices[0].delta.content;
  if (content) {
    process.stdout.write(content);
  }
}
```

#### Simplified Completion

```typescript
// Create a simple completion with just messages
client.chat.complete(
  messages: ChatMessage[],
  model: string,
  options?: Omit<ChatCompletionOptions, 'messages' | 'model'>
): Promise<string>
```

Example:

```typescript
const response = await client.chat.complete(
  [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Hello, how are you?' },
  ],
  'gpt-3.5-turbo',
  { temperature: 0.7 }
);

console.log(response); // Just the content string
```

### Chain Execution

#### Creating a Chain

```typescript
// Create a new chain
client.chains.createChain(definition: ChainDefinition): Promise<ChainDefinition>
```

Example:

```typescript
const chain = await client.chains.createChain({
  id: 'my-chain',
  name: 'My First Chain',
  nodes: [
    {
      id: 'node1',
      type: 'llm',
      name: 'LLM Node',
      model: 'gpt-3.5-turbo',
      prompt: 'Summarize the following text: {{input}}',
    },
    {
      id: 'node2',
      type: 'function',
      name: 'Function Node',
      function: 'formatOutput',
    },
  ],
  edges: [
    {
      source: 'node1',
      target: 'node2',
    },
  ],
});
```

#### Executing a Chain

```typescript
// Execute a chain
client.chains.executeChain(request: ChainExecutionRequest): Promise<ChainExecutionResponse>
```

Example:

```typescript
const result = await client.chains.executeChain({
  chain: 'my-chain',
  inputs: {
    input: 'This is a long text that needs to be summarized...',
  },
});

console.log(result.outputs);
```

#### Streaming Chain Execution

```typescript
// Execute a chain with streaming response
client.chains.executeChainStream(request: ChainExecutionRequest): Promise<ReadableStream<ChainExecutionChunk>>
```

Example:

```typescript
const stream = await client.chains.executeChainStream({
  chain: 'my-chain',
  inputs: {
    input: 'This is a long text that needs to be summarized...',
  },
});

for await (const chunk of stream) {
  console.log(chunk);
}
```

### Configuration Management

```typescript
// Get the current configuration
client.config.getConfig(): IntelliRouterConfig

// Update the configuration
client.config.updateConfig(config: Partial<IntelliRouterConfig>): IntelliRouterConfig
```

Example:

```typescript
// Get current config
const config = client.config.getConfig();
console.log(config);

// Update config
const updatedConfig = client.config.updateConfig({
  timeout: 60000,
  maxRetries: 5,
});
console.log(updatedConfig);
```

### Error Handling

The SDK provides a comprehensive error handling system:

```typescript
try {
  const completion = await client.chat.createCompletion({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'user', content: 'Hello' },
    ],
  });
} catch (error) {
  if (error instanceof ValidationError) {
    console.error('Validation error:', error.message);
  } else if (error instanceof AuthenticationError) {
    console.error('Authentication error:', error.message);
  } else if (error instanceof AuthorizationError) {
    console.error('Authorization error:', error.message);
  } else if (error instanceof NotFoundError) {
    console.error('Not found error:', error.message);
  } else if (error instanceof RateLimitError) {
    console.error('Rate limit error:', error.message);
  } else if (error instanceof ServerError) {
    console.error('Server error:', error.message, error.status);
  } else if (error instanceof TimeoutError) {
    console.error('Timeout error:', error.message);
  } else if (error instanceof ApiError) {
    console.error('API error:', error.message, error.status);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Advanced Usage

### Function Calling

```typescript
const completion = await client.chat.createCompletion({
  model: 'gpt-3.5-turbo',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'What\'s the weather in San Francisco?' },
  ],
  functions: [
    {
      name: 'get_weather',
      description: 'Get the current weather in a location',
      parameters: {
        type: 'object',
        properties: {
          location: {
            type: 'string',
            description: 'The city and state, e.g. San Francisco, CA',
          },
          unit: {
            type: 'string',
            enum: ['celsius', 'fahrenheit'],
            description: 'The unit of temperature',
          },
        },
        required: ['location'],
      },
    },
  ],
  function_call: 'auto',
});

if (completion.choices[0].message.function_call) {
  const functionCall = completion.choices[0].message.function_call;
  console.log(`Function: ${functionCall.name}`);
  console.log(`Arguments: ${functionCall.arguments}`);
  
  // Parse arguments
  const args = JSON.parse(functionCall.arguments);
  
  // Call your function
  const weatherData = await getWeather(args.location, args.unit);
  
  // Continue the conversation with the function result
  const followUpCompletion = await client.chat.createCompletion({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'What\'s the weather in San Francisco?' },
      { 
        role: 'assistant', 
        content: null,
        function_call: {
          name: 'get_weather',
          arguments: functionCall.arguments,
        },
      },
      {
        role: 'function',
        name: 'get_weather',
        content: JSON.stringify(weatherData),
      },
    ],
  });
  
  console.log(followUpCompletion.choices[0].message.content);
}
```

### Tool Calling

```typescript
const completion = await client.chat.createCompletion({
  model: 'gpt-3.5-turbo',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'What\'s the weather in San Francisco and Tokyo?' },
  ],
  tools: [
    {
      type: 'function',
      function: {
        name: 'get_weather',
        description: 'Get the current weather in a location',
        parameters: {
          type: 'object',
          properties: {
            location: {
              type: 'string',
              description: 'The city and state, e.g. San Francisco, CA',
            },
            unit: {
              type: 'string',
              enum: ['celsius', 'fahrenheit'],
              description: 'The unit of temperature',
            },
          },
          required: ['location'],
        },
      },
    },
  ],
  tool_choice: 'auto',
});

if (completion.choices[0].message.tool_calls) {
  const toolCalls = completion.choices[0].message.tool_calls;
  
  // Process each tool call
  const toolResults = await Promise.all(
    toolCalls.map(async (toolCall) => {
      if (toolCall.type === 'function') {
        const functionCall = toolCall.function;
        console.log(`Function: ${functionCall.name}`);
        console.log(`Arguments: ${functionCall.arguments}`);
        
        // Parse arguments
        const args = JSON.parse(functionCall.arguments);
        
        // Call your function
        const result = await getWeather(args.location, args.unit);
        
        return {
          tool_call_id: toolCall.id,
          role: 'tool',
          name: functionCall.name,
          content: JSON.stringify(result),
        };
      }
    })
  );
  
  // Continue the conversation with the tool results
  const followUpCompletion = await client.chat.createCompletion({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'What\'s the weather in San Francisco and Tokyo?' },
      { 
        role: 'assistant', 
        content: null,
        tool_calls: completion.choices[0].message.tool_calls,
      },
      ...toolResults,
    ],
  });
  
  console.log(followUpCompletion.choices[0].message.content);
}
```

## License

MIT