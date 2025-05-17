Quick Start
==========

This guide will help you get started with the IntelliRouter Python SDK.

Installation
-----------

First, install the SDK:

.. code-block:: bash

    pip install intellirouter

Basic Usage
----------

Initialize the Client
~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    from intellirouter import IntelliRouter

    # Initialize the client with your API key
    client = IntelliRouter(api_key="your-api-key")

    # Or, if you have set the INTELLIROUTER_API_KEY environment variable:
    # client = IntelliRouter()

Chat Completions
~~~~~~~~~~~~~~

Create a basic chat completion:

.. code-block:: python

    # Create a chat completion
    completion = client.chat.create(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello, how are you?"}
        ]
    )

    # Print the response
    print(completion.choices[0].message.content)

Streaming Chat Completions
~~~~~~~~~~~~~~~~~~~~~~~

Stream the response token by token:

.. code-block:: python

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

Chain Execution
~~~~~~~~~~~~~

Create and execute a chain:

.. code-block:: python

    # Create a chain
    chain = client.chains.create(
        name="Simple Chain",
        description="A simple chain that generates a response",
        steps={
            "step1": {
                "id": "step1",
                "type": "llm",
                "name": "Generate Response",
                "description": "Generate a response to the input",
                "inputs": {"prompt": "string"},
                "outputs": {"response": "string"},
            }
        }
    )

    # Execute the chain
    result = client.chains.run(
        chain_id=chain.id,
        inputs={"prompt": "Hello, world!"}
    )

    # Print the result
    print(result.outputs["response"])

Asynchronous Usage
----------------

The SDK also supports asynchronous operations:

.. code-block:: python

    import asyncio
    from intellirouter import IntelliRouter

    client = IntelliRouter(api_key="your-api-key")

    async def main():
        # Create a chat completion asynchronously
        completion = await client.chat.acreate(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "Hello, how are you?"}
            ]
        )
        
        print(completion.choices[0].message.content)
        
        # Create a chain asynchronously
        chain = await client.chains.acreate(
            name="Async Chain",
            description="A chain created asynchronously",
            steps={
                "step1": {
                    "id": "step1",
                    "type": "llm",
                    "name": "Generate Response",
                    "description": "Generate a response to the input",
                    "inputs": {"prompt": "string"},
                    "outputs": {"response": "string"},
                }
            }
        )
        
        # Execute the chain asynchronously
        result = await client.chains.arun(
            chain_id=chain.id,
            inputs={"prompt": "Hello, world!"}
        )
        
        print(result.outputs["response"])

    # Run the async function
    asyncio.run(main())

Error Handling
------------

Handle errors gracefully:

.. code-block:: python

    from intellirouter import IntelliRouter
    from intellirouter.exceptions import (
        AuthenticationError,
        RateLimitError,
        ServerError,
        ValidationError,
    )

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
    except ValidationError as e:
        print(f"Validation error: {str(e)}")
    except Exception as e:
        print(f"An error occurred: {str(e)}")

Next Steps
---------

Now that you've learned the basics, you can:

- Explore the :doc:`api` for detailed documentation
- Check out the :doc:`examples` for more advanced usage
- Learn about :doc:`configuration` options
- Understand :doc:`error_handling` in depth