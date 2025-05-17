from typing import Dict, List, Any, Optional, Union, Iterator, AsyncIterator
import json
from ..transport import Transport
from ..types import Role
from ..exceptions import ValidationError
from .models import (
    ChatMessage,
    ChatCompletionRequest,
    ChatCompletion,
    ChatCompletionChunk,
)

class ChatClient:
    """
    Client for the chat completions API.
    
    This client provides methods for creating chat completions.
    """
    
    def __init__(self, transport: Transport):
        """
        Initialize the chat client.
        
        Args:
            transport: The transport layer to use for API requests.
        """
        self.transport = transport
    
    def create(
        self,
        model: str,
        messages: List[Union[Dict[str, Any], ChatMessage]],
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        n: Optional[int] = None,
        stop: Optional[Union[str, List[str]]] = None,
        max_tokens: Optional[int] = None,
        presence_penalty: Optional[float] = None,
        frequency_penalty: Optional[float] = None,
        logit_bias: Optional[Dict[str, float]] = None,
        user: Optional[str] = None,
        stream: bool = False,
    ) -> Union[ChatCompletion, Iterator[ChatCompletionChunk]]:
        """
        Create a chat completion.
        
        Args:
            model: The model to use for the completion.
            messages: The messages to generate a completion for.
            temperature: Controls randomness. Higher values (e.g., 0.8) make output more random, lower values (e.g., 0.2) make it more deterministic.
            top_p: Controls diversity via nucleus sampling. 0.1 means only tokens with the top 10% probability mass are considered.
            n: How many completions to generate for each prompt.
            stop: Up to 4 sequences where the API will stop generating further tokens.
            max_tokens: The maximum number of tokens to generate.
            presence_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far.
            frequency_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on their frequency in the text so far.
            logit_bias: Modify the likelihood of specified tokens appearing in the completion.
            user: A unique identifier representing your end-user.
            stream: Whether to stream the response.
        
        Returns:
            If stream is False, returns a ChatCompletion object.
            If stream is True, returns an iterator of ChatCompletionChunk objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Convert messages to the expected format
        formatted_messages = self._format_messages(messages)
        
        # Prepare the request data
        data = {
            "model": model,
            "messages": formatted_messages,
            "stream": stream,
        }
        
        # Add optional parameters
        if temperature is not None:
            data["temperature"] = temperature
        if top_p is not None:
            data["top_p"] = top_p
        if n is not None:
            data["n"] = n
        if stop is not None:
            data["stop"] = stop
        if max_tokens is not None:
            data["max_tokens"] = max_tokens
        if presence_penalty is not None:
            data["presence_penalty"] = presence_penalty
        if frequency_penalty is not None:
            data["frequency_penalty"] = frequency_penalty
        if logit_bias is not None:
            data["logit_bias"] = logit_bias
        if user is not None:
            data["user"] = user
        
        # Validate the request
        try:
            ChatCompletionRequest(**data)
        except Exception as e:
            raise ValidationError(f"Invalid chat completion request: {str(e)}")
        
        if stream:
            return self.stream(
                model=model,
                messages=messages,
                temperature=temperature,
                top_p=top_p,
                n=n,
                stop=stop,
                max_tokens=max_tokens,
                presence_penalty=presence_penalty,
                frequency_penalty=frequency_penalty,
                logit_bias=logit_bias,
                user=user,
            )
        
        # Make the request
        response = self.transport.request(
            method="POST",
            path="/v1/chat/completions",
            data=data,
        )
        
        # Parse the response
        try:
            return ChatCompletion(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chat completion response: {str(e)}")
    
    def stream(
        self,
        model: str,
        messages: List[Union[Dict[str, Any], ChatMessage]],
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        n: Optional[int] = None,
        stop: Optional[Union[str, List[str]]] = None,
        max_tokens: Optional[int] = None,
        presence_penalty: Optional[float] = None,
        frequency_penalty: Optional[float] = None,
        logit_bias: Optional[Dict[str, float]] = None,
        user: Optional[str] = None,
    ) -> Iterator[ChatCompletionChunk]:
        """
        Create a streaming chat completion.
        
        Args:
            model: The model to use for the completion.
            messages: The messages to generate a completion for.
            temperature: Controls randomness. Higher values (e.g., 0.8) make output more random, lower values (e.g., 0.2) make it more deterministic.
            top_p: Controls diversity via nucleus sampling. 0.1 means only tokens with the top 10% probability mass are considered.
            n: How many completions to generate for each prompt.
            stop: Up to 4 sequences where the API will stop generating further tokens.
            max_tokens: The maximum number of tokens to generate.
            presence_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far.
            frequency_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on their frequency in the text so far.
            logit_bias: Modify the likelihood of specified tokens appearing in the completion.
            user: A unique identifier representing your end-user.
        
        Returns:
            An iterator of ChatCompletionChunk objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Convert messages to the expected format
        formatted_messages = self._format_messages(messages)
        
        # Prepare the request data
        data = {
            "model": model,
            "messages": formatted_messages,
            "stream": True,
        }
        
        # Add optional parameters
        if temperature is not None:
            data["temperature"] = temperature
        if top_p is not None:
            data["top_p"] = top_p
        if n is not None:
            data["n"] = n
        if stop is not None:
            data["stop"] = stop
        if max_tokens is not None:
            data["max_tokens"] = max_tokens
        if presence_penalty is not None:
            data["presence_penalty"] = presence_penalty
        if frequency_penalty is not None:
            data["frequency_penalty"] = frequency_penalty
        if logit_bias is not None:
            data["logit_bias"] = logit_bias
        if user is not None:
            data["user"] = user
        
        # Validate the request
        try:
            ChatCompletionRequest(**data)
        except Exception as e:
            raise ValidationError(f"Invalid chat completion request: {str(e)}")
        
        # Make the streaming request
        for chunk in self.transport.stream(
            method="POST",
            path="/v1/chat/completions",
            data=data,
        ):
            try:
                yield ChatCompletionChunk(**chunk)
            except Exception as e:
                raise ValidationError(f"Invalid chat completion chunk: {str(e)}")
    
    async def acreate(
        self,
        model: str,
        messages: List[Union[Dict[str, Any], ChatMessage]],
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        n: Optional[int] = None,
        stop: Optional[Union[str, List[str]]] = None,
        max_tokens: Optional[int] = None,
        presence_penalty: Optional[float] = None,
        frequency_penalty: Optional[float] = None,
        logit_bias: Optional[Dict[str, float]] = None,
        user: Optional[str] = None,
        stream: bool = False,
    ) -> Union[ChatCompletion, AsyncIterator[ChatCompletionChunk]]:
        """
        Create a chat completion asynchronously.
        
        Args:
            model: The model to use for the completion.
            messages: The messages to generate a completion for.
            temperature: Controls randomness. Higher values (e.g., 0.8) make output more random, lower values (e.g., 0.2) make it more deterministic.
            top_p: Controls diversity via nucleus sampling. 0.1 means only tokens with the top 10% probability mass are considered.
            n: How many completions to generate for each prompt.
            stop: Up to 4 sequences where the API will stop generating further tokens.
            max_tokens: The maximum number of tokens to generate.
            presence_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far.
            frequency_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on their frequency in the text so far.
            logit_bias: Modify the likelihood of specified tokens appearing in the completion.
            user: A unique identifier representing your end-user.
            stream: Whether to stream the response.
        
        Returns:
            If stream is False, returns a ChatCompletion object.
            If stream is True, returns an async iterator of ChatCompletionChunk objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Convert messages to the expected format
        formatted_messages = self._format_messages(messages)
        
        # Prepare the request data
        data = {
            "model": model,
            "messages": formatted_messages,
            "stream": stream,
        }
        
        # Add optional parameters
        if temperature is not None:
            data["temperature"] = temperature
        if top_p is not None:
            data["top_p"] = top_p
        if n is not None:
            data["n"] = n
        if stop is not None:
            data["stop"] = stop
        if max_tokens is not None:
            data["max_tokens"] = max_tokens
        if presence_penalty is not None:
            data["presence_penalty"] = presence_penalty
        if frequency_penalty is not None:
            data["frequency_penalty"] = frequency_penalty
        if logit_bias is not None:
            data["logit_bias"] = logit_bias
        if user is not None:
            data["user"] = user
        
        # Validate the request
        try:
            ChatCompletionRequest(**data)
        except Exception as e:
            raise ValidationError(f"Invalid chat completion request: {str(e)}")
        
        if stream:
            return self.astream(
                model=model,
                messages=messages,
                temperature=temperature,
                top_p=top_p,
                n=n,
                stop=stop,
                max_tokens=max_tokens,
                presence_penalty=presence_penalty,
                frequency_penalty=frequency_penalty,
                logit_bias=logit_bias,
                user=user,
            )
        
        # Make the request
        response = await self.transport.arequest(
            method="POST",
            path="/v1/chat/completions",
            data=data,
        )
        
        # Parse the response
        try:
            return ChatCompletion(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chat completion response: {str(e)}")
    
    async def astream(
        self,
        model: str,
        messages: List[Union[Dict[str, Any], ChatMessage]],
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        n: Optional[int] = None,
        stop: Optional[Union[str, List[str]]] = None,
        max_tokens: Optional[int] = None,
        presence_penalty: Optional[float] = None,
        frequency_penalty: Optional[float] = None,
        logit_bias: Optional[Dict[str, float]] = None,
        user: Optional[str] = None,
    ) -> AsyncIterator[ChatCompletionChunk]:
        """
        Create a streaming chat completion asynchronously.
        
        Args:
            model: The model to use for the completion.
            messages: The messages to generate a completion for.
            temperature: Controls randomness. Higher values (e.g., 0.8) make output more random, lower values (e.g., 0.2) make it more deterministic.
            top_p: Controls diversity via nucleus sampling. 0.1 means only tokens with the top 10% probability mass are considered.
            n: How many completions to generate for each prompt.
            stop: Up to 4 sequences where the API will stop generating further tokens.
            max_tokens: The maximum number of tokens to generate.
            presence_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far.
            frequency_penalty: Number between -2.0 and 2.0. Positive values penalize new tokens based on their frequency in the text so far.
            logit_bias: Modify the likelihood of specified tokens appearing in the completion.
            user: A unique identifier representing your end-user.
        
        Returns:
            An async iterator of ChatCompletionChunk objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Convert messages to the expected format
        formatted_messages = self._format_messages(messages)
        
        # Prepare the request data
        data = {
            "model": model,
            "messages": formatted_messages,
            "stream": True,
        }
        
        # Add optional parameters
        if temperature is not None:
            data["temperature"] = temperature
        if top_p is not None:
            data["top_p"] = top_p
        if n is not None:
            data["n"] = n
        if stop is not None:
            data["stop"] = stop
        if max_tokens is not None:
            data["max_tokens"] = max_tokens
        if presence_penalty is not None:
            data["presence_penalty"] = presence_penalty
        if frequency_penalty is not None:
            data["frequency_penalty"] = frequency_penalty
        if logit_bias is not None:
            data["logit_bias"] = logit_bias
        if user is not None:
            data["user"] = user
        
        # Validate the request
        try:
            ChatCompletionRequest(**data)
        except Exception as e:
            raise ValidationError(f"Invalid chat completion request: {str(e)}")
        
        # Make the streaming request
        async for chunk in self.transport.astream(
            method="POST",
            path="/v1/chat/completions",
            data=data,
        ):
            try:
                yield ChatCompletionChunk(**chunk)
            except Exception as e:
                raise ValidationError(f"Invalid chat completion chunk: {str(e)}")
    
    def _format_messages(
        self,
        messages: List[Union[Dict[str, Any], ChatMessage]],
    ) -> List[Dict[str, Any]]:
        """
        Format messages for the API request.
        
        Args:
            messages: The messages to format.
        
        Returns:
            The formatted messages.
        
        Raises:
            ValidationError: If a message is invalid.
        """
        formatted_messages = []
        
        for message in messages:
            if isinstance(message, ChatMessage):
                formatted_message = message.dict(exclude_none=True)
            elif isinstance(message, dict):
                # Validate the message
                try:
                    formatted_message = ChatMessage(**message).dict(exclude_none=True)
                except Exception as e:
                    raise ValidationError(f"Invalid chat message: {str(e)}")
            else:
                raise ValidationError(f"Invalid chat message type: {type(message)}")
            
            formatted_messages.append(formatted_message)
        
        return formatted_messages