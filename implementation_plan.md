# Implementation Plan for Feature Flags in Test Helpers

## Identified Test Helpers Used in Production Code

1. **Audit Module Communication Helpers**:
   - The audit module in the main codebase is using test helpers from the `intellirouter-test-utils` crate
   - These helpers are used for testing communication between services
   - The imports are already conditionally included with `#[cfg(feature = "test-utils")]`

2. **Test Client Binary**:
   - The `test_client.rs` binary is already conditionally compiled with `#[cfg(feature = "test-utils")]`

3. **Mock Model Backend**:
   - The `mock_backend.rs` module is used in production code but is primarily for testing
   - It's re-exported in the main `llm_proxy/mod.rs` file without feature flags

## Implementation Plan

1. **Update Audit Module**:
   - The audit module already has proper feature flags
   - Verify that all imports from `intellirouter-test-utils` are properly guarded

2. **Update Mock Backend**:
   - Add feature flags to the mock backend module
   - Update the re-export in `llm_proxy/mod.rs` to be conditional on the `test-utils` feature

3. **Update Router Integration**:
   - Add feature flags to the mock router service functions in `router_integration.rs`
   - These functions are used in production code but are primarily for testing

4. **Update Documentation**:
   - Add documentation explaining the purpose of the feature flags
   - Explain how to use these helpers in non-production environments