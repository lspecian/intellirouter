# IntelliRouter Python SDK

The IntelliRouter Python SDK provides a clean, idiomatic interface for interacting with IntelliRouter, including support for chat completions, streaming, and chain execution.

## Installation

```bash
pip install intellirouter
```

## Usage

### Basic Chat Completion

```python
from intellirouter import IntelliRouter

client = IntelliRouter(api_key="your-api-key")

response = client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ]
)

print(response.choices[0].message.content)
```

### Streaming Chat Completion

```python
from intellirouter import IntelliRouter

client = IntelliRouter(api_key="your-api-key")

for chunk in client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"}
    ],
    stream=True
):
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

### Chain Execution

```python
from intellirouter import IntelliRouter

client = IntelliRouter(api_key="your-api-key")

chain = client.chains.get("my-chain")
result = chain.execute(
    inputs={
        "query": "What is the capital of France?"
    }
)

print(result)
```

## API Reference

### IntelliRouter

```python
IntelliRouter(
    api_key: str = None,
    base_url: str = "http://localhost:8000",
    timeout: float = 60.0,
    max_retries: int = 3
)
```

### Chat Completions

```python
client.chat.completions.create(
    model: str,
    messages: List[Dict],
    temperature: float = 1.0,
    top_p: float = 1.0,
    n: int = 1,
    stream: bool = False,
    stop: Union[str, List[str]] = None,
    max_tokens: int = None,
    presence_penalty: float = 0.0,
    frequency_penalty: float = 0.0,
    logit_bias: Dict[str, float] = None,
    user: str = None
)
```

### Chains

```python
client.chains.list()
client.chains.get(chain_id: str)
client.chains.create(definition: Dict)
client.chains.update(chain_id: str, definition: Dict)
client.chains.delete(chain_id: str)
chain.execute(inputs: Dict)
```

## Error Handling

```python
from intellirouter import IntelliRouter, IntelliRouterError

client = IntelliRouter(api_key="your-api-key")

try:
    response = client.chat.completions.create(
        model="non-existent-model",
        messages=[
            {"role": "user", "content": "Hello!"}
        ]
    )
except IntelliRouterError as e:
    print(f"Error: {e}")
```

## Development

### Setup

```bash
git clone https://github.com/yourusername/intellirouter.git
cd intellirouter/sdk/python
pip install -e ".[dev]"
```

### Testing

```bash
pytest
```

### Building

```bash
python -m build