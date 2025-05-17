"""
IntelliRouter Python SDK

A Python SDK for interacting with the IntelliRouter API.
"""

from .client import IntelliRouter
from .exceptions import (
    IntelliRouterError,
    APIError,
    AuthenticationError,
    RateLimitError,
    ServerError,
    ValidationError,
    ConfigurationError,
)
from .types import (
    Role,
    JSONDict,
    JSONList,
    JSONValue,
    Model,
    ModelList,
)
from .chat import (
    ChatMessage,
    ChatCompletion,
    ChatCompletionChoice,
    ChatCompletionChunk,
    ChatCompletionChunkChoice,
    ChatCompletionChunkDelta,
)
from .chains import (
    Chain,
    ChainStep,
    ChainDependency,
    ChainExecution,
    ChainExecutionEvent,
    ChainExecutionStepResult,
)

__version__ = "0.1.0"

__all__ = [
    "IntelliRouter",
    "IntelliRouterError",
    "APIError",
    "AuthenticationError",
    "RateLimitError",
    "ServerError",
    "ValidationError",
    "ConfigurationError",
    "Role",
    "JSONDict",
    "JSONList",
    "JSONValue",
    "Model",
    "ModelList",
    "ChatMessage",
    "ChatCompletion",
    "ChatCompletionChoice",
    "ChatCompletionChunk",
    "ChatCompletionChunkChoice",
    "ChatCompletionChunkDelta",
    "Chain",
    "ChainStep",
    "ChainDependency",
    "ChainExecution",
    "ChainExecutionEvent",
    "ChainExecutionStepResult",
]