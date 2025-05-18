#!/bin/bash
# Simple CLI script to test the router role

# Set variables
HOST=${1:-localhost}
PORT=${2:-9000}
MODEL=${3:-mock-llama}

echo "Testing IntelliRouter Router Role"
echo "=================================="
echo "Host: $HOST"
echo "Port: $PORT"
echo "Model: $MODEL"
echo

# Create a simple test request
echo "Sending test request to /v1/chat/completions..."
curl -s -X POST "http://$HOST:$PORT/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"messages\": [
      {
        \"role\": \"user\",
        \"content\": \"Hello from the test script!\"
      }
    ],
    \"temperature\": 0.7,
    \"max_tokens\": 100
  }" | jq .

echo
echo "Test completed!"