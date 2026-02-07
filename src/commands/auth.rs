//! Authentication and organization management commands.
//!
//! Manages API tokens, organization authentication, and team/state caching.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries;
use crate::config::{CachedTeam, Config};
use crate::error::LinError;
use crate::models::{TeamsResponse, ViewerResponse, WorkflowStatesResponse};
use crate::output::{OutputFormat, output};
use serde::Serialize;
use std::collections::HashMap;

/// Authenticate with a Linear organization and sync data.
///
/// Validates the token, stores it, and syncs all teams and their workflow states.
pub fn auth_add(name: String, token: String, format: OutputFormat) -> Result<()> {
    // 1. Validate token by making a test API call
    let client = GraphQLClient::new(&token);
    validate_token(&client)?;

    // 2. Load config and add/update org
    let mut config = Config::load()?;
    config.add_org(name.clone(), token)?;
    config.switch_org(&name)?;
    config.save()?;

    // 3. Sync all teams and their workflow states
    let teams = sync_org_data(&client, &mut config)?;

    // 4. Output success
    let team_keys: Vec<String> = teams.iter().map(|t| t.0.clone()).collect();
    let state_count: usize = teams.iter().map(|t| t.1).sum();

    let response = AuthAddResponse {
        organization: name,
        teams: team_keys.clone(),
        team_count: team_keys.len(),
        state_count,
    };

    output(&response, format);
    Ok(())
}

/// Switch to a different organization.
pub fn auth_switch(name: String, format: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;
    config.switch_org(&name)?;
    config.save()?;

    let response = AuthSwitchResponse { organization: name };

    output(&response, format);
    Ok(())
}

/// List all authenticated organizations.
pub fn auth_list(format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
    let active = config.active_org.as_deref();

    let orgs: Vec<AuthOrgInfo> = config
        .orgs
        .iter()
        .map(|(name, org_config)| AuthOrgInfo {
            name: name.clone(),
            active: Some(name.as_str()) == active,
            team_count: org_config.cache.teams.len(),
            last_sync: org_config.cache.last_sync.clone(),
        })
        .collect();

    let response = AuthListResponse {
        organizations: orgs,
    };

    output(&response, format);
    Ok(())
}

/// Remove an organization from the configuration.
pub fn auth_remove(name: String, format: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;
    config.remove_org(&name)?;
    config.save()?;

    let response = AuthRemoveResponse { organization: name };

    output(&response, format);
    Ok(())
}

/// Show status of the current active organization.
pub fn auth_status(format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
    let org_name = config.get_active_org_name()?;
    let org_config = config.get_active_org()?;

    let teams: Vec<String> = org_config.cache.teams.keys().cloned().collect();

    let response = AuthStatusResponse {
        organization: org_name.to_string(),
        teams,
        last_sync: org_config.cache.last_sync.clone(),
    };

    output(&response, format);
    Ok(())
}

/// Manually sync the current organization's teams and states.
pub fn auth_sync(format: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;
    let org_config = config.get_active_org()?;
    let client = GraphQLClient::new(&org_config.token);

    let teams = sync_org_data(&client, &mut config)?;

    let team_keys: Vec<String> = teams.iter().map(|t| t.0.clone()).collect();
    let state_count: usize = teams.iter().map(|t| t.1).sum();

    let response = AuthSyncResponse {
        teams: team_keys.clone(),
        team_count: team_keys.len(),
        state_count,
    };

    output(&response, format);
    Ok(())
}

/// Parse an estimate scale type string into a HashMap of name -> value mappings.
///
/// For t-shirt scales, maps ["xs","s","m","l","xl"] to [1,2,3,5,8].
/// For numeric scales (linear, fibonacci, exponential), uses hardcoded default values.
/// Returns an empty HashMap for "notUsed" or unknown types.
fn parse_estimate_scale(estimate_type: &Option<String>) -> HashMap<String, f64> {
    let Some(est_type) = estimate_type else {
        return HashMap::new();
    };

    match est_type.as_str() {
        "tShirt" => {
            let names = ["xs", "s", "m", "l", "xl"];
            let values = [1.0, 2.0, 3.0, 5.0, 8.0];
            names
                .iter()
                .zip(values.iter())
                .map(|(&name, &val)| (name.to_string(), val))
                .collect()
        }
        "linear" => [1.0, 2.0, 3.0, 4.0, 5.0]
            .iter()
            .map(|&val| (format!("{}", val as i64), val))
            .collect(),
        "fibonacci" => [1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0]
            .iter()
            .map(|&val| (format!("{}", val as i64), val))
            .collect(),
        "exponential" => [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0]
            .iter()
            .map(|&val| (format!("{}", val as i64), val))
            .collect(),
        _ => HashMap::new(),
    }
}

/// Sync all teams and their workflow states for the active organization.
///
/// Returns a vector of (team_key, state_count) tuples.
fn sync_org_data(client: &GraphQLClient, config: &mut Config) -> Result<Vec<(String, usize)>> {
    // Query all teams
    let teams_response: TeamsResponse = client.query(
        queries::team::TEAMS_QUERY,
        serde_json::json!({ "first": 100 }),
    )?;

    let mut results = Vec::new();

    // For each team, query and cache workflow states
    for team in teams_response.teams.nodes {
        // Query workflow states for this team
        let states_response: WorkflowStatesResponse = client.query(
            queries::workflow::WORKFLOW_STATES_QUERY,
            serde_json::json!({ "id": team.id }),
        )?;

        // Build state name -> state ID map (lowercase for case-insensitive lookup)
        let states: HashMap<String, String> = states_response
            .team
            .states
            .nodes
            .into_iter()
            .map(|s| (s.name.to_lowercase(), s.id))
            .collect();

        let state_count = states.len();

        // Cache the team
        let cached_team = CachedTeam {
            id: team.id,
            name: team.name,
            states,
            estimates: parse_estimate_scale(&team.issue_estimate_type),
        };

        config.cache_team(team.key.clone(), cached_team)?;
        results.push((team.key, state_count));
    }

    // Query and cache all projects
    use crate::models::ProjectsResponse;
    let projects_response: ProjectsResponse = client.query(
        queries::project::PROJECTS_QUERY,
        serde_json::json!({ "first": 250 }),
    )?;

    let projects: Vec<(String, String)> = projects_response
        .projects
        .nodes
        .into_iter()
        .map(|p| (p.id, p.name))
        .collect();

    config.cache_projects(projects)?;

    // Update last sync time
    config.update_last_sync()?;
    config.save()?;

    Ok(results)
}

/// Validate a token by querying the viewer endpoint.
fn validate_token(client: &GraphQLClient) -> Result<()> {
    let _response: ViewerResponse = client
        .query(queries::user::VIEWER_QUERY, serde_json::json!({}))
        .map_err(|e| {
            LinError::api(format!(
                "Invalid or expired API token. Please check your token and try again. Error: {}",
                e
            ))
        })?;

    Ok(())
}

// Response types for JSON output

#[derive(Debug, Serialize)]
struct AuthAddResponse {
    organization: String,
    teams: Vec<String>,
    team_count: usize,
    state_count: usize,
}

impl crate::output::HumanDisplay for AuthAddResponse {
    fn human_fmt(&self) -> String {
        [
            format!("✓ Authenticated as '{}'", self.organization),
            format!(
                "✓ Synced {} teams: {}",
                self.team_count,
                self.teams.join(", ")
            ),
            format!("✓ Cached {} workflow states", self.state_count),
        ]
        .join("\n")
    }
}

#[derive(Debug, Serialize)]
struct AuthSwitchResponse {
    organization: String,
}

impl crate::output::HumanDisplay for AuthSwitchResponse {
    fn human_fmt(&self) -> String {
        format!("✓ Switched to '{}'", self.organization)
    }
}

#[derive(Debug, Serialize)]
struct AuthOrgInfo {
    name: String,
    active: bool,
    team_count: usize,
    last_sync: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthListResponse {
    organizations: Vec<AuthOrgInfo>,
}

impl crate::output::HumanDisplay for AuthListResponse {
    fn human_fmt(&self) -> String {
        if self.organizations.is_empty() {
            return "No organizations configured. Run: lin auth <name> <token>".to_string();
        }

        let mut lines = Vec::new();
        for org in &self.organizations {
            let marker = if org.active { "* " } else { "  " };
            let sync_info = org
                .last_sync
                .as_ref()
                .map(|s| format!(" (last sync: {})", s))
                .unwrap_or_default();
            lines.push(format!(
                "{}{} ({} teams{})",
                marker, org.name, org.team_count, sync_info
            ));
        }
        lines.join("\n")
    }
}

#[derive(Debug, Serialize)]
struct AuthRemoveResponse {
    organization: String,
}

impl crate::output::HumanDisplay for AuthRemoveResponse {
    fn human_fmt(&self) -> String {
        format!("✓ Removed organization '{}'", self.organization)
    }
}

#[derive(Debug, Serialize)]
struct AuthStatusResponse {
    organization: String,
    teams: Vec<String>,
    last_sync: Option<String>,
}

impl crate::output::HumanDisplay for AuthStatusResponse {
    fn human_fmt(&self) -> String {
        let mut lines = vec![format!("Active organization: {}", self.organization)];
        lines.push(format!("Teams: {}", self.teams.join(", ")));
        if let Some(last_sync) = &self.last_sync {
            lines.push(format!("Last sync: {}", last_sync));
        }
        lines.join("\n")
    }
}

#[derive(Debug, Serialize)]
struct AuthSyncResponse {
    teams: Vec<String>,
    team_count: usize,
    state_count: usize,
}

impl crate::output::HumanDisplay for AuthSyncResponse {
    fn human_fmt(&self) -> String {
        format!(
            "✓ Synced {} teams: {}\n✓ Cached {} workflow states",
            self.team_count,
            self.teams.join(", "),
            self.state_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::HumanDisplay;

    // Basic smoke tests - full integration tests would require mockito

    #[test]
    fn test_auth_add_response_human_fmt() {
        let response = AuthAddResponse {
            organization: "test-org".to_string(),
            teams: vec!["ENG".to_string(), "DESIGN".to_string()],
            team_count: 2,
            state_count: 10,
        };

        let output = response.human_fmt();
        assert!(output.contains("test-org"));
        assert!(output.contains("2 teams"));
        assert!(output.contains("ENG, DESIGN"));
        assert!(output.contains("10 workflow states"));
    }

    #[test]
    fn test_auth_list_response_empty() {
        let response = AuthListResponse {
            organizations: vec![],
        };

        let output = response.human_fmt();
        assert!(output.contains("No organizations"));
    }

    #[test]
    fn test_auth_list_response_with_orgs() {
        let response = AuthListResponse {
            organizations: vec![
                AuthOrgInfo {
                    name: "org1".to_string(),
                    active: true,
                    team_count: 3,
                    last_sync: Some("2024-01-01T00:00:00Z".to_string()),
                },
                AuthOrgInfo {
                    name: "org2".to_string(),
                    active: false,
                    team_count: 5,
                    last_sync: None,
                },
            ],
        };

        let output = response.human_fmt();
        assert!(output.contains("* org1"));
        assert!(output.contains("  org2"));
        assert!(output.contains("3 teams"));
        assert!(output.contains("5 teams"));
    }

    #[test]
    fn test_parse_estimate_scale_tshirt() {
        let est = Some("tShirt".to_string());
        let scale = parse_estimate_scale(&est);
        assert_eq!(scale.len(), 5);
        assert_eq!(scale["xs"], 1.0);
        assert_eq!(scale["s"], 2.0);
        assert_eq!(scale["m"], 3.0);
        assert_eq!(scale["l"], 5.0);
        assert_eq!(scale["xl"], 8.0);
    }

    #[test]
    fn test_parse_estimate_scale_linear() {
        let est = Some("linear".to_string());
        let scale = parse_estimate_scale(&est);
        assert_eq!(scale.len(), 5);
        assert_eq!(scale["1"], 1.0);
        assert_eq!(scale["2"], 2.0);
        assert_eq!(scale["3"], 3.0);
        assert_eq!(scale["4"], 4.0);
        assert_eq!(scale["5"], 5.0);
    }

    #[test]
    fn test_parse_estimate_scale_fibonacci() {
        let est = Some("fibonacci".to_string());
        let scale = parse_estimate_scale(&est);
        assert_eq!(scale.len(), 7);
        assert_eq!(scale["1"], 1.0);
        assert_eq!(scale["2"], 2.0);
        assert_eq!(scale["3"], 3.0);
        assert_eq!(scale["5"], 5.0);
        assert_eq!(scale["8"], 8.0);
        assert_eq!(scale["13"], 13.0);
        assert_eq!(scale["21"], 21.0);
    }

    #[test]
    fn test_parse_estimate_scale_exponential() {
        let est = Some("exponential".to_string());
        let scale = parse_estimate_scale(&est);
        assert_eq!(scale.len(), 7);
        assert_eq!(scale["1"], 1.0);
        assert_eq!(scale["2"], 2.0);
        assert_eq!(scale["4"], 4.0);
        assert_eq!(scale["8"], 8.0);
        assert_eq!(scale["16"], 16.0);
        assert_eq!(scale["32"], 32.0);
        assert_eq!(scale["64"], 64.0);
    }

    #[test]
    fn test_parse_estimate_scale_not_used() {
        let est = Some("notUsed".to_string());
        let scale = parse_estimate_scale(&est);
        assert!(scale.is_empty());
    }

    #[test]
    fn test_parse_estimate_scale_none() {
        let est: Option<String> = None;
        let scale = parse_estimate_scale(&est);
        assert!(scale.is_empty());
    }
}
