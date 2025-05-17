# IntelliRouter Rust SDK

The IntelliRouter Rust SDK provides a clean, idiomatic interface for interacting with IntelliRouter, including support for chat completions, streaming, and chain execution.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
intellirouter = "0.1.0"
```

## Usage

### Basic Chat Completion

```rust
use intellirouter::{IntelliRouter, ChatCompletionRequest, Message, Role};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = IntelliRouter::new("your-api-key");

    let request = ChatCompletionRequest::new("gpt-3.5-turbo")
        .add_message(Message::new(Role::System, "You are a helpful assistant."))
        .add_message(Message::new(Role::User, "Hello!"));

    let response = client.chat_completions().create(request).await?;
    
    if let Some(message) = response.choices.first().map(|c| &c.message) {
        println!("{}", message.content);
    }

    Ok(())
}
```

### Streaming Chat Completion

```rust
use intellirouter::{IntelliRouter, ChatCompletionRequest, Message, Role};
use anyhow::Result;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let client = IntelliRouter::new("your-api-key");

    let request = ChatCompletionRequest::new("gpt-3.5-turbo")
        .add_message(Message::new(Role::System, "You are a helpful assistant."))
        .add_message(Message::new(Role::User, "Hello!"))
        .stream(true);

    let mut stream = client.chat_completions().create_stream(request).await?;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
            print!("{}", content);
        }
    }

    Ok(())
}
```

### Chain Execution

```rust
use intellirouter::{IntelliRouter, ChainExecutionRequest};
use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let client = IntelliRouter::new("your-api-key");

    let chain = client.chains().get("my-chain").await?;
    
    let request = ChainExecutionRequest::new()
        .add_input("query", "What is the capital of France?");
    
    let result = chain.execute(request).await?;
    
    println!("{}", result);

    Ok(())
}
```

## API Reference

### IntelliRouter

```rust
IntelliRouter::new(api_key: impl Into<String>)
IntelliRouter::with_config(config: ClientConfig)
```

### Chat Completions

```rust
client.chat_completions().create(request: ChatCompletionRequest) -> Result<ChatCompletionResponse>
client.chat_completions().create_stream(request: ChatCompletionRequest) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>>
```

### Chains

```rust
client.chains().list() -> Result<Vec<Chain>>
client.chains().get(chain_id: impl Into<String>) -> Result<Chain>
client.chains().create(definition: impl Serialize) -> Result<Chain>
client.chains().update(chain_id: impl Into<String>, definition: impl Serialize) -> Result<Chain>
client.chains().delete(chain_id: impl Into<String>) -> Result<()>
chain.execute(request: ChainExecutionRequest) -> Result<Value>
```

## Error Handling

```rust
use intellirouter::{IntelliRouter, ChatCompletionRequest, Message, Role, Error};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = IntelliRouter::new("your-api-key");

    let request = ChatCompletionRequest::new("non-existent-model")
        .add_message(Message::new(Role::User, "Hello!"));

    match client.chat_completions().create(request).await {
        Ok(response) => {
            if let Some(message) = response.choices.first().map(|c| &c.message) {
                println!("{}", message.content);
            }
        }
        Err(err) => {
            match err {
                Error::ApiError(api_err) => {
                    println!("API Error: {} ({})", api_err.message, api_err.code);
                }
                Error::HttpError(status) => {
                    println!("HTTP Error: {}", status);
                }
                Error::IoError(io_err) => {
                    println!("IO Error: {}", io_err);
                }
                Error::SerdeError(serde_err) => {
                    println!("Serialization Error: {}", serde_err);
                }
            }
        }
    }

    Ok(())
}
```

## Development

### Setup

```bash
git clone https://github.com/yourusername/intellirouter.git
cd intellirouter/sdk/rust
cargo build
```

### Testing

```bash
cargo test
```

### Building

```bash
cargo build --release