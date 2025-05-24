//! # Assertion Libraries and Validation Framework
//!
//! This module provides comprehensive assertion utilities for IntelliRouter testing,
//! including domain-specific assertions, a validation rules engine, and support for
//! different test categories.

mod core;
mod domain;
mod matchers;
mod rules;
mod validation;

// Re-export core assertion types and functions
pub use core::{
    Assert, AssertionContext, AssertionError, AssertionOutcome, AssertionResult, Assertions,
};

// Re-export domain-specific assertions
pub use domain::{
    chain::ChainAssertions, grpc::GrpcAssertions, http::HttpAssertions, llm::LlmAssertions,
    performance::PerformanceAssertions, rag::RagAssertions, router::RouterAssertions,
    security::SecurityAssertions,
};

// Re-export matchers
pub use matchers::{
    ContainsMatcher, EqualsMatcher, ErrorRateMatcher, HeaderMatcher, JsonMatcher,
    JsonSchemaMatcher, LatencyMatcher, Matcher, RegexMatcher, ResponseTimeMatcher,
    StatusCodeMatcher, ThroughputMatcher, TypeMatcher, XmlMatcher,
};

// Re-export validation rules engine
pub use rules::{
    Rule, RuleEngine, RuleSet, ValidationContext, ValidationResult, ValidationSeverity,
    ValidationStatus,
};

// Re-export validation framework
pub use validation::{
    AsyncValidator, SyncValidator, ValidationConfig, ValidationFormatter, ValidationRegistry,
    ValidationReport, ValidationReporter, Validator, ValidatorBuilder,
};

// Convenience functions
pub use core::{assert_context, assert_that};
