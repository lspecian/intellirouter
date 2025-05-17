from typing import Dict, Any, Optional, Union, AsyncIterator, Iterator
import requests
import aiohttp
import json
import sseclient
import asyncio
from ..config import Configuration
from ..exceptions import APIError, AuthenticationError, RateLimitError, ServerError
from .base import Transport

class HTTPTransport(Transport):
    """
    HTTP transport layer for making requests to the IntelliRouter API.
    
    Args:
        config: Configuration object.
    """
    
    def __init__(self, config: Configuration):
        self.config = config
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {config.api_key}",
            "Content-Type": "application/json",
        })
    
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
        
        Raises:
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        url = f"{self.config.base_url}{path}"
        
        try:
            response = self.session.request(
                method=method,
                url=url,
                params=params,
                json=data,
                stream=stream,
                timeout=self.config.timeout,
            )
            
            if stream:
                return response
            
            if response.status_code >= 400:
                self._handle_error_response(response)
            
            return response.json()
        except requests.exceptions.RequestException as e:
            raise APIError(f"Request failed: {str(e)}")
    
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
        
        Raises:
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        url = f"{self.config.base_url}{path}"
        
        async with aiohttp.ClientSession() as session:
            headers = {
                "Authorization": f"Bearer {self.config.api_key}",
                "Content-Type": "application/json",
            }
            
            try:
                async with session.request(
                    method=method,
                    url=url,
                    params=params,
                    json=data,
                    headers=headers,
                    timeout=self.config.timeout,
                ) as response:
                    if stream:
                        return response
                    
                    if response.status >= 400:
                        await self._ahandle_error_response(response)
                    
                    return await response.json()
            except aiohttp.ClientError as e:
                raise APIError(f"Request failed: {str(e)}")
    
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
        
        Raises:
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        response = self.request(method, path, params, data, stream=True)
        
        if response.status_code >= 400:
            self._handle_error_response(response)
        
        client = sseclient.SSEClient(response)
        
        for event in client.events():
            if event.data == "[DONE]":
                break
            
            try:
                yield json.loads(event.data)
            except json.JSONDecodeError:
                raise APIError(f"Invalid JSON in stream: {event.data}")
    
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
        
        Raises:
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        response = await self.arequest(method, path, params, data, stream=True)
        
        if response.status >= 400:
            await self._ahandle_error_response(response)
        
        async for line in response.content:
            line = line.decode("utf-8").strip()
            
            if not line:
                continue
            
            if line.startswith("data:"):
                data = line[5:].strip()
                
                if data == "[DONE]":
                    break
                
                try:
                    yield json.loads(data)
                except json.JSONDecodeError:
                    raise APIError(f"Invalid JSON in stream: {data}")
    
    def _handle_error_response(self, response: requests.Response) -> None:
        """
        Handle an error response from the API.
        
        Args:
            response: Response object.
        
        Raises:
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
            APIError: For other API errors.
        """
        try:
            error_data = response.json()
            error_message = error_data.get("error", {}).get("message", "Unknown error")
        except (json.JSONDecodeError, KeyError):
            error_message = response.text or "Unknown error"
        
        if response.status_code == 401:
            raise AuthenticationError(f"Authentication failed: {error_message}")
        elif response.status_code == 429:
            raise RateLimitError(f"Rate limit exceeded: {error_message}")
        elif response.status_code >= 500:
            raise ServerError(f"Server error: {error_message}")
        else:
            raise APIError(f"API error: {error_message}", response.status_code)
    
    async def _ahandle_error_response(self, response: aiohttp.ClientResponse) -> None:
        """
        Handle an error response from the API asynchronously.
        
        Args:
            response: Response object.
        
        Raises:
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
            APIError: For other API errors.
        """
        try:
            error_data = await response.json()
            error_message = error_data.get("error", {}).get("message", "Unknown error")
        except (json.JSONDecodeError, KeyError):
            error_message = await response.text() or "Unknown error"
        
        if response.status == 401:
            raise AuthenticationError(f"Authentication failed: {error_message}")
        elif response.status == 429:
            raise RateLimitError(f"Rate limit exceeded: {error_message}")
        elif response.status >= 500:
            raise ServerError(f"Server error: {error_message}")
        else:
            raise APIError(f"API error: {error_message}", response.status)