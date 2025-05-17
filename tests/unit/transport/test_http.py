import unittest
from unittest.mock import MagicMock, patch
import json
import requests
import aiohttp
from aiohttp.client_reqrep import ClientResponse
import asyncio

from intellirouter.transport.http import HTTPTransport
from intellirouter.config.settings import Configuration
from intellirouter.exceptions import (
    APIError,
    AuthenticationError,
    RateLimitError,
    ServerError,
    ValidationError,
)


class TestHTTPTransport(unittest.TestCase):
    """Test the HTTP transport layer."""

    def setUp(self):
        """Set up the test environment."""
        self.config = Configuration(
            api_key="test-api-key",
            base_url="http://test-url.com",
        )
        self.transport = HTTPTransport(self.config)
        
        # Mock response for successful request
        self.mock_response = MagicMock()
        self.mock_response.status_code = 200
        self.mock_response.json.return_value = {"result": "success"}
        
        # Mock response for streaming request
        self.mock_stream_response = MagicMock()
        self.mock_stream_response.status_code = 200
        self.mock_stream_response.iter_lines.return_value = [
            b'data: {"chunk": 1}',
            b'data: {"chunk": 2}',
            b'data: [DONE]'
        ]

    @patch("requests.request")
    def test_request(self, mock_request):
        """Test the request method."""
        mock_request.return_value = self.mock_response
        
        result = self.transport.request(
            method="POST",
            path="/test",
            data={"test": "data"},
            params={"param": "value"}
        )
        
        # Check that requests.request was called correctly
        mock_request.assert_called_once_with(
            method="POST",
            url="http://test-url.com/test",
            json={"test": "data"},
            params={"param": "value"},
            headers={
                "Authorization": "Bearer test-api-key",
                "Content-Type": "application/json",
            },
            timeout=60,
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(result, {"result": "success"})

    @patch("requests.request")
    def test_request_with_authentication_error(self, mock_request):
        """Test the request method with an authentication error."""
        mock_response = MagicMock()
        mock_response.status_code = 401
        mock_response.json.return_value = {"error": "Invalid API key"}
        mock_request.return_value = mock_response
        
        with self.assertRaises(AuthenticationError):
            self.transport.request(
                method="POST",
                path="/test"
            )

    @patch("requests.request")
    def test_request_with_rate_limit_error(self, mock_request):
        """Test the request method with a rate limit error."""
        mock_response = MagicMock()
        mock_response.status_code = 429
        mock_response.json.return_value = {"error": "Rate limit exceeded"}
        mock_request.return_value = mock_response
        
        with self.assertRaises(RateLimitError):
            self.transport.request(
                method="POST",
                path="/test"
            )

    @patch("requests.request")
    def test_request_with_server_error(self, mock_request):
        """Test the request method with a server error."""
        mock_response = MagicMock()
        mock_response.status_code = 500
        mock_response.json.return_value = {"error": "Server error"}
        mock_request.return_value = mock_response
        
        with self.assertRaises(ServerError):
            self.transport.request(
                method="POST",
                path="/test"
            )

    @patch("requests.request")
    def test_request_with_validation_error(self, mock_request):
        """Test the request method with a validation error."""
        mock_response = MagicMock()
        mock_response.status_code = 400
        mock_response.json.return_value = {"error": "Validation error"}
        mock_request.return_value = mock_response
        
        with self.assertRaises(ValidationError):
            self.transport.request(
                method="POST",
                path="/test"
            )

    @patch("requests.request")
    def test_request_with_api_error(self, mock_request):
        """Test the request method with an API error."""
        mock_response = MagicMock()
        mock_response.status_code = 404
        mock_response.json.return_value = {"error": "Not found"}
        mock_request.return_value = mock_response
        
        with self.assertRaises(APIError):
            self.transport.request(
                method="POST",
                path="/test"
            )

    @patch("requests.request")
    def test_stream(self, mock_request):
        """Test the stream method."""
        mock_request.return_value = self.mock_stream_response
        
        chunks = list(self.transport.stream(
            method="POST",
            path="/test",
            data={"test": "data"},
            params={"param": "value"}
        ))
        
        # Check that requests.request was called correctly
        mock_request.assert_called_once_with(
            method="POST",
            url="http://test-url.com/test",
            json={"test": "data"},
            params={"param": "value"},
            headers={
                "Authorization": "Bearer test-api-key",
                "Content-Type": "application/json",
                "Accept": "text/event-stream",
            },
            timeout=60,
            stream=True,
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(len(chunks), 2)  # [DONE] is filtered out
        self.assertEqual(chunks[0], {"chunk": 1})
        self.assertEqual(chunks[1], {"chunk": 2})

    @patch("aiohttp.ClientSession.request")
    async def test_arequest(self, mock_request):
        """Test the arequest method."""
        # Mock the response
        mock_response = MagicMock(spec=ClientResponse)
        mock_response.status = 200
        mock_response.json = MagicMock(return_value=asyncio.Future())
        mock_response.json.return_value.set_result({"result": "success"})
        mock_request.return_value.__aenter__.return_value = mock_response
        
        result = await self.transport.arequest(
            method="POST",
            path="/test",
            data={"test": "data"},
            params={"param": "value"}
        )
        
        # Check that aiohttp.ClientSession.request was called correctly
        mock_request.assert_called_once()
        call_args = mock_request.call_args[1]
        self.assertEqual(call_args["method"], "POST")
        self.assertEqual(call_args["url"], "http://test-url.com/test")
        self.assertEqual(call_args["json"], {"test": "data"})
        self.assertEqual(call_args["params"], {"param": "value"})
        self.assertEqual(call_args["headers"]["Authorization"], "Bearer test-api-key")
        
        # Check that the response was parsed correctly
        self.assertEqual(result, {"result": "success"})

    @patch("aiohttp.ClientSession.request")
    async def test_astream(self, mock_request):
        """Test the astream method."""
        # Mock the response
        mock_response = MagicMock(spec=ClientResponse)
        mock_response.status = 200
        
        # Mock the content attribute to return an async iterator
        async def mock_content_iterator():
            yield b'data: {"chunk": 1}\n\n'
            yield b'data: {"chunk": 2}\n\n'
            yield b'data: [DONE]\n\n'
        
        mock_response.content.iter_any = mock_content_iterator
        mock_request.return_value.__aenter__.return_value = mock_response
        
        chunks = []
        async for chunk in self.transport.astream(
            method="POST",
            path="/test",
            data={"test": "data"},
            params={"param": "value"}
        ):
            chunks.append(chunk)
        
        # Check that aiohttp.ClientSession.request was called correctly
        mock_request.assert_called_once()
        call_args = mock_request.call_args[1]
        self.assertEqual(call_args["method"], "POST")
        self.assertEqual(call_args["url"], "http://test-url.com/test")
        self.assertEqual(call_args["json"], {"test": "data"})
        self.assertEqual(call_args["params"], {"param": "value"})
        self.assertEqual(call_args["headers"]["Authorization"], "Bearer test-api-key")
        
        # Check that the response was parsed correctly
        self.assertEqual(len(chunks), 2)  # [DONE] is filtered out
        self.assertEqual(chunks[0], {"chunk": 1})
        self.assertEqual(chunks[1], {"chunk": 2})


if __name__ == "__main__":
    unittest.main()