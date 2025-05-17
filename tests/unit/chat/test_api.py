import unittest
from unittest.mock import MagicMock, patch
import json

from intellirouter.chat.api import ChatClient
from intellirouter.chat.models import ChatMessage, ChatCompletion, ChatCompletionChunk
from intellirouter.exceptions import ValidationError


class TestChatAPI(unittest.TestCase):
    """Test the chat API."""

    def setUp(self):
        """Set up the test environment."""
        self.transport = MagicMock()
        self.client = ChatClient(self.transport)
        
        # Mock response for chat completion
        self.mock_completion_response = {
            "id": "test-id",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-3.5-turbo",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello, how can I help you?"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 10,
                "total_tokens": 20
            }
        }
        
        # Mock response for streaming chat completion
        self.mock_chunk_response = {
            "id": "test-id",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-3.5-turbo",
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "content": "Hello"
                    },
                    "finish_reason": None
                }
            ]
        }
        
        # Set up the transport mock
        self.transport.request.return_value = self.mock_completion_response
        self.transport.stream.return_value = [self.mock_chunk_response]

    def test_create(self):
        """Test the create method."""
        completion = self.client.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello"}
            ]
        )
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_once_with(
            method="POST",
            path="/v1/chat/completions",
            data={
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": "Hello"}],
                "stream": False
            }
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(completion.id, "test-id")
        self.assertEqual(completion.choices[0].message.content, "Hello, how can I help you?")

    def test_create_with_options(self):
        """Test the create method with options."""
        completion = self.client.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello"}
            ],
            temperature=0.5,
            max_tokens=100,
            stop=["END"],
            presence_penalty=0.5,
            frequency_penalty=0.5,
            logit_bias={"50256": -100},
            user="test-user"
        )
        
        # Check that the transport was called with the correct options
        self.transport.request.assert_called_once()
        call_args = self.transport.request.call_args[1]
        self.assertEqual(call_args["data"]["temperature"], 0.5)
        self.assertEqual(call_args["data"]["max_tokens"], 100)
        self.assertEqual(call_args["data"]["stop"], ["END"])
        self.assertEqual(call_args["data"]["presence_penalty"], 0.5)
        self.assertEqual(call_args["data"]["frequency_penalty"], 0.5)
        self.assertEqual(call_args["data"]["logit_bias"], {"50256": -100})
        self.assertEqual(call_args["data"]["user"], "test-user")

    def test_create_with_invalid_message(self):
        """Test that the create method raises a ValidationError for invalid messages."""
        with self.assertRaises(ValidationError):
            self.client.create(
                model="gpt-3.5-turbo",
                messages=[
                    {"role": "invalid-role", "content": "Hello"}
                ]
            )

    def test_stream(self):
        """Test the stream method."""
        chunks = list(self.client.stream(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello"}
            ]
        ))
        
        # Check that the transport was called correctly
        self.transport.stream.assert_called_once_with(
            method="POST",
            path="/v1/chat/completions",
            data={
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": "Hello"}],
                "stream": True
            }
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(len(chunks), 1)
        self.assertEqual(chunks[0].choices[0].delta.content, "Hello")

    @patch("intellirouter.chat.api.ChatClient.stream")
    def test_create_with_stream(self, mock_stream):
        """Test the create method with stream=True."""
        mock_stream.return_value = [
            ChatCompletionChunk(**self.mock_chunk_response)
        ]
        
        chunks = list(self.client.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello"}
            ],
            stream=True
        ))
        
        # Check that the stream method was called
        mock_stream.assert_called_once_with(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": "Hello"}
            ],
            temperature=None,
            top_p=None,
            n=None,
            stop=None,
            max_tokens=None,
            presence_penalty=None,
            frequency_penalty=None,
            logit_bias=None,
            user=None
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(len(chunks), 1)
        self.assertEqual(chunks[0].choices[0].delta.content, "Hello")

    def test_format_messages(self):
        """Test the _format_messages method."""
        # Test with dict messages
        messages = [
            {"role": "user", "content": "Hello"}
        ]
        formatted_messages = self.client._format_messages(messages)
        self.assertEqual(formatted_messages, [{"role": "user", "content": "Hello"}])
        
        # Test with ChatMessage objects
        messages = [
            ChatMessage(role="user", content="Hello")
        ]
        formatted_messages = self.client._format_messages(messages)
        self.assertEqual(formatted_messages, [{"role": "user", "content": "Hello"}])
        
        # Test with invalid message type
        with self.assertRaises(ValidationError):
            self.client._format_messages(["invalid"])


if __name__ == "__main__":
    unittest.main()