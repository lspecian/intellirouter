//! Tests for the Model Registry module
//!
//! This module contains tests for the Model Registry module, including
//! concurrency tests and persistence tests.

#[cfg(test)]
pub mod concurrency_tests;
#[cfg(test)]
pub mod persistence_tests;

#[cfg(test)]
use crate::modules::model_registry::*;

#[cfg(test)]
#[test]
fn test_global_registry() {
    // Clear the global registry first
    global_registry().clear();

    // Register a model
    let model = ModelMetadata::new(
        "gpt-4".to_string(),
        "GPT-4".to_string(),
        "openai".to_string(),
        "1.0".to_string(),
        "https://api.openai.com/v1/chat/completions".to_string(),
    );
    register_model(model.clone()).unwrap();

    // Get the model
    let retrieved = get_model("gpt-4").unwrap();
    assert_eq!(retrieved.id, "gpt-4");

    // List all models
    let all_models = list_models();
    assert_eq!(all_models.len(), 1);

    // Remove the model
    let removed = remove_model("gpt-4").unwrap();
    assert_eq!(removed.id, "gpt-4");

    // List all models again
    let all_models = list_models();
    assert_eq!(all_models.len(), 0);
}
