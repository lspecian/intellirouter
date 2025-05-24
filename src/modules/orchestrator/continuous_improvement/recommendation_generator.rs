//! Recommendation Generator
//!
//! This module provides the RecommendationGenerator implementation for generating
//! improvement recommendations based on analysis results.

use std::collections::HashSet;

use crate::modules::orchestrator::reporting::{
    ImprovementSuggestion, ReportGenerator, SuggestionPriority,
};
use crate::modules::orchestrator::types::OrchestratorError;

use super::types::{AnalysisRecommendation, AnalysisResult};

/// Recommendation generator
#[derive(Debug, Default)]
pub struct RecommendationGenerator {
    /// Recommendation filters
    filters: Vec<Box<dyn RecommendationFilter>>,
    /// Recommendation deduplicator
    deduplicator: RecommendationDeduplicator,
    /// Recommendation prioritizer
    prioritizer: RecommendationPrioritizer,
}

impl RecommendationGenerator {
    /// Create a new recommendation generator
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            deduplicator: RecommendationDeduplicator::new(),
            prioritizer: RecommendationPrioritizer::new(),
        }
    }

    /// Add a recommendation filter
    pub fn with_filter<F: RecommendationFilter + 'static>(mut self, filter: F) -> Self {
        self.filters.push(Box::new(filter));
        self
    }

    /// Set the recommendation deduplicator
    pub fn with_deduplicator(mut self, deduplicator: RecommendationDeduplicator) -> Self {
        self.deduplicator = deduplicator;
        self
    }

    /// Set the recommendation prioritizer
    pub fn with_prioritizer(mut self, prioritizer: RecommendationPrioritizer) -> Self {
        self.prioritizer = prioritizer;
        self
    }

    /// Generate improvement suggestions from analysis results
    pub fn generate_suggestions(
        &self,
        results: &[AnalysisResult],
    ) -> Result<Vec<ImprovementSuggestion>, OrchestratorError> {
        // Extract recommendations from analysis results
        let mut recommendations: Vec<AnalysisRecommendation> = results
            .iter()
            .flat_map(|r| r.recommendations.clone())
            .collect();

        // Apply filters
        for filter in &self.filters {
            recommendations = filter.filter_recommendations(&recommendations);
        }

        // Deduplicate recommendations
        recommendations = self
            .deduplicator
            .deduplicate_recommendations(&recommendations);

        // Prioritize recommendations
        recommendations = self
            .prioritizer
            .prioritize_recommendations(&recommendations);

        // Convert to improvement suggestions
        let suggestions: Vec<ImprovementSuggestion> = recommendations
            .iter()
            .map(|r| r.to_improvement_suggestion())
            .collect();

        Ok(suggestions)
    }

    /// Generate a report from analysis results
    pub fn generate_report(
        &self,
        results: &[AnalysisResult],
        report_generator: &ReportGenerator,
    ) -> Result<(), OrchestratorError> {
        // Generate suggestions
        let suggestions = self.generate_suggestions(results)?;

        // Add suggestions to report
        for suggestion in suggestions {
            report_generator
                .add_improvement_suggestion(suggestion)
                .map_err(|e| {
                    OrchestratorError::Other(format!("Failed to add improvement suggestion: {}", e))
                })?;
        }

        // Add analysis results to report
        for result in results {
            // Add metrics
            for (key, value) in &result.metrics {
                report_generator.add_metric(key, *value).map_err(|e| {
                    OrchestratorError::Other(format!("Failed to add metric: {}", e))
                })?;
            }

            // Add findings
            for finding in &result.findings {
                report_generator
                    .add_finding(
                        &finding.id,
                        &finding.title,
                        &finding.description,
                        format!("{:?}", finding.severity),
                    )
                    .map_err(|e| {
                        OrchestratorError::Other(format!("Failed to add finding: {}", e))
                    })?;
            }
        }

        Ok(())
    }
}

/// Recommendation filter trait
pub trait RecommendationFilter: Send + Sync + std::fmt::Debug {
    /// Filter recommendations
    fn filter_recommendations(
        &self,
        recommendations: &[AnalysisRecommendation],
    ) -> Vec<AnalysisRecommendation>;
}

/// Recommendation deduplicator
#[derive(Debug, Default)]
pub struct RecommendationDeduplicator {
    /// Similarity threshold
    similarity_threshold: f64,
}

impl RecommendationDeduplicator {
    /// Create a new recommendation deduplicator
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.8,
        }
    }

    /// Create a new recommendation deduplicator with a custom similarity threshold
    pub fn with_similarity_threshold(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
        }
    }

    /// Deduplicate recommendations
    pub fn deduplicate_recommendations(
        &self,
        recommendations: &[AnalysisRecommendation],
    ) -> Vec<AnalysisRecommendation> {
        let mut result = Vec::new();
        let mut seen_ids = HashSet::new();

        for recommendation in recommendations {
            // Skip if we've already seen this ID
            if seen_ids.contains(&recommendation.id) {
                continue;
            }

            // Check if this recommendation is similar to any we've already added
            let mut is_duplicate = false;
            for existing in &result {
                if self.are_similar(recommendation, existing) {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                seen_ids.insert(recommendation.id.clone());
                result.push(recommendation.clone());
            }
        }

        result
    }

    /// Check if two recommendations are similar
    fn are_similar(&self, a: &AnalysisRecommendation, b: &AnalysisRecommendation) -> bool {
        // Simple implementation: check if titles are similar
        // In a real implementation, this would use more sophisticated similarity metrics
        let a_words: HashSet<&str> = a.title.split_whitespace().collect();
        let b_words: HashSet<&str> = b.title.split_whitespace().collect();

        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();

        if union == 0 {
            return false;
        }

        (intersection as f64 / union as f64) >= self.similarity_threshold
    }
}

/// Recommendation prioritizer
#[derive(Debug, Default)]
pub struct RecommendationPrioritizer {
    /// Maximum number of high priority recommendations
    max_high_priority: usize,
    /// Maximum number of medium priority recommendations
    max_medium_priority: usize,
}

impl RecommendationPrioritizer {
    /// Create a new recommendation prioritizer
    pub fn new() -> Self {
        Self {
            max_high_priority: 3,
            max_medium_priority: 5,
        }
    }

    /// Create a new recommendation prioritizer with custom limits
    pub fn with_limits(max_high_priority: usize, max_medium_priority: usize) -> Self {
        Self {
            max_high_priority,
            max_medium_priority,
        }
    }

    /// Prioritize recommendations
    pub fn prioritize_recommendations(
        &self,
        recommendations: &[AnalysisRecommendation],
    ) -> Vec<AnalysisRecommendation> {
        let mut result = recommendations.to_vec();

        // Sort by priority, then by impact, then by difficulty (easier first)
        result.sort_by(|a, b| {
            let priority_cmp = b.priority.cmp(&a.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            let impact_cmp = b.impact.cmp(&a.impact);
            if impact_cmp != std::cmp::Ordering::Equal {
                return impact_cmp;
            }

            a.difficulty.cmp(&b.difficulty)
        });

        // Limit the number of high and medium priority recommendations
        let mut high_count = 0;
        let mut medium_count = 0;

        result = result
            .into_iter()
            .filter(|r| {
                match r.priority {
                    SuggestionPriority::High => {
                        if high_count < self.max_high_priority {
                            high_count += 1;
                            true
                        } else {
                            // Downgrade to medium priority
                            let mut downgraded = r.clone();
                            downgraded.priority = SuggestionPriority::Medium;
                            if medium_count < self.max_medium_priority {
                                medium_count += 1;
                                true
                            } else {
                                // Downgrade to low priority
                                downgraded.priority = SuggestionPriority::Low;
                                true
                            }
                        }
                    }
                    SuggestionPriority::Medium => {
                        if medium_count < self.max_medium_priority {
                            medium_count += 1;
                            true
                        } else {
                            // Downgrade to low priority
                            let mut downgraded = r.clone();
                            downgraded.priority = SuggestionPriority::Low;
                            true
                        }
                    }
                    SuggestionPriority::Low => true,
                }
            })
            .collect();

        result
    }
}

/// Priority filter
#[derive(Debug)]
pub struct PriorityFilter {
    /// Minimum priority
    min_priority: SuggestionPriority,
}

impl PriorityFilter {
    /// Create a new priority filter
    pub fn _new(min_priority: SuggestionPriority) -> Self {
        Self { min_priority }
    }
}

impl RecommendationFilter for PriorityFilter {
    fn filter_recommendations(
        &self,
        recommendations: &[AnalysisRecommendation],
    ) -> Vec<AnalysisRecommendation> {
        recommendations
            .iter()
            .filter(|r| r.priority >= self.min_priority)
            .cloned()
            .collect()
    }
}

/// Impact filter
#[derive(Debug)]
pub struct ImpactFilter {
    /// Minimum impact
    min_impact: super::types::EstimatedImpact,
}

impl ImpactFilter {
    /// Create a new impact filter
    pub fn _new(min_impact: super::types::EstimatedImpact) -> Self {
        Self { min_impact }
    }
}

impl RecommendationFilter for ImpactFilter {
    fn filter_recommendations(
        &self,
        recommendations: &[AnalysisRecommendation],
    ) -> Vec<AnalysisRecommendation> {
        recommendations
            .iter()
            .filter(|r| r.impact >= self.min_impact)
            .cloned()
            .collect()
    }
}

/// Difficulty filter
#[derive(Debug)]
pub struct DifficultyFilter {
    /// Maximum difficulty
    max_difficulty: super::types::ImplementationDifficulty,
}

impl DifficultyFilter {
    /// Create a new difficulty filter
    pub fn _new(max_difficulty: super::types::ImplementationDifficulty) -> Self {
        Self { max_difficulty }
    }
}

impl RecommendationFilter for DifficultyFilter {
    fn filter_recommendations(
        &self,
        recommendations: &[AnalysisRecommendation],
    ) -> Vec<AnalysisRecommendation> {
        recommendations
            .iter()
            .filter(|r| r.difficulty <= self.max_difficulty)
            .cloned()
            .collect()
    }
}
