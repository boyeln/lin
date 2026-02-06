//! Team and state resolution with caching support.
//!
//! Resolves team keys to team IDs and state names to state IDs,
//! using the config cache when available or querying the API.

use std::collections::HashMap;

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries;
use crate::commands::issue::is_uuid;
use crate::config::{CachedTeam, Config};
use crate::error::LinError;
use crate::models::{IssueTeamResponse, TeamsResponse, WorkflowStatesResponse};

/// Resolve a team from an optional argument, falling back to the current team.
///
/// If `team_key_or_id` is provided, resolves it normally.
/// If `team_key_or_id` is None, falls back to the current team from config.
///
/// # Arguments
///
/// * `client` - GraphQL client for API queries
/// * `team_key_or_id` - Optional team key (e.g., "ENG") or UUID
/// * `use_cache` - Whether to use cached data
///
/// # Returns
///
/// The team UUID.
pub fn resolve_team_or_current(
    client: &GraphQLClient,
    team_key_or_id: Option<&str>,
    use_cache: bool,
) -> Result<String> {
    if let Some(team) = team_key_or_id {
        // Team was provided explicitly, resolve it
        resolve_team_id(client, team, use_cache)
    } else {
        // No team provided, try to use current team
        let config = Config::load()?;
        match config.get_current_team() {
            Some(current_team) => resolve_team_id(client, &current_team, use_cache),
            None => Err(LinError::config(
                "No team specified. Use --team or set a default team with 'lin team switch <key>'"
                    .to_string(),
            )),
        }
    }
}

/// Resolve a team key or UUID to a team UUID.
///
/// If `use_cache` is true, uses cached data from the config.
/// Otherwise, queries the API directly (env var mode).
///
/// # Arguments
///
/// * `client` - GraphQL client for API queries
/// * `team_key_or_id` - Team key (e.g., "ENG") or UUID
/// * `use_cache` - Whether to use cached data
///
/// # Returns
///
/// The team UUID.
pub fn resolve_team_id(
    client: &GraphQLClient,
    team_key_or_id: &str,
    use_cache: bool,
) -> Result<String> {
    // 1. UUID passthrough
    if is_uuid(team_key_or_id) {
        return Ok(team_key_or_id.to_string());
    }

    // 2. Cached mode - check cache first
    if use_cache {
        let config = Config::load()?;
        if let Some(id) = config.get_team_id(team_key_or_id) {
            return Ok(id); // Cache hit
        }

        // Cache miss - try to sync this team
        match sync_team_to_cache(client, team_key_or_id) {
            Ok(cached_team) => {
                let mut config = Config::load()?;
                let team_id = cached_team.id.clone();
                config.cache_team(team_key_or_id.to_uppercase(), cached_team)?;
                return Ok(team_id);
            }
            Err(_) => {
                // Sync failed - show helpful error
                let available_teams = config.get_all_team_keys();
                let suggestion = if available_teams.is_empty() {
                    "Run 'lin team list' to see available teams.".to_string()
                } else {
                    format!("Available teams: {}", available_teams.join(", "))
                };
                return Err(LinError::api(format!(
                    "Team '{}' not found. {}",
                    team_key_or_id, suggestion
                )));
            }
        }
    }

    // 3. Env var mode - direct API query (no cache)
    match query_team_by_key(client, team_key_or_id) {
        Ok(team_id) => Ok(team_id),
        Err(_) => Err(LinError::api(format!(
            "Team '{}' not found. Run 'lin team list' to see available teams.",
            team_key_or_id
        ))),
    }
}

/// Resolve a state name or UUID to a state UUID.
///
/// Requires team context since states are team-specific.
/// If `use_cache` is true, uses cached data from the config.
///
/// # Arguments
///
/// * `client` - GraphQL client for API queries
/// * `team_key` - Team key for context (e.g., "ENG")
/// * `state_name_or_id` - State name (e.g., "todo") or UUID
/// * `use_cache` - Whether to use cached data
///
/// # Returns
///
/// The state UUID.
pub fn resolve_state_id(
    client: &GraphQLClient,
    team_key: &str,
    state_name_or_id: &str,
    use_cache: bool,
) -> Result<String> {
    // 1. UUID passthrough
    if is_uuid(state_name_or_id) {
        return Ok(state_name_or_id.to_string());
    }

    // 2. Ensure we have team ID (resolve if needed)
    let team_id = resolve_team_id(client, team_key, use_cache)?;

    let state_lower = state_name_or_id.to_lowercase();

    // 3. Cached mode - check cache
    if use_cache {
        let config = Config::load()?;
        if let Some(id) = config.get_state_id(team_key, &state_lower) {
            return Ok(id); // Cache hit
        }

        // Cache miss - sync team's states and retry
        sync_team_to_cache(client, team_key)?;
        let config = Config::load()?;
        match config.get_state_id(team_key, &state_lower) {
            Some(id) => Ok(id),
            None => {
                // List available states in error
                let available = config.get_all_states_for_team(team_key);
                Err(LinError::api(format!(
                    "State '{}' not found for team '{}'. Available states: {}",
                    state_name_or_id,
                    team_key,
                    available.join(", ")
                )))
            }
        }
    } else {
        // 4. Env var mode - direct API query
        let states = query_workflow_states(client, &team_id)?;
        match states.iter().find(|s| s.name.to_lowercase() == state_lower) {
            Some(state) => Ok(state.id.clone()),
            None => {
                let available: Vec<_> = states.iter().map(|s| s.name.as_str()).collect();
                Err(LinError::api(format!(
                    "State '{}' not found. Available states: {}",
                    state_name_or_id,
                    available.join(", ")
                )))
            }
        }
    }
}

/// Get the team ID for a given issue.
///
/// Used when updating an issue's state - we need to know which team
/// the issue belongs to in order to resolve state names.
pub fn get_issue_team_id(client: &GraphQLClient, issue_id: &str) -> Result<String> {
    let response: IssueTeamResponse = client.query(
        queries::issue::ISSUE_TEAM_QUERY,
        serde_json::json!({ "id": issue_id }),
    )?;

    Ok(response.issue.team.id)
}

/// Get the team key for a given team ID.
///
/// Used when we have a team ID but need the key for cache lookups.
pub fn get_team_key(client: &GraphQLClient, team_id: &str) -> Result<String> {
    // First try to find it in the cache
    let config = Config::load()?;
    if let Ok(org) = config.get_active_org() {
        for (key, team) in &org.cache.teams {
            if team.id == team_id {
                return Ok(key.clone());
            }
        }
    }

    // Not in cache - query the API
    let response: crate::models::TeamResponse = client.query(
        queries::team::TEAM_QUERY,
        serde_json::json!({ "id": team_id }),
    )?;

    Ok(response.team.key)
}

/// Resolve an estimate name or numeric string to a numeric value.
///
/// If `use_cache` is true, tries to resolve the estimate from cached team data.
/// Otherwise, or if the value is already numeric, returns the parsed value.
///
/// # Arguments
///
/// * `estimate_str` - Estimate name (e.g., "XS", "M", "L") or numeric value (e.g., "3", "5.0")
/// * `team_key` - Team key for context (e.g., "ENG") - required for cache lookups
/// * `use_cache` - Whether to use cached data
///
/// # Returns
///
/// The numeric estimate value.
pub fn resolve_estimate_value(
    estimate_str: &str,
    team_key: Option<&str>,
    use_cache: bool,
) -> Result<f64> {
    // 1. Try to parse as a number first
    if let Ok(value) = estimate_str.parse::<f64>() {
        return Ok(value);
    }

    // 2. If not a number, try to resolve from cache (if enabled and team provided)
    if use_cache {
        if let Some(team) = team_key {
            let config = Config::load()?;
            if let Some(value) = config.get_estimate_value(team, estimate_str) {
                return Ok(value);
            }

            // Cache miss - show helpful error
            let available = config.get_all_estimates_for_team(team);
            if available.is_empty() {
                return Err(LinError::config(format!(
                    "Estimate '{}' is not a number and no estimates are configured for team '{}'. \
                    Configure estimates using the config file or provide a numeric value.",
                    estimate_str, team
                )));
            } else {
                return Err(LinError::config(format!(
                    "Estimate '{}' not found for team '{}'. Available estimates: {}. \
                    You can also use a numeric value directly.",
                    estimate_str,
                    team,
                    available.join(", ")
                )));
            }
        }
    }

    // 3. Not a number and cache not available/enabled
    Err(LinError::parse(format!(
        "Invalid estimate '{}': must be a numeric value",
        estimate_str
    )))
}

/// Sync a single team's data to the cache.
///
/// Queries the team by key and all its workflow states, returning a CachedTeam.
fn sync_team_to_cache(client: &GraphQLClient, team_key: &str) -> Result<CachedTeam> {
    // Query team by key
    let teams_response: TeamsResponse = client.query(
        queries::team::TEAM_BY_KEY_QUERY,
        serde_json::json!({
            "filter": {
                "key": { "eq": team_key.to_uppercase() }
            }
        }),
    )?;

    if teams_response.teams.nodes.is_empty() {
        return Err(LinError::api(format!("Team '{}' not found", team_key)));
    }

    let team = &teams_response.teams.nodes[0];

    // Query workflow states for this team
    let states = query_workflow_states(client, &team.id)?;

    // Build state map (lowercase keys for case-insensitive lookup)
    let state_map: HashMap<String, String> = states
        .into_iter()
        .map(|s| (s.name.to_lowercase(), s.id))
        .collect();

    Ok(CachedTeam {
        id: team.id.clone(),
        name: team.name.clone(),
        states: state_map,
        estimates: HashMap::new(),
    })
}

/// Query a team by its key.
///
/// Returns the team ID on success.
fn query_team_by_key(client: &GraphQLClient, team_key: &str) -> Result<String> {
    let teams_response: TeamsResponse = client.query(
        queries::team::TEAM_BY_KEY_QUERY,
        serde_json::json!({
            "filter": {
                "key": { "eq": team_key.to_uppercase() }
            }
        }),
    )?;

    if teams_response.teams.nodes.is_empty() {
        return Err(LinError::api(format!("Team '{}' not found", team_key)));
    }

    Ok(teams_response.teams.nodes[0].id.clone())
}

/// Query workflow states for a team.
///
/// Returns a vector of workflow states.
fn query_workflow_states(
    client: &GraphQLClient,
    team_id: &str,
) -> Result<Vec<crate::models::WorkflowState>> {
    let states_response: WorkflowStatesResponse = client.query(
        queries::workflow::WORKFLOW_STATES_QUERY,
        serde_json::json!({ "id": team_id }),
    )?;

    Ok(states_response.team.states.nodes)
}

/// Resolve a milestone name or UUID to a milestone UUID.
///
/// Requires project context since milestones are project-specific.
///
/// # Arguments
///
/// * `client` - GraphQL client for API queries
/// * `milestone_name_or_id` - Milestone name (e.g., "Sprint 1") or UUID
/// * `project_id` - Project UUID for context
///
/// # Returns
///
/// The milestone UUID.
pub fn resolve_milestone_id(
    client: &GraphQLClient,
    milestone_name_or_id: &str,
    project_id: &str,
) -> Result<String> {
    // 1. UUID passthrough
    if is_uuid(milestone_name_or_id) {
        return Ok(milestone_name_or_id.to_string());
    }

    // 2. Query milestones for the project
    let response: crate::models::ProjectMilestonesResponse = client.query(
        queries::milestone::PROJECT_MILESTONES_QUERY,
        serde_json::json!({
            "projectId": project_id,
            "first": 100
        }),
    )?;

    // 3. Find milestone by name (case-insensitive)
    let milestone_lower = milestone_name_or_id.to_lowercase();
    match response
        .project_milestones
        .nodes
        .iter()
        .find(|m| m.name.to_lowercase() == milestone_lower)
    {
        Some(milestone) => Ok(milestone.id.clone()),
        None => {
            let available: Vec<_> = response
                .project_milestones
                .nodes
                .iter()
                .map(|m| m.name.as_str())
                .collect();
            if available.is_empty() {
                Err(LinError::api(format!(
                    "No milestones found for project '{}'",
                    project_id
                )))
            } else {
                Err(LinError::api(format!(
                    "Milestone '{}' not found. Available milestones: {}",
                    milestone_name_or_id,
                    available.join(", ")
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These are basic unit tests. Full integration tests would require mockito.

    #[test]
    fn test_resolve_team_id_uuid_passthrough() {
        let client = GraphQLClient::new("test-token");
        let uuid = "550e8400-e29b-41d4-a716-446655440000";

        // UUID should pass through without any API calls
        let result = resolve_team_id(&client, uuid, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid);
    }

    #[test]
    fn test_resolve_state_id_uuid_passthrough() {
        let client = GraphQLClient::new("test-token");
        let uuid = "550e8400-e29b-41d4-a716-446655440000";

        // UUID should pass through without any API calls
        let result = resolve_state_id(&client, "ENG", uuid, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid);
    }

    #[test]
    fn test_is_uuid_detection() {
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(is_uuid("550e8400e29b41d4a716446655440000"));
        assert!(!is_uuid("ENG"));
        assert!(!is_uuid("ENG-123"));
    }
}
