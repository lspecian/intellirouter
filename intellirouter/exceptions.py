class IntelliRouterError(Exception):
    """Base class for all IntelliRouter exceptions."""
    pass

class APIError(IntelliRouterError):
    """Exception raised when the API returns an error."""
    
    def __init__(self, message: str, status_code: int = None):
        self.status_code = status_code
        super().__init__(message)

class AuthenticationError(APIError):
    """Exception raised when authentication fails."""
    pass

class RateLimitError(APIError):
    """Exception raised when the rate limit is exceeded."""
    pass

class ServerError(APIError):
    """Exception raised when the server returns an error."""
    pass

class ValidationError(IntelliRouterError):
    """Exception raised when validation fails."""
    pass

class ConfigurationError(IntelliRouterError):
    """Exception raised when there is a configuration error."""
    pass