//! Attachment output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::{Attachment, AttachmentWithIssue};

impl HumanDisplay for Attachment {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));
        parts.push(format!("  {}: {}", "URL".dimmed(), self.url.cyan()));

        if let Some(subtitle) = &self.subtitle {
            parts.push(format!("  {}: {}", "Description".dimmed(), subtitle));
        }

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        // Format date (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };
        parts.push(format!("  {}: {}", "Created".dimmed(), date));

        parts.join("\n")
    }
}

impl HumanDisplay for AttachmentWithIssue {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));
        parts.push(format!("  {}: {}", "URL".dimmed(), self.url.cyan()));

        if let Some(subtitle) = &self.subtitle {
            parts.push(format!("  {}: {}", "Description".dimmed(), subtitle));
        }

        if let Some(issue) = &self.issue {
            parts.push(format!(
                "  {}: {}",
                "Issue".dimmed(),
                issue.identifier.cyan()
            ));
        }

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        // Format date (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };
        parts.push(format!("  {}: {}", "Created".dimmed(), date));

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AttachmentIssue, User};

    #[test]
    fn test_attachment_human_display() {
        let attachment = Attachment {
            id: "attach-123".to_string(),
            title: "Screenshot.png".to_string(),
            subtitle: Some("Bug screenshot".to_string()),
            url: "https://example.com/screenshot.png".to_string(),
            metadata: None,
            created_at: "2024-01-15T00:00:00.000Z".to_string(),
            updated_at: "2024-01-15T00:00:00.000Z".to_string(),
            creator: Some(User {
                id: "user-1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                display_name: None,
                active: true,
            }),
        };
        let output = attachment.human_fmt();
        assert!(output.contains("Screenshot.png"));
        assert!(output.contains("attach-123"));
        assert!(output.contains("https://example.com/screenshot.png"));
        assert!(output.contains("Bug screenshot"));
        assert!(output.contains("John Doe"));
        assert!(output.contains("2024-01-15"));
    }

    #[test]
    fn test_attachment_with_issue_human_display() {
        let attachment = AttachmentWithIssue {
            id: "attach-456".to_string(),
            title: "Log file".to_string(),
            subtitle: None,
            url: "https://example.com/log.txt".to_string(),
            metadata: None,
            created_at: "2024-01-20T00:00:00.000Z".to_string(),
            updated_at: "2024-01-20T00:00:00.000Z".to_string(),
            creator: None,
            issue: Some(AttachmentIssue {
                id: "issue-123".to_string(),
                identifier: "ENG-456".to_string(),
            }),
        };
        let output = attachment.human_fmt();
        assert!(output.contains("Log file"));
        assert!(output.contains("attach-456"));
        assert!(output.contains("https://example.com/log.txt"));
        assert!(output.contains("ENG-456"));
        assert!(output.contains("2024-01-20"));
    }
}
