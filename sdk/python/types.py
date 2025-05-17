from typing import Dict, List, Any, Optional, Union, Literal

# Role types for chat messages
Role = Literal["system", "user", "assistant", "function", "tool"]

# Common types for API requests and responses
JSONDict = Dict[str, Any]
JSONList = List[Any]
JSONValue = Union[str, int, float, bool, None, JSONDict, JSONList]

# Chat message types
ChatMessage = Dict[str, Any]
ChatCompletion = Dict[str, Any]
ChatCompletionChunk = Dict[str, Any]

# Model types
Model = Dict[str, Any]
ModelList = List[Model]