//! Issue output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::{Comment, FullIssueRelation, Issue, IssueWithComments, NormalizedRelation};

impl HumanDisplay for Issue {
    fn human_fmt(&self) -> String {
        let identifier = self.identifier.bold().cyan();
        let mut parts = vec![format!("{} {}", identifier, self.title)];

        if let Some(state) = &self.state {
            // Color status based on workflow state type
            let status_colored = match state.type_.as_str() {
                "completed" => state.name.green(),
                "canceled" => state.name.red().dimmed(),
                "started" => state.name.yellow(),
                "backlog" | "unstarted" => state.name.dimmed(),
                _ => state.name.normal(),
            };
            parts.push(format!("  {}: {}", "Status".dimmed(), status_colored));
        }

        // Color priority based on level
        let priority_colored = match self.priority {
            0 => None,
            1 => Some("Urgent".red().bold()),
            2 => Some("High".yellow()),
            3 => Some("Normal".normal()),
            4 => Some("Low".dimmed()),
            _ => Some("Unknown".normal()),
        };
        if let Some(p) = priority_colored {
            parts.push(format!("  {}: {}", "Priority".dimmed(), p));
        }

        if let Some(assignee) = &self.assignee {
            parts.push(format!("  {}: {}", "Assignee".dimmed(), assignee.name));
        }

        if let Some(team) = &self.team {
            parts.push(format!("  {}: {}", "Team".dimmed(), team.name));
        }

        parts.join("\n")
    }
}

impl HumanDisplay for Comment {
    fn human_fmt(&self) -> String {
        let author = self
            .user
            .as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Format timestamp (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };

        let header = format!("{} {}", author.bold(), date.dimmed());

        // Indent the body for readability
        let body_lines: Vec<String> = self
            .body
            .lines()
            .map(|line| format!("  {}", line))
            .collect();
        let body = body_lines.join("\n");

        format!("{}\n{}", header, body)
    }
}

impl HumanDisplay for IssueWithComments {
    fn human_fmt(&self) -> String {
        let identifier = self.identifier.bold().cyan();
        let mut parts = vec![format!("{} {}", identifier, self.title)];

        if let Some(state) = &self.state {
            let status_colored = match state.type_.as_str() {
                "completed" => state.name.green(),
                "canceled" => state.name.red().dimmed(),
                "started" => state.name.yellow(),
                "backlog" | "unstarted" => state.name.dimmed(),
                _ => state.name.normal(),
            };
            parts.push(format!("  {}: {}", "Status".dimmed(), status_colored));
        }

        let priority_colored = match self.priority {
            0 => None,
            1 => Some("Urgent".red().bold()),
            2 => Some("High".yellow()),
            3 => Some("Normal".normal()),
            4 => Some("Low".dimmed()),
            _ => Some("Unknown".normal()),
        };
        if let Some(p) = priority_colored {
            parts.push(format!("  {}: {}", "Priority".dimmed(), p));
        }

        if let Some(assignee) = &self.assignee {
            parts.push(format!("  {}: {}", "Assignee".dimmed(), assignee.name));
        }

        if let Some(team) = &self.team {
            parts.push(format!("  {}: {}", "Team".dimmed(), team.name));
        }

        // Add comments section
        let comment_count = self.comments.nodes.len();
        parts.push(format!("\n  {} ({})", "Comments".bold(), comment_count));

        if self.comments.nodes.is_empty() {
            parts.push("  No comments yet.".dimmed().to_string());
        } else {
            for comment in &self.comments.nodes {
                parts.push(String::new()); // blank line before each comment
                // Indent comment output
                for line in comment.human_fmt().lines() {
                    parts.push(format!("  {}", line));
                }
            }
        }

        parts.join("\n")
    }
}

impl HumanDisplay for NormalizedRelation {
    fn human_fmt(&self) -> String {
        // Color the relation type based on its meaning
        let type_colored = match self.relation_type.as_str() {
            "parent" => "Parent".cyan().bold(),
            "child" => "Child".cyan(),
            "blocks" => "Blocks".red().bold(),
            "blocked_by" => "Blocked by".red(),
            "related" => "Related to".yellow(),
            "duplicate" => "Duplicate of".magenta(),
            other => other.normal(),
        };

        format!(
            "{} {} {}",
            type_colored,
            self.related_issue.identifier.bold().cyan(),
            self.related_issue.title
        )
    }
}

impl HumanDisplay for FullIssueRelation {
    fn human_fmt(&self) -> String {
        // Color the relation type
        let type_colored = match self.type_.as_str() {
            "blocks" => "blocks".red().bold(),
            "related" => "related to".yellow(),
            "duplicate" => "duplicate of".magenta(),
            other => other.normal(),
        };

        let source = self
            .issue
            .as_ref()
            .map(|i| format!("{} {}", i.identifier.bold().cyan(), i.title))
            .unwrap_or_else(|| "(unknown)".dimmed().to_string());

        let target = self
            .related_issue
            .as_ref()
            .map(|i| format!("{} {}", i.identifier.bold().cyan(), i.title))
            .unwrap_or_else(|| "(unknown)".dimmed().to_string());

        format!("{} {} {}", source, type_colored, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RelatedIssue, WorkflowState};

    #[test]
    fn test_issue_human_display() {
        let issue = Issue {
            id: "issue-123".to_string(),
            identifier: "ENG-123".to_string(),
            title: "Fix the bug".to_string(),
            description: None,
            priority: 2,
            state: Some(WorkflowState {
                id: "state-1".to_string(),
                name: "In Progress".to_string(),
                color: "#0066ff".to_string(),
                type_: "started".to_string(),
            }),
            team: None,
            assignee: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-02".to_string(),
        };
        let output = issue.human_fmt();
        assert!(output.contains("ENG-123"));
        assert!(output.contains("Fix the bug"));
        assert!(output.contains("Status: In Progress"));
        assert!(output.contains("Priority: High"));
    }

    #[test]
    fn test_normalized_relation_human_display() {
        let relation = NormalizedRelation {
            id: "rel-123".to_string(),
            relation_type: "blocks".to_string(),
            related_issue: RelatedIssue {
                id: "issue-456".to_string(),
                identifier: "ENG-456".to_string(),
                title: "Blocked issue".to_string(),
            },
        };
        let output = relation.human_fmt();
        assert!(output.contains("Blocks"));
        assert!(output.contains("ENG-456"));
        assert!(output.contains("Blocked issue"));
    }

    #[test]
    fn test_normalized_relation_parent_display() {
        let relation = NormalizedRelation {
            id: "parent:issue-100".to_string(),
            relation_type: "parent".to_string(),
            related_issue: RelatedIssue {
                id: "issue-100".to_string(),
                identifier: "ENG-100".to_string(),
                title: "Parent issue".to_string(),
            },
        };
        let output = relation.human_fmt();
        assert!(output.contains("Parent"));
        assert!(output.contains("ENG-100"));
        assert!(output.contains("Parent issue"));
    }

    #[test]
    fn test_full_issue_relation_human_display() {
        let relation = FullIssueRelation {
            id: "rel-new".to_string(),
            type_: "blocks".to_string(),
            issue: Some(RelatedIssue {
                id: "issue-1".to_string(),
                identifier: "ENG-1".to_string(),
                title: "Source issue".to_string(),
            }),
            related_issue: Some(RelatedIssue {
                id: "issue-2".to_string(),
                identifier: "ENG-2".to_string(),
                title: "Target issue".to_string(),
            }),
        };
        let output = relation.human_fmt();
        assert!(output.contains("ENG-1"));
        assert!(output.contains("Source issue"));
        assert!(output.contains("blocks"));
        assert!(output.contains("ENG-2"));
        assert!(output.contains("Target issue"));
    }
}
