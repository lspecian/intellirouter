import unittest
from unittest.mock import MagicMock, patch
import os

from intellirouter import IntelliRouter
from intellirouter.transport import Transport
from intellirouter.exceptions import ConfigurationError


class TestClient(unittest.TestCase):
    """Test the IntelliRouter client."""

    def setUp(self):
        """Set up the test environment."""
        # Save the original environment variables
        self.original_env = os.environ.copy()
        
        # Set up environment variables for testing
        os.environ["INTELLIROUTER_API_KEY"] = "test-api-key"
        os.environ["INTELLIROUTER_BASE_URL"] = "http://test-url.com"
        
    def tearDown(self):
        """Tear down the test environment."""
        # Restore the original environment variables
        os.environ.clear()
        os.environ.update(self.original_env)

    def test_client_initialization(self):
        """Test that the client initializes correctly."""
        client = IntelliRouter(api_key="test-key")
        self.assertEqual(client.config.api_key, "test-key")
        self.assertEqual(client.config.base_url, "http://localhost:8000")

    def test_client_initialization_from_env(self):
        """Test that the client initializes correctly from environment variables."""
        client = IntelliRouter()
        self.assertEqual(client.config.api_key, "test-api-key")
        self.assertEqual(client.config.base_url, "http://test-url.com")

    def test_client_initialization_with_custom_transport(self):
        """Test that the client can be initialized with a custom transport."""
        transport = MagicMock(spec=Transport)
        client = IntelliRouter(api_key="test-key", transport=transport)
        self.assertEqual(client.transport, transport)

    def test_client_initialization_without_api_key(self):
        """Test that the client raises an error when no API key is provided."""
        # Remove the API key from the environment
        os.environ.pop("INTELLIROUTER_API_KEY")
        
        with self.assertRaises(ConfigurationError):
            IntelliRouter()

    def test_client_chat_property(self):
        """Test that the chat property returns a ChatClient."""
        client = IntelliRouter(api_key="test-key")
        self.assertIsNotNone(client.chat)
        self.assertEqual(client._chat, client.chat)  # Test caching

    def test_client_chains_property(self):
        """Test that the chains property returns a ChainClient."""
        client = IntelliRouter(api_key="test-key")
        self.assertIsNotNone(client.chains)
        self.assertEqual(client._chains, client.chains)  # Test caching

    def test_client_models_property(self):
        """Test that the models property returns a ModelClient."""
        client = IntelliRouter(api_key="test-key")
        self.assertIsNotNone(client.models)
        self.assertEqual(client._models, client.models)  # Test caching


if __name__ == "__main__":
    unittest.main()