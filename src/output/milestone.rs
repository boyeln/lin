//! Project milestone output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::ProjectMilestone;

impl HumanDisplay for ProjectMilestone {
    fn human_fmt(&self) -> String {
        // Color milestone status
        let status_colored = match self.status.as_str() {
            "done" => self.status.green(),
            "next" => self.status.yellow(),
            "overdue" => self.status.red(),
            "unstarted" => self.status.dimmed(),
            _ => self.status.normal(),
        };

        let mut parts = vec![format!("{}", self.name.bold())];
        parts.push(format!("  {}: {}", "Status".dimmed(), status_colored));
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        if let Some(target) = &self.target_date {
            parts.push(format!("  {}: {}", "Target".dimmed(), target));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        parts.join("\n")
    }
}
