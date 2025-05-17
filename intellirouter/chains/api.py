from typing import Dict, List, Any, Optional, Union, Iterator, AsyncIterator
import json
import time
from ..transport import Transport
from ..exceptions import ValidationError
from .models import (
    Chain,
    ChainStep,
    ChainDependency,
    ChainExecution,
    ChainExecutionEvent,
)

class ChainClient:
    """
    Client for the chain execution API.
    
    This client provides methods for creating and executing chains.
    """
    
    def __init__(self, transport: Transport):
        """
        Initialize the chain client.
        
        Args:
            transport: The transport layer to use for API requests.
        """
        self.transport = transport
    
    def create(
        self,
        name: str,
        description: Optional[str] = None,
        steps: Optional[Dict[str, Union[ChainStep, Dict[str, Any]]]] = None,
        dependencies: Optional[List[Union[ChainDependency, Dict[str, Any]]]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> Chain:
        """
        Create a new chain.
        
        Args:
            name: The name of the chain.
            description: A description of the chain.
            steps: The steps in the chain.
            dependencies: The dependencies between steps.
            config: Additional configuration for the chain.
        
        Returns:
            The created chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {
            "name": name,
        }
        
        if description is not None:
            data["description"] = description
        
        if steps is not None:
            formatted_steps = {}
            for step_id, step in steps.items():
                if isinstance(step, ChainStep):
                    formatted_steps[step_id] = step.dict(exclude_none=True)
                elif isinstance(step, dict):
                    # Validate the step
                    try:
                        formatted_steps[step_id] = ChainStep(**step).dict(exclude_none=True)
                    except Exception as e:
                        raise ValidationError(f"Invalid chain step: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain step type: {type(step)}")
            
            data["steps"] = formatted_steps
        
        if dependencies is not None:
            formatted_dependencies = []
            for dependency in dependencies:
                if isinstance(dependency, ChainDependency):
                    formatted_dependencies.append(dependency.dict(exclude_none=True))
                elif isinstance(dependency, dict):
                    # Validate the dependency
                    try:
                        formatted_dependencies.append(ChainDependency(**dependency).dict(exclude_none=True))
                    except Exception as e:
                        raise ValidationError(f"Invalid chain dependency: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain dependency type: {type(dependency)}")
            
            data["dependencies"] = formatted_dependencies
        
        if config is not None:
            data["config"] = config
        
        # Make the request
        response = self.transport.request(
            method="POST",
            path="/v1/chains",
            data=data,
        )
        
        # Parse the response
        try:
            return Chain(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain response: {str(e)}")
    
    def get(self, chain_id: str) -> Chain:
        """
        Get a chain by ID.
        
        Args:
            chain_id: The ID of the chain.
        
        Returns:
            The chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Make the request
        response = self.transport.request(
            method="GET",
            path=f"/v1/chains/{chain_id}",
        )
        
        # Parse the response
        try:
            return Chain(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain response: {str(e)}")
    
    def list(
        self,
        limit: Optional[int] = None,
        offset: Optional[int] = None,
    ) -> List[Chain]:
        """
        List chains.
        
        Args:
            limit: The maximum number of chains to return.
            offset: The offset to start from.
        
        Returns:
            A list of chains.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the query parameters
        params = {}
        
        if limit is not None:
            params["limit"] = limit
        
        if offset is not None:
            params["offset"] = offset
        
        # Make the request
        response = self.transport.request(
            method="GET",
            path="/v1/chains",
            params=params,
        )
        
        # Parse the response
        try:
            return [Chain(**chain) for chain in response["chains"]]
        except Exception as e:
            raise ValidationError(f"Invalid chain list response: {str(e)}")
    
    def update(
        self,
        chain_id: str,
        name: Optional[str] = None,
        description: Optional[str] = None,
        steps: Optional[Dict[str, Union[ChainStep, Dict[str, Any]]]] = None,
        dependencies: Optional[List[Union[ChainDependency, Dict[str, Any]]]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> Chain:
        """
        Update a chain.
        
        Args:
            chain_id: The ID of the chain.
            name: The name of the chain.
            description: A description of the chain.
            steps: The steps in the chain.
            dependencies: The dependencies between steps.
            config: Additional configuration for the chain.
        
        Returns:
            The updated chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {}
        
        if name is not None:
            data["name"] = name
        
        if description is not None:
            data["description"] = description
        
        if steps is not None:
            formatted_steps = {}
            for step_id, step in steps.items():
                if isinstance(step, ChainStep):
                    formatted_steps[step_id] = step.dict(exclude_none=True)
                elif isinstance(step, dict):
                    # Validate the step
                    try:
                        formatted_steps[step_id] = ChainStep(**step).dict(exclude_none=True)
                    except Exception as e:
                        raise ValidationError(f"Invalid chain step: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain step type: {type(step)}")
            
            data["steps"] = formatted_steps
        
        if dependencies is not None:
            formatted_dependencies = []
            for dependency in dependencies:
                if isinstance(dependency, ChainDependency):
                    formatted_dependencies.append(dependency.dict(exclude_none=True))
                elif isinstance(dependency, dict):
                    # Validate the dependency
                    try:
                        formatted_dependencies.append(ChainDependency(**dependency).dict(exclude_none=True))
                    except Exception as e:
                        raise ValidationError(f"Invalid chain dependency: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain dependency type: {type(dependency)}")
            
            data["dependencies"] = formatted_dependencies
        
        if config is not None:
            data["config"] = config
        
        # Make the request
        response = self.transport.request(
            method="PATCH",
            path=f"/v1/chains/{chain_id}",
            data=data,
        )
        
        # Parse the response
        try:
            return Chain(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain response: {str(e)}")
    
    def delete(self, chain_id: str) -> None:
        """
        Delete a chain.
        
        Args:
            chain_id: The ID of the chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Make the request
        self.transport.request(
            method="DELETE",
            path=f"/v1/chains/{chain_id}",
        )
    
    def run(
        self,
        chain_id: str,
        inputs: Dict[str, Any],
        config: Optional[Dict[str, Any]] = None,
        stream: bool = False,
    ) -> Union[ChainExecution, Iterator[ChainExecutionEvent]]:
        """
        Execute a chain.
        
        Args:
            chain_id: The ID of the chain.
            inputs: The inputs for the chain.
            config: Additional configuration for the execution.
            stream: Whether to stream the execution events.
        
        Returns:
            If stream is False, returns a ChainExecution object.
            If stream is True, returns an iterator of ChainExecutionEvent objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {
            "inputs": inputs,
            "stream": stream,
        }
        
        if config is not None:
            data["config"] = config
        
        if stream:
            return self.stream(chain_id, inputs, config)
        
        # Make the request
        response = self.transport.request(
            method="POST",
            path=f"/v1/chains/{chain_id}/run",
            data=data,
        )
        
        # Parse the response
        try:
            return ChainExecution(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain execution response: {str(e)}")
    
    def stream(
        self,
        chain_id: str,
        inputs: Dict[str, Any],
        config: Optional[Dict[str, Any]] = None,
    ) -> Iterator[ChainExecutionEvent]:
        """
        Execute a chain with streaming events.
        
        Args:
            chain_id: The ID of the chain.
            inputs: The inputs for the chain.
            config: Additional configuration for the execution.
        
        Returns:
            An iterator of ChainExecutionEvent objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {
            "inputs": inputs,
            "stream": True,
        }
        
        if config is not None:
            data["config"] = config
        
        # Make the streaming request
        for event in self.transport.stream(
            method="POST",
            path=f"/v1/chains/{chain_id}/run",
            data=data,
        ):
            try:
                yield ChainExecutionEvent(**event)
            except Exception as e:
                raise ValidationError(f"Invalid chain execution event: {str(e)}")
    
    async def aget(self, chain_id: str) -> Chain:
        """
        Get a chain by ID asynchronously.
        
        Args:
            chain_id: The ID of the chain.
        
        Returns:
            The chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Make the request
        response = await self.transport.arequest(
            method="GET",
            path=f"/v1/chains/{chain_id}",
        )
        
        # Parse the response
        try:
            return Chain(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain response: {str(e)}")
    
    async def alist(
        self,
        limit: Optional[int] = None,
        offset: Optional[int] = None,
    ) -> List[Chain]:
        """
        List chains asynchronously.
        
        Args:
            limit: The maximum number of chains to return.
            offset: The offset to start from.
        
        Returns:
            A list of chains.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the query parameters
        params = {}
        
        if limit is not None:
            params["limit"] = limit
        
        if offset is not None:
            params["offset"] = offset
        
        # Make the request
        response = await self.transport.arequest(
            method="GET",
            path="/v1/chains",
            params=params,
        )
        
        # Parse the response
        try:
            return [Chain(**chain) for chain in response["chains"]]
        except Exception as e:
            raise ValidationError(f"Invalid chain list response: {str(e)}")
    
    async def acreate(
        self,
        name: str,
        description: Optional[str] = None,
        steps: Optional[Dict[str, Union[ChainStep, Dict[str, Any]]]] = None,
        dependencies: Optional[List[Union[ChainDependency, Dict[str, Any]]]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> Chain:
        """
        Create a new chain asynchronously.
        
        Args:
            name: The name of the chain.
            description: A description of the chain.
            steps: The steps in the chain.
            dependencies: The dependencies between steps.
            config: Additional configuration for the chain.
        
        Returns:
            The created chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {
            "name": name,
        }
        
        if description is not None:
            data["description"] = description
        
        if steps is not None:
            formatted_steps = {}
            for step_id, step in steps.items():
                if isinstance(step, ChainStep):
                    formatted_steps[step_id] = step.dict(exclude_none=True)
                elif isinstance(step, dict):
                    # Validate the step
                    try:
                        formatted_steps[step_id] = ChainStep(**step).dict(exclude_none=True)
                    except Exception as e:
                        raise ValidationError(f"Invalid chain step: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain step type: {type(step)}")
            
            data["steps"] = formatted_steps
        
        if dependencies is not None:
            formatted_dependencies = []
            for dependency in dependencies:
                if isinstance(dependency, ChainDependency):
                    formatted_dependencies.append(dependency.dict(exclude_none=True))
                elif isinstance(dependency, dict):
                    # Validate the dependency
                    try:
                        formatted_dependencies.append(ChainDependency(**dependency).dict(exclude_none=True))
                    except Exception as e:
                        raise ValidationError(f"Invalid chain dependency: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain dependency type: {type(dependency)}")
            
            data["dependencies"] = formatted_dependencies
        
        if config is not None:
            data["config"] = config
        
        # Make the request
        response = await self.transport.arequest(
            method="POST",
            path="/v1/chains",
            data=data,
        )
        
        # Parse the response
        try:
            return Chain(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain response: {str(e)}")
    
    async def aupdate(
        self,
        chain_id: str,
        name: Optional[str] = None,
        description: Optional[str] = None,
        steps: Optional[Dict[str, Union[ChainStep, Dict[str, Any]]]] = None,
        dependencies: Optional[List[Union[ChainDependency, Dict[str, Any]]]] = None,
        config: Optional[Dict[str, Any]] = None,
    ) -> Chain:
        """
        Update a chain asynchronously.
        
        Args:
            chain_id: The ID of the chain.
            name: The name of the chain.
            description: A description of the chain.
            steps: The steps in the chain.
            dependencies: The dependencies between steps.
            config: Additional configuration for the chain.
        
        Returns:
            The updated chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {}
        
        if name is not None:
            data["name"] = name
        
        if description is not None:
            data["description"] = description
        
        if steps is not None:
            formatted_steps = {}
            for step_id, step in steps.items():
                if isinstance(step, ChainStep):
                    formatted_steps[step_id] = step.dict(exclude_none=True)
                elif isinstance(step, dict):
                    # Validate the step
                    try:
                        formatted_steps[step_id] = ChainStep(**step).dict(exclude_none=True)
                    except Exception as e:
                        raise ValidationError(f"Invalid chain step: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain step type: {type(step)}")
            
            data["steps"] = formatted_steps
        
        if dependencies is not None:
            formatted_dependencies = []
            for dependency in dependencies:
                if isinstance(dependency, ChainDependency):
                    formatted_dependencies.append(dependency.dict(exclude_none=True))
                elif isinstance(dependency, dict):
                    # Validate the dependency
                    try:
                        formatted_dependencies.append(ChainDependency(**dependency).dict(exclude_none=True))
                    except Exception as e:
                        raise ValidationError(f"Invalid chain dependency: {str(e)}")
                else:
                    raise ValidationError(f"Invalid chain dependency type: {type(dependency)}")
            
            data["dependencies"] = formatted_dependencies
        
        if config is not None:
            data["config"] = config
        
        # Make the request
        response = await self.transport.arequest(
            method="PATCH",
            path=f"/v1/chains/{chain_id}",
            data=data,
        )
        
        # Parse the response
        try:
            return Chain(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain response: {str(e)}")
    
    async def adelete(self, chain_id: str) -> None:
        """
        Delete a chain asynchronously.
        
        Args:
            chain_id: The ID of the chain.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Make the request
        await self.transport.arequest(
            method="DELETE",
            path=f"/v1/chains/{chain_id}",
        )
    
    async def arun(
        self,
        chain_id: str,
        inputs: Dict[str, Any],
        config: Optional[Dict[str, Any]] = None,
        stream: bool = False,
    ) -> Union[ChainExecution, AsyncIterator[ChainExecutionEvent]]:
        """
        Execute a chain asynchronously.
        
        Args:
            chain_id: The ID of the chain.
            inputs: The inputs for the chain.
            config: Additional configuration for the execution.
            stream: Whether to stream the execution events.
        
        Returns:
            If stream is False, returns a ChainExecution object.
            If stream is True, returns an async iterator of ChainExecutionEvent objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {
            "inputs": inputs,
            "stream": stream,
        }
        
        if config is not None:
            data["config"] = config
        
        if stream:
            return self.astream(chain_id, inputs, config)
        
        # Make the request
        response = await self.transport.arequest(
            method="POST",
            path=f"/v1/chains/{chain_id}/run",
            data=data,
        )
        
        # Parse the response
        try:
            return ChainExecution(**response)
        except Exception as e:
            raise ValidationError(f"Invalid chain execution response: {str(e)}")
    
    async def astream(
        self,
        chain_id: str,
        inputs: Dict[str, Any],
        config: Optional[Dict[str, Any]] = None,
    ) -> AsyncIterator[ChainExecutionEvent]:
        """
        Execute a chain with streaming events asynchronously.
        
        Args:
            chain_id: The ID of the chain.
            inputs: The inputs for the chain.
            config: Additional configuration for the execution.
        
        Returns:
            An async iterator of ChainExecutionEvent objects.
        
        Raises:
            ValidationError: If the request is invalid.
            APIError: If the API returns an error.
            AuthenticationError: If authentication fails.
            RateLimitError: If the rate limit is exceeded.
            ServerError: If the server returns an error.
        """
        # Prepare the request data
        data = {
            "inputs": inputs,
            "stream": True,
        }
        
        if config is not None:
            data["config"] = config
        
        # Make the streaming request
        async for event in self.transport.astream(
            method="POST",
            path=f"/v1/chains/{chain_id}/run",
            data=data,
        ):
            try:
                yield ChainExecutionEvent(**event)
            except Exception as e:
                raise ValidationError(f"Invalid chain execution event: {str(e)}")