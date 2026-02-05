//! Cycle output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::{Cycle, CycleWithIssues};

impl HumanDisplay for Cycle {
    fn human_fmt(&self) -> String {
        // Display cycle name or number
        let title = self
            .name
            .as_ref()
            .map(|n| format!("{}", n.bold()))
            .unwrap_or_else(|| format!("{}", format!("Cycle {}", self.number).bold()));

        let mut parts = vec![title];
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        // Format dates (show date portion only)
        if let Some(starts) = &self.starts_at {
            let date = if starts.len() >= 10 {
                &starts[..10]
            } else {
                starts
            };
            parts.push(format!("  {}: {}", "Starts".dimmed(), date));
        }

        if let Some(ends) = &self.ends_at {
            let date = if ends.len() >= 10 { &ends[..10] } else { ends };
            parts.push(format!("  {}: {}", "Ends".dimmed(), date));
        }

        if let Some(completed) = &self.completed_at {
            let date = if completed.len() >= 10 {
                &completed[..10]
            } else {
                completed
            };
            parts.push(format!("  {}: {}", "Completed".dimmed(), date.green()));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        parts.join("\n")
    }
}

impl HumanDisplay for CycleWithIssues {
    fn human_fmt(&self) -> String {
        // Display cycle name or number
        let title = self
            .name
            .as_ref()
            .map(|n| format!("{}", n.bold()))
            .unwrap_or_else(|| format!("{}", format!("Cycle {}", self.number).bold()));

        let mut parts = vec![title];
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        // Format dates (show date portion only)
        if let Some(starts) = &self.starts_at {
            let date = if starts.len() >= 10 {
                &starts[..10]
            } else {
                starts
            };
            parts.push(format!("  {}: {}", "Starts".dimmed(), date));
        }

        if let Some(ends) = &self.ends_at {
            let date = if ends.len() >= 10 { &ends[..10] } else { ends };
            parts.push(format!("  {}: {}", "Ends".dimmed(), date));
        }

        if let Some(completed) = &self.completed_at {
            let date = if completed.len() >= 10 {
                &completed[..10]
            } else {
                completed
            };
            parts.push(format!("  {}: {}", "Completed".dimmed(), date.green()));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        // Add issues section
        let issue_count = self.issues.nodes.len();
        parts.push(format!("\n  {} ({})", "Issues".bold(), issue_count));

        if self.issues.nodes.is_empty() {
            parts.push("  No issues in this cycle.".dimmed().to_string());
        } else {
            for issue in &self.issues.nodes {
                parts.push(String::new()); // blank line before each issue
                                           // Indent issue output
                for line in issue.human_fmt().lines() {
                    parts.push(format!("  {}", line));
                }
            }
        }

        parts.join("\n")
    }
}
