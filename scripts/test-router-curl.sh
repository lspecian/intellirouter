#!/bin/bash
# Simple curl test for the router role

set -e  # Exit on error

echo "===== Testing IntelliRouter Router Role with curl ====="
echo

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
  "max_tokens": 100,
  "stream": false
}'

echo "Testing endpoint: $ENDPOINT"
echo "Payload:"
echo "$PAYLOAD" | jq .
echo

# Send request
echo "Sending request..."
RESPONSE=$(curl -s -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d "$PAYLOAD")

echo "Response:"
echo "Raw response: $RESPONSE"
echo "$RESPONSE" | jq . || echo "Failed to parse response as JSON"

# Check if the response has the expected fields
if echo "$RESPONSE" | jq -e '.id' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.choices' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.choices[0].message' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.choices[0].finish_reason' > /dev/null 2>&1 && \
   echo "$RESPONSE" | jq -e '.usage' > /dev/null 2>&1; then
  echo
  echo "✅ Test PASSED: Response contains all expected fields"
  exit 0
else
  echo
  echo "❌ Test FAILED: Response is missing expected fields"
  exit 1
fi