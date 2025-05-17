Configuration
============

The IntelliRouter Python SDK can be configured in multiple ways:

1. Constructor arguments
2. Environment variables
3. Configuration file

Constructor Arguments
-------------------

The most direct way to configure the SDK is through constructor arguments:

.. code-block:: python

    from intellirouter import IntelliRouter

    client = IntelliRouter(
        api_key="your-api-key",
        base_url="http://localhost:8000",
        timeout=60,
        max_retries=3,
    )

Environment Variables
-------------------

The SDK can also be configured using environment variables:

.. code-block:: bash

    # Set environment variables
    export INTELLIROUTER_API_KEY="your-api-key"
    export INTELLIROUTER_BASE_URL="http://localhost:8000"
    export INTELLIROUTER_TIMEOUT=60
    export INTELLIROUTER_MAX_RETRIES=3

.. code-block:: python

    from intellirouter import IntelliRouter

    # The SDK will automatically read from environment variables
    client = IntelliRouter()

Available environment variables:

- ``INTELLIROUTER_API_KEY``: API key for authentication
- ``INTELLIROUTER_BASE_URL``: Base URL for the API (default: http://localhost:8000)
- ``INTELLIROUTER_TIMEOUT``: Timeout for API requests in seconds (default: 60)
- ``INTELLIROUTER_MAX_RETRIES``: Maximum number of retries for failed requests (default: 3)
- ``INTELLIROUTER_CONFIG_FILE``: Path to configuration file

Configuration File
----------------

The SDK can also be configured using a JSON configuration file:

.. code-block:: json

    {
      "api_key": "your-api-key",
      "base_url": "http://localhost:8000",
      "timeout": 60,
      "max_retries": 3
    }

The default location for the configuration file is ``~/.intellirouter/config.json``. You can specify a different location using the ``INTELLIROUTER_CONFIG_FILE`` environment variable:

.. code-block:: bash

    export INTELLIROUTER_CONFIG_FILE="/path/to/config.json"

.. code-block:: python

    from intellirouter import IntelliRouter

    # The SDK will automatically read from the configuration file
    client = IntelliRouter()

Configuration Precedence
----------------------

The SDK follows this order of precedence when determining configuration values:

1. Constructor arguments
2. Environment variables
3. Configuration file
4. Default values

For example, if you provide an API key as a constructor argument, it will take precedence over an API key specified in an environment variable or configuration file.

Default Values
------------

The SDK uses the following default values if not otherwise specified:

- ``base_url``: http://localhost:8000
- ``timeout``: 60 seconds
- ``max_retries``: 3

Custom Transport
--------------

You can also provide a custom transport layer to the SDK:

.. code-block:: python

    from intellirouter import IntelliRouter
    from intellirouter.transport import Transport

    class CustomTransport(Transport):
        def request(self, method, path, data=None, params=None):
            # Custom implementation
            pass

        def stream(self, method, path, data=None, params=None):
            # Custom implementation
            pass

        async def arequest(self, method, path, data=None, params=None):
            # Custom implementation
            pass

        async def astream(self, method, path, data=None, params=None):
            # Custom implementation
            pass

    client = IntelliRouter(
        api_key="your-api-key",
        transport=CustomTransport(),
    )