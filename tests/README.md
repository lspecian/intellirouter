# IntelliRouter Test Harness

This directory contains a comprehensive test suite for the IntelliRouter, with a particular focus on the `router` role. The tests are organized into a three-tiered structure to ensure complete validation of the router's functionality.

## Test Structure

### 1. Schema Compliance Tests
Located in `tests/unit/router/schema_validation_tests.rs`, these tests validate:
- Incoming JSON against OpenAI `/v1/chat/completions` spec
- All required fields exist and types match
- Response formatting before returning to client

### 2. Mock Unit Tests
Located in `tests/unit/router/mock_backend_tests.rs`, these tests:
- Use `MockModelBackend` to return static messages
- Test `router_core` and `llm_proxy` with mock backend
- Assert routing logic works for known models, unknown models, and malformed input

### 3. Integration Tests
Located in `tests/integration/router_http_tests.rs`, these tests:
- Boot `router` on a test port using `spawn_test_server()`
- Send real HTTP POST requests to `/v1/chat/completions`
- Assert HTTP 200 status and validate response structure

### 4. External API Tests (Optional)
Located in `tests/integration/router_http_tests.rs` (marked with `#[ignore]`), these tests:
- Use `OPENAI_API_KEY` environment variable to make real API calls
- Route requests to live OpenAI backend
- Assert schema and correctness

## Running the Tests

### Running Local Tests

To run all tests except the ignored external API tests:

```bash
cargo test
```

This will run all unit and integration tests that don't require external API keys.

### Running Specific Test Categories

To run only the schema validation tests:

```bash
cargo test unit::router::schema_validation_tests
```

To run only the mock backend tests:

```bash
cargo test unit::router::mock_backend_tests
```

To run only the integration tests:

```bash
cargo test integration::router_http_tests
```

### Running External API Tests

To run the external API tests that require an OpenAI API key:

1. Set your OpenAI API key as an environment variable:
   ```bash
   export OPENAI_API_KEY=your_api_key_here
   ```

2. Run the ignored tests:
   ```bash
   cargo test -- --ignored
   ```

## Manual Testing

You can also test the router manually using curl:

```bash
# Start the router
cargo run -- run --role router

# In another terminal, send a request
curl -X POST "http://localhost:8080/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "mock-llama",
    "messages": [
      {
        "role": "user",
        "content": "Hello from curl!"
      }
    ],
    "temperature": 0.7,
    "max_tokens": 100
  }'
```

## Test Coverage

The test suite covers:

1. **Request Validation**
   - Valid requests are accepted
   - Invalid requests are rejected with appropriate error messages
   - All required fields are validated

2. **Response Validation**
   - Responses match the OpenAI API format
   - All required fields are present
   - Types match the expected schema

3. **Routing Logic**
   - Requests are routed to the appropriate model
   - Unknown models are handled gracefully
   - Different routing strategies work correctly

4. **Error Handling**
   - Malformed requests return appropriate error responses
   - Model errors are handled gracefully
   - Timeouts and other errors are handled appropriately

5. **Streaming**
   - Streaming requests work correctly
   - Streaming responses match the expected format

## Adding New Tests

When adding new functionality to the router, please add corresponding tests to maintain test coverage. Follow these guidelines:

1. **Unit Tests**: Add tests for new functions or methods in the appropriate unit test file.
2. **Integration Tests**: Add tests for new endpoints or functionality in the integration test file.
3. **Schema Tests**: Update schema validation tests when the API schema changes.