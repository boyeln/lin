//! Configuration management for the lin CLI.
//!
//! Handles multi-organization authentication with embedded cache for teams and workflow states.
//! Config is stored in `~/.config/lin/config.json`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
}

/// Cached data for an organization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrgCache {
    /// Map of team keys (e.g., "ENG") to cached team data
    #[serde(default)]
    pub teams: HashMap<String, CachedTeam>,
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
}

impl Config {
    /// Returns the path to the global configuration file.
    ///
    /// The config file is stored at `~/.config/lin/config.json` on Unix systems
    /// or the equivalent XDG config directory on other platforms.
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("lin").join("config.json")
    }

    /// Load configuration from the config file.
    ///
    /// If the config file doesn't exist, returns a default empty configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&contents)
            .map_err(|e| LinError::parse(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save the configuration to the config file.
    ///
    /// Creates the config directory and any parent directories if they don't exist.
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
        };
        let team2 = CachedTeam {
            id: "team-2".to_string(),
            name: "Design".to_string(),
            states: HashMap::new(),
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
}
