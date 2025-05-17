from typing import Dict, Any, Optional, Union, AsyncIterator, Iterator
from abc import ABC, abstractmethod

class Transport(ABC):
    """
    Abstract base class for transport layers.
    
    The transport layer is responsible for making requests to the IntelliRouter API.
    """
    
    @abstractmethod
    def request(
        self,
        method: str,
        path: str,
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
        stream: bool = False,
    ) -> Dict[str, Any]:
        """
        Make a synchronous request to the IntelliRouter API.
        
        Args:
            method: HTTP method (GET, POST, etc.).
            path: API path.
            params: Query parameters.
            data: Request body.
            stream: Whether to stream the response.
        
        Returns:
            Response data.
        """
        pass
    
    @abstractmethod
    async def arequest(
        self,
        method: str,
        path: str,
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
        stream: bool = False,
    ) -> Dict[str, Any]:
        """
        Make an asynchronous request to the IntelliRouter API.
        
        Args:
            method: HTTP method (GET, POST, etc.).
            path: API path.
            params: Query parameters.
            data: Request body.
            stream: Whether to stream the response.
        
        Returns:
            Response data.
        """
        pass
    
    @abstractmethod
    def stream(
        self,
        method: str,
        path: str,
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
    ) -> Iterator[Dict[str, Any]]:
        """
        Make a streaming synchronous request to the IntelliRouter API.
        
        Args:
            method: HTTP method (GET, POST, etc.).
            path: API path.
            params: Query parameters.
            data: Request body.
        
        Returns:
            Iterator of response chunks.
        """
        pass
    
    @abstractmethod
    async def astream(
        self,
        method: str,
        path: str,
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
    ) -> AsyncIterator[Dict[str, Any]]:
        """
        Make a streaming asynchronous request to the IntelliRouter API.
        
        Args:
            method: HTTP method (GET, POST, etc.).
            path: API path.
            params: Query parameters.
            data: Request body.
        
        Returns:
            Async iterator of response chunks.
        """
        pass