use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::modules::rag_manager::source::ContextSource;
use crate::modules::rag_manager::types::{ContextChunk, RagError};

/// A simple file-based context source
///
/// This implementation reads content from a file or string and returns it as context.
/// For the MVP, it doesn't do any sophisticated chunking or relevance scoring.
pub struct FileContextSource {
    /// The content of the file
    content: String,

    /// The name of the source
    source_name: String,
}

impl FileContextSource {
    /// Create a new file context source from a string
    ///
    /// # Arguments
    ///
    /// * `content` - The content to use as context
    /// * `source_name` - The name of the source
    pub fn new(content: String, source_name: String) -> Self {
        Self {
            content,
            source_name,
        }
    }

    /// Create a new file context source from a file
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file
    /// * `source_name` - Optional name of the source. If not provided, the file path is used.
    ///
    /// # Returns
    ///
    /// A new file context source, or an error if the file cannot be read
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        source_name: Option<String>,
    ) -> Result<Self, RagError> {
        let content = std::fs::read_to_string(&path)?;
        let source_name =
            source_name.unwrap_or_else(|| path.as_ref().to_string_lossy().to_string());

        Ok(Self {
            content,
            source_name,
        })
    }
}

#[async_trait]
impl ContextSource for FileContextSource {
    async fn get_context(
        &self,
        _query: &str,
        max_chunks: usize,
    ) -> Result<Vec<ContextChunk>, RagError> {
        // For MVP, just return the entire content as a single chunk
        // In a more sophisticated implementation, we would:
        // 1. Split the content into chunks
        // 2. Calculate relevance scores based on the query
        // 3. Return the most relevant chunks

        let chunk = ContextChunk {
            content: self.content.clone(),
            source: self.source_name.clone(),
            relevance_score: 1.0, // Maximum relevance for MVP
            metadata: HashMap::new(),
        };

        Ok(vec![chunk].into_iter().take(max_chunks).collect())
    }

    fn get_name(&self) -> String {
        self.source_name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_context_source_from_string() {
        let source = FileContextSource::new(
            "This is a test document.".to_string(),
            "test.txt".to_string(),
        );

        let chunks = source.get_context("test", 1).await.unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "This is a test document.");
        assert_eq!(chunks[0].source, "test.txt");
        assert_eq!(chunks[0].relevance_score, 1.0);
    }

    #[tokio::test]
    async fn test_file_context_source_from_file() {
        // Create a temporary file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is a test document.").unwrap();

        // Create a file context source
        let source = FileContextSource::from_file(&file_path, None).unwrap();

        // Get context
        let chunks = source.get_context("test", 1).await.unwrap();

        // Verify the context
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "This is a test document.\n");
        assert_eq!(chunks[0].source, file_path.to_string_lossy());
        assert_eq!(chunks[0].relevance_score, 1.0);
    }
}
