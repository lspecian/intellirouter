from typing import Dict, List, Any, Optional, Union, Literal
from pydantic import BaseModel, Field

class ChainStep(BaseModel):
    """
    A step in a chain.
    
    Args:
        id: The ID of the step.
        type: The type of the step.
        name: The name of the step.
        description: A description of the step.
        inputs: The inputs for the step.
        outputs: The outputs for the step.
        config: Additional configuration for the step.
    """
    id: str
    type: str
    name: Optional[str] = None
    description: Optional[str] = None
    inputs: Dict[str, Any] = Field(default_factory=dict)
    outputs: Dict[str, str] = Field(default_factory=dict)
    config: Dict[str, Any] = Field(default_factory=dict)

class ChainDependency(BaseModel):
    """
    A dependency between steps in a chain.
    
    Args:
        dependent_step: The ID of the dependent step.
        required_step: The ID of the required step.
        type: The type of dependency.
    """
    dependent_step: str
    required_step: str
    type: Literal["simple", "conditional"] = "simple"
    condition: Optional[Dict[str, Any]] = None

class Chain(BaseModel):
    """
    A chain of steps.
    
    Args:
        id: The ID of the chain.
        name: The name of the chain.
        description: A description of the chain.
        steps: The steps in the chain.
        dependencies: The dependencies between steps.
        config: Additional configuration for the chain.
    """
    id: str
    name: Optional[str] = None
    description: Optional[str] = None
    steps: Dict[str, ChainStep] = Field(default_factory=dict)
    dependencies: List[ChainDependency] = Field(default_factory=list)
    config: Dict[str, Any] = Field(default_factory=dict)

class ChainExecutionStepResult(BaseModel):
    """
    The result of executing a step in a chain.
    
    Args:
        step_id: The ID of the step.
        outputs: The outputs of the step.
        error: An error message if the step failed.
        execution_time: The time it took to execute the step.
    """
    step_id: str
    outputs: Dict[str, Any] = Field(default_factory=dict)
    error: Optional[str] = None
    execution_time: Optional[float] = None

class ChainExecution(BaseModel):
    """
    The result of executing a chain.
    
    Args:
        chain_id: The ID of the chain.
        status: The status of the execution.
        step_results: The results of each step.
        outputs: The outputs of the chain.
        error: An error message if the chain failed.
        execution_time: The time it took to execute the chain.
    """
    chain_id: str
    status: Literal["running", "completed", "failed"]
    step_results: Dict[str, ChainExecutionStepResult] = Field(default_factory=dict)
    outputs: Dict[str, Any] = Field(default_factory=dict)
    error: Optional[str] = None
    execution_time: Optional[float] = None

class ChainExecutionEvent(BaseModel):
    """
    An event in a streaming chain execution.
    
    Args:
        event_type: The type of event.
        chain_id: The ID of the chain.
        step_id: The ID of the step.
        data: The data associated with the event.
    """
    event_type: Literal["step_started", "step_completed", "step_failed", "chain_completed", "chain_failed"]
    chain_id: str
    step_id: Optional[str] = None
    data: Dict[str, Any] = Field(default_factory=dict)