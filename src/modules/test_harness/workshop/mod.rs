//! Team Workshop Module
//!
//! This module provides functionality for conducting interactive workshops for the test harness.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, error, info, warn};

use super::docs::{DocumentationFormat, DocumentationSection};
use super::training::{TrainingDifficulty, TrainingMaterial};
use crate::modules::test_harness::types::TestHarnessError;

/// Workshop activity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkshopActivityType {
    /// Presentation
    Presentation,
    /// Demonstration
    Demonstration,
    /// Hands-on exercise
    HandsOn,
    /// Group discussion
    Discussion,
    /// Quiz
    Quiz,
    /// Code review
    CodeReview,
    /// Pair programming
    PairProgramming,
}

impl fmt::Display for WorkshopActivityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkshopActivityType::Presentation => write!(f, "Presentation"),
            WorkshopActivityType::Demonstration => write!(f, "Demonstration"),
            WorkshopActivityType::HandsOn => write!(f, "Hands-on Exercise"),
            WorkshopActivityType::Discussion => write!(f, "Group Discussion"),
            WorkshopActivityType::Quiz => write!(f, "Quiz"),
            WorkshopActivityType::CodeReview => write!(f, "Code Review"),
            WorkshopActivityType::PairProgramming => write!(f, "Pair Programming"),
        }
    }
}

/// Workshop activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopActivity {
    /// Activity ID
    pub id: String,
    /// Activity title
    pub title: String,
    /// Activity description
    pub description: String,
    /// Activity type
    pub activity_type: WorkshopActivityType,
    /// Activity duration in minutes
    pub duration_minutes: u32,
    /// Activity content
    pub content: String,
    /// Activity materials
    pub materials: Vec<TrainingMaterial>,
    /// Activity sections
    pub sections: Vec<DocumentationSection>,
    /// Activity metadata
    pub metadata: HashMap<String, String>,
}

impl WorkshopActivity {
    /// Create a new workshop activity
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        activity_type: WorkshopActivityType,
        duration_minutes: u32,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            activity_type,
            duration_minutes,
            content: String::new(),
            materials: Vec::new(),
            sections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the activity content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Add a material to the activity
    pub fn with_material(mut self, material: TrainingMaterial) -> Self {
        self.materials.push(material);
        self
    }

    /// Add multiple materials to the activity
    pub fn with_materials(mut self, materials: Vec<TrainingMaterial>) -> Self {
        self.materials.extend(materials);
        self
    }

    /// Add a section to the activity
    pub fn with_section(mut self, section: DocumentationSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add multiple sections to the activity
    pub fn with_sections(mut self, sections: Vec<DocumentationSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add metadata to the activity
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Render the activity to markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // Add title
        result.push_str(&format!("# {}\n\n", self.title));

        // Add metadata
        result.push_str(&format!("**Type:** {}\n\n", self.activity_type));
        result.push_str(&format!(
            "**Duration:** {} minutes\n\n",
            self.duration_minutes
        ));

        // Add description
        result.push_str(&format!("## Description\n\n{}\n\n", self.description));

        // Add content
        if !self.content.is_empty() {
            result.push_str("## Content\n\n");
            result.push_str(&self.content);
            result.push_str("\n\n");
        }

        // Add materials
        if !self.materials.is_empty() {
            result.push_str("## Materials\n\n");

            for material in &self.materials {
                result.push_str(&format!("### {}\n\n", material.title));
                result.push_str(&format!("**Type:** {}\n\n", material.material_type));
                result.push_str(&format!("**Difficulty:** {}\n\n", material.difficulty));
                result.push_str(&format!("{}\n\n", material.description));
            }
        }

        // Add sections
        for section in &self.sections {
            result.push_str(&section.to_markdown(2));
        }

        result
    }
}

/// Workshop session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopSession {
    /// Session ID
    pub id: String,
    /// Session title
    pub title: String,
    /// Session description
    pub description: String,
    /// Session difficulty
    pub difficulty: TrainingDifficulty,
    /// Session duration in minutes
    pub duration_minutes: u32,
    /// Session activities
    pub activities: Vec<WorkshopActivity>,
    /// Session prerequisites
    pub prerequisites: Vec<String>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl WorkshopSession {
    /// Create a new workshop session
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        difficulty: TrainingDifficulty,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            difficulty,
            duration_minutes: 0,
            activities: Vec::new(),
            prerequisites: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the session duration
    pub fn with_duration(mut self, duration_minutes: u32) -> Self {
        self.duration_minutes = duration_minutes;
        self
    }

    /// Add an activity to the session
    pub fn with_activity(mut self, activity: WorkshopActivity) -> Self {
        self.duration_minutes += activity.duration_minutes;
        self.activities.push(activity);
        self
    }

    /// Add multiple activities to the session
    pub fn with_activities(mut self, activities: Vec<WorkshopActivity>) -> Self {
        for activity in &activities {
            self.duration_minutes += activity.duration_minutes;
        }
        self.activities.extend(activities);
        self
    }

    /// Add a prerequisite to the session
    pub fn with_prerequisite(mut self, prerequisite: impl Into<String>) -> Self {
        self.prerequisites.push(prerequisite.into());
        self
    }

    /// Add multiple prerequisites to the session
    pub fn with_prerequisites(
        mut self,
        prerequisites: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        for prerequisite in prerequisites {
            self.prerequisites.push(prerequisite.into());
        }
        self
    }

    /// Add metadata to the session
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Render the session to markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // Add title
        result.push_str(&format!("# {}\n\n", self.title));

        // Add metadata
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

        // Add agenda
        result.push_str("## Agenda\n\n");

        let mut current_time = 0;
        for (i, activity) in self.activities.iter().enumerate() {
            let start_time = current_time;
            let end_time = current_time + activity.duration_minutes;

            result.push_str(&format!(
                "{}. **{}** ({} min, {:02}:{:02} - {:02}:{:02})\n",
                i + 1,
                activity.title,
                activity.duration_minutes,
                start_time / 60,
                start_time % 60,
                end_time / 60,
                end_time % 60
            ));
            result.push_str(&format!("   {}\n", activity.description));

            current_time = end_time;
        }

        result.push_str("\n");

        // Add activities
        result.push_str("## Activities\n\n");

        for activity in &self.activities {
            result.push_str(&format!("### {}\n\n", activity.title));
            result.push_str(&format!("**Type:** {}\n\n", activity.activity_type));
            result.push_str(&format!(
                "**Duration:** {} minutes\n\n",
                activity.duration_minutes
            ));
            result.push_str(&format!("{}\n\n", activity.description));

            if !activity.content.is_empty() {
                result.push_str("#### Content\n\n");
                result.push_str(&activity.content);
                result.push_str("\n\n");
            }
        }

        result
    }
}

/// Workshop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workshop {
    /// Workshop ID
    pub id: String,
    /// Workshop title
    pub title: String,
    /// Workshop description
    pub description: String,
    /// Workshop sessions
    pub sessions: Vec<WorkshopSession>,
    /// Workshop metadata
    pub metadata: HashMap<String, String>,
}

impl Workshop {
    /// Create a new workshop
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            sessions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a session to the workshop
    pub fn with_session(mut self, session: WorkshopSession) -> Self {
        self.sessions.push(session);
        self
    }

    /// Add multiple sessions to the workshop
    pub fn with_sessions(mut self, sessions: Vec<WorkshopSession>) -> Self {
        self.sessions.extend(sessions);
        self
    }

    /// Add metadata to the workshop
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Calculate the total duration of the workshop
    pub fn total_duration(&self) -> u32 {
        self.sessions.iter().map(|s| s.duration_minutes).sum()
    }

    /// Render the workshop to markdown
    pub fn to_markdown(&self) -> String {
        let mut result = String::new();

        // Add title
        result.push_str(&format!("# {}\n\n", self.title));

        // Add description
        result.push_str(&format!("{}\n\n", self.description));

        // Add metadata
        result.push_str(&format!(
            "**Total Duration:** {} minutes\n\n",
            self.total_duration()
        ));

        // Add sessions
        result.push_str("## Sessions\n\n");

        for (i, session) in self.sessions.iter().enumerate() {
            result.push_str(&format!("### Session {}: {}\n\n", i + 1, session.title));
            result.push_str(&format!("**Difficulty:** {}\n\n", session.difficulty));
            result.push_str(&format!(
                "**Duration:** {} minutes\n\n",
                session.duration_minutes
            ));
            result.push_str(&format!("{}\n\n", session.description));
            result.push_str(&format!("[Go to session]({})\n\n", session.id));
        }

        result
    }
}

/// Workshop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopConfig {
    /// Workshop title
    pub title: String,
    /// Workshop description
    pub description: Option<String>,
    /// Workshop version
    pub version: String,
    /// Workshop author
    pub author: Option<String>,
    /// Workshop output directory
    pub output_dir: PathBuf,
    /// Workshop formats
    pub formats: Vec<DocumentationFormat>,
    /// Workshop template directory
    pub template_dir: Option<PathBuf>,
    /// Workshop assets directory
    pub assets_dir: Option<PathBuf>,
    /// Workshop metadata
    pub metadata: HashMap<String, String>,
}

impl Default for WorkshopConfig {
    fn default() -> Self {
        Self {
            title: "IntelliRouter Test Harness Workshop".to_string(),
            description: Some("Workshop materials for the IntelliRouter test harness".to_string()),
            version: "1.0.0".to_string(),
            author: Some("IntelliRouter Team".to_string()),
            output_dir: PathBuf::from("workshops"),
            formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
            template_dir: None,
            assets_dir: None,
            metadata: HashMap::new(),
        }
    }
}

/// Workshop generator
pub struct WorkshopGenerator {
    /// Workshop configuration
    config: WorkshopConfig,
}

impl WorkshopGenerator {
    /// Create a new workshop generator
    pub fn new(config: WorkshopConfig) -> Self {
        Self { config }
    }

    /// Generate workshop materials
    pub async fn generate(&self, workshops: Vec<Workshop>) -> Result<(), TestHarnessError> {
        info!("Generating workshop materials");

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)
            .await
            .map_err(|e| {
                TestHarnessError::IoError(format!("Failed to create output directory: {}", e))
            })?;

        // Generate workshop files
        for workshop in &workshops {
            self.generate_workshop(workshop).await?;
        }

        info!("Workshop materials generated successfully");

        Ok(())
    }

    /// Generate workshop files
    async fn generate_workshop(&self, workshop: &Workshop) -> Result<(), TestHarnessError> {
        // Create workshop directory
        let workshop_dir = self.config.output_dir.join(&workshop.id);
        fs::create_dir_all(&workshop_dir).await.map_err(|e| {
            TestHarnessError::IoError(format!("Failed to create workshop directory: {}", e))
        })?;

        // Generate workshop files
        for format in &self.config.formats {
            match format {
                DocumentationFormat::Markdown => {
                    let markdown = workshop.to_markdown();
                    let path = workshop_dir.join(format!("index.{}", format.extension()));

                    fs::write(&path, markdown).await.map_err(|e| {
                        TestHarnessError::IoError(format!("Failed to write markdown file: {}", e))
                    })?;

                    info!("Generated markdown workshop: {:?}", path);
                }
                _ => {
                    warn!("Documentation format not implemented: {}", format);
                }
            }
        }

        // Generate session files
        for session in &workshop.sessions {
            self.generate_session(workshop, session).await?;
        }

        Ok(())
    }

    /// Generate session files
    async fn generate_session(
        &self,
        workshop: &Workshop,
        session: &WorkshopSession,
    ) -> Result<(), TestHarnessError> {
        // Create session directory
        let session_dir = self.config.output_dir.join(&workshop.id).join(&session.id);
        fs::create_dir_all(&session_dir).await.map_err(|e| {
            TestHarnessError::IoError(format!("Failed to create session directory: {}", e))
        })?;

        // Generate session files
        for format in &self.config.formats {
            match format {
                DocumentationFormat::Markdown => {
                    let markdown = session.to_markdown();
                    let path = session_dir.join(format!("index.{}", format.extension()));

                    fs::write(&path, markdown).await.map_err(|e| {
                        TestHarnessError::IoError(format!("Failed to write markdown file: {}", e))
                    })?;

                    info!("Generated markdown session: {:?}", path);
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
    use super::super::training::TrainingMaterialType;
    use super::*;

    #[test]
    fn test_workshop_activity() {
        let activity = WorkshopActivity::new(
            "intro",
            "Introduction to the Test Harness",
            "An introduction to the test harness and its components",
            WorkshopActivityType::Presentation,
            30,
        )
        .with_content("This is the content of the presentation.")
        .with_section(DocumentationSection::new(
            "overview",
            "Overview",
            "An overview of the test harness",
        ));

        assert_eq!(activity.id, "intro");
        assert_eq!(activity.title, "Introduction to the Test Harness");
        assert_eq!(
            activity.description,
            "An introduction to the test harness and its components"
        );
        assert_eq!(activity.activity_type, WorkshopActivityType::Presentation);
        assert_eq!(activity.duration_minutes, 30);
        assert_eq!(activity.content, "This is the content of the presentation.");
        assert_eq!(activity.sections.len(), 1);
    }
}
