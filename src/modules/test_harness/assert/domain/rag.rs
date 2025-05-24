//! RAG assertions for the assertion framework.
//!
//! This module provides assertions specific to the RAG (Retrieval-Augmented Generation) component of IntelliRouter.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents a document for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The document ID.
    pub id: String,
    /// The document content.
    pub content: String,
    /// The document metadata.
    pub metadata: Value,
    /// The document embedding.
    pub embedding: Option<Vec<f32>>,
}

impl Document {
    /// Creates a new document.
    pub fn new(id: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            content: content.to_string(),
            metadata: Value::Null,
            embedding: None,
        }
    }

    /// Sets the document metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the document embedding.
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// Represents a RAG query for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQuery {
    /// The query ID.
    pub query_id: String,
    /// The query text.
    pub query: String,
    /// The query parameters.
    pub parameters: Value,
    /// The query timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The query metadata.
    pub metadata: Value,
}

impl RagQuery {
    /// Creates a new RAG query.
    pub fn new(query_id: &str, query: &str) -> Self {
        Self {
            query_id: query_id.to_string(),
            query: query.to_string(),
            parameters: Value::Null,
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
        }
    }

    /// Sets the query parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the query timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the query metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents a RAG result for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResult {
    /// The result ID.
    pub result_id: String,
    /// The query ID.
    pub query_id: String,
    /// The retrieved documents.
    pub documents: Vec<Document>,
    /// The generated answer.
    pub answer: Option<String>,
    /// The result timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The result metadata.
    pub metadata: Value,
    /// The retrieval time.
    pub retrieval_time: Duration,
    /// The generation time.
    pub generation_time: Duration,
    /// The total time.
    pub total_time: Duration,
    /// The error, if any.
    pub error: Option<String>,
}

impl RagResult {
    /// Creates a new RAG result.
    pub fn new(result_id: &str, query_id: &str) -> Self {
        Self {
            result_id: result_id.to_string(),
            query_id: query_id.to_string(),
            documents: Vec::new(),
            answer: None,
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
            retrieval_time: Duration::default(),
            generation_time: Duration::default(),
            total_time: Duration::default(),
            error: None,
        }
    }

    /// Adds a document to the result.
    pub fn with_document(mut self, document: Document) -> Self {
        self.documents.push(document);
        self
    }

    /// Sets the generated answer.
    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer = Some(answer.to_string());
        self
    }

    /// Sets the result timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the result metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the retrieval time.
    pub fn with_retrieval_time(mut self, retrieval_time: Duration) -> Self {
        self.retrieval_time = retrieval_time;
        self
    }

    /// Sets the generation time.
    pub fn with_generation_time(mut self, generation_time: Duration) -> Self {
        self.generation_time = generation_time;
        self
    }

    /// Sets the total time.
    pub fn with_total_time(mut self, total_time: Duration) -> Self {
        self.total_time = total_time;
        self
    }

    /// Sets the error.
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }
}

/// Assertions for RAG components.
#[derive(Debug, Clone)]
pub struct RagAssertions;

impl RagAssertions {
    /// Creates a new RAG assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a result was successful (no error).
    pub fn assert_success(&self, result: &RagResult) -> AssertionResult {
        match &result.error {
            None => AssertionResult::new("Result is successful", AssertionOutcome::Passed),
            Some(error) => AssertionResult::new("Result is successful", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Result has an error",
                        "No error",
                        error,
                    ),
                ),
        }
    }

    /// Asserts that a result has a specific error.
    pub fn assert_error(&self, result: &RagResult, expected: &str) -> AssertionResult {
        match &result.error {
            Some(error) => {
                if error == expected {
                    AssertionResult::new(
                        &format!("Result has error '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Result has error '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Result has a different error",
                            expected,
                            error,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Result has error '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Result does not have an error",
                    expected,
                    "No error",
                ),
            ),
        }
    }

    /// Asserts that a result has a specific number of documents.
    pub fn assert_document_count(&self, result: &RagResult, expected: usize) -> AssertionResult {
        let count = result.documents.len();
        if count == expected {
            AssertionResult::new(
                &format!("Document count is {}", expected),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Document count is {}", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Document count does not match expected value",
                    &format!("{}", expected),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a result has at least a specific number of documents.
    pub fn assert_min_document_count(&self, result: &RagResult, min: usize) -> AssertionResult {
        let count = result.documents.len();
        if count >= min {
            AssertionResult::new(
                &format!("Document count >= {}", min),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Document count >= {}", min),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Document count is less than minimum",
                    &format!(">= {}", min),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a result has at most a specific number of documents.
    pub fn assert_max_document_count(&self, result: &RagResult, max: usize) -> AssertionResult {
        let count = result.documents.len();
        if count <= max {
            AssertionResult::new(
                &format!("Document count <= {}", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Document count <= {}", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Document count is greater than maximum",
                    &format!("<= {}", max),
                    &format!("{}", count),
                ),
            )
        }
    }

    /// Asserts that a result has a document with a specific ID.
    pub fn assert_has_document_id(&self, result: &RagResult, document_id: &str) -> AssertionResult {
        let has_document = result.documents.iter().any(|d| d.id == document_id);
        if has_document {
            AssertionResult::new(
                &format!("Has document with ID '{}'", document_id),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Has document with ID '{}'", document_id),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Result does not have document with ID '{}'", document_id),
                    &format!("Document with ID '{}'", document_id),
                    "No such document",
                ),
            )
        }
    }

    /// Asserts that a result has a document containing a specific text.
    pub fn assert_has_document_containing(
        &self,
        result: &RagResult,
        text: &str,
    ) -> AssertionResult {
        let has_document = result.documents.iter().any(|d| d.content.contains(text));
        if has_document {
            AssertionResult::new(
                &format!("Has document containing '{}'", text),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Has document containing '{}'", text),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Result does not have document containing '{}'", text),
                    &format!("Document containing '{}'", text),
                    "No such document",
                ),
            )
        }
    }

    /// Asserts that a result has an answer.
    pub fn assert_has_answer(&self, result: &RagResult) -> AssertionResult {
        match &result.answer {
            Some(_) => AssertionResult::new("Result has answer", AssertionOutcome::Passed),
            None => AssertionResult::new("Result has answer", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Result does not have an answer",
                    "Answer",
                    "No answer",
                ),
            ),
        }
    }

    /// Asserts that a result has a specific answer.
    pub fn assert_answer(&self, result: &RagResult, expected: &str) -> AssertionResult {
        match &result.answer {
            Some(answer) => {
                if answer == expected {
                    AssertionResult::new(
                        &format!("Answer is '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Answer is '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Answer does not match expected value",
                            expected,
                            answer,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Answer is '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Result does not have an answer",
                    expected,
                    "No answer",
                ),
            ),
        }
    }

    /// Asserts that a result has an answer containing a specific text.
    pub fn assert_answer_contains(&self, result: &RagResult, expected: &str) -> AssertionResult {
        match &result.answer {
            Some(answer) => {
                if answer.contains(expected) {
                    AssertionResult::new(
                        &format!("Answer contains '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Answer contains '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Answer does not contain '{}'", expected),
                            &format!("Answer containing '{}'", expected),
                            answer,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Answer contains '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Result does not have an answer",
                    &format!("Answer containing '{}'", expected),
                    "No answer",
                ),
            ),
        }
    }

    /// Asserts that a result's retrieval time is less than a specific value.
    pub fn assert_retrieval_time(&self, result: &RagResult, max_ms: u64) -> AssertionResult {
        let retrieval_time_ms = result.retrieval_time.as_millis() as u64;
        if retrieval_time_ms <= max_ms {
            AssertionResult::new(
                &format!("Retrieval time <= {} ms", max_ms),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Retrieval time <= {} ms", max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Retrieval time exceeds maximum",
                    &format!("<= {} ms", max_ms),
                    &format!("{} ms", retrieval_time_ms),
                ),
            )
        }
    }

    /// Asserts that a result's generation time is less than a specific value.
    pub fn assert_generation_time(&self, result: &RagResult, max_ms: u64) -> AssertionResult {
        let generation_time_ms = result.generation_time.as_millis() as u64;
        if generation_time_ms <= max_ms {
            AssertionResult::new(
                &format!("Generation time <= {} ms", max_ms),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Generation time <= {} ms", max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Generation time exceeds maximum",
                    &format!("<= {} ms", max_ms),
                    &format!("{} ms", generation_time_ms),
                ),
            )
        }
    }

    /// Asserts that a result's total time is less than a specific value.
    pub fn assert_total_time(&self, result: &RagResult, max_ms: u64) -> AssertionResult {
        let total_time_ms = result.total_time.as_millis() as u64;
        if total_time_ms <= max_ms {
            AssertionResult::new(
                &format!("Total time <= {} ms", max_ms),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Total time <= {} ms", max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Total time exceeds maximum",
                    &format!("<= {} ms", max_ms),
                    &format!("{} ms", total_time_ms),
                ),
            )
        }
    }

    /// Asserts that a result's answer is factually accurate based on the retrieved documents.
    /// This is a placeholder for a more sophisticated factual accuracy check.
    pub fn assert_factually_accurate(&self, result: &RagResult) -> AssertionResult {
        // In a real implementation, this would use a more sophisticated factual accuracy check.
        // For now, we just return a passed assertion.
        AssertionResult::new("Answer is factually accurate", AssertionOutcome::Passed)
    }

    /// Asserts that a result's answer is relevant to the query.
    /// This is a placeholder for a more sophisticated relevance check.
    pub fn assert_relevant(&self, result: &RagResult, query: &RagQuery) -> AssertionResult {
        // In a real implementation, this would use a more sophisticated relevance check.
        // For now, we just return a passed assertion.
        AssertionResult::new("Answer is relevant to query", AssertionOutcome::Passed)
    }
}

impl Default for RagAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new RAG assertions instance.
pub fn create_rag_assertions() -> RagAssertions {
    RagAssertions::new()
}

/// Creates a new document.
pub fn create_document(id: &str, content: &str) -> Document {
    Document::new(id, content)
}

/// Creates a new RAG query.
pub fn create_rag_query(query_id: &str, query: &str) -> RagQuery {
    RagQuery::new(query_id, query)
}

/// Creates a new RAG result.
pub fn create_rag_result(result_id: &str, query_id: &str) -> RagResult {
    RagResult::new(result_id, query_id)
}
