"""
Example script for using the chat completions API.
"""

import os
import asyncio
from intellirouter import IntelliRouter

# Set your API key
api_key = os.environ.get("INTELLIROUTER_API_KEY", "your-api-key")

# Initialize the client
client = IntelliRouter(api_key=api_key)

# Example messages
messages = [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello, how are you?"},
]

def sync_example():
    """
    Example of using the synchronous API.
    """
    print("Synchronous example:")
    
    # Create a chat completion
    completion = client.chat.create(
        model="gpt-3.5-turbo",
        messages=messages,
    )
    
    # Print the response
    print(f"Response: {completion.choices[0].message.content}")
    print()
    
    # Create a streaming chat completion
    print("Streaming example:")
    
    # Buffer to collect the full response
    full_response = ""
    
    # Stream the response
    for chunk in client.chat.create(
        model="gpt-3.5-turbo",
        messages=messages,
        stream=True,
    ):
        # Get the content from the chunk
        content = chunk.choices[0].delta.content
        
        # Print the content
        if content:
            print(content, end="", flush=True)
            full_response += content
    
    print("\n")
    print(f"Full response: {full_response}")
    print()

async def async_example():
    """
    Example of using the asynchronous API.
    """
    print("Asynchronous example:")
    
    # Create a chat completion
    completion = await client.chat.acreate(
        model="gpt-3.5-turbo",
        messages=messages,
    )
    
    # Print the response
    print(f"Response: {completion.choices[0].message.content}")
    print()
    
    # Create a streaming chat completion
    print("Async streaming example:")
    
    # Buffer to collect the full response
    full_response = ""
    
    # Stream the response
    async for chunk in client.chat.acreate(
        model="gpt-3.5-turbo",
        messages=messages,
        stream=True,
    ):
        # Get the content from the chunk
        content = chunk.choices[0].delta.content
        
        # Print the content
        if content:
            print(content, end="", flush=True)
            full_response += content
    
    print("\n")
    print(f"Full response: {full_response}")
    print()

if __name__ == "__main__":
    # Run the synchronous example
    sync_example()
    
    # Run the asynchronous example
    asyncio.run(async_example())