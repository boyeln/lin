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
        let estimates = if let Some(est_type) = &self.issue_estimate_type {
            if !est_type.values.is_empty() {
                let scale_display = match est_type.id.as_str() {
                    "tshirt" => "XS, S, M, L, XL".to_string(),
                    _ => est_type
                        .values
                        .iter()
                        .map(|v| {
                            if *v == v.floor() {
                                format!("{}", *v as i64)
                            } else {
                                format!("{}", v)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                };
                format!("\n  Estimates: {}", scale_display.dimmed())
            } else {
                String::new()
            }
        } else {
            String::new()
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
        use crate::models::IssueEstimationType;

        let team = Team {
            id: "team-456".to_string(),
            key: "ENG".to_string(),
            name: "Engineering".to_string(),
            description: Some("The engineering team".to_string()),
            issue_estimate_type: Some(IssueEstimationType {
                id: "tshirt".to_string(),
                name: "T-Shirt Sizes".to_string(),
                values: vec![1.0, 2.0, 3.0, 5.0, 8.0],
            }),
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
