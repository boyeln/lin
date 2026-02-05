//! Configuration management for the lin CLI.
//!
//! Handles loading, saving, and managing organization configurations
//! including API tokens stored in `~/.config/lin/config.json` (global)
//! or `.lin/config.json` (local/project-specific).

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::LinError;

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

/// Scope for config operations (local or global).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
    /// Local config (.lin/config.json in project)
    Local,
    /// Global config (~/.config/lin/config.json)
    Global,
}

impl Config {
    /// Returns the path to the global configuration file.
    ///
    /// The config file is stored at `~/.config/lin/config.json` on Unix systems
    /// or the equivalent XDG config directory on other platforms.
    pub fn config_path() -> PathBuf {
        Self::global_config_path()
    }

    /// Returns the path to the global configuration file.
    pub fn global_config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("lin").join("config.json")
    }

    /// Find the local configuration file by walking up the directory tree.
    ///
    /// Looks for `.lin/config.json` starting from the current directory
    /// and walking up to the root.
    ///
    /// Returns None if no local config file is found.
    pub fn find_local_config_path() -> Option<PathBuf> {
        let mut current_dir = env::current_dir().ok()?;

        loop {
            let candidate = current_dir.join(".lin").join("config.json");
            if candidate.exists() {
                return Some(candidate);
            }

            // Try to move to parent directory
            if !current_dir.pop() {
                // Reached root, no config found
                return None;
            }
        }
    }

    /// Load configuration from a specific path.
    fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)
            .map_err(|e| LinError::parse(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Load the global configuration.
    pub fn load_global() -> Result<Self> {
        Self::load_from_path(&Self::global_config_path())
    }

    /// Load the local configuration if it exists.
    pub fn load_local() -> Result<Option<Self>> {
        if let Some(path) = Self::find_local_config_path() {
            Ok(Some(Self::load_from_path(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Merge two configs, with the local config taking precedence.
    ///
    /// Organizations from both configs are combined, with local tokens
    /// overriding global tokens for the same organization name.
    /// The default_org from local config (if set) overrides the global one.
    pub fn merge(global: Self, local: Self) -> Self {
        let mut merged = Config {
            organizations: global.organizations.clone(),
            default_org: global.default_org.clone(),
        };

        // Merge organizations (local overrides global)
        for (name, token) in local.organizations {
            merged.organizations.insert(name, token);
        }

        // Local default_org overrides global
        if local.default_org.is_some() {
            merged.default_org = local.default_org;
        }

        merged
    }

    /// Load configuration from both local and global config files.
    ///
    /// Local config (.lin/config.json) takes precedence over global config
    /// (~/.config/lin/config.json). Organizations and settings are merged,
    /// with local values overriding global ones.
    ///
    /// If neither config file exists, returns a default empty configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if a config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let global = Self::load_global()?;
        let local = Self::load_local()?;

        match local {
            Some(local_config) => Ok(Self::merge(global, local_config)),
            None => Ok(global),
        }
    }

    /// Save the configuration to a specific path.
    fn save_to_path(&self, path: &Path) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| LinError::parse(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, contents)?;
        Ok(())
    }

    /// Save the configuration to the global config file.
    ///
    /// Creates the config directory and any parent directories if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be written.
    pub fn save(&self) -> Result<()> {
        self.save_global()
    }

    /// Save to the global config file.
    pub fn save_global(&self) -> Result<()> {
        self.save_to_path(&Self::global_config_path())
    }

    /// Save to the local config file (.lin/config.json in current directory).
    pub fn save_local(&self) -> Result<()> {
        let local_path = PathBuf::from(".lin").join("config.json");
        self.save_to_path(&local_path)
    }

    /// Save to the appropriate config file based on scope.
    ///
    /// If scope is None, saves to local config if it exists, otherwise global.
    pub fn save_with_scope(&self, scope: Option<ConfigScope>) -> Result<()> {
        match scope {
            Some(ConfigScope::Global) => self.save_global(),
            Some(ConfigScope::Local) => self.save_local(),
            None => {
                // Auto-detect: save to local if it exists, otherwise global
                if Self::find_local_config_path().is_some() {
                    self.save_local()
                } else {
                    self.save_global()
                }
            }
        }
    }

    /// Add or update an organization with the given token.
    ///
    /// If this is the first organization, it will be set as the default.
    pub fn add_org(&mut self, name: &str, token: &str) {
        self.organizations
            .insert(name.to_string(), token.to_string());

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
                     Use 'lin config set token <value>' to add an organization or 'lin config set default-org <name>' to set a default."
                )
            })?,
        };

        self.organizations.get(org_name).cloned().ok_or_else(|| {
            LinError::config(format!(
                "Organization '{}' not found in configuration. Use 'lin config set token <value> --org {}' to add it.",
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
                "Organization '{}' not found in configuration. Add it first with 'lin config set token <value> --org {}'.",
                name, name
            )));
        }

        self.default_org = Some(name.to_string());
        Ok(())
    }

    /// Validate the configuration file.
    ///
    /// Performs the following checks:
    /// - Valid JSON syntax (already validated on load)
    /// - API tokens have correct format (start with `lin_api_`)
    /// - Default organization exists in the organizations list
    ///
    /// # Returns
    ///
    /// Returns a `ConfigValidationResult` with validation status and any issues found.
    pub fn validate(&self) -> ConfigValidationResult {
        let mut issues = Vec::new();

        // Check API token format for each organization
        for (org_name, token) in &self.organizations {
            if !token.starts_with("lin_api_") {
                issues.push(ConfigValidationIssue {
                    severity: ValidationSeverity::Warning,
                    field: format!("organizations.{}", org_name),
                    message: format!(
                        "Token for organization '{}' may be invalid: does not start with 'lin_api_'",
                        org_name
                    ),
                });
            }

            if token.is_empty() {
                issues.push(ConfigValidationIssue {
                    severity: ValidationSeverity::Error,
                    field: format!("organizations.{}", org_name),
                    message: format!("Token for organization '{}' is empty", org_name),
                });
            }
        }

        // Check default organization exists
        if let Some(default) = &self.default_org {
            if !self.organizations.contains_key(default) {
                issues.push(ConfigValidationIssue {
                    severity: ValidationSeverity::Error,
                    field: "default_org".to_string(),
                    message: format!(
                        "Default organization '{}' does not exist in the organizations list",
                        default
                    ),
                });
            }
        }

        // Check for empty organization names
        for org_name in self.organizations.keys() {
            if org_name.trim().is_empty() {
                issues.push(ConfigValidationIssue {
                    severity: ValidationSeverity::Error,
                    field: "organizations".to_string(),
                    message: "Organization name cannot be empty".to_string(),
                });
            }
        }

        let is_valid = !issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error);

        ConfigValidationResult { is_valid, issues }
    }

    /// Load and validate the raw configuration file.
    ///
    /// This validates the JSON syntax separately from the Config::load() method
    /// to provide more detailed error messages.
    ///
    /// # Returns
    ///
    /// Returns a tuple of (Config, Vec<ConfigValidationIssue>) with parse errors if any.
    pub fn load_and_validate_file() -> Result<(Self, Vec<ConfigValidationIssue>)> {
        let path = Self::config_path();
        let mut parse_issues = Vec::new();

        if !path.exists() {
            parse_issues.push(ConfigValidationIssue {
                severity: ValidationSeverity::Warning,
                field: "config_file".to_string(),
                message: format!("Configuration file does not exist at {}", path.display()),
            });
            return Ok((Config::default(), parse_issues));
        }

        let contents = fs::read_to_string(&path)?;

        // Try to parse as JSON first to check syntax
        let config: Config = match serde_json::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                return Err(LinError::parse(format!(
                    "Invalid JSON syntax in config file: {}",
                    e
                )));
            }
        };

        Ok((config, parse_issues))
    }

    /// Mask an API token for display, showing only the first 12 characters.
    ///
    /// # Arguments
    ///
    /// * `token` - The token to mask
    ///
    /// # Returns
    ///
    /// A masked version of the token like "lin_api_xxxx..."
    pub fn mask_token(token: &str) -> String {
        if token.len() <= 12 {
            "*".repeat(token.len())
        } else {
            format!("{}...", &token[..12])
        }
    }
}

/// Result of configuration validation.
#[derive(Debug, Clone)]
pub struct ConfigValidationResult {
    /// Whether the configuration is valid (no errors, may have warnings).
    pub is_valid: bool,
    /// List of validation issues found.
    pub issues: Vec<ConfigValidationIssue>,
}

/// A single validation issue found in the configuration.
#[derive(Debug, Clone)]
pub struct ConfigValidationIssue {
    /// Severity of the issue.
    pub severity: ValidationSeverity,
    /// The field or section with the issue.
    pub field: String,
    /// Human-readable description of the issue.
    pub message: String,
}

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Error - configuration is invalid.
    Error,
    /// Warning - configuration may work but has potential issues.
    Warning,
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
        assert_eq!(
            config.organizations.get("org1"),
            Some(&"token1".to_string())
        );
        assert_eq!(
            config.organizations.get("org2"),
            Some(&"token2".to_string())
        );
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
        assert!(!config.organizations.contains_key("org2"));
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No organization specified")
        );
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

    #[test]
    fn test_validate_valid_config() {
        let mut config = Config::default();
        config.add_org("my-org", "lin_api_abc123xyz");

        let result = config.validate();
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_invalid_token_format() {
        let mut config = Config::default();
        config.add_org("my-org", "invalid_token_format");

        let result = config.validate();
        assert!(result.is_valid); // Warning doesn't make it invalid
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].severity, ValidationSeverity::Warning);
        assert!(
            result.issues[0]
                .message
                .contains("does not start with 'lin_api_'")
        );
    }

    #[test]
    fn test_validate_empty_token() {
        let mut config = Config::default();
        config
            .organizations
            .insert("my-org".to_string(), String::new());
        config.default_org = Some("my-org".to_string());

        let result = config.validate();
        assert!(!result.is_valid);
        // Should have both empty token error and invalid format warning
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.severity == ValidationSeverity::Error)
        );
        assert!(result.issues.iter().any(|i| i.message.contains("is empty")));
    }

    #[test]
    fn test_validate_invalid_default_org() {
        let mut config = Config::default();
        config.add_org("existing-org", "lin_api_abc123xyz");
        config.default_org = Some("nonexistent-org".to_string());

        let result = config.validate();
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].severity, ValidationSeverity::Error);
        assert!(
            result.issues[0]
                .message
                .contains("does not exist in the organizations list")
        );
    }

    #[test]
    fn test_validate_empty_config() {
        let config = Config::default();

        let result = config.validate();
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_multiple_issues() {
        let mut config = Config::default();
        config
            .organizations
            .insert("org1".to_string(), "bad_token1".to_string());
        config
            .organizations
            .insert("org2".to_string(), "bad_token2".to_string());
        config.default_org = Some("nonexistent".to_string());

        let result = config.validate();
        assert!(!result.is_valid);
        // Should have warnings for both invalid tokens and error for invalid default
        assert!(result.issues.len() >= 3);
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.severity == ValidationSeverity::Error)
        );
    }

    #[test]
    fn test_mask_token_long() {
        let token = "lin_api_abcdefghijklmnop";
        let masked = Config::mask_token(token);
        assert_eq!(masked, "lin_api_abcd...");
    }

    #[test]
    fn test_mask_token_short() {
        let token = "short";
        let masked = Config::mask_token(token);
        assert_eq!(masked, "*****");
    }

    #[test]
    fn test_mask_token_exactly_12() {
        let token = "lin_api_abcd";
        let masked = Config::mask_token(token);
        assert_eq!(masked, "************");
    }
}
