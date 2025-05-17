import unittest
from unittest.mock import MagicMock, patch, mock_open
import os
import json
import tempfile

from intellirouter.config.settings import Configuration
from intellirouter.exceptions import ConfigurationError


class TestConfiguration(unittest.TestCase):
    """Test the configuration settings."""

    def setUp(self):
        """Set up the test environment."""
        # Save the original environment variables
        self.original_env = os.environ.copy()
        
        # Set up environment variables for testing
        os.environ["INTELLIROUTER_API_KEY"] = "env-api-key"
        os.environ["INTELLIROUTER_BASE_URL"] = "http://env-url.com"
        os.environ["INTELLIROUTER_TIMEOUT"] = "30"
        os.environ["INTELLIROUTER_MAX_RETRIES"] = "5"
        
    def tearDown(self):
        """Tear down the test environment."""
        # Restore the original environment variables
        os.environ.clear()
        os.environ.update(self.original_env)

    def test_configuration_with_explicit_values(self):
        """Test configuration with explicit values."""
        config = Configuration(
            api_key="explicit-api-key",
            base_url="http://explicit-url.com",
            timeout=10,
            max_retries=2
        )
        
        self.assertEqual(config.api_key, "explicit-api-key")
        self.assertEqual(config.base_url, "http://explicit-url.com")
        self.assertEqual(config.timeout, 10)
        self.assertEqual(config.max_retries, 2)

    def test_configuration_from_environment(self):
        """Test configuration from environment variables."""
        config = Configuration()
        
        self.assertEqual(config.api_key, "env-api-key")
        self.assertEqual(config.base_url, "http://env-url.com")
        self.assertEqual(config.timeout, 30)
        self.assertEqual(config.max_retries, 5)

    def test_configuration_with_defaults(self):
        """Test configuration with defaults."""
        # Remove environment variables
        os.environ.pop("INTELLIROUTER_API_KEY")
        os.environ.pop("INTELLIROUTER_BASE_URL")
        os.environ.pop("INTELLIROUTER_TIMEOUT")
        os.environ.pop("INTELLIROUTER_MAX_RETRIES")
        
        config = Configuration(api_key="test-api-key")
        
        self.assertEqual(config.api_key, "test-api-key")
        self.assertEqual(config.base_url, "http://localhost:8000")
        self.assertEqual(config.timeout, 60)
        self.assertEqual(config.max_retries, 3)

    @patch("os.path.exists")
    @patch("builtins.open", new_callable=mock_open, read_data='{"api_key": "file-api-key", "base_url": "http://file-url.com", "timeout": 20, "max_retries": 4}')
    def test_configuration_from_file(self, mock_file, mock_exists):
        """Test configuration from file."""
        # Remove environment variables
        os.environ.pop("INTELLIROUTER_API_KEY")
        os.environ.pop("INTELLIROUTER_BASE_URL")
        os.environ.pop("INTELLIROUTER_TIMEOUT")
        os.environ.pop("INTELLIROUTER_MAX_RETRIES")
        
        # Set up the config file path
        os.environ["INTELLIROUTER_CONFIG_FILE"] = "/path/to/config.json"
        
        # Mock the file existence
        mock_exists.return_value = True
        
        config = Configuration()
        
        self.assertEqual(config.api_key, "file-api-key")
        self.assertEqual(config.base_url, "http://file-url.com")
        self.assertEqual(config.timeout, 20)
        self.assertEqual(config.max_retries, 4)

    def test_configuration_precedence(self):
        """Test configuration precedence."""
        # Set up a config file
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as f:
            json.dump({
                "api_key": "file-api-key",
                "base_url": "http://file-url.com",
                "timeout": 20,
                "max_retries": 4
            }, f)
            config_file = f.name
        
        try:
            # Set the config file path
            os.environ["INTELLIROUTER_CONFIG_FILE"] = config_file
            
            # Test precedence: explicit > environment > file > default
            config = Configuration(
                api_key="explicit-api-key",
                base_url="http://explicit-url.com"
            )
            
            self.assertEqual(config.api_key, "explicit-api-key")  # Explicit
            self.assertEqual(config.base_url, "http://explicit-url.com")  # Explicit
            self.assertEqual(config.timeout, 30)  # Environment
            self.assertEqual(config.max_retries, 5)  # Environment
            
            # Remove environment variables for timeout and max_retries
            os.environ.pop("INTELLIROUTER_TIMEOUT")
            os.environ.pop("INTELLIROUTER_MAX_RETRIES")
            
            config = Configuration(
                api_key="explicit-api-key",
                base_url="http://explicit-url.com"
            )
            
            self.assertEqual(config.api_key, "explicit-api-key")  # Explicit
            self.assertEqual(config.base_url, "http://explicit-url.com")  # Explicit
            self.assertEqual(config.timeout, 20)  # File
            self.assertEqual(config.max_retries, 4)  # File
            
            # Remove the config file path
            os.environ.pop("INTELLIROUTER_CONFIG_FILE")
            
            config = Configuration(
                api_key="explicit-api-key",
                base_url="http://explicit-url.com"
            )
            
            self.assertEqual(config.api_key, "explicit-api-key")  # Explicit
            self.assertEqual(config.base_url, "http://explicit-url.com")  # Explicit
            self.assertEqual(config.timeout, 60)  # Default
            self.assertEqual(config.max_retries, 3)  # Default
        finally:
            # Clean up the temporary file
            os.unlink(config_file)

    def test_configuration_without_api_key(self):
        """Test configuration without API key."""
        # Remove the API key from the environment
        os.environ.pop("INTELLIROUTER_API_KEY")
        
        # Configuration should not raise an error, but the API key will be None
        config = Configuration()
        self.assertIsNone(config.api_key)


if __name__ == "__main__":
    unittest.main()