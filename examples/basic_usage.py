"""
Basic usage example for the IntelliRouter Python SDK.
"""

import os
import sys
import asyncio

# Add the parent directory to the path so we can import the SDK
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from intellirouter import IntelliRouter

# Example API key (replace with your own)
API_KEY = "your-api-key"

def sync_example():
    """
    Example of using the SDK synchronously.
    """
    print("=== Synchronous Example ===")
    
    # Initialize the client
    client = IntelliRouter(api_key=API_KEY)
    
    # This is a placeholder for when the chat API is implemented
    # In a real implementation, this would make an actual API call
    print("Chat API will be implemented in subtask 13.3")
    
    # This is a placeholder for when the chains API is implemented
    # In a real implementation, this would make an actual API call
    print("Chains API will be implemented in subtask 13.4")
    
    # This is a placeholder for when the models API is implemented
    # In a real implementation, this would make an actual API call
    print("Models API will be implemented in later subtasks")
    
    print()

async def async_example():
    """
    Example of using the SDK asynchronously.
    """
    print("=== Asynchronous Example ===")
    
    # Initialize the client
    client = IntelliRouter(api_key=API_KEY)
    
    # This is a placeholder for when the chat API is implemented
    # In a real implementation, this would make an actual API call
    print("Async Chat API will be implemented in subtask 13.3")
    
    # This is a placeholder for when the chains API is implemented
    # In a real implementation, this would make an actual API call
    print("Async Chains API will be implemented in subtask 13.4")
    
    # This is a placeholder for when the models API is implemented
    # In a real implementation, this would make an actual API call
    print("Async Models API will be implemented in later subtasks")
    
    print()

def main():
    """
    Run the examples.
    """
    # Run the synchronous example
    sync_example()
    
    # Run the asynchronous example
    asyncio.run(async_example())

if __name__ == "__main__":
    main()