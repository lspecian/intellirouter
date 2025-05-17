# IntelliRouter TypeScript SDK

The IntelliRouter TypeScript SDK provides a clean, idiomatic interface for interacting with IntelliRouter, including support for chat completions, streaming, and chain execution.

## Installation

```bash
npm install intellirouter
# or
yarn add intellirouter
# or
pnpm add intellirouter
```

## Usage

### Basic Chat Completion

```typescript
import { IntelliRouter } from 'intellirouter';

const client = new IntelliRouter({
  apiKey: 'your-api-key',
});

async function main() {
  const response = await client.chat.completions.create({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello!' },
    ],
  });

  console.log(response.choices[0].message.content);
}

main();
```

### Streaming Chat Completion

```typescript
import { IntelliRouter } from 'intellirouter';

const client = new IntelliRouter({
  apiKey: 'your-api-key',
});

async function main() {
  const stream = await client.chat.completions.create({
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Hello!' },
    ],
    stream: true,
  });

  for await (const chunk of stream) {
    if (chunk.choices[0]?.delta?.content) {
      process.stdout.write(chunk.choices[0].delta.content);
    }
  }
}

main();
```

### Chain Execution

```typescript
import { IntelliRouter } from 'intellirouter';

const client = new IntelliRouter({
  apiKey: 'your-api-key',
});

async function main() {
  const chain = await client.chains.get('my-chain');
  const result = await chain.execute({
    query: 'What is the capital of France?',
  });

  console.log(result);
}

main();
```

## API Reference

### IntelliRouter

```typescript
new IntelliRouter({
  apiKey?: string;
  baseUrl?: string;
  timeout?: number;
  maxRetries?: number;
})
```

### Chat Completions

```typescript
client.chat.completions.create({
  model: string;
  messages: Array<{
    role: 'system' | 'user' | 'assistant' | 'function';
    content: string;
    name?: string;
  }>;
  temperature?: number;
  top_p?: number;
  n?: number;
  stream?: boolean;
  stop?: string | string[];
  max_tokens?: number;
  presence_penalty?: number;
  frequency_penalty?: number;
  logit_bias?: Record<string, number>;
  user?: string;
})
```

### Chains

```typescript
client.chains.list()
client.chains.get(chainId: string)
client.chains.create(definition: object)
client.chains.update(chainId: string, definition: object)
client.chains.delete(chainId: string)
chain.execute(inputs: object)
```

## Error Handling

```typescript
import { IntelliRouter, IntelliRouterError } from 'intellirouter';

const client = new IntelliRouter({
  apiKey: 'your-api-key',
});

async function main() {
  try {
    const response = await client.chat.completions.create({
      model: 'non-existent-model',
      messages: [
        { role: 'user', content: 'Hello!' },
      ],
    });
  } catch (error) {
    if (error instanceof IntelliRouterError) {
      console.error(`Error: ${error.message}`);
    } else {
      console.error('Unknown error:', error);
    }
  }
}

main();
```

## Development

### Setup

```bash
git clone https://github.com/yourusername/intellirouter.git
cd intellirouter/sdk/typescript
npm install
```

### Testing

```bash
npm test
```

### Building

```bash
npm run build