//! Configuration management for the lin CLI.
//!
//! Handles loading, saving, and managing organization configurations
//! including API tokens stored in `~/.config/lin/config.json`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::LinError;
use crate::Result;

/// Configuration for the lin CLI.
///
/// Stores organization credentials and settings in a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Map of organization names to their API tokens.
    pub organizations: HashMap<String, String>,
    /// The default organization to use when none is specified.
    pub default_org: Option<String>,
}

impl Config {
    /// Returns the path to the configuration file.
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
    /// Creates the config directory if it doesn't exist.
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
    /// If this is the first organization, it will be set as the default.
    pub fn add_org(&mut self, name: &str, token: &str) {
        self.organizations.insert(name.to_string(), token.to_string());

        // Set as default if this is the first org
        if self.default_org.is_none() {
            self.default_org = Some(name.to_string());
        }
    }

    /// Remove an organization from the configuration.
    ///
    /// If the removed organization was the default, the default will be cleared.
    ///
    /// # Errors
    ///
    /// Returns an error if the organization doesn't exist.
    pub fn remove_org(&mut self, name: &str) -> Result<()> {
        if self.organizations.remove(name).is_none() {
            return Err(LinError::config(format!(
                "Organization '{}' not found in configuration",
                name
            )));
        }

        // Clear default if it was the removed org
        if self.default_org.as_deref() == Some(name) {
            self.default_org = None;
        }

        Ok(())
    }

    /// Get the API token for the specified organization or the default.
    ///
    /// # Arguments
    ///
    /// * `org` - Optional organization name. If None, uses the default org.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No organization is specified and no default is set
    /// - The specified organization doesn't exist in the config
    pub fn get_token(&self, org: Option<&str>) -> Result<String> {
        let org_name = match org {
            Some(name) => name,
            None => self.default_org.as_deref().ok_or_else(|| {
                LinError::config(
                    "No organization specified and no default organization is set. \
                     Use 'lin org add <name>' to add an organization or 'lin org set-default <name>' to set a default."
                )
            })?,
        };

        self.organizations
            .get(org_name)
            .cloned()
            .ok_or_else(|| {
                LinError::config(format!(
                    "Organization '{}' not found in configuration. Use 'lin org add {}' to add it.",
                    org_name, org_name
                ))
            })
    }

    /// List all configured organization names.
    pub fn list_orgs(&self) -> Vec<&str> {
        self.organizations.keys().map(|s| s.as_str()).collect()
    }

    /// Set the default organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the organization doesn't exist in the config.
    pub fn set_default(&mut self, name: &str) -> Result<()> {
        if !self.organizations.contains_key(name) {
            return Err(LinError::config(format!(
                "Organization '{}' not found in configuration. Add it first with 'lin org add {}'.",
                name, name
            )));
        }

        self.default_org = Some(name.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.organizations.is_empty());
        assert!(config.default_org.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.add_org("test-org", "test-token-123");

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.organizations.get("test-org"),
            Some(&"test-token-123".to_string())
        );
        assert_eq!(deserialized.default_org, Some("test-org".to_string()));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "organizations": {
                "org1": "token1",
                "org2": "token2"
            },
            "default_org": "org1"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.organizations.len(), 2);
        assert_eq!(config.organizations.get("org1"), Some(&"token1".to_string()));
        assert_eq!(config.organizations.get("org2"), Some(&"token2".to_string()));
        assert_eq!(config.default_org, Some("org1".to_string()));
    }

    #[test]
    fn test_add_org_sets_default_for_first() {
        let mut config = Config::default();
        assert!(config.default_org.is_none());

        config.add_org("first-org", "token1");
        assert_eq!(config.default_org, Some("first-org".to_string()));

        // Second org shouldn't change the default
        config.add_org("second-org", "token2");
        assert_eq!(config.default_org, Some("first-org".to_string()));
    }

    #[test]
    fn test_add_org_updates_existing() {
        let mut config = Config::default();
        config.add_org("my-org", "old-token");
        config.add_org("my-org", "new-token");

        assert_eq!(
            config.organizations.get("my-org"),
            Some(&"new-token".to_string())
        );
        assert_eq!(config.organizations.len(), 1);
    }

    #[test]
    fn test_remove_org_success() {
        let mut config = Config::default();
        config.add_org("org1", "token1");
        config.add_org("org2", "token2");

        let result = config.remove_org("org2");
        assert!(result.is_ok());
        assert_eq!(config.organizations.len(), 1);
        assert!(config.organizations.get("org2").is_none());
    }

    #[test]
    fn test_remove_org_clears_default() {
        let mut config = Config::default();
        config.add_org("org1", "token1");
        assert_eq!(config.default_org, Some("org1".to_string()));

        config.remove_org("org1").unwrap();
        assert!(config.default_org.is_none());
    }

    #[test]
    fn test_remove_org_not_found() {
        let mut config = Config::default();
        let result = config.remove_org("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_get_token_with_org_name() {
        let mut config = Config::default();
        config.add_org("my-org", "my-token");

        let token = config.get_token(Some("my-org")).unwrap();
        assert_eq!(token, "my-token");
    }

    #[test]
    fn test_get_token_uses_default() {
        let mut config = Config::default();
        config.add_org("default-org", "default-token");

        let token = config.get_token(None).unwrap();
        assert_eq!(token, "default-token");
    }

    #[test]
    fn test_get_token_no_default_error() {
        let config = Config::default();
        let result = config.get_token(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No organization specified"));
    }

    #[test]
    fn test_get_token_org_not_found() {
        let config = Config::default();
        let result = config.get_token(Some("nonexistent"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_list_orgs() {
        let mut config = Config::default();
        config.add_org("org-a", "token-a");
        config.add_org("org-b", "token-b");
        config.add_org("org-c", "token-c");

        let orgs = config.list_orgs();
        assert_eq!(orgs.len(), 3);
        assert!(orgs.contains(&"org-a"));
        assert!(orgs.contains(&"org-b"));
        assert!(orgs.contains(&"org-c"));
    }

    #[test]
    fn test_set_default_success() {
        let mut config = Config::default();
        config.add_org("org1", "token1");
        config.add_org("org2", "token2");

        config.set_default("org2").unwrap();
        assert_eq!(config.default_org, Some("org2".to_string()));
    }

    #[test]
    fn test_set_default_not_found() {
        let mut config = Config::default();
        let result = config.set_default("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_config_path_contains_lin() {
        let path = Config::config_path();
        assert!(path.to_string_lossy().contains("lin"));
        assert!(path.to_string_lossy().contains("config.json"));
    }
}
