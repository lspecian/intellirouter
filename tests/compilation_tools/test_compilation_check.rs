//! Tests for the compilation status check tool.
//!
//! These tests verify that the compilation status check correctly identifies
//! compilation errors in different parts of the codebase.
//!
//! This test is only available when the `test-utils` feature is enabled.
#![cfg(feature = "test-utils")]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Temporary directory for test files
const TEMP_DIR: &str = "target/test_compilation";

/// Setup function to create a temporary directory for test files
fn setup() {
    let temp_dir = PathBuf::from(TEMP_DIR);
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to remove temporary directory");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temporary directory");
    fs::create_dir_all(temp_dir.join("src")).expect("Failed to create src directory");
}

/// Cleanup function to remove temporary test files
fn cleanup() {
    let temp_dir = PathBuf::from(TEMP_DIR);
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to remove temporary directory");
    }
}

/// Create a Cargo.toml file for testing
fn create_cargo_toml(path: &Path, lib: bool, bin: bool) {
    let mut content = r#"[package]
name = "test_compilation"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    .to_string();

    if lib {
        content.push_str("\n[lib]\npath = \"src/lib.rs\"\n");
    }

    if bin {
        content.push_str("\n[[bin]]\nname = \"test_bin\"\npath = \"src/main.rs\"\n");
    }

    let mut file = File::create(path.join("Cargo.toml")).expect("Failed to create Cargo.toml");
    file.write_all(content.as_bytes())
        .expect("Failed to write to Cargo.toml");
}

/// Run cargo check on the test project
fn run_cargo_check(path: &Path, target: &str) -> bool {
    let output = Command::new("cargo")
        .current_dir(path)
        .args(&["check", target])
        .output()
        .expect("Failed to execute cargo check");

    output.status.success()
}

/// Test that the compilation status check correctly identifies errors in library code
#[test]
fn test_lib_compilation_error() {
    setup();
    let temp_dir = PathBuf::from(TEMP_DIR);

    // Create Cargo.toml with lib
    create_cargo_toml(&temp_dir, true, false);

    // Create a lib.rs file with a compilation error
    let lib_content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    // Missing semicolon - compilation error
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    // Syntax error - missing closing brace
    a - b
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Run cargo check on lib
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should fail due to compilation error
    assert!(!result, "Compilation should fail with syntax error");

    // Fix the compilation error
    let fixed_lib_content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    // Fixed - added semicolon
    a + b;
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    // Fixed - added closing brace
    a - b
}
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(fixed_lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Run cargo check on lib again
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should succeed now
    assert!(result, "Compilation should succeed after fixing the error");

    cleanup();
}

/// Test that the compilation status check correctly identifies errors in binary code
#[test]
fn test_bin_compilation_error() {
    setup();
    let temp_dir = PathBuf::from(TEMP_DIR);

    // Create Cargo.toml with bin
    create_cargo_toml(&temp_dir, false, true);

    // Create a main.rs file with a compilation error
    let bin_content = r#"
fn main() {
    // Undefined variable - compilation error
    println!("{}", undefined_variable);
}
"#;

    let mut file = File::create(temp_dir.join("src/main.rs")).expect("Failed to create main.rs");
    file.write_all(bin_content.as_bytes())
        .expect("Failed to write to main.rs");

    // Run cargo check on bin
    let result = run_cargo_check(&temp_dir, "--bins");

    // Should fail due to compilation error
    assert!(!result, "Compilation should fail with undefined variable");

    // Fix the compilation error
    let fixed_bin_content = r#"
fn main() {
    // Fixed - defined the variable
    let undefined_variable = "Hello, world!";
    println!("{}", undefined_variable);
}
"#;

    let mut file = File::create(temp_dir.join("src/main.rs")).expect("Failed to create main.rs");
    file.write_all(fixed_bin_content.as_bytes())
        .expect("Failed to write to main.rs");

    // Run cargo check on bin again
    let result = run_cargo_check(&temp_dir, "--bins");

    // Should succeed now
    assert!(result, "Compilation should succeed after fixing the error");

    cleanup();
}

/// Test that the compilation status check correctly identifies module visibility errors
#[test]
fn test_module_visibility_error() {
    setup();
    let temp_dir = PathBuf::from(TEMP_DIR);

    // Create Cargo.toml with lib
    create_cargo_toml(&temp_dir, true, false);

    // Create a lib.rs file with a module
    let lib_content = r#"
mod utils;

pub fn use_utils() -> String {
    // Trying to use a private function from utils module - visibility error
    utils::helper()
}
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Create a utils.rs file with a private function
    fs::create_dir_all(temp_dir.join("src")).expect("Failed to create src directory");
    let utils_content = r#"
// Private function
fn helper() -> String {
    "helper function".to_string()
}
"#;

    let mut file = File::create(temp_dir.join("src/utils.rs")).expect("Failed to create utils.rs");
    file.write_all(utils_content.as_bytes())
        .expect("Failed to write to utils.rs");

    // Run cargo check on lib
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should fail due to visibility error
    assert!(!result, "Compilation should fail with visibility error");

    // Fix the visibility error
    let fixed_utils_content = r#"
// Public function
pub fn helper() -> String {
    "helper function".to_string()
}
"#;

    let mut file = File::create(temp_dir.join("src/utils.rs")).expect("Failed to create utils.rs");
    file.write_all(fixed_utils_content.as_bytes())
        .expect("Failed to write to utils.rs");

    // Run cargo check on lib again
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should succeed now
    assert!(
        result,
        "Compilation should succeed after fixing the visibility error"
    );

    cleanup();
}

/// Test that the compilation status check correctly identifies trait implementation errors
#[test]
fn test_trait_implementation_error() {
    setup();
    let temp_dir = PathBuf::from(TEMP_DIR);

    // Create Cargo.toml with lib
    create_cargo_toml(&temp_dir, true, false);

    // Create a lib.rs file with a trait and incomplete implementation
    let lib_content = r#"
pub trait Greeter {
    fn greet(&self) -> String;
    fn greet_loudly(&self) -> String;
}

pub struct Person {
    name: String,
}

// Incomplete trait implementation - missing greet_loudly
impl Greeter for Person {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
    // Missing greet_loudly implementation
}
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Run cargo check on lib
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should fail due to incomplete trait implementation
    assert!(
        !result,
        "Compilation should fail with incomplete trait implementation"
    );

    // Fix the trait implementation
    let fixed_lib_content = r#"
pub trait Greeter {
    fn greet(&self) -> String;
    fn greet_loudly(&self) -> String;
}

pub struct Person {
    name: String,
}

// Complete trait implementation
impl Greeter for Person {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
    
    fn greet_loudly(&self) -> String {
        format!("HELLO, {}!!!", self.name.to_uppercase())
    }
}
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(fixed_lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Run cargo check on lib again
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should succeed now
    assert!(
        result,
        "Compilation should succeed after fixing the trait implementation"
    );

    cleanup();
}

/// Test that the compilation status check correctly identifies import path errors
#[test]
fn test_import_path_error() {
    setup();
    let temp_dir = PathBuf::from(TEMP_DIR);

    // Create directory structure
    fs::create_dir_all(temp_dir.join("src/modules")).expect("Failed to create modules directory");

    // Create Cargo.toml with lib
    create_cargo_toml(&temp_dir, true, false);

    // Create module files
    let models_content = r#"
pub struct User {
    pub name: String,
    pub age: u32,
}
"#;

    let mut file =
        File::create(temp_dir.join("src/modules/models.rs")).expect("Failed to create models.rs");
    file.write_all(models_content.as_bytes())
        .expect("Failed to write to models.rs");

    // Create mod.rs to export modules
    let mod_content = r#"
pub mod models;
"#;

    let mut file =
        File::create(temp_dir.join("src/modules/mod.rs")).expect("Failed to create mod.rs");
    file.write_all(mod_content.as_bytes())
        .expect("Failed to write to mod.rs");

    // Create lib.rs with incorrect import path
    let lib_content = r#"
// Incorrect import path
use crate::models::User;

pub fn create_user(name: &str, age: u32) -> User {
    User {
        name: name.to_string(),
        age,
    }
}
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Run cargo check on lib
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should fail due to incorrect import path
    assert!(
        !result,
        "Compilation should fail with incorrect import path"
    );

    // Fix the import path
    let fixed_lib_content = r#"
// Correct import path
use crate::modules::models::User;

pub fn create_user(name: &str, age: u32) -> User {
    User {
        name: name.to_string(),
        age,
    }
}
"#;

    let mut file = File::create(temp_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    file.write_all(fixed_lib_content.as_bytes())
        .expect("Failed to write to lib.rs");

    // Run cargo check on lib again
    let result = run_cargo_check(&temp_dir, "--lib");

    // Should succeed now
    assert!(
        result,
        "Compilation should succeed after fixing the import path"
    );

    cleanup();
}
