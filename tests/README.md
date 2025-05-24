# IntelliRouter Test Structure

This directory contains a comprehensive test suite for the IntelliRouter project. The tests are organized into a structured hierarchy to ensure complete validation of all components and their interactions.

## Directory Structure

```
tests/
├── unit/                  # Unit tests mirroring src/ structure
│   └── modules/           # Tests for specific modules
│       ├── audit/         # Tests for audit module
│       ├── ipc/           # Tests for IPC module
│       └── ...            # Tests for other modules
├── integration/           # Integration tests between components
│   ├── chain_tests.rs     # Tests for chain execution
│   └── ...                # Other integration tests
├── e2e/                   # End-to-end tests for complete workflows
│   ├── api/               # API-focused end-to-end tests
│   ├── performance/       # Performance and load tests
│   └── scenarios/         # Scenario-based end-to-end tests
├── bin/                   # Test binaries
│   └── run_tests.rs       # Test runner (moved from src/bin/)
├── templates/             # Test templates for new tests
│   ├── unit_test_template.rs
│   ├── integration_test_template.rs
│   └── e2e_test_template.rs
├── framework/             # Test framework components
│   └── test_harness/      # Test harness components
├── compilation_tools/     # Tools for testing compilation
├── openai_compatibility/  # Tests for OpenAI API compatibility
└── test_plans/            # Test plans and documentation
```

This structure represents the new organization of tests, which has been updated to improve maintainability and clarity. The key changes include:

1. Moving all module-specific tests from `src/modules/*/tests.rs` to `tests/unit/modules/*/`
2. Moving the test runner from `src/bin/run_tests.rs` to `tests/bin/run_tests.rs`
3. Creating dedicated directories for integration and end-to-end tests
4. Providing test templates for new tests
5. Separating test utilities into the `intellirouter-test-utils` crate

## Test Categories

### 1. Unit Tests

Located in `tests/unit/`, these tests focus on individual components in isolation:

- Each module has its own directory structure mirroring the `src/modules/` structure
- Tests validate the behavior of individual functions, methods, and classes
- Mock dependencies are used to isolate the component being tested
- Fast to run and provide immediate feedback

### 2. Integration Tests

Located in `tests/integration/`, these tests validate interactions between components:

- Test how different modules work together
- Validate API contracts between components
- May use real or mock dependencies depending on the test
- More comprehensive but slower than unit tests

### 3. End-to-End Tests

Located in `tests/e2e/`, these tests validate complete workflows:

- Test the entire system from end to end
- Validate real-world scenarios and user workflows
- Use real dependencies and services
- Comprehensive but slow to run

### 4. Test Templates

Located in `tests/templates/`, these provide starting points for new tests:

- `unit_test_template.rs`: Template for unit tests
- `integration_test_template.rs`: Template for integration tests
- `e2e_test_template.rs`: Template for end-to-end tests

## Running Tests

### Running All Tests

```bash
cargo test
```

### Running Specific Test Categories

```bash
# Run only unit tests
cargo test --test 'unit_*'

# Run only integration tests
cargo test --test 'integration_*'

# Run only e2e tests
cargo test --test 'e2e_*'

# Run only e2e tests including ignored tests
cargo test --test 'e2e_*' -- --ignored

# Run specific module tests
cargo test unit::modules::router_core
```

### Running Tests for Specific Modules

```bash
# Run tests for the audit module
cargo test unit::modules::audit

# Run tests for the IPC module
cargo test unit::modules::ipc
```

## Writing New Tests

When adding new functionality, please add corresponding tests:

1. Choose the appropriate test category (unit, integration, e2e)
2. Use the corresponding template from `tests/templates/`
3. Place the test in the appropriate directory
4. Follow the test-first development approach as outlined in the project guidelines
5. Ensure tests are comprehensive and cover edge cases

For detailed guidance on writing tests, see the [Testing Guide](../docs/testing_guide.md).

## Router-Specific Tests

The following sections detail the router-specific tests that were previously documented:

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

The test suite aims to cover:

1. **Unit-level functionality**
   - Individual functions and methods
   - Error handling and edge cases
   - Component-specific behavior

2. **Component interactions**
   - API contracts between components
   - Data flow between modules
   - Error propagation

3. **End-to-end workflows**
   - Complete user scenarios
   - System resilience and recovery
   - Performance under load

## Adding New Tests

When adding new functionality to the project, please add corresponding tests to maintain test coverage. Follow these guidelines:

1. **Unit Tests**: Add tests for new functions or methods in the appropriate unit test file under `tests/unit/modules/`, mirroring the source code structure.
2. **Integration Tests**: Add tests for new component interactions in the `tests/integration/` directory.
3. **E2E Tests**: Add tests for new end-to-end workflows in the `tests/e2e/` directory.
4. **Schema Tests**: Update schema validation tests when the API schema changes.

### Test-First Development

Remember to follow the test-first development approach:

1. Write tests before implementing functionality
2. Verify that tests fail appropriately
3. Implement the minimum code needed to make tests pass
4. Refactor while maintaining passing tests

For more details on the test-first approach, see [Testing Policy](../docs/testing_policy.md).

## Test Utilities

The `intellirouter-test-utils` crate provides common utilities for testing:

- **Fixtures**: Common test data and setup
- **Helpers**: Utility functions for testing
- **Mocks**: Mock implementations of dependencies

For more details on the test utilities, see the [Test Utilities README](../intellirouter-test-utils/README.md).

## Test Harness

The test harness provides a comprehensive framework for testing different aspects of the system. It is located in the `tests/framework/test_harness/` directory and includes components for:

- Assertions
- Benchmarking
- Configuration
- Data management
- Mocking
- Scenario testing
- Security testing

For more details on the test harness, see [Test Harness Documentation](../docs/test_harness.md).