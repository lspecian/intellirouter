//! Test templates for IntelliRouter
//!
//! This module contains templates for writing tests following IntelliRouter's test-first approach.
//! These templates are meant to be copied and adapted, not used directly.
//!
//! The entire module is guarded with `#[cfg(test)]` to ensure it's only compiled during tests.

#![cfg(test)]

// Re-export the templates
pub mod integration_test_template;
pub mod unit_test_template;
