import unittest
from unittest.mock import MagicMock, patch
import json

from intellirouter.chains.api import ChainClient
from intellirouter.chains.models import Chain, ChainStep, ChainDependency, ChainExecutionResult, ChainExecutionEvent
from intellirouter.exceptions import ValidationError


class TestChainsAPI(unittest.TestCase):
    """Test the chains API."""

    def setUp(self):
        """Set up the test environment."""
        self.transport = MagicMock()
        self.client = ChainClient(self.transport)
        
        # Mock response for chain creation
        self.mock_chain_response = {
            "id": "test-chain-id",
            "name": "Test Chain",
            "description": "A test chain",
            "steps": {
                "step1": {
                    "id": "step1",
                    "type": "llm",
                    "name": "Test Step",
                    "description": "A test step",
                    "inputs": {"prompt": "string"},
                    "outputs": {"response": "string"},
                    "config": {}
                }
            },
            "dependencies": []
        }
        
        # Mock response for chain execution
        self.mock_execution_response = {
            "chain_id": "test-chain-id",
            "status": "completed",
            "step_results": {
                "step1": {
                    "status": "completed",
                    "outputs": {"response": "Hello, world!"}
                }
            },
            "outputs": {"step1.response": "Hello, world!"},
            "execution_time": 1.0
        }
        
        # Mock response for streaming chain execution
        self.mock_event_response = {
            "event_type": "step_completed",
            "step_id": "step1",
            "data": {
                "status": "completed",
                "outputs": {"response": "Hello, world!"}
            }
        }
        
        # Set up the transport mock
        self.transport.request.side_effect = [
            self.mock_chain_response,  # For create
            self.mock_chain_response,  # For get
            [self.mock_chain_response],  # For list
            self.mock_chain_response,  # For update
            self.mock_execution_response,  # For run
            None,  # For delete
        ]
        self.transport.stream.return_value = [self.mock_event_response]

    def test_create(self):
        """Test the create method."""
        chain = self.client.create(
            name="Test Chain",
            description="A test chain",
            steps={
                "step1": {
                    "id": "step1",
                    "type": "llm",
                    "name": "Test Step",
                    "description": "A test step",
                    "inputs": {"prompt": "string"},
                    "outputs": {"response": "string"}
                }
            }
        )
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_with(
            method="POST",
            path="/v1/chains",
            data={
                "name": "Test Chain",
                "description": "A test chain",
                "steps": {
                    "step1": {
                        "id": "step1",
                        "type": "llm",
                        "name": "Test Step",
                        "description": "A test step",
                        "inputs": {"prompt": "string"},
                        "outputs": {"response": "string"}
                    }
                },
                "dependencies": []
            }
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(chain.id, "test-chain-id")
        self.assertEqual(chain.name, "Test Chain")
        self.assertEqual(len(chain.steps), 1)
        self.assertEqual(chain.steps["step1"].id, "step1")

    def test_get(self):
        """Test the get method."""
        chain = self.client.get("test-chain-id")
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_with(
            method="GET",
            path="/v1/chains/test-chain-id"
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(chain.id, "test-chain-id")
        self.assertEqual(chain.name, "Test Chain")

    def test_list(self):
        """Test the list method."""
        chains = self.client.list()
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_with(
            method="GET",
            path="/v1/chains",
            params={"limit": 100, "offset": 0}
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(len(chains), 1)
        self.assertEqual(chains[0].id, "test-chain-id")

    def test_update(self):
        """Test the update method."""
        chain = self.client.update(
            chain_id="test-chain-id",
            description="Updated description"
        )
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_with(
            method="PATCH",
            path="/v1/chains/test-chain-id",
            data={"description": "Updated description"}
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(chain.id, "test-chain-id")
        self.assertEqual(chain.description, "A test chain")  # Using mock response

    def test_run(self):
        """Test the run method."""
        result = self.client.run(
            chain_id="test-chain-id",
            inputs={"prompt": "Hello"}
        )
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_with(
            method="POST",
            path="/v1/chains/test-chain-id/run",
            data={"inputs": {"prompt": "Hello"}, "stream": False}
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(result.chain_id, "test-chain-id")
        self.assertEqual(result.status, "completed")
        self.assertEqual(result.outputs["step1.response"], "Hello, world!")

    def test_stream(self):
        """Test the stream method."""
        events = list(self.client.stream(
            chain_id="test-chain-id",
            inputs={"prompt": "Hello"}
        ))
        
        # Check that the transport was called correctly
        self.transport.stream.assert_called_once_with(
            method="POST",
            path="/v1/chains/test-chain-id/run",
            data={"inputs": {"prompt": "Hello"}, "stream": True}
        )
        
        # Check that the response was parsed correctly
        self.assertEqual(len(events), 1)
        self.assertEqual(events[0].event_type, "step_completed")
        self.assertEqual(events[0].step_id, "step1")
        self.assertEqual(events[0].data["outputs"]["response"], "Hello, world!")

    def test_delete(self):
        """Test the delete method."""
        self.client.delete("test-chain-id")
        
        # Check that the transport was called correctly
        self.transport.request.assert_called_with(
            method="DELETE",
            path="/v1/chains/test-chain-id"
        )


if __name__ == "__main__":
    unittest.main()