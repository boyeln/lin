//! Label output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::Label;

impl HumanDisplay for Label {
    fn human_fmt(&self) -> String {
        let group_indicator = if self.is_group {
            " [Group]".cyan().to_string()
        } else {
            String::new()
        };

        let mut parts = vec![format!("{}{}", self.name.bold(), group_indicator)];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));
        parts.push(format!("  {}: {}", "Color".dimmed(), self.color));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_human_display() {
        let label = Label {
            id: "label-123".to_string(),
            name: "Bug".to_string(),
            description: Some("Bug reports".to_string()),
            color: "#ff0000".to_string(),
            is_group: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-02".to_string(),
        };
        let output = label.human_fmt();
        assert!(output.contains("Bug"));
        assert!(output.contains("label-123"));
        assert!(output.contains("#ff0000"));
        assert!(output.contains("Bug reports"));
        assert!(!output.contains("[Group]"));
    }

    #[test]
    fn test_label_group_human_display() {
        let label = Label {
            id: "label-456".to_string(),
            name: "Feature".to_string(),
            description: None,
            color: "#00ff00".to_string(),
            is_group: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let output = label.human_fmt();
        assert!(output.contains("Feature"));
        assert!(output.contains("[Group]"));
        assert!(output.contains("label-456"));
        assert!(output.contains("#00ff00"));
    }
}
