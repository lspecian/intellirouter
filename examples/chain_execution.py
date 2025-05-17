"""
Example of using the Chain Execution Framework in the IntelliRouter Python SDK.
"""

import os
import json
from intellirouter import IntelliRouter
from intellirouter.chains import Chain, ChainStep, ChainDependency

# Initialize the client
client = IntelliRouter(
    api_key=os.environ.get("INTELLIROUTER_API_KEY", "your-api-key"),
    base_url=os.environ.get("INTELLIROUTER_BASE_URL", "http://localhost:8000"),
)

# Create a chain
chain = client.chains.create(
    name="Simple Text Processing Chain",
    description="A chain that processes text through multiple steps",
    steps={
        "tokenize": ChainStep(
            id="tokenize",
            type="text_processor",
            name="Tokenize Text",
            description="Split text into tokens",
            inputs={"text": "string"},
            outputs={"tokens": "tokens"},
            config={"lowercase": True},
        ),
        "filter": ChainStep(
            id="filter",
            type="text_processor",
            name="Filter Tokens",
            description="Filter out stopwords",
            inputs={"tokens": "tokens"},
            outputs={"filtered_tokens": "tokens"},
            config={"stopwords": ["the", "a", "an"]},
        ),
        "join": ChainStep(
            id="join",
            type="text_processor",
            name="Join Tokens",
            description="Join tokens back into text",
            inputs={"tokens": "tokens"},
            outputs={"processed_text": "string"},
            config={"separator": " "},
        ),
    },
    dependencies=[
        ChainDependency(
            dependent_step="filter",
            required_step="tokenize",
        ),
        ChainDependency(
            dependent_step="join",
            required_step="filter",
        ),
    ],
)

print(f"Created chain: {chain.id}")

# Get the chain
retrieved_chain = client.chains.get(chain.id)
print(f"Retrieved chain: {retrieved_chain.name}")

# List chains
chains = client.chains.list(limit=10)
print(f"Found {len(chains)} chains")

# Update the chain
updated_chain = client.chains.update(
    chain_id=chain.id,
    description="An updated chain that processes text through multiple steps",
)
print(f"Updated chain: {updated_chain.description}")

# Execute the chain
result = client.chains.run(
    chain_id=chain.id,
    inputs={"text": "The quick brown fox jumps over the lazy dog"},
)
print(f"Chain execution status: {result.status}")
print(f"Chain outputs: {json.dumps(result.outputs, indent=2)}")

# Execute the chain with streaming
print("Streaming chain execution:")
for event in client.chains.stream(
    chain_id=chain.id,
    inputs={"text": "The quick brown fox jumps over the lazy dog"},
):
    print(f"Event: {event.event_type}, Step: {event.step_id}")
    if event.event_type == "chain_completed":
        print(f"Chain completed with outputs: {json.dumps(event.data.get('outputs', {}), indent=2)}")

# Delete the chain
client.chains.delete(chain.id)
print(f"Deleted chain: {chain.id}")

# Async example (uncomment to run)
"""
import asyncio

async def async_example():
    # Create a chain asynchronously
    chain = await client.chains.acreate(
        name="Async Chain",
        description="A chain created asynchronously",
    )
    print(f"Created async chain: {chain.id}")
    
    # Get the chain asynchronously
    retrieved_chain = await client.chains.aget(chain.id)
    print(f"Retrieved async chain: {retrieved_chain.name}")
    
    # Execute the chain asynchronously
    result = await client.chains.arun(
        chain_id=chain.id,
        inputs={"text": "Async processing example"},
    )
    print(f"Async chain execution status: {result.status}")
    
    # Execute the chain with streaming asynchronously
    print("Streaming async chain execution:")
    async for event in client.chains.astream(
        chain_id=chain.id,
        inputs={"text": "Async streaming example"},
    ):
        print(f"Async event: {event.event_type}, Step: {event.step_id}")
    
    # Delete the chain asynchronously
    await client.chains.adelete(chain.id)
    print(f"Deleted async chain: {chain.id}")

# Run the async example
# asyncio.run(async_example())
"""