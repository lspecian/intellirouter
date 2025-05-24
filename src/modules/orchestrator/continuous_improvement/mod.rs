//! Continuous Improvement System
//!
//! This module provides functionality for analyzing testing results and suggesting
//! improvements to the testing strategy.

mod analyzer;
mod coverage_analyzer;
mod error_analyzer;
mod performance_analyzer;
mod recommendation_generator;
mod reporting;
mod types;

// Re-export types
pub use types::{
    AnalysisFinding, AnalysisRecommendation, AnalysisResult, EstimatedImpact, FindingSeverity,
    ImplementationDifficulty,
};

// Re-export analyzer trait
pub use analyzer::Analyzer;

// Re-export reporting traits
pub use reporting::{OrchestratorReporting, QualityAnalyzer};

// Re-export analyzer implementations
pub use coverage_analyzer::CoverageAnalyzer;
pub use error_analyzer::ErrorAnalyzer;
pub use performance_analyzer::PerformanceAnalyzer;

// Re-export recommendation generator
pub use recommendation_generator::RecommendationGenerator;

/// Continuous Improvement System
///
/// This struct coordinates the analysis of testing results and generation of improvement
/// suggestions.
#[derive(Debug)]
pub struct ContinuousImprovementSystem {
    /// Analyzers
    analyzers: Vec<Box<dyn Analyzer>>,
    /// Recommendation generator
    recommendation_generator: RecommendationGenerator,
}

impl ContinuousImprovementSystem {
    /// Create a new continuous improvement system
    pub fn new() -> Self {
        Self {
            analyzers: Vec::new(),
            recommendation_generator: RecommendationGenerator::new(),
        }
    }

    /// Add an analyzer
    pub fn with_analyzer<A: Analyzer + 'static>(mut self, analyzer: A) -> Self {
        self.analyzers.push(Box::new(analyzer));
        self
    }

    /// Set the recommendation generator
    pub fn with_recommendation_generator(mut self, generator: RecommendationGenerator) -> Self {
        self.recommendation_generator = generator;
        self
    }

    /// Get a reference to the analyzers
    pub fn analyzers(&self) -> &[Box<dyn Analyzer>] {
        &self.analyzers
    }

    /// Get a reference to the recommendation generator
    pub fn recommendation_generator(&self) -> &RecommendationGenerator {
        &self.recommendation_generator
    }
}

impl Default for ContinuousImprovementSystem {
    fn default() -> Self {
        Self::new()
            .with_analyzer(PerformanceAnalyzer::new())
            .with_analyzer(CoverageAnalyzer::new())
            .with_analyzer(ErrorAnalyzer::new())
    }
}
