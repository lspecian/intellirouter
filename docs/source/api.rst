API Reference
============

This section provides detailed documentation for the IntelliRouter Python SDK API.

Client
------

.. autoclass:: intellirouter.client.IntelliRouter
   :members:
   :undoc-members:
   :show-inheritance:

Chat
----

.. autoclass:: intellirouter.chat.api.ChatClient
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chat.models.ChatMessage
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chat.models.ChatCompletion
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chat.models.ChatCompletionChunk
   :members:
   :undoc-members:
   :show-inheritance:

Chains
------

.. autoclass:: intellirouter.chains.api.ChainClient
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chains.models.Chain
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chains.models.ChainStep
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chains.models.ChainDependency
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chains.models.ChainExecutionResult
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.chains.models.ChainExecutionEvent
   :members:
   :undoc-members:
   :show-inheritance:

Models
------

.. autoclass:: intellirouter.models.api.ModelClient
   :members:
   :undoc-members:
   :show-inheritance:

Configuration
------------

.. autoclass:: intellirouter.config.settings.Configuration
   :members:
   :undoc-members:
   :show-inheritance:

Transport
---------

.. autoclass:: intellirouter.transport.base.Transport
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.transport.http.HTTPTransport
   :members:
   :undoc-members:
   :show-inheritance:

Exceptions
----------

.. autoclass:: intellirouter.exceptions.IntelliRouterError
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.exceptions.APIError
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.exceptions.AuthenticationError
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.exceptions.RateLimitError
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.exceptions.ServerError
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.exceptions.ValidationError
   :members:
   :undoc-members:
   :show-inheritance:

.. autoclass:: intellirouter.exceptions.ConfigurationError
   :members:
   :undoc-members:
   :show-inheritance: