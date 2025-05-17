Examples
========

This section provides examples of how to use the IntelliRouter Python SDK.

Chat Completions
---------------

Basic Chat Completion
~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

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

Streaming Chat Completion
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    from intellirouter import IntelliRouter

    # Initialize the client
    client = IntelliRouter(api_key="your-api-key")

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

Asynchronous Chat Completion
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    import asyncio
    from intellirouter import IntelliRouter

    # Initialize the client
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

Chain Execution
--------------

Creating and Running a Chain
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    from intellirouter import IntelliRouter
    from intellirouter.chains import Chain, ChainStep, ChainDependency

    # Initialize the client
    client = IntelliRouter(api_key="your-api-key")

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

    # Execute the chain
    result = client.chains.run(
        chain_id=chain.id,
        inputs={"text": "The quick brown fox jumps over the lazy dog"},
    )

    print(f"Chain execution status: {result.status}")
    print(f"Chain outputs: {result.outputs}")

Streaming Chain Execution
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    from intellirouter import IntelliRouter

    # Initialize the client
    client = IntelliRouter(api_key="your-api-key")

    # Get an existing chain
    chain = client.chains.get("chain-id")

    # Execute the chain with streaming
    for event in client.chains.stream(
        chain_id=chain.id,
        inputs={"text": "The quick brown fox jumps over the lazy dog"},
    ):
        print(f"Event: {event.event_type}, Step: {event.step_id}")
        if event.event_type == "chain_completed":
            print(f"Chain completed with outputs: {event.data.get('outputs', {})}")

Asynchronous Chain Execution
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    import asyncio
    from intellirouter import IntelliRouter

    # Initialize the client
    client = IntelliRouter(api_key="your-api-key")

    async def main():
        # Create a chain asynchronously
        chain = await client.chains.acreate(
            name="Async Chain",
            description="A chain created asynchronously",
            steps={
                "step1": {
                    "id": "step1",
                    "type": "llm",
                    "name": "Generate Text",
                    "description": "Generate text based on input",
                    "inputs": {"prompt": "string"},
                    "outputs": {"response": "string"},
                }
            }
        )
        
        # Execute the chain asynchronously
        result = await client.chains.arun(
            chain_id=chain.id,
            inputs={"prompt": "Hello, world!"},
        )
        
        print(f"Chain execution status: {result.status}")
        print(f"Chain outputs: {result.outputs}")
        
        # Execute the chain with streaming asynchronously
        async for event in client.chains.astream(
            chain_id=chain.id,
            inputs={"prompt": "Hello, world!"},
        ):
            print(f"Event: {event.event_type}, Step: {event.step_id}")

    asyncio.run(main())

Model Management
--------------

Listing Available Models
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    from intellirouter import IntelliRouter

    # Initialize the client
    client = IntelliRouter(api_key="your-api-key")

    # List available models
    models = client.models.list()
    for model in models:
        print(f"{model.id} - {model.name}")

Getting Model Details
~~~~~~~~~~~~~~~~~~~

.. code-block:: python

    from intellirouter import IntelliRouter

    # Initialize the client
    client = IntelliRouter(api_key="your-api-key")

    # Get model details
    model = client.models.get("gpt-3.5-turbo")
    print(f"Model: {model.name}")
    print(f"Description: {model.description}")
    print(f"Context Length: {model.context_length}")