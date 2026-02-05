//! Project output formatting.

use colored::Colorize;

use super::HumanDisplay;
use crate::models::Project;

impl HumanDisplay for Project {
    fn human_fmt(&self) -> String {
        // Color project state
        let state_colored = match self.state.as_str() {
            "completed" => self.state.green(),
            "canceled" => self.state.red().dimmed(),
            "started" => self.state.yellow(),
            "paused" => self.state.cyan(),
            "planned" | "backlog" => self.state.dimmed(),
            _ => self.state.normal(),
        };

        let mut parts = vec![format!("{}", self.name.bold())];
        parts.push(format!("  {}: {}", "State".dimmed(), state_colored));
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        if let Some(start) = &self.start_date {
            parts.push(format!("  {}: {}", "Start".dimmed(), start));
        }

        if let Some(target) = &self.target_date {
            parts.push(format!("  {}: {}", "Target".dimmed(), target));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        parts.join("\n")
    }
}
