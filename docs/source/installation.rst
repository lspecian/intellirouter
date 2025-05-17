Installation
===========

This guide will help you install the IntelliRouter Python SDK.

Requirements
-----------

- Python 3.7 or higher
- pip (Python package installer)

Basic Installation
----------------

You can install the IntelliRouter Python SDK using pip:

.. code-block:: bash

    pip install intellirouter

This will install the SDK and all its dependencies.

Installation from Source
----------------------

You can also install the SDK from source:

.. code-block:: bash

    git clone https://github.com/intellirouter/intellirouter-python.git
    cd intellirouter-python
    pip install -e .

Development Installation
---------------------

If you want to contribute to the SDK, you can install it with development dependencies:

.. code-block:: bash

    pip install -e ".[dev]"

This will install additional dependencies for development, such as pytest, black, and sphinx.

Verifying Installation
--------------------

You can verify that the SDK is installed correctly by running:

.. code-block:: python

    import intellirouter
    print(intellirouter.__version__)

This should print the version of the SDK.

Dependencies
----------

The IntelliRouter Python SDK depends on the following packages:

- ``requests``: For making HTTP requests
- ``aiohttp``: For making asynchronous HTTP requests
- ``sseclient-py``: For handling server-sent events (streaming)

These dependencies will be automatically installed when you install the SDK.

Troubleshooting
-------------

If you encounter any issues during installation, try the following:

1. Upgrade pip:

   .. code-block:: bash

       pip install --upgrade pip

2. Install with verbose output:

   .. code-block:: bash

       pip install -v intellirouter

3. Check for conflicting dependencies:

   .. code-block:: bash

       pip check

4. If you're using a virtual environment, make sure it's activated:

   .. code-block:: bash

       source venv/bin/activate  # On Unix/macOS
       venv\Scripts\activate     # On Windows

5. If you're behind a proxy, make sure to configure pip to use it:

   .. code-block:: bash

       pip install --proxy http://user:password@proxy.server:port intellirouter