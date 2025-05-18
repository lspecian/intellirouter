#!/bin/bash
# Test script for the router role

set -e  # Exit on error

echo "===== Testing IntelliRouter Router Role ====="
echo

# Start router role
echo "Starting router role..."
cargo run -- run --role router &
ROUTER_PID=$!

# Wait for router to start
echo "Waiting for router to start..."
sleep 5

# Test endpoint
ENDPOINT="http://localhost:8080/v1/chat/completions"

# Test payload
PAYLOAD='{
  "model": "mock-llama",
  "messages": [
    {
      "role": "user",
      "content": "Hello from the test script!"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 100
}'

echo "Testing endpoint: $ENDPOINT"
echo "Payload:"
echo "$PAYLOAD" | jq .
echo

# Send request
echo "Sending request..."
RESPONSE=$(curl -s -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD")

echo "Response:"
echo "$RESPONSE" | jq .

# Check if the response has the expected fields
if echo "$RESPONSE" | jq -e '.id' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.choices' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.choices[0].message' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.choices[0].finish_reason' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.usage' > /dev/null 2>&1; then
  echo
  echo "✅ Test PASSED: Response contains all expected fields"
  TEST_RESULT=0
else
  echo
  echo "❌ Test FAILED: Response is missing expected fields"
  TEST_RESULT=1
fi

# Clean up
echo "Stopping router..."
kill $ROUTER_PID

echo "Test completed."
exit $TEST_RESULT