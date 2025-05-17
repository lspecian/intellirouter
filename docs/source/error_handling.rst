Error Handling
=============

The IntelliRouter Python SDK provides a comprehensive exception hierarchy for error handling.

Exception Hierarchy
-----------------

- ``IntelliRouterError``: Base class for all exceptions
- ``APIError``: Base class for API errors
  - ``AuthenticationError``: Authentication failed
  - ``RateLimitError``: Rate limit exceeded
  - ``ServerError``: Server error
- ``ValidationError``: Validation failed
- ``ConfigurationError``: Configuration error

Basic Error Handling
------------------

.. code-block:: python

    from intellirouter import IntelliRouter
    from intellirouter.exceptions import (
        IntelliRouterError,
        APIError,
        AuthenticationError,
        RateLimitError,
        ServerError,
        ValidationError,
        ConfigurationError,
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
    except APIError as e:
        print(f"API error: {str(e)}")
    except IntelliRouterError as e:
        print(f"IntelliRouter error: {str(e)}")
    except Exception as e:
        print(f"Unexpected error: {str(e)}")

Handling Specific Error Types
---------------------------

Authentication Errors
~~~~~~~~~~~~~~~~~~~

Authentication errors occur when the API key is invalid or missing:

.. code-block:: python

    try:
        completion = client.chat.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello, how are you?"}
            ]
        )
    except AuthenticationError:
        print("Authentication failed. Check your API key.")
        # Prompt the user to enter a new API key
        new_api_key = input("Enter a new API key: ")
        client = IntelliRouter(api_key=new_api_key)

Rate Limit Errors
~~~~~~~~~~~~~~~

Rate limit errors occur when you exceed the API rate limits:

.. code-block:: python

    import time
    from intellirouter.exceptions import RateLimitError

    max_retries = 5
    retry_count = 0
    backoff_time = 1  # seconds

    while retry_count < max_retries:
        try:
            completion = client.chat.create(
                model="gpt-3.5-turbo",
                messages=[
                    {"role": "user", "content": "Hello, how are you?"}
                ]
            )
            break  # Success, exit the loop
        except RateLimitError:
            retry_count += 1
            if retry_count < max_retries:
                print(f"Rate limit exceeded. Retrying in {backoff_time} seconds...")
                time.sleep(backoff_time)
                backoff_time *= 2  # Exponential backoff
            else:
                print("Maximum retries reached. Please try again later.")

Server Errors
~~~~~~~~~~~

Server errors occur when the API server encounters an error:

.. code-block:: python

    try:
        completion = client.chat.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello, how are you?"}
            ]
        )
    except ServerError:
        print("Server error. Please try again later.")
        # Try a different model or endpoint
        try:
            completion = client.chat.create(
                model="gpt-4",  # Try a different model
                messages=[
                    {"role": "user", "content": "Hello, how are you?"}
                ]
            )
        except ServerError:
            print("All servers are experiencing issues. Please try again later.")

Validation Errors
~~~~~~~~~~~~~~

Validation errors occur when the request is invalid:

.. code-block:: python

    try:
        completion = client.chat.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "invalid-role", "content": "Hello, how are you?"}
            ]
        )
    except ValidationError as e:
        print(f"Validation error: {str(e)}")
        # Fix the validation error
        completion = client.chat.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello, how are you?"}
            ]
        )

Configuration Errors
~~~~~~~~~~~~~~~~~

Configuration errors occur when the SDK is not properly configured:

.. code-block:: python

    try:
        client = IntelliRouter()  # No API key provided
    except ConfigurationError as e:
        print(f"Configuration error: {str(e)}")
        # Prompt the user to enter an API key
        api_key = input("Enter your API key: ")
        client = IntelliRouter(api_key=api_key)

Error Handling with Async Code
----------------------------

When using the asynchronous API, you can handle errors using try/except blocks within async functions:

.. code-block:: python

    import asyncio
    from intellirouter import IntelliRouter
    from intellirouter.exceptions import APIError

    client = IntelliRouter(api_key="your-api-key")

    async def main():
        try:
            completion = await client.chat.acreate(
                model="gpt-3.5-turbo",
                messages=[
                    {"role": "system", "content": "You are a helpful assistant."},
                    {"role": "user", "content": "Hello, how are you?"}
                ]
            )
            print(completion.choices[0].message.content)
        except APIError as e:
            print(f"API error: {str(e)}")

    asyncio.run(main())