//! This file provides a template for writing integration tests following the test-first approach.
//! Copy and adapt this template when creating new integration tests.

// Import the modules being tested
use intellirouter::module_a;
use intellirouter::module_b;
use intellirouter::test_utils;

// Basic integration test
#[test]
fn test_modules_work_together() {
    // Arrange: Set up the test data and environment
    let input_data = "test input";
    let expected_output = "expected result";

    // Initialize components
    let component_a = module_a::initialize().expect("Failed to initialize component A");
    let component_b = module_b::initialize().expect("Failed to initialize component B");

    // Act: Test the interaction between components
    let result = component_a.process_with(component_b, input_data);

    // Assert: Verify the result matches expectations
    assert_eq!(result, expected_output);
}

// Integration test with test fixtures
#[test]
fn test_with_test_fixtures() {
    // Arrange: Set up test fixtures
    let test_fixture = test_utils::create_test_fixture();
    let expected_output = "expected result";

    // Act: Use the test fixture
    let result = test_fixture.execute_test_scenario();

    // Assert: Verify the result
    assert_eq!(result, expected_output);
}

// Integration test with mocked external dependencies
#[test]
fn test_with_mocked_dependencies() {
    // Arrange: Create and configure mocks
    let mut mock_external_service = test_utils::MockExternalService::new();

    mock_external_service
        .expect_call()
        .with(test_utils::eq("expected input"))
        .times(1)
        .returning(|_| Ok("mocked response".to_string()));

    // Initialize system under test with the mock
    let system = module_a::SystemUnderTest::new(Box::new(mock_external_service));

    // Act: Execute the test
    let result = system.execute_operation("test input");

    // Assert: Verify the result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "expected result");
}

// Integration test for error handling
#[test]
fn test_error_propagation() {
    // Arrange: Set up error condition
    let mut mock_service = test_utils::MockExternalService::new();

    mock_service
        .expect_call()
        .with(test_utils::eq("error trigger"))
        .times(1)
        .returning(|_| Err("service error".to_string()));

    // Initialize system with the mock
    let system_a = module_a::SystemUnderTest::new(Box::new(mock_service));
    let system_b = module_b::initialize().expect("Failed to initialize system B");

    // Act: Test error propagation between components
    let result = system_a.interact_with(system_b, "error trigger");

    // Assert: Verify error is properly propagated
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Error from service: service error"
    );
}

// Async integration test
#[tokio::test]
async fn test_async_integration() {
    // Arrange: Set up async components
    let component_a = module_a::initialize_async()
        .await
        .expect("Failed to initialize async component A");
    let component_b = module_b::initialize_async()
        .await
        .expect("Failed to initialize async component B");

    // Act: Test async interaction
    let result = component_a
        .process_with_async(component_b, "test input")
        .await;

    // Assert: Verify the result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "expected async result");
}

// Database integration test
#[test]
fn test_database_integration() {
    // Arrange: Set up test database
    let db = test_utils::setup_test_database();

    // Initialize component with database
    let component =
        module_a::initialize_with_db(db.clone()).expect("Failed to initialize component with DB");

    // Act: Perform database operation
    let result = component.store_data("test key", "test value");

    // Assert: Verify operation succeeded
    assert!(result.is_ok());

    // Verify data was stored correctly
    let stored_value = db.get("test key").expect("Failed to get value from DB");
    assert_eq!(stored_value, "test value");
}

// HTTP API integration test
#[tokio::test]
async fn test_http_api_integration() {
    // Arrange: Start test server
    let server = test_utils::start_test_server().await;
    let client = reqwest::Client::new();

    // Act: Send request to the server
    let response = client
        .post(&format!("http://{}/api/endpoint", server.address()))
        .json(&serde_json::json!({
            "key": "value"
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Assert: Verify response
    assert_eq!(response.status(), 200);

    let body = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response body");

    assert_eq!(body["result"], "success");
}

// End-to-end test with multiple components
#[test]
fn test_end_to_end_flow() {
    // Arrange: Set up the complete test environment
    let test_env = test_utils::setup_test_environment();

    // Act: Execute the complete flow
    let result = test_env.execute_complete_flow("test input");

    // Assert: Verify the end-to-end result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "expected end-to-end result");

    // Verify side effects
    assert!(test_env.verify_side_effects());
}
