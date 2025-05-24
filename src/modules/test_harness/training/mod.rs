//! Training Materials Generator Module
//!
//! This module provides functionality for generating training materials for the test harness.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, error, info, warn};

use super::docs::{DocumentationFormat, DocumentationSection};
use crate::modules::test_harness::types::TestHarnessError;

/// Training material type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingMaterialType {
    /// Tutorial
    Tutorial,
    /// Workshop
    Workshop,
    /// Exercise
    Exercise,
    /// Quiz
    Quiz,
    /// Cheat sheet
    CheatSheet,
}

impl fmt::Display for TrainingMaterialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrainingMaterialType::Tutorial => write!(f, "Tutorial"),
            TrainingMaterialType::Workshop => write!(f, "Workshop"),
            TrainingMaterialType::Exercise => write!(f, "Exercise"),
            TrainingMaterialType::Quiz => write!(f, "Quiz"),
            TrainingMaterialType::CheatSheet => write!(f, "Cheat Sheet"),
        }
    }
}

/// Training material difficulty
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingDifficulty {
    /// Beginner difficulty
    Beginner,
    /// Intermediate difficulty
    Intermediate,
    /// Advanced difficulty
    Advanced,
    /// Expert difficulty
    Expert,
}

impl fmt::Display for TrainingDifficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrainingDifficulty::Beginner => write!(f, "Beginner"),
            TrainingDifficulty::Intermediate => write!(f, "Intermediate"),
            TrainingDifficulty::Advanced => write!(f, "Advanced"),
            TrainingDifficulty::Expert => write!(f, "Expert"),
        }
    }
}

/// Training material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMaterial {
    /// Material ID
    pub id: String,
    /// Material title
    pub title: String,
    /// Material description
    pub description: String,
    /// Material type
    pub material_type: TrainingMaterialType,
    /// Material difficulty
    pub difficulty: TrainingDifficulty,
    /// Material duration in minutes
    pub duration_minutes: u32,
    /// Material prerequisites
    pub prerequisites: Vec<String>,
    /// Material content
    pub content: String,
    /// Material sections
    pub sections: Vec<DocumentationSection>,
    /// Material metadata
    pub metadata: HashMap<String, String>,
}

impl TrainingMaterial {
    /// Create a new training material
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        material_type: TrainingMaterialType,
        difficulty: TrainingDifficulty,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            material_type,
            difficulty,
            duration_minutes: 0,
            prerequisites: Vec::new(),
            content: String::new(),
            sections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the material duration
    pub fn with_duration(mut self, duration_minutes: u32) -> Self {
        self.duration_minutes = duration_minutes;
        self
    }

    /// Add a prerequisite to the material
    pub fn with_prerequisite(mut self, prerequisite: impl Into<String>) -> Self {
        self.prerequisites.push(prerequisite.into());
        self
    }

    /// Add multiple prerequisites to the material
    pub fn with_prerequisites(
        mut self,
        prerequisites: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        for prerequisite in prerequisites {
            self.prerequisites.push(prerequisite.into());
        }
        self
    }

    /// Set the material content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Add a section to the material
    pub fn with_section(mut self, section: DocumentationSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add multiple sections to the material
    pub fn with_sections(mut self, sections: Vec<DocumentationSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add metadata to the material
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Render the material to markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // Add title
        result.push_str(&format!("# {}\n\n", self.title));

        // Add metadata
        result.push_str(&format!("**Type:** {}\n\n", self.material_type));
        result.push_str(&format!("**Difficulty:** {}\n\n", self.difficulty));
        result.push_str(&format!(
            "**Duration:** {} minutes\n\n",
            self.duration_minutes
        ));

        // Add description
        result.push_str(&format!("## Description\n\n{}\n\n", self.description));

        // Add prerequisites
        if !self.prerequisites.is_empty() {
            result.push_str("## Prerequisites\n\n");

            for prerequisite in &self.prerequisites {
                result.push_str(&format!("- {}\n", prerequisite));
            }

            result.push_str("\n");
        }

        // Add content
        if !self.content.is_empty() {
            result.push_str("## Content\n\n");
            result.push_str(&self.content);
            result.push_str("\n\n");
        }

        // Add sections
        for section in &self.sections {
            result.push_str(&section.to_markdown(2));
        }

        result
    }

    /// Render the material to HTML
    pub fn to_html(&self) -> String {
        let mut result = String::new();

        // Add HTML header
        result.push_str("<!DOCTYPE html>\n");
        result.push_str("<html lang=\"en\">\n");
        result.push_str("<head>\n");
        result.push_str("  <meta charset=\"UTF-8\">\n");
        result.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        result.push_str(&format!("  <title>{}</title>\n", self.title));
        result.push_str("  <style>\n");
        result.push_str("    body { font-family: Arial, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
        result.push_str("    h1, h2, h3, h4, h5, h6 { color: #333; }\n");
        result.push_str(
            "    code { background-color: #f4f4f4; padding: 2px 5px; border-radius: 3px; }\n",
        );
        result.push_str("    pre { background-color: #f4f4f4; padding: 10px; border-radius: 5px; overflow-x: auto; }\n");
        result.push_str("    .metadata { background-color: #f0f0f0; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
        result.push_str("  </style>\n");
        result.push_str("</head>\n");
        result.push_str("<body>\n");

        // Add title
        result.push_str(&format!("<h1>{}</h1>\n\n", self.title));

        // Add metadata
        result.push_str("<div class=\"metadata\">\n");
        result.push_str("<ul>\n");
        result.push_str(&format!(
            "  <li><strong>Type:</strong> {}</li>\n",
            self.material_type
        ));
        result.push_str(&format!(
            "  <li><strong>Difficulty:</strong> {}</li>\n",
            self.difficulty
        ));
        result.push_str(&format!(
            "  <li><strong>Duration:</strong> {} minutes</li>\n",
            self.duration_minutes
        ));
        result.push_str("</ul>\n");
        result.push_str("</div>\n\n");

        // Add description
        result.push_str("<h2>Description</h2>\n");
        result.push_str(&format!("<p>{}</p>\n\n", self.description));

        // Add prerequisites
        if !self.prerequisites.is_empty() {
            result.push_str("<h2>Prerequisites</h2>\n");
            result.push_str("<ul>\n");

            for prerequisite in &self.prerequisites {
                result.push_str(&format!("  <li>{}</li>\n", prerequisite));
            }

            result.push_str("</ul>\n\n");
        }

        // Add content
        if !self.content.is_empty() {
            result.push_str("<h2>Content</h2>\n");
            result.push_str(&format!(
                "<div class=\"content\">{}</div>\n\n",
                self.content
            ));
        }

        // Add sections
        for section in &self.sections {
            result.push_str(&format!("<div id=\"{}\" class=\"section\">\n", section.id));
            result.push_str(&section.to_html(2));
            result.push_str("</div>\n\n");
        }

        // Add HTML footer
        result.push_str("</body>\n");
        result.push_str("</html>\n");

        result
    }
}

/// Training course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCourse {
    /// Course ID
    pub id: String,
    /// Course title
    pub title: String,
    /// Course description
    pub description: String,
    /// Course materials
    pub materials: Vec<TrainingMaterial>,
    /// Course metadata
    pub metadata: HashMap<String, String>,
}

impl TrainingCourse {
    /// Create a new training course
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            materials: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a material to the course
    pub fn with_material(mut self, material: TrainingMaterial) -> Self {
        self.materials.push(material);
        self
    }

    /// Add multiple materials to the course
    pub fn with_materials(mut self, materials: Vec<TrainingMaterial>) -> Self {
        self.materials.extend(materials);
        self
    }

    /// Add metadata to the course
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Render the course to markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // Add title
        result.push_str(&format!("# {}\n\n", self.title));

        // Add description
        result.push_str(&format!("{}\n\n", self.description));

        // Add table of contents
        result.push_str("## Table of Contents\n\n");

        for (i, material) in self.materials.iter().enumerate() {
            result.push_str(&format!(
                "{}. [{}]({})\n",
                i + 1,
                material.title,
                material.id
            ));
        }

        result.push_str("\n");

        // Add materials
        for material in &self.materials {
            result.push_str(&format!("## {}\n\n", material.title));
            result.push_str(&format!("**Type:** {}\n\n", material.material_type));
            result.push_str(&format!("**Difficulty:** {}\n\n", material.difficulty));
            result.push_str(&format!(
                "**Duration:** {} minutes\n\n",
                material.duration_minutes
            ));
            result.push_str(&format!("{}\n\n", material.description));
            result.push_str(&format!("[Go to material]({})\n\n", material.id));
        }

        result
    }
}

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Training title
    pub title: String,
    /// Training description
    pub description: Option<String>,
    /// Training version
    pub version: String,
    /// Training author
    pub author: Option<String>,
    /// Training output directory
    pub output_dir: PathBuf,
    /// Training formats
    pub formats: Vec<DocumentationFormat>,
    /// Training template directory
    pub template_dir: Option<PathBuf>,
    /// Training assets directory
    pub assets_dir: Option<PathBuf>,
    /// Training metadata
    pub metadata: HashMap<String, String>,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            title: "IntelliRouter Test Harness Training".to_string(),
            description: Some("Training materials for the IntelliRouter test harness".to_string()),
            version: "1.0.0".to_string(),
            author: Some("IntelliRouter Team".to_string()),
            output_dir: PathBuf::from("training"),
            formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
            template_dir: None,
            assets_dir: None,
            metadata: HashMap::new(),
        }
    }
}

/// Training generator
pub struct TrainingGenerator {
    /// Training configuration
    config: TrainingConfig,
}

impl TrainingGenerator {
    /// Create a new training generator
    pub fn new(config: TrainingConfig) -> Self {
        Self { config }
    }

    /// Generate training materials
    pub async fn generate(&self, courses: Vec<TrainingCourse>) -> Result<(), TestHarnessError> {
        info!("Generating training materials");

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)
            .await
            .map_err(|e| {
                TestHarnessError::IoError(format!("Failed to create output directory: {}", e))
            })?;

        // Generate course files
        for course in &courses {
            self.generate_course(course).await?;
        }

        info!("Training materials generated successfully");

        Ok(())
    }

    /// Generate course files
    async fn generate_course(&self, course: &TrainingCourse) -> Result<(), TestHarnessError> {
        // Create course directory
        let course_dir = self.config.output_dir.join(&course.id);
        fs::create_dir_all(&course_dir).await.map_err(|e| {
            TestHarnessError::IoError(format!("Failed to create course directory: {}", e))
        })?;

        // Generate course files
        for format in &self.config.formats {
            match format {
                DocumentationFormat::Markdown => {
                    let markdown = course.to_markdown();
                    let path = course_dir.join(format!("index.{}", format.extension()));

                    fs::write(&path, markdown).await.map_err(|e| {
                        TestHarnessError::IoError(format!("Failed to write markdown file: {}", e))
                    })?;

                    info!("Generated markdown course: {:?}", path);
                }
                _ => {
                    warn!("Documentation format not implemented: {}", format);
                }
            }
        }

        // Generate material files
        for material in &course.materials {
            self.generate_material(course, material).await?;
        }

        Ok(())
    }

    /// Generate material files
    async fn generate_material(
        &self,
        course: &TrainingCourse,
        material: &TrainingMaterial,
    ) -> Result<(), TestHarnessError> {
        // Create material directory
        let material_dir = self.config.output_dir.join(&course.id);

        // Generate material files
        for format in &self.config.formats {
            match format {
                DocumentationFormat::Markdown => {
                    let markdown = material.to_markdown();
                    let path = material_dir.join(format!("{}.{}", material.id, format.extension()));

                    fs::write(&path, markdown).await.map_err(|e| {
                        TestHarnessError::IoError(format!("Failed to write markdown file: {}", e))
                    })?;

                    info!("Generated markdown material: {:?}", path);
                }
                _ => {
                    warn!("Documentation format not implemented: {}", format);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::docs::DocumentationSection;
    use super::*;

    #[test]
    fn test_training_material() {
        let material = TrainingMaterial::new(
            "getting-started",
            "Getting Started with the Test Harness",
            "Learn how to use the test harness for basic testing",
            TrainingMaterialType::Tutorial,
            TrainingDifficulty::Beginner,
        )
        .with_duration(30)
        .with_prerequisite("Basic Rust knowledge")
        .with_prerequisite("IntelliRouter installed")
        .with_content("This is the content of the tutorial.")
        .with_section(DocumentationSection::new(
            "installation",
            "Installation",
            "How to install the test harness",
        ))
        .with_section(DocumentationSection::new(
            "configuration",
            "Configuration",
            "How to configure the test harness",
        ));

        assert_eq!(material.id, "getting-started");
        assert_eq!(material.title, "Getting Started with the Test Harness");
        assert_eq!(
            material.description,
            "Learn how to use the test harness for basic testing"
        );
        assert_eq!(material.material_type, TrainingMaterialType::Tutorial);
        assert_eq!(material.difficulty, TrainingDifficulty::Beginner);
        assert_eq!(material.duration_minutes, 30);
        assert_eq!(
            material.prerequisites,
            vec!["Basic Rust knowledge", "IntelliRouter installed"]
        );
        assert_eq!(material.content, "This is the content of the tutorial.");
        assert_eq!(material.sections.len(), 2);
    }
}
