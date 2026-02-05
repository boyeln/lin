//! Document output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::{Document, DocumentWithContent};

impl HumanDisplay for Document {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        if let Some(project) = &self.project {
            parts.push(format!("  {}: {}", "Project".dimmed(), project.name));
        }

        // Format date (show date portion only)
        let date = if self.updated_at.len() >= 10 {
            &self.updated_at[..10]
        } else {
            &self.updated_at
        };
        parts.push(format!("  {}: {}", "Updated".dimmed(), date));

        parts.join("\n")
    }
}

impl HumanDisplay for DocumentWithContent {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        if let Some(project) = &self.project {
            parts.push(format!("  {}: {}", "Project".dimmed(), project.name));
        }

        // Format date (show date portion only)
        let date = if self.updated_at.len() >= 10 {
            &self.updated_at[..10]
        } else {
            &self.updated_at
        };
        parts.push(format!("  {}: {}", "Updated".dimmed(), date));

        // Add content section
        parts.push(format!("\n{}", "Content".bold()));
        if let Some(content) = &self.content {
            if content.is_empty() {
                parts.push("  (empty)".dimmed().to_string());
            } else {
                // Indent content for readability
                for line in content.lines() {
                    parts.push(format!("  {}", line));
                }
            }
        } else {
            parts.push("  (no content)".dimmed().to_string());
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DocumentProject, User};

    #[test]
    fn test_document_human_display() {
        let document = Document {
            id: "doc-123".to_string(),
            title: "Project Overview".to_string(),
            icon: None,
            color: None,
            created_at: "2024-01-01T00:00:00.000Z".to_string(),
            updated_at: "2024-01-15T00:00:00.000Z".to_string(),
            creator: Some(User {
                id: "user-1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                display_name: None,
                active: true,
            }),
            project: Some(DocumentProject {
                id: "project-1".to_string(),
                name: "Q1 Roadmap".to_string(),
            }),
        };
        let output = document.human_fmt();
        assert!(output.contains("Project Overview"));
        assert!(output.contains("doc-123"));
        assert!(output.contains("John Doe"));
        assert!(output.contains("Q1 Roadmap"));
        assert!(output.contains("2024-01-15"));
    }

    #[test]
    fn test_document_with_content_human_display() {
        let document = DocumentWithContent {
            id: "doc-789".to_string(),
            title: "Technical Spec".to_string(),
            content: Some("# Overview\n\nThis is the content.".to_string()),
            icon: None,
            color: None,
            created_at: "2024-01-01T00:00:00.000Z".to_string(),
            updated_at: "2024-01-20T00:00:00.000Z".to_string(),
            creator: None,
            project: None,
        };
        let output = document.human_fmt();
        assert!(output.contains("Technical Spec"));
        assert!(output.contains("doc-789"));
        assert!(output.contains("Content"));
        assert!(output.contains("# Overview"));
        assert!(output.contains("This is the content."));
    }

    #[test]
    fn test_document_with_empty_content_human_display() {
        let document = DocumentWithContent {
            id: "doc-empty".to_string(),
            title: "Empty Doc".to_string(),
            content: None,
            icon: None,
            color: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            creator: None,
            project: None,
        };
        let output = document.human_fmt();
        assert!(output.contains("Empty Doc"));
        assert!(output.contains("(no content)"));
    }
}
