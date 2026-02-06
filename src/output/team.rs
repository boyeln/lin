//! Team and workflow state output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::{Team, WorkflowState};

impl HumanDisplay for Team {
    fn human_fmt(&self) -> String {
        let desc = self
            .description
            .as_ref()
            .map(|d| format!("\n  {}", d.dimmed()))
            .unwrap_or_default();

        // Add estimate scale info if present
        let estimates = match self.issue_estimate_type.as_deref() {
            Some("tShirt") => format!("\n  Estimates: {}", "XS, S, M, L, XL".dimmed()),
            Some("linear") => format!("\n  Estimates: {}", "1, 2, 3, 4, 5".dimmed()),
            Some("fibonacci") => {
                format!("\n  Estimates: {}", "1, 2, 3, 5, 8, 13, 21".dimmed())
            }
            Some("exponential") => {
                format!("\n  Estimates: {}", "1, 2, 4, 8, 16, 32, 64".dimmed())
            }
            _ => String::new(),
        };

        format!(
            "{} {}{}{}",
            format!("[{}]", self.key).cyan(),
            self.name.bold(),
            desc,
            estimates
        )
    }
}

impl HumanDisplay for WorkflowState {
    fn human_fmt(&self) -> String {
        let type_label = match self.type_.as_str() {
            "backlog" => "Backlog",
            "unstarted" => "Unstarted",
            "started" => "Started",
            "completed" => "Completed",
            "canceled" => "Canceled",
            other => other,
        };
        format!(
            "{} [{}]\n  ID: {}\n  Color: {}",
            self.name, type_label, self.id, self.color
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_human_display() {
        let team = Team {
            id: "team-123".to_string(),
            key: "ENG".to_string(),
            name: "Engineering".to_string(),
            description: Some("The engineering team".to_string()),
            issue_estimate_type: None,
        };
        let output = team.human_fmt();
        assert!(output.contains("[ENG]"));
        assert!(output.contains("Engineering"));
        assert!(output.contains("The engineering team"));
    }

    #[test]
    fn test_team_human_display_with_estimates() {
        let team = Team {
            id: "team-456".to_string(),
            key: "ENG".to_string(),
            name: "Engineering".to_string(),
            description: Some("The engineering team".to_string()),
            issue_estimate_type: Some("tShirt".to_string()),
        };
        let output = team.human_fmt();
        assert!(output.contains("[ENG]"));
        assert!(output.contains("Engineering"));
        assert!(output.contains("The engineering team"));
        assert!(output.contains("Estimates: XS, S, M, L, XL"));
    }

    #[test]
    fn test_workflow_state_human_display() {
        let state = WorkflowState {
            id: "state-123".to_string(),
            name: "In Progress".to_string(),
            color: "#0066ff".to_string(),
            type_: "started".to_string(),
        };
        let output = state.human_fmt();
        assert!(output.contains("In Progress"));
        assert!(output.contains("[Started]"));
        assert!(output.contains("state-123"));
        assert!(output.contains("#0066ff"));
    }
}
