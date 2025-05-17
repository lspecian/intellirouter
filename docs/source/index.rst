Welcome to IntelliRouter Python SDK's documentation!
===================================================

The IntelliRouter Python SDK provides a simple and intuitive interface for interacting with the IntelliRouter API.

.. toctree::
   :maxdepth: 2
   :caption: Contents:

   installation
   quickstart
   api
   examples
   configuration
   error_handling

Installation
===========

You can install the IntelliRouter Python SDK using pip:

.. code-block:: bash

   pip install intellirouter

Quick Start
==========

.. code-block:: python

   import os
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

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`