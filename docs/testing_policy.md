# Test-First Development Policy

This document outlines IntelliRouter's test-first development policy, which ensures that all code is tested before it's considered complete.

## Table of Contents

- [Introduction](#introduction)
- [Test-First Principles](#test-first-principles)
- [Implementation Process](#implementation-process)
- [Testing Tools](#testing-tools)
- [Test Templates](#test-templates)
- [Verification Process](#verification-process)
- [Exceptions and Bypasses](#exceptions-and-bypasses)
- [Integration with CI/CD](#integration-with-cicd)
- [FAQ](#faq)

## Introduction

IntelliRouter follows a test-first development approach, also known as Test-Driven Development (TDD). This means that tests are written before implementing the actual functionality. This approach ensures that:

1. All code is testable by design
2. All code has tests
3. Implementation meets requirements
4. Edge cases are considered from the start
5. Refactoring can be done safely

## Test-First Principles

The core principles of our test-first approach are:

1. **Write Tests First**: Tests should be written before implementing the functionality they test
2. **Verify Test Failure**: Tests should fail initially (since the functionality doesn't exist yet)
3. **Implement Minimally**: Implement the minimum code needed to make tests pass
4. **Refactor Safely**: Refactor the code while ensuring tests continue to pass

## Implementation Process

### Step 1: Understand Requirements

Before writing any tests, ensure you understand the requirements:
- What functionality is needed?
- What are the expected inputs and outputs?
- What edge cases need to be handled?
- What error conditions should be considered?

### Step 2: Write Tests

Write tests that verify the expected behavior:
- Create a test module or file
- Write test cases for normal operation
- Write test cases for edge cases
- Write test cases for error conditions

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_initialization() {
        let config = RouterConfig {
            strategy: RoutingStrategy::ContentBased,
        };
        
        let result = init(config);
        assert!(result.is_ok());
        
        let router = result.unwrap();
        assert_eq!(router.strategy, RoutingStrategy::ContentBased);
    }
    
    #[test]
    fn test_router_initialization_with_invalid_config() {
        let config = RouterConfig {
            strategy: RoutingStrategy::Invalid,
        };
        
        let result = init(config);
        assert!(result.is_err());
    }
}
```

### Step 3: Verify Test Failure

Run the tests to verify that they fail:
```bash
cargo test
```

Document the test failures, as they will be useful for code reviews and PR descriptions.

### Step 4: Implement Functionality

Implement the minimum code needed to make the tests pass:

```rust
pub struct Router {
    pub strategy: RoutingStrategy,
}

pub fn init(config: RouterConfig) -> Result<Router, Error> {
    if config.strategy == RoutingStrategy::Invalid {
        return Err(Error::InvalidStrategy);
    }
    
    Ok(Router {
        strategy: config.strategy,
    })
}
```

### Step 5: Verify Tests Pass

Run the tests again to verify that they pass:
```bash
cargo test
```

### Step 6: Refactor

Refactor the code to improve its design, readability, and performance, while ensuring the tests continue to pass.

## Testing Tools

IntelliRouter uses the following testing tools:

- **Rust's Built-in Testing Framework**: For unit and integration tests
- **Mockall**: For creating mock implementations
- **Proptest**: For property-based testing
- **Tarpaulin**: For test coverage reporting

## Test Templates

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name_scenario_being_tested() {
        // Arrange
        let input = // ...
        
        // Act
        let result = function_being_tested(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### Integration Test Template

```rust
#[test]
fn test_component_interaction() {
    // Arrange
    let component1 = // ...
    let component2 = // ...
    
    // Act
    let result = component1.interact_with(component2);
    
    // Assert
    assert!(result.is_ok());
    // Additional assertions...
}
```

### Mock Test Template

```rust
#[test]
fn test_with_mock() {
    // Arrange
    let mut mock = MockComponent::new();
    
    mock.expect_method()
        .with(eq(expected_input))
        .times(1)
        .returning(|_| Ok(expected_output));
    
    // Act
    let result = system_under_test.use_component(mock);
    
    // Assert
    assert_eq!(result, expected_result);
}
```

## Verification Process

### Code Review

During code review, reviewers should verify that:
- Tests were written before implementation
- Tests cover normal operation, edge cases, and error conditions
- Tests are well-structured and follow the Arrange-Act-Assert pattern
- Implementation passes all tests
- Test coverage is adequate

### PR Requirements

Pull requests should include:
- Evidence that tests were written first (commit history or test failure screenshots)
- Explanation of the testing approach
- Test coverage report

## Exceptions and Bypasses

In rare cases, it may be necessary to bypass the test-first approach. This should be done only in exceptional circumstances and with proper justification.

To bypass the pre-commit hook check:
```bash
git commit --no-verify
```

When bypassing the test-first approach, you must:
1. Document the reason for the bypass
2. Add tests as soon as possible after implementation
3. Get approval from a team lead or maintainer

## Integration with CI/CD

Our CI/CD pipeline enforces the test-first approach by:
- Running all tests
- Checking test coverage
- Verifying that tests exist for all code
- Failing the build if these checks don't pass

## FAQ

### Q: What if I'm fixing a bug?
A: Write a test that reproduces the bug first, then fix the bug to make the test pass.

### Q: What if I'm refactoring code?
A: Ensure that tests exist for the code being refactored. If not, write tests for the current behavior before refactoring.

### Q: What if I'm working on exploratory code?
A: Use a feature branch and document that it's exploratory. Add tests before merging to the main branch.

### Q: How detailed should my tests be?
A: Tests should verify all expected behaviors, edge cases, and error conditions. They should be detailed enough to serve as documentation for how the code should behave.

### Q: What's the minimum acceptable test coverage?
A: We aim for at least 80% test coverage overall, with critical components having 90%+ coverage.