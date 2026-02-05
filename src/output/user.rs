//! User output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::User;

impl HumanDisplay for User {
    fn human_fmt(&self) -> String {
        let name = self.name.bold();
        let status = if self.active {
            String::new()
        } else {
            format!(" {}", "(inactive)".red())
        };
        let display = self
            .display_name
            .as_ref()
            .map(|d| format!(" {}", format!("({})", d).dimmed()))
            .unwrap_or_default();
        format!("{}{}{}\n  {}", name, display, status, self.email.dimmed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_human_display() {
        let user = User {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            display_name: Some("JD".to_string()),
            active: true,
        };
        let output = user.human_fmt();
        assert!(output.contains("John Doe"));
        assert!(output.contains("(JD)"));
        assert!(output.contains("john@example.com"));
    }

    #[test]
    fn test_user_human_display_inactive() {
        let user = User {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            display_name: None,
            active: false,
        };
        let output = user.human_fmt();
        assert!(output.contains("(inactive)"));
    }
}
