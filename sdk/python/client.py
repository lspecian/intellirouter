from typing import Optional, Dict, Any
import os
from .config import Configuration
from .transport import Transport, HTTPTransport
from .exceptions import ConfigurationError

class IntelliRouter:
    """
    Main client for interacting with the IntelliRouter API.
    
    This client provides access to all IntelliRouter functionality, including
    chat completions, chain execution, and model management.
    
    Args:
        api_key: API key for authentication. If not provided, will be read from
            the INTELLIROUTER_API_KEY environment variable.
        base_url: Base URL for the IntelliRouter API. Defaults to http://localhost:8000.
        config: Optional configuration object. If not provided, a default
            configuration will be created.
        transport: Optional transport layer. If not provided, a default HTTP
            transport will be created.
    """
    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        config: Optional[Configuration] = None,
        transport: Optional[Transport] = None,
    ):
        self.config = config or Configuration(api_key=api_key, base_url=base_url)
        
        # Validate configuration
        if not self.config.api_key:
            raise ConfigurationError(
                "API key is required. Provide it as an argument, set the INTELLIROUTER_API_KEY "
                "environment variable, or include it in your configuration file."
            )
        
        self.transport = transport or HTTPTransport(self.config)
        
        # Initialize sub-clients
        self._chat = None
        self._chains = None
        self._models = None
    
    @property
    def chat(self):
        """
        Access the chat completions API.
        
        Returns:
            ChatClient: Client for chat completions.
        """
        if self._chat is None:
            from .chat import ChatClient
            self._chat = ChatClient(self.transport)
        return self._chat
    
    @property
    def chains(self):
        """
        Access the chain execution API.
        
        Returns:
            ChainClient: Client for chain execution.
        """
        if self._chains is None:
            from .chains import ChainClient
            self._chains = ChainClient(self.transport)
        return self._chains
    
    @property
    def models(self):
        """
        Access the model management API.
        
        Returns:
            ModelClient: Client for model management.
        """
        if self._models is None:
            from .models import ModelClient
            self._models = ModelClient(self.transport)
        return self._models