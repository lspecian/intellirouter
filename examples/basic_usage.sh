#!/bin/bash
# Basic Usage Example for IntelliRouter
# This script demonstrates how to start IntelliRouter and send a basic request

# Set variables
HOST="localhost"
PORT="8080"
MODEL="gpt-3.5-turbo"

echo "IntelliRouter Basic Usage Example"
echo "=================================="

# Step 1: Start IntelliRouter (in background)
echo "Step 1: Starting IntelliRouter..."
echo "Running: intellirouter run --role router"
echo "Note: In a real scenario, you would run this in a separate terminal"
echo "      or use 'intellirouter run --role router &' to run in background"
echo ""

# Step 2: Wait for the server to start
echo "Step 2: Waiting for server to start..."
echo "In a real scenario, you might want to add a sleep command here"
echo "sleep 5"
echo ""

# Step 3: Send a test request
echo "Step 3: Sending a test request to IntelliRouter..."
echo "Running: curl -X POST http://$HOST:$PORT/v1/chat/completions \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"What is the capital of France?\"}]}'"
echo ""

# Step 4: Example response (simulated)
echo "Step 4: Example response:"
echo '{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-3.5-turbo",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "The capital of France is Paris."
    },
    "finish_reason": "stop"
  }]
}'
echo ""

# Step 5: Streaming example
echo "Step 5: Sending a streaming request..."
echo "Running: curl -X POST http://$HOST:$PORT/v1/chat/completions \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Tell me about Paris.\"}],\"stream\":true}'"
echo ""

echo "Step 6: Stopping IntelliRouter..."
echo "In a real scenario, you would press Ctrl+C in the terminal where IntelliRouter is running"
echo "or use 'kill <PID>' if running in background"
echo ""

echo "Example completed"