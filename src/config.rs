//! Configuration management for the lin CLI.
//!
//! Handles multi-organization authentication with embedded cache for teams and workflow states.
//! Config is stored in `~/.config/lin/config.json`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::Result;
use crate::error::LinError;

/// Configuration for the lin CLI with multi-org support.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// The active organization name (used when no env var override)
    pub active_org: Option<String>,
    /// Map of organization names to their configuration
    #[serde(default)]
    pub orgs: HashMap<String, OrgConfig>,
}

/// Configuration for a single organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgConfig {
    /// Linear API token for this organization
    pub token: String,
    /// Cached data for this organization
    #[serde(default)]
    pub cache: OrgCache,
    /// Current/default team key for this organization
    #[serde(default)]
    pub current_team: Option<String>,
}

/// Cached data for an organization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgCache {
    /// Map of team keys (e.g., "ENG") to cached team data
    #[serde(default)]
    pub teams: HashMap<String, CachedTeam>,
    /// Map of project slugs to project UUIDs
    #[serde(default)]
    pub projects: HashMap<String, String>,
    /// Last time the cache was synced (ISO 8601 timestamp)
    pub last_sync: Option<String>,
}

/// Cached data for a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTeam {
    /// Team UUID
    pub id: String,
    /// Team name (e.g., "Engineering")
    pub name: String,
    /// Map of state names (lowercase) to state UUIDs
    pub states: HashMap<String, String>,
    /// Map of estimate names (lowercase) to numeric values
    #[serde(default)]
    pub estimates: HashMap<String, f64>,
}

impl Config {
    /// Returns the path to the global configuration file.
    ///
    /// The config file is stored at `~/.config/lin/config.json` on all platforms.
    /// This provides consistency across Unix-like systems and follows CLI tool conventions.
    pub fn config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".config").join("lin").join("config.json")
    }

    /// Returns the legacy config path (macOS only).
    ///
    /// On macOS, configs were previously stored in `~/Library/Application Support/lin/config.json`.
    /// This is used for automatic migration to the new location.
    #[cfg(target_os = "macos")]
    fn legacy_config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir
            .join("Library")
            .join("Application Support")
            .join("lin")
            .join("config.json")
    }

    /// Load configuration from the config file.
    ///
    /// If the config file doesn't exist, returns a default empty configuration.
    /// On macOS, automatically migrates from legacy location if found.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        // Check if config exists at new location
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&contents)
                .map_err(|e| LinError::parse(format!("Failed to parse config file: {}", e)))?;
            return Ok(config);
        }

        // On macOS, check legacy location and migrate if found
        #[cfg(target_os = "macos")]
        {
            let legacy_path = Self::legacy_config_path();
            if legacy_path.exists() {
                // Read from legacy location
                let contents = fs::read_to_string(&legacy_path)?;
                let config: Config = serde_json::from_str(&contents)
                    .map_err(|e| LinError::parse(format!("Failed to parse config file: {}", e)))?;

                // Save to new location (this will set proper permissions)
                config.save()?;

                // Remove legacy file
                let _ = fs::remove_file(&legacy_path);

                return Ok(config);
            }
        }

        Ok(Config::default())
    }

    /// Save the configuration to the config file.
    ///
    /// Creates the config directory and any parent directories if they don't exist.
    /// On Unix systems, sets file permissions to 600 (owner read/write only) to protect API tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| LinError::parse(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, contents)?;

        // Set file permissions to 600 (owner read/write only) on Unix systems
        // This protects API tokens from being readable by other users
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(&path, permissions)?;
        }

        Ok(())
    }

    /// Add or update an organization with the given token.
    ///
    /// If this is the first organization, it will be set as active.
    /// If updating an existing org, it remains or becomes active.
    pub fn add_org(&mut self, name: String, token: String) -> Result<()> {
        let org_config = OrgConfig {
            token,
            cache: OrgCache::default(),
            current_team: None,
        };

        self.orgs.insert(name.clone(), org_config);

        // Set as active if this is the first org or if it's being updated
        if self.active_org.is_none() || self.active_org.as_ref() == Some(&name) {
            self.active_org = Some(name);
        }

        Ok(())
    }

    /// Remove an organization from the configuration.
    ///
    /// If the removed organization was active, the active org will be cleared.
    ///
    /// # Errors
    ///
    /// Returns an error if the organization doesn't exist.
    pub fn remove_org(&mut self, name: &str) -> Result<()> {
        if self.orgs.remove(name).is_none() {
            return Err(LinError::config(format!(
                "Organization '{}' not found in configuration",
                name
            )));
        }

        // Clear active if it was the removed org
        if self.active_org.as_deref() == Some(name) {
            // Set to any other org if available
            self.active_org = self.orgs.keys().next().map(|s| s.to_string());
        }

        Ok(())
    }

    /// Switch the active organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the organization doesn't exist.
    pub fn switch_org(&mut self, name: &str) -> Result<()> {
        if !self.orgs.contains_key(name) {
            return Err(LinError::config(format!(
                "Organization '{}' not found in configuration",
                name
            )));
        }

        self.active_org = Some(name.to_string());
        Ok(())
    }

    /// Get the active organization configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set or if it doesn't exist.
    pub fn get_active_org(&self) -> Result<&OrgConfig> {
        let org_name = self.active_org.as_deref().ok_or_else(|| {
            LinError::config("No active organization. Run: lin auth <name> <token>")
        })?;

        self.orgs.get(org_name).ok_or_else(|| {
            LinError::config(format!(
                "Active organization '{}' not found in configuration",
                org_name
            ))
        })
    }

    /// Get the active organization configuration mutably.
    pub fn get_active_org_mut(&mut self) -> Result<&mut OrgConfig> {
        let org_name = self
            .active_org
            .as_deref()
            .ok_or_else(|| {
                LinError::config("No active organization. Run: lin auth <name> <token>")
            })?
            .to_string();

        self.orgs.get_mut(&org_name).ok_or_else(|| {
            LinError::config(format!(
                "Active organization '{}' not found in configuration",
                org_name
            ))
        })
    }

    /// Get the active organization name.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set.
    pub fn get_active_org_name(&self) -> Result<&str> {
        self.active_org
            .as_deref()
            .ok_or_else(|| LinError::config("No active organization. Run: lin auth <name> <token>"))
    }

    /// Get the API token for a specific organization, or the active organization if None.
    ///
    /// # Arguments
    ///
    /// * `org_name` - Optional organization name. If None, uses the active organization.
    ///
    /// # Errors
    ///
    /// Returns an error if no organization is found or no active organization is set.
    pub fn get_token(&self, org_name: Option<&str>) -> Result<String> {
        let org = if let Some(name) = org_name {
            self.orgs
                .get(name)
                .ok_or_else(|| LinError::config(format!("Organization '{}' not found", name)))?
        } else {
            self.get_active_org()?
        };

        Ok(org.token.clone())
    }

    /// Get a team ID from the cache by team key.
    ///
    /// Returns None if the team is not in the cache.
    pub fn get_team_id(&self, team_key: &str) -> Option<String> {
        let org = self.get_active_org().ok()?;
        org.cache.teams.get(team_key).map(|t| t.id.clone())
    }

    /// Get a state ID from the cache by team key and state name.
    ///
    /// State name lookup is case-insensitive.
    /// Returns None if the team or state is not in the cache.
    pub fn get_state_id(&self, team_key: &str, state_name: &str) -> Option<String> {
        let org = self.get_active_org().ok()?;
        let team = org.cache.teams.get(team_key)?;
        team.states.get(&state_name.to_lowercase()).cloned()
    }

    /// Cache a team's data in the active organization.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set.
    pub fn cache_team(&mut self, key: String, team: CachedTeam) -> Result<()> {
        let org = self.get_active_org_mut()?;
        org.cache.teams.insert(key, team);
        Ok(())
    }

    /// Get all team keys from the active organization's cache.
    pub fn get_all_team_keys(&self) -> Vec<String> {
        self.get_active_org()
            .ok()
            .map(|org| org.cache.teams.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all state names for a team from the active organization's cache.
    pub fn get_all_states_for_team(&self, team_key: &str) -> Vec<String> {
        self.get_active_org()
            .ok()
            .and_then(|org| org.cache.teams.get(team_key))
            .map(|team| team.states.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get an estimate value from the cache by team key and estimate name.
    ///
    /// Estimate name lookup is case-insensitive.
    /// Returns None if the team or estimate is not in the cache.
    pub fn get_estimate_value(&self, team_key: &str, estimate_name: &str) -> Option<f64> {
        let org = self.get_active_org().ok()?;
        let team = org.cache.teams.get(team_key)?;
        team.estimates.get(&estimate_name.to_lowercase()).copied()
    }

    /// Get all estimate names for a team from the active organization's cache.
    pub fn get_all_estimates_for_team(&self, team_key: &str) -> Vec<String> {
        self.get_active_org()
            .ok()
            .and_then(|org| org.cache.teams.get(team_key))
            .map(|team| team.estimates.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Set estimates for a team in the active organization's cache.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set or the team is not cached.
    pub fn set_team_estimates(
        &mut self,
        team_key: &str,
        estimates: HashMap<String, f64>,
    ) -> Result<()> {
        let org = self.get_active_org_mut()?;
        let team =
            org.cache.teams.get_mut(team_key).ok_or_else(|| {
                LinError::config(format!("Team '{}' not found in cache", team_key))
            })?;

        // Store estimates with lowercase keys for case-insensitive lookup
        team.estimates = estimates
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        Ok(())
    }

    /// Update the last sync time for the active organization.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set.
    pub fn update_last_sync(&mut self) -> Result<()> {
        let org = self.get_active_org_mut()?;
        org.cache.last_sync = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// List all configured organization names.
    pub fn list_orgs(&self) -> Vec<&str> {
        self.orgs.keys().map(|s| s.as_str()).collect()
    }

    /// Set the current team for the active organization.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set or the team is not in the cache.
    pub fn set_current_team(&mut self, team_key: &str) -> Result<()> {
        let org = self.get_active_org_mut()?;

        // Verify team exists in cache
        if !org.cache.teams.contains_key(team_key) {
            return Err(LinError::config(format!(
                "Team '{}' not found in cache. Run 'lin auth sync' to sync teams.",
                team_key
            )));
        }

        org.current_team = Some(team_key.to_string());
        Ok(())
    }

    /// Get the current team for the active organization.
    ///
    /// Returns None if no current team is set.
    pub fn get_current_team(&self) -> Option<String> {
        self.get_active_org().ok()?.current_team.clone()
    }

    /// Clear the current team for the active organization.
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set.
    pub fn clear_current_team(&mut self) -> Result<()> {
        let org = self.get_active_org_mut()?;
        org.current_team = None;
        Ok(())
    }

    /// Slugify a project name for use as a human-readable identifier.
    ///
    /// Converts to lowercase, replaces spaces and special characters with dashes,
    /// and removes multiple consecutive dashes.
    pub fn slugify(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    '\0' // Will be filtered out
                }
            })
            .filter(|&c| c != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Generate a unique slug for a project, handling conflicts by appending UUID prefix.
    ///
    /// If the base slug conflicts with an existing slug for a different project,
    /// appends characters from the UUID until unique (starting with 3 chars).
    fn generate_unique_slug(
        base_slug: &str,
        project_id: &str,
        existing_slugs: &HashMap<String, String>,
    ) -> String {
        // If no conflict, use base slug
        if !existing_slugs.contains_key(base_slug) {
            return base_slug.to_string();
        }

        // If conflict but it's the same project, keep the slug
        if existing_slugs.get(base_slug) == Some(&project_id.to_string()) {
            return base_slug.to_string();
        }

        // Conflict with different project - append UUID prefix
        let uuid_chars: Vec<char> = project_id.chars().filter(|c| c.is_alphanumeric()).collect();
        let mut suffix_len = 3;

        loop {
            let suffix: String = uuid_chars.iter().take(suffix_len).collect();
            let candidate = format!("{}-{}", base_slug, suffix);

            // Check if this candidate is available or already assigned to this project
            if !existing_slugs.contains_key(&candidate)
                || existing_slugs.get(&candidate) == Some(&project_id.to_string())
            {
                return candidate;
            }

            suffix_len += 1;
            if suffix_len > uuid_chars.len() {
                // Fallback to full UUID if we run out of characters
                return format!("{}-{}", base_slug, project_id);
            }
        }
    }

    /// Cache projects for the active organization.
    ///
    /// Generates slugs for each project, handling conflicts automatically.
    ///
    /// # Arguments
    ///
    /// * `projects` - List of projects to cache (tuples of id and name)
    ///
    /// # Errors
    ///
    /// Returns an error if no active organization is set.
    pub fn cache_projects(&mut self, projects: Vec<(String, String)>) -> Result<()> {
        let org = self.get_active_org_mut()?;
        let mut slug_map = HashMap::new();

        for (id, name) in projects {
            let base_slug = Self::slugify(&name);
            let unique_slug = Self::generate_unique_slug(&base_slug, &id, &slug_map);
            slug_map.insert(unique_slug, id);
        }

        org.cache.projects = slug_map;
        Ok(())
    }

    /// Get a project UUID from the cache by slug or UUID.
    ///
    /// If the input looks like a UUID (contains dashes and is long), returns it directly.
    /// Otherwise, looks it up in the slug cache.
    ///
    /// Returns None if the slug is not in the cache.
    pub fn get_project_id(&self, slug_or_id: &str) -> Option<String> {
        let org = self.get_active_org().ok()?;

        // If it looks like a UUID, return it directly
        if slug_or_id.len() > 30 && slug_or_id.contains('-') {
            return Some(slug_or_id.to_string());
        }

        // Otherwise, look up in cache
        org.cache.projects.get(slug_or_id).cloned()
    }

    /// Get a project slug from the cache by UUID.
    ///
    /// Returns None if the UUID is not in the cache.
    pub fn get_project_slug(&self, project_id: &str) -> Option<String> {
        let org = self.get_active_org().ok()?;

        // Find the slug that maps to this UUID
        org.cache
            .projects
            .iter()
            .find(|(_, id)| id.as_str() == project_id)
            .map(|(slug, _)| slug.clone())
    }

    /// Get all project slugs from the active organization's cache.
    pub fn get_all_project_slugs(&self) -> Vec<String> {
        self.get_active_org()
            .ok()
            .map(|org| org.cache.projects.keys().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.orgs.is_empty());
        assert!(config.active_org.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config
            .add_org("test-org".to_string(), "test-token-123".to_string())
            .unwrap();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert!(deserialized.orgs.contains_key("test-org"));
        assert_eq!(
            deserialized.orgs.get("test-org").unwrap().token,
            "test-token-123"
        );
        assert_eq!(deserialized.active_org, Some("test-org".to_string()));
    }

    #[test]
    fn test_add_org_sets_active_for_first() {
        let mut config = Config::default();
        assert!(config.active_org.is_none());

        config
            .add_org("first-org".to_string(), "token1".to_string())
            .unwrap();
        assert_eq!(config.active_org, Some("first-org".to_string()));

        // Second org doesn't change active
        config
            .add_org("second-org".to_string(), "token2".to_string())
            .unwrap();
        assert_eq!(config.active_org, Some("first-org".to_string()));
    }

    #[test]
    fn test_add_org_updates_existing() {
        let mut config = Config::default();
        config
            .add_org("my-org".to_string(), "old-token".to_string())
            .unwrap();
        config
            .add_org("my-org".to_string(), "new-token".to_string())
            .unwrap();

        assert_eq!(config.orgs.get("my-org").unwrap().token, "new-token");
        assert_eq!(config.orgs.len(), 1);
    }

    #[test]
    fn test_remove_org_success() {
        let mut config = Config::default();
        config
            .add_org("org1".to_string(), "token1".to_string())
            .unwrap();
        config
            .add_org("org2".to_string(), "token2".to_string())
            .unwrap();

        let result = config.remove_org("org2");
        assert!(result.is_ok());
        assert_eq!(config.orgs.len(), 1);
        assert!(!config.orgs.contains_key("org2"));
    }

    #[test]
    fn test_remove_org_switches_active() {
        let mut config = Config::default();
        config
            .add_org("org1".to_string(), "token1".to_string())
            .unwrap();
        config
            .add_org("org2".to_string(), "token2".to_string())
            .unwrap();
        config.switch_org("org2").unwrap();
        assert_eq!(config.active_org, Some("org2".to_string()));

        config.remove_org("org2").unwrap();
        // Should switch to remaining org
        assert_eq!(config.active_org, Some("org1".to_string()));
    }

    #[test]
    fn test_remove_org_not_found() {
        let mut config = Config::default();
        let result = config.remove_org("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_switch_org_success() {
        let mut config = Config::default();
        config
            .add_org("org1".to_string(), "token1".to_string())
            .unwrap();
        config
            .add_org("org2".to_string(), "token2".to_string())
            .unwrap();

        config.switch_org("org2").unwrap();
        assert_eq!(config.active_org, Some("org2".to_string()));
    }

    #[test]
    fn test_switch_org_not_found() {
        let mut config = Config::default();
        let result = config.switch_org("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_get_active_org() {
        let mut config = Config::default();
        config
            .add_org("my-org".to_string(), "my-token".to_string())
            .unwrap();

        let org = config.get_active_org().unwrap();
        assert_eq!(org.token, "my-token");
    }

    #[test]
    fn test_get_active_org_none_set() {
        let config = Config::default();
        let result = config.get_active_org();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No active organization")
        );
    }

    #[test]
    fn test_get_active_org_name() {
        let mut config = Config::default();
        config
            .add_org("my-org".to_string(), "token".to_string())
            .unwrap();

        let name = config.get_active_org_name().unwrap();
        assert_eq!(name, "my-org");
    }

    #[test]
    fn test_cache_team() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let team = CachedTeam {
            id: "team-123".to_string(),
            name: "Engineering".to_string(),
            states: HashMap::new(),
            estimates: HashMap::new(),
        };

        config.cache_team("ENG".to_string(), team).unwrap();

        let team_id = config.get_team_id("ENG");
        assert_eq!(team_id, Some("team-123".to_string()));
    }

    #[test]
    fn test_get_team_id_not_cached() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let team_id = config.get_team_id("NOTEXIST");
        assert!(team_id.is_none());
    }

    #[test]
    fn test_get_state_id() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let mut states = HashMap::new();
        states.insert("todo".to_string(), "state-123".to_string());
        states.insert("done".to_string(), "state-456".to_string());

        let team = CachedTeam {
            id: "team-123".to_string(),
            name: "Engineering".to_string(),
            states,
            estimates: HashMap::new(),
        };

        config.cache_team("ENG".to_string(), team).unwrap();

        // Case-insensitive lookup
        assert_eq!(
            config.get_state_id("ENG", "todo"),
            Some("state-123".to_string())
        );
        assert_eq!(
            config.get_state_id("ENG", "TODO"),
            Some("state-123".to_string())
        );
        assert_eq!(
            config.get_state_id("ENG", "Done"),
            Some("state-456".to_string())
        );
    }

    #[test]
    fn test_get_state_id_not_cached() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let state_id = config.get_state_id("ENG", "todo");
        assert!(state_id.is_none());
    }

    #[test]
    fn test_get_all_team_keys() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let team1 = CachedTeam {
            id: "team-1".to_string(),
            name: "Engineering".to_string(),
            states: HashMap::new(),
            estimates: HashMap::new(),
        };
        let team2 = CachedTeam {
            id: "team-2".to_string(),
            name: "Design".to_string(),
            states: HashMap::new(),
            estimates: HashMap::new(),
        };

        config.cache_team("ENG".to_string(), team1).unwrap();
        config.cache_team("DESIGN".to_string(), team2).unwrap();

        let keys = config.get_all_team_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"ENG".to_string()));
        assert!(keys.contains(&"DESIGN".to_string()));
    }

    #[test]
    fn test_get_all_states_for_team() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let mut states = HashMap::new();
        states.insert("todo".to_string(), "state-1".to_string());
        states.insert("done".to_string(), "state-2".to_string());

        let team = CachedTeam {
            id: "team-123".to_string(),
            name: "Engineering".to_string(),
            states,
            estimates: HashMap::new(),
        };

        config.cache_team("ENG".to_string(), team).unwrap();

        let state_names = config.get_all_states_for_team("ENG");
        assert_eq!(state_names.len(), 2);
        assert!(state_names.contains(&"todo".to_string()));
        assert!(state_names.contains(&"done".to_string()));
    }

    #[test]
    fn test_list_orgs() {
        let mut config = Config::default();
        config
            .add_org("org-a".to_string(), "token-a".to_string())
            .unwrap();
        config
            .add_org("org-b".to_string(), "token-b".to_string())
            .unwrap();
        config
            .add_org("org-c".to_string(), "token-c".to_string())
            .unwrap();

        let orgs = config.list_orgs();
        assert_eq!(orgs.len(), 3);
        assert!(orgs.contains(&"org-a"));
        assert!(orgs.contains(&"org-b"));
        assert!(orgs.contains(&"org-c"));
    }

    #[test]
    fn test_config_path_contains_lin() {
        let path = Config::config_path();
        assert!(path.to_string_lossy().contains("lin"));
        assert!(path.to_string_lossy().contains("config.json"));
    }

    #[test]
    fn test_get_estimate_value() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let mut estimates = HashMap::new();
        estimates.insert("xs".to_string(), 1.0);
        estimates.insert("s".to_string(), 2.0);
        estimates.insert("m".to_string(), 3.0);
        estimates.insert("l".to_string(), 5.0);
        estimates.insert("xl".to_string(), 8.0);

        let team = CachedTeam {
            id: "team-123".to_string(),
            name: "Engineering".to_string(),
            states: HashMap::new(),
            estimates,
        };

        config.cache_team("ENG".to_string(), team).unwrap();

        // Case-insensitive lookup
        assert_eq!(config.get_estimate_value("ENG", "xs"), Some(1.0));
        assert_eq!(config.get_estimate_value("ENG", "XS"), Some(1.0));
        assert_eq!(config.get_estimate_value("ENG", "m"), Some(3.0));
        assert_eq!(config.get_estimate_value("ENG", "M"), Some(3.0));
        assert_eq!(config.get_estimate_value("ENG", "xl"), Some(8.0));
    }

    #[test]
    fn test_get_estimate_value_not_cached() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let estimate = config.get_estimate_value("ENG", "xs");
        assert!(estimate.is_none());
    }

    #[test]
    fn test_get_all_estimates_for_team() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let mut estimates = HashMap::new();
        estimates.insert("xs".to_string(), 1.0);
        estimates.insert("s".to_string(), 2.0);
        estimates.insert("m".to_string(), 3.0);

        let team = CachedTeam {
            id: "team-123".to_string(),
            name: "Engineering".to_string(),
            states: HashMap::new(),
            estimates,
        };

        config.cache_team("ENG".to_string(), team).unwrap();

        let estimate_names = config.get_all_estimates_for_team("ENG");
        assert_eq!(estimate_names.len(), 3);
        assert!(estimate_names.contains(&"xs".to_string()));
        assert!(estimate_names.contains(&"s".to_string()));
        assert!(estimate_names.contains(&"m".to_string()));
    }

    #[test]
    fn test_set_team_estimates() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let team = CachedTeam {
            id: "team-123".to_string(),
            name: "Engineering".to_string(),
            states: HashMap::new(),
            estimates: HashMap::new(),
        };

        config.cache_team("ENG".to_string(), team).unwrap();

        let mut new_estimates = HashMap::new();
        new_estimates.insert("XS".to_string(), 1.0);
        new_estimates.insert("S".to_string(), 2.0);
        new_estimates.insert("M".to_string(), 3.0);

        config.set_team_estimates("ENG", new_estimates).unwrap();

        // Verify estimates were stored with lowercase keys
        assert_eq!(config.get_estimate_value("ENG", "xs"), Some(1.0));
        assert_eq!(config.get_estimate_value("ENG", "s"), Some(2.0));
        assert_eq!(config.get_estimate_value("ENG", "m"), Some(3.0));
    }

    #[test]
    fn test_set_team_estimates_team_not_found() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let estimates = HashMap::new();
        let result = config.set_team_estimates("NOTEXIST", estimates);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(
            Config::slugify("Q1 Backend Redesign"),
            "q1-backend-redesign"
        );
        assert_eq!(Config::slugify("Mobile App v2"), "mobile-app-v2");
        assert_eq!(Config::slugify("Infrastructure"), "infrastructure");
        assert_eq!(Config::slugify("Project @ 2024"), "project-2024");
        assert_eq!(Config::slugify("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(Config::slugify("dash-separated"), "dash-separated");
        assert_eq!(Config::slugify("under_score"), "under-score");
    }

    #[test]
    fn test_generate_unique_slug_no_conflict() {
        let existing = HashMap::new();
        let slug = Config::generate_unique_slug("test-project", "abc123", &existing);
        assert_eq!(slug, "test-project");
    }

    #[test]
    fn test_generate_unique_slug_with_conflict() {
        let mut existing = HashMap::new();
        existing.insert("test-project".to_string(), "different-id".to_string());

        let slug = Config::generate_unique_slug("test-project", "abc123", &existing);
        assert_eq!(slug, "test-project-abc");

        // If that's also taken, try longer suffix
        existing.insert("test-project-abc".to_string(), "another-id".to_string());
        let slug = Config::generate_unique_slug("test-project", "abc123", &existing);
        assert_eq!(slug, "test-project-abc1");
    }

    #[test]
    fn test_generate_unique_slug_same_project() {
        let mut existing = HashMap::new();
        existing.insert("test-project".to_string(), "abc123".to_string());

        // Same project ID should keep the same slug
        let slug = Config::generate_unique_slug("test-project", "abc123", &existing);
        assert_eq!(slug, "test-project");
    }

    #[test]
    fn test_cache_projects() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let projects = vec![
            ("proj-1".to_string(), "Q1 Backend".to_string()),
            ("proj-2".to_string(), "Frontend".to_string()),
            ("proj-3".to_string(), "Backend".to_string()),
        ];

        config.cache_projects(projects).unwrap();

        let org = config.get_active_org().unwrap();
        assert_eq!(
            org.cache.projects.get("q1-backend"),
            Some(&"proj-1".to_string())
        );
        assert_eq!(
            org.cache.projects.get("frontend"),
            Some(&"proj-2".to_string())
        );
        assert_eq!(
            org.cache.projects.get("backend"),
            Some(&"proj-3".to_string())
        );
    }

    #[test]
    fn test_cache_projects_with_conflicts() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let projects = vec![
            ("abc123".to_string(), "Mobile App".to_string()),
            ("xyz789".to_string(), "Mobile App".to_string()),
        ];

        config.cache_projects(projects).unwrap();

        let org = config.get_active_org().unwrap();
        // First project gets the base slug
        assert_eq!(
            org.cache.projects.get("mobile-app"),
            Some(&"abc123".to_string())
        );
        // Second project gets a suffix
        assert!(org.cache.projects.contains_key("mobile-app-xyz"));
    }

    #[test]
    fn test_get_project_id_from_slug() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let projects = vec![("proj-123".to_string(), "Test Project".to_string())];
        config.cache_projects(projects).unwrap();

        let id = config.get_project_id("test-project");
        assert_eq!(id, Some("proj-123".to_string()));
    }

    #[test]
    fn test_get_project_id_from_uuid() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        // UUID-like strings should be returned directly
        let id = config.get_project_id("abc-123-def-456-ghi-789-long-uuid");
        assert_eq!(id, Some("abc-123-def-456-ghi-789-long-uuid".to_string()));
    }

    #[test]
    fn test_get_project_id_not_found() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let id = config.get_project_id("nonexistent");
        assert!(id.is_none());
    }

    #[test]
    fn test_get_project_slug() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let projects = vec![("proj-123".to_string(), "Test Project".to_string())];
        config.cache_projects(projects).unwrap();

        let slug = config.get_project_slug("proj-123");
        assert_eq!(slug, Some("test-project".to_string()));
    }

    #[test]
    fn test_get_all_project_slugs() {
        let mut config = Config::default();
        config
            .add_org("org".to_string(), "token".to_string())
            .unwrap();

        let projects = vec![
            ("proj-1".to_string(), "Project A".to_string()),
            ("proj-2".to_string(), "Project B".to_string()),
        ];
        config.cache_projects(projects).unwrap();

        let slugs = config.get_all_project_slugs();
        assert_eq!(slugs.len(), 2);
        assert!(slugs.contains(&"project-a".to_string()));
        assert!(slugs.contains(&"project-b".to_string()));
    }
}
