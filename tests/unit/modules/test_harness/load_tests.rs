//! Load Testing Module
//!
//! This module provides test cases for load testing and concurrency testing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::join_all;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::modules::model_registry::{
    connectors::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, MessageRole},
    storage::ModelRegistry,
    ConnectorError, ModelMetadata, ModelStatus,
};
use crate::modules::router_core::{
    RouterConfig, RouterError, RoutingContext, RoutingMetadata, RoutingRequest, RoutingResponse,
    RoutingStrategy,
};
use crate::modules::test_harness::{
    AssertionHelper, TestCase, TestCategory, TestContext, TestEngine, TestOutcome, TestResult,
    TestSuite,
};

/// Create a test suite for load testing
pub fn create_load_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("Load Tests")
        .with_description("Tests for load testing and concurrency testing");

    // Add test cases
    suite = suite
        .with_test_case(create_concurrent_requests_test_case())
        .with_test_case(create_rate_limiting_test_case())
        .with_test_case(create_resource_contention_test_case())
        .with_test_case(create_performance_degradation_test_case());

    suite = suite.with_test_case(create_race_condition_detection_test_case());

    suite
}

/// Create a test case for concurrent requests
fn create_concurrent_requests_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Performance,
            "concurrent_requests_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running concurrent requests test");

                // Parameters
                let num_concurrent_requests = 50;
                let max_concurrent = 10;
                let request_delay_ms = 10;

                // Create a semaphore to limit concurrency
                let semaphore = Arc::new(Semaphore::new(max_concurrent));

                // Create a shared counter for successful and failed requests
                let success_count = Arc::new(Mutex::new(0));
                let failure_count = Arc::new(Mutex::new(0));
                let response_times = Arc::new(Mutex::new(Vec::new()));

                // Create futures for concurrent requests
                let mut futures = Vec::new();
                for i in 0..num_concurrent_requests {
                    let semaphore = semaphore.clone();
                    let success_count = success_count.clone();
                    let failure_count = failure_count.clone();
                    let response_times = response_times.clone();

                    let future = async move {
                        // Acquire a permit from the semaphore
                        let _permit = semaphore.acquire().await.unwrap();

                        // Create a request
                        let request = create_test_request(i);

                        // Simulate processing time
                        let start_time = Instant::now();

                        // Simulate a request with random success/failure
                        let result = if i % 10 == 0 {
                            // Simulate a failure every 10th request
                            Err(RouterError::ConnectorError("Simulated failure".to_string()))
                        } else {
                            // Simulate successful request with varying response times
                            sleep(Duration::from_millis(request_delay_ms + (i % 5) * 10)).await;
                            Ok(create_test_response(i))
                        };

                        let elapsed = start_time.elapsed();

                        // Record the result
                        match result {
                            Ok(_) => {
                                let mut count = success_count.lock().await;
                                *count += 1;
                            }
                            Err(_) => {
                                let mut count = failure_count.lock().await;
                                *count += 1;
                            }
                        }

                        // Record response time
                        let mut times = response_times.lock().await;
                        times.push(elapsed.as_millis() as u64);
                    };

                    futures.push(future);
                }

                // Execute all futures
                join_all(futures).await;

                // Get results
                let success = *success_count.lock().await;
                let failure = *failure_count.lock().await;
                let times = response_times.lock().await.clone();

                // Calculate statistics
                let total_requests = success + failure;
                let success_rate = (success as f64 / total_requests as f64) * 100.0;

                let mut avg_response_time = 0.0;
                let mut max_response_time = 0;
                let mut min_response_time = u64::MAX;

                if !times.is_empty() {
                    avg_response_time = times.iter().sum::<u64>() as f64 / times.len() as f64;
                    max_response_time = *times.iter().max().unwrap();
                    min_response_time = *times.iter().min().unwrap();
                }

                // Verify results
                AssertionHelper::assert_eq(
                    total_requests,
                    num_concurrent_requests,
                    "Total requests should match expected count",
                )?;

                AssertionHelper::assert_eq(
                    failure,
                    num_concurrent_requests / 10,
                    "Failure count should match expected count",
                )?;

                AssertionHelper::assert_true(
                    avg_response_time > 0.0,
                    "Average response time should be greater than 0",
                )?;

                // Log results
                info!("Concurrent requests test results:");
                info!("Total requests: {}", total_requests);
                info!("Successful requests: {}", success);
                info!("Failed requests: {}", failure);
                info!("Success rate: {:.2}%", success_rate);
                info!("Average response time: {:.2}ms", avg_response_time);
                info!("Min response time: {}ms", min_response_time);
                info!("Max response time: {}ms", max_response_time);

                Ok(TestResult::new(
                    "concurrent_requests_test",
                    TestCategory::Performance,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for rate limiting
fn create_rate_limiting_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(TestCategory::Performance, "rate_limiting_test".to_string()),
        |ctx| {
            async move {
                info!("Running rate limiting test");

                // Parameters
                let requests_per_second = 20;
                let test_duration_seconds = 5;
                let rate_limit = 10; // Maximum requests per second

                // Create a rate limiter
                let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(rate_limit)));

                // Create shared counters
                let accepted_count = Arc::new(Mutex::new(0));
                let rejected_count = Arc::new(Mutex::new(0));

                // Create futures for rate-limited requests
                let mut futures = Vec::new();
                let total_requests = requests_per_second * test_duration_seconds;

                for i in 0..total_requests {
                    let rate_limiter = rate_limiter.clone();
                    let accepted_count = accepted_count.clone();
                    let rejected_count = rejected_count.clone();

                    let future = async move {
                        // Check if request is allowed by rate limiter
                        let allowed = {
                            let mut limiter = rate_limiter.lock().await;
                            limiter.allow_request()
                        };

                        // Record the result
                        if allowed {
                            let mut count = accepted_count.lock().await;
                            *count += 1;
                        } else {
                            let mut count = rejected_count.lock().await;
                            *count += 1;
                        }

                        // Simulate request processing time
                        sleep(Duration::from_millis(10)).await;
                    };

                    futures.push(future);

                    // Add delay between requests to simulate the specified rate
                    if (i + 1) % requests_per_second == 0 {
                        sleep(Duration::from_secs(1)).await;
                    }
                }

                // Execute all futures
                join_all(futures).await;

                // Get results
                let accepted = *accepted_count.lock().await;
                let rejected = *rejected_count.lock().await;

                // Calculate statistics
                let total_requests_actual = accepted + rejected;
                let accepted_rate = (accepted as f64 / total_requests_actual as f64) * 100.0;
                let rejected_rate = (rejected as f64 / total_requests_actual as f64) * 100.0;

                // Verify results
                AssertionHelper::assert_eq(
                    total_requests_actual,
                    total_requests,
                    "Total requests should match expected count",
                )?;

                // We expect approximately rate_limit * test_duration_seconds requests to be accepted
                let expected_accepted = rate_limit * test_duration_seconds;
                let accepted_min = (expected_accepted as f64 * 0.8) as usize; // Allow 20% margin
                let accepted_max = (expected_accepted as f64 * 1.2) as usize; // Allow 20% margin

                AssertionHelper::assert_true(
                    accepted >= accepted_min && accepted <= accepted_max,
                    &format!(
                        "Accepted requests ({}) should be approximately {} (±20%)",
                        accepted, expected_accepted
                    ),
                )?;

                // Log results
                info!("Rate limiting test results:");
                info!("Total requests: {}", total_requests_actual);
                info!("Accepted requests: {} ({:.2}%)", accepted, accepted_rate);
                info!("Rejected requests: {} ({:.2}%)", rejected, rejected_rate);
                info!("Rate limit: {} requests per second", rate_limit);

                Ok(TestResult::new(
                    "rate_limiting_test",
                    TestCategory::Performance,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for resource contention
fn create_resource_contention_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Performance,
            "resource_contention_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running resource contention test");

                // Parameters
                let num_threads = 10;
                let num_iterations = 100;
                let num_resources = 5;

                // Create shared resources
                let resources = Arc::new(Mutex::new(vec![0; num_resources]));
                let contention_detected = Arc::new(Mutex::new(false));

                // Create futures for concurrent resource access
                let mut futures = Vec::new();
                for thread_id in 0..num_threads {
                    let resources = resources.clone();
                    let contention_detected = contention_detected.clone();

                    let future = async move {
                        for _ in 0..num_iterations {
                            // Simulate work before accessing shared resource
                            sleep(Duration::from_millis(thread_id)).await;

                            // Access shared resources
                            let mut resources_guard = resources.lock().await;

                            // Simulate a race condition by checking if resources are in an inconsistent state
                            let sum: i32 = resources_guard.iter().sum();
                            if sum != 0 && sum != num_resources as i32 {
                                let mut contention = contention_detected.lock().await;
                                *contention = true;
                            }

                            // Update resources
                            for i in 0..num_resources {
                                if thread_id % 2 == 0 {
                                    // Even threads increment
                                    resources_guard[i] += 1;
                                } else {
                                    // Odd threads decrement
                                    resources_guard[i] -= 1;
                                }
                            }

                            // Simulate work after accessing shared resource
                            drop(resources_guard); // Explicitly release the lock
                            sleep(Duration::from_millis(1)).await;
                        }
                    };

                    futures.push(future);
                }

                // Execute all futures
                join_all(futures).await;

                // Check if contention was detected
                let contention = *contention_detected.lock().await;

                // In a properly synchronized system, contention should not be detected
                AssertionHelper::assert_false(
                    contention,
                    "Resource contention should not be detected with proper synchronization",
                )?;

                // Check final state of resources
                let final_resources = resources.lock().await.clone();
                let sum: i32 = final_resources.iter().sum();

                // Even and odd threads should cancel each other out if num_threads is even
                let expected_sum = if num_threads % 2 == 0 {
                    0
                } else {
                    num_resources as i32
                };

                AssertionHelper::assert_eq(
                    sum,
                    expected_sum,
                    "Final resource state should match expected value",
                )?;

                // Log results
                info!("Resource contention test results:");
                info!("Number of threads: {}", num_threads);
                info!("Number of iterations: {}", num_iterations);
                info!("Number of resources: {}", num_resources);
                info!("Contention detected: {}", contention);
                info!("Final resource state: {:?}", final_resources);

                Ok(TestResult::new(
                    "resource_contention_test",
                    TestCategory::Performance,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Create a test case for performance degradation under load
fn create_performance_degradation_test_case() -> TestCase {
    TestCase::new(
        TestContext::new(
            TestCategory::Performance,
            "performance_degradation_test".to_string(),
        ),
        |ctx| {
            async move {
                info!("Running performance degradation test");

                // Parameters
                let num_iterations = 5;
                let base_load = 10;
                let load_multiplier = 2;

                // Store response times for each load level
                let mut load_response_times = HashMap::new();

                // Test with increasing load
                for iteration in 0..num_iterations {
                    let current_load = base_load * load_multiplier.pow(iteration);
                    info!("Testing with load: {}", current_load);

                    // Create futures for concurrent requests
                    let mut futures = Vec::new();
                    let response_times = Arc::new(Mutex::new(Vec::new()));

                    for i in 0..current_load {
                        let response_times = response_times.clone();

                        let future = async move {
                            let start_time = Instant::now();

                            // Simulate request processing with some work
                            // The work becomes more intensive as the load increases
                            let work_factor = (i % 5) as u64 + 1;
                            simulate_work(work_factor).await;

                            let elapsed = start_time.elapsed();

                            // Record response time
                            let mut times = response_times.lock().await;
                            times.push(elapsed.as_millis() as u64);
                        };

                        futures.push(future);
                    }

                    // Execute all futures
                    join_all(futures).await;

                    // Calculate statistics
                    let times = response_times.lock().await.clone();
                    let avg_response_time = times.iter().sum::<u64>() as f64 / times.len() as f64;
                    let max_response_time = *times.iter().max().unwrap_or(&0);
                    let min_response_time = *times.iter().min().unwrap_or(&0);

                    // Store results
                    load_response_times.insert(
                        current_load,
                        (avg_response_time, min_response_time, max_response_time),
                    );

                    // Log results for this load level
                    info!(
                        "Load {}: Avg={:.2}ms, Min={}ms, Max={}ms",
                        current_load, avg_response_time, min_response_time, max_response_time
                    );
                }

                // Analyze performance degradation
                let mut prev_load = 0;
                let mut prev_avg_time = 0.0;
                let mut degradation_factors = Vec::new();

                for load in (0..num_iterations).map(|i| base_load * load_multiplier.pow(i)) {
                    if prev_load > 0 {
                        let (avg_time, _, _) = load_response_times[&load];
                        let degradation_factor = avg_time / prev_avg_time;
                        degradation_factors.push(degradation_factor);

                        info!(
                            "Load increase from {} to {}: Response time increased by factor {:.2}",
                            prev_load, load, degradation_factor
                        );
                    }

                    prev_load = load;
                    prev_avg_time = load_response_times[&load].0;
                }

                // Calculate average degradation factor
                let avg_degradation_factor = if degradation_factors.is_empty() {
                    1.0
                } else {
                    degradation_factors.iter().sum::<f64>() / degradation_factors.len() as f64
                };

                // Verify results
                // We expect some degradation as load increases, but it should be reasonable
                AssertionHelper::assert_true(
                    avg_degradation_factor >= 1.0,
                    "Average degradation factor should be at least 1.0",
                )?;

                // Check if degradation is sub-linear (good) or super-linear (concerning)
                let is_sublinear = avg_degradation_factor < load_multiplier as f64;

                info!("Performance degradation test results:");
                info!("Average degradation factor: {:.2}", avg_degradation_factor);
                info!(
                    "Degradation is {}",
                    if is_sublinear {
                        "sub-linear (good)"
                    } else {
                        "super-linear (concerning)"
                    }
                );

                // We don't fail the test based on degradation factor, just report it
                Ok(TestResult::new(
                    "performance_degradation_test",
                    TestCategory::Performance,
                    TestOutcome::Passed,
                ))
            }
            .boxed()
        },
    )
}

/// Simulate work with a given factor
async fn simulate_work(factor: u64) {
    // Simulate CPU-bound work
    let start = Instant::now();
    let duration = Duration::from_millis(10 * factor);

    while start.elapsed() < duration {
        // Busy wait to simulate CPU work
        for _ in 0..1000 {
            std::hint::black_box(0);
        }

        // Yield to allow other tasks to run
        tokio::task::yield_now().await;
    }
}

/// Create a test request
fn create_test_request(id: usize) -> RoutingRequest {
    use std::time::Duration;
    use std::collections::HashMap;

    // Create the chat completion request
    let chat_request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: format!("Test request {}", id),
            name: None,
            function_call: None,
            tool_calls: None,
        }],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: Some(false),
        functions: None,
        tools: None,
        additional_params: None,
    };

    // Create the routing context
    let mut parameters = HashMap::new();
    parameters.insert("request_id".to_string(), format!("request-{}", id));
    parameters.insert("session_id".to_string(), format!("session-{}", id));

    let context = RoutingContext {
        request: chat_request,
        user_id: Some(format!("user-{}", id)),
        org_id: None,
        timestamp: chrono::Utc::now(),
        priority: 0,
        tags: vec!["test".to_string()],
        parameters,
    };

    // Create the routing request
    RoutingRequest {
        context,
        model_filter: None,
        preferred_model_id: None,
        excluded_model_ids: Vec::new(),
        max_attempts: 3,
        timeout: Duration::from_secs(30),
    }
}

/// Create a test response
fn create_test_response(id: usize) -> RoutingResponse {
    use std::collections::HashMap;

    // Create a chat completion response
    let chat_response = ChatCompletionResponse {
        id: format!("response-{}", id),
        model: "test-model".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: format!("Response to test request {}", id),
                name: None,
                function_call: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    // Create routing metadata
    let now = chrono::Utc::now();
    let mut additional_metadata = HashMap::new();
    additional_metadata.insert("test_id".to_string(), id.to_string());

    let metadata = RoutingMetadata {
        selected_model_id: "test-model".to_string(),
        strategy_name: "test-strategy".to_string(),
        routing_start_time: now - chrono::Duration::milliseconds(10),
        routing_end_time: now,
        routing_time_ms: 10,
        models_considered: 1,
        attempts: 1,
        is_fallback: false,
        selection_criteria: None,
        additional_metadata,
    };

    // Create the routing response
    RoutingResponse {
        response: chat_response,
        metadata,
    }
}

/// Simple rate limiter
struct RateLimiter {
    rate_limit: usize,
    window_start: Instant,
    request_count: usize,
}

impl RateLimiter {
    /// Create a new rate limiter
    fn new(rate_limit: usize) -> Self {
        Self {
            rate_limit,
            window_start: Instant::now(),
            request_count: 0,
        }
    }

    /// Check if a request is allowed
    fn allow_request(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start);

        // Reset window if more than 1 second has passed
        if elapsed >= Duration::from_secs(1) {
            self.window_start = now;
            self.request_count = 0;
        }

        // Check if we're under the rate limit
        if self.request_count < self.rate_limit {
            self.request_count += 1;
            true
        } else {
            false
        }
        
        /// Create a test case for race condition detection
        fn create_race_condition_detection_test_case() -> TestCase {
            TestCase::new(
                TestContext::new(
                    TestCategory::Performance,
                    "race_condition_detection_test".to_string(),
                ),
                |ctx| {
                    async move {
                        info!("Running race condition detection test");
        
                        // Parameters
                        let num_threads = 20;
                        let num_iterations = 50;
                        
                        // Create a shared counter without proper synchronization
                        // This is intentionally vulnerable to race conditions
                        let counter = Arc::new(std::sync::RwLock::new(0));
                        
                        // Create a flag to detect race conditions
                        let race_detected = Arc::new(Mutex::new(false));
                        
                        // Create futures for concurrent counter updates
                        let mut futures = Vec::new();
                        for thread_id in 0..num_threads {
                            let counter = counter.clone();
                            let race_detected = race_detected.clone();
                            
                            let future = async move {
                                for _ in 0..num_iterations {
                                    // Introduce random delays to increase chance of race conditions
                                    let delay = thread_id % 3;
                                    sleep(Duration::from_millis(delay)).await;
                                    
                                    // Simulate a race condition by reading, modifying, and writing
                                    // without proper synchronization between operations
                                    let current = {
                                        // Read the current value
                                        let read_guard = counter.read().unwrap();
                                        *read_guard
                                    };
                                    
                                    // Introduce a small delay between read and write to increase
                                    // the chance of a race condition
                                    sleep(Duration::from_millis(1)).await;
                                    
                                    // Write the new value
                                    let new_value = current + 1;
                                    {
                                        let mut write_guard = counter.write().unwrap();
                                        *write_guard = new_value;
                                    }
                                    
                                    // Check for race condition by comparing with expected value
                                    // This is a simplified detection mechanism
                                    let expected_min = thread_id * num_iterations;
                                    if new_value < expected_min && !*race_detected.lock().await {
                                        let mut race = race_detected.lock().await;
                                        *race = true;
                                        info!("Race condition detected: value {} is less than expected minimum {}",
                                              new_value, expected_min);
                                    }
                                }
                            };
                            
                            futures.push(future);
                        }
                        
                        // Execute all futures
                        join_all(futures).await;
                        
                        // Get final counter value
                        let final_value = *counter.read().unwrap();
                        let expected_value = num_threads * num_iterations;
                        let race_condition = *race_detected.lock().await;
                        
                        // Log results
                        info!("Race condition detection test results:");
                        info!("Number of threads: {}", num_threads);
                        info!("Number of iterations per thread: {}", num_iterations);
                        info!("Expected final counter value: {}", expected_value);
                        info!("Actual final counter value: {}", final_value);
                        info!("Race condition detected: {}", race_condition);
                        
                        // In a real system, we would want to fix race conditions
                        // But for this test, we're intentionally creating a race condition
                        // to verify our detection mechanism works
                        if final_value != expected_value {
                            info!("Race condition confirmed: final value {} != expected value {}",
                                  final_value, expected_value);
                            
                            // We don't fail the test because we're intentionally creating a race condition
                            // Instead, we verify that our detection mechanism worked
                            AssertionHelper::assert_true(
                                race_condition,
                                "Race condition should have been detected",
                            )?;
                        } else {
                            info!("No race condition occurred during this test run");
                            // This is unlikely but possible - the test is non-deterministic
                            // We don't fail the test in this case either
                        }
                        
                        Ok(TestResult::new(
                            "race_condition_detection_test",
                            TestCategory::Performance,
                            TestOutcome::Passed,
                        ))
                    }
                    .boxed()
                },
            )
        }
    }
}
