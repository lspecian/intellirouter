import unittest

from intellirouter.exceptions import (
    IntelliRouterError,
    APIError,
    AuthenticationError,
    RateLimitError,
    ServerError,
    ValidationError,
    ConfigurationError,
)


class TestExceptions(unittest.TestCase):
    """Test the exception hierarchy."""

    def test_intellirouter_error(self):
        """Test the IntelliRouterError class."""
        error = IntelliRouterError("Test error")
        self.assertEqual(str(error), "Test error")
        self.assertIsInstance(error, Exception)

    def test_api_error(self):
        """Test the APIError class."""
        error = APIError("API error", status_code=400, response={"error": "Bad request"})
        self.assertEqual(str(error), "API error")
        self.assertEqual(error.status_code, 400)
        self.assertEqual(error.response, {"error": "Bad request"})
        self.assertIsInstance(error, IntelliRouterError)

    def test_authentication_error(self):
        """Test the AuthenticationError class."""
        error = AuthenticationError("Authentication failed", status_code=401, response={"error": "Unauthorized"})
        self.assertEqual(str(error), "Authentication failed")
        self.assertEqual(error.status_code, 401)
        self.assertEqual(error.response, {"error": "Unauthorized"})
        self.assertIsInstance(error, APIError)
        self.assertIsInstance(error, IntelliRouterError)

    def test_rate_limit_error(self):
        """Test the RateLimitError class."""
        error = RateLimitError("Rate limit exceeded", status_code=429, response={"error": "Too many requests"})
        self.assertEqual(str(error), "Rate limit exceeded")
        self.assertEqual(error.status_code, 429)
        self.assertEqual(error.response, {"error": "Too many requests"})
        self.assertIsInstance(error, APIError)
        self.assertIsInstance(error, IntelliRouterError)

    def test_server_error(self):
        """Test the ServerError class."""
        error = ServerError("Server error", status_code=500, response={"error": "Internal server error"})
        self.assertEqual(str(error), "Server error")
        self.assertEqual(error.status_code, 500)
        self.assertEqual(error.response, {"error": "Internal server error"})
        self.assertIsInstance(error, APIError)
        self.assertIsInstance(error, IntelliRouterError)

    def test_validation_error(self):
        """Test the ValidationError class."""
        error = ValidationError("Validation error", status_code=400, response={"error": "Invalid request"})
        self.assertEqual(str(error), "Validation error")
        self.assertEqual(error.status_code, 400)
        self.assertEqual(error.response, {"error": "Invalid request"})
        self.assertIsInstance(error, APIError)
        self.assertIsInstance(error, IntelliRouterError)

    def test_configuration_error(self):
        """Test the ConfigurationError class."""
        error = ConfigurationError("Configuration error")
        self.assertEqual(str(error), "Configuration error")
        self.assertIsInstance(error, IntelliRouterError)

    def test_api_error_with_minimal_args(self):
        """Test the APIError class with minimal arguments."""
        error = APIError("API error")
        self.assertEqual(str(error), "API error")
        self.assertIsNone(error.status_code)
        self.assertIsNone(error.response)
        self.assertIsInstance(error, IntelliRouterError)

    def test_api_error_with_status_code_only(self):
        """Test the APIError class with status code only."""
        error = APIError("API error", status_code=400)
        self.assertEqual(str(error), "API error")
        self.assertEqual(error.status_code, 400)
        self.assertIsNone(error.response)
        self.assertIsInstance(error, IntelliRouterError)

    def test_api_error_with_response_only(self):
        """Test the APIError class with response only."""
        error = APIError("API error", response={"error": "Bad request"})
        self.assertEqual(str(error), "API error")
        self.assertIsNone(error.status_code)
        self.assertEqual(error.response, {"error": "Bad request"})
        self.assertIsInstance(error, IntelliRouterError)


if __name__ == "__main__":
    unittest.main()