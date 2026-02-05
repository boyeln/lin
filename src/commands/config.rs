//! Configuration management commands.
//!
//! Commands for managing configuration including API tokens and organization settings.

use std::io::{self, BufRead, Write};

use colored::Colorize;
use serde::Serialize;

use crate::Result;
use crate::config::{Config, ConfigScope, ConfigValidationIssue, ValidationSeverity};
use crate::output::{HumanDisplay, OutputFormat, output_success};

/// Response for the config set command.
#[derive(Serialize)]
pub struct ConfigSetResponse {
    pub message: String,
    pub key: String,
    pub value_or_org: String,
    pub config_path: String,
}

/// Response for the config get command.
#[derive(Serialize)]
pub struct ConfigGetResponse {
    pub key: String,
    pub value: String,
    pub source: Option<String>,
    pub organization: Option<String>,
}

/// Response for the config list command.
#[derive(Serialize)]
pub struct ConfigListResponse {
    pub default_org: Option<String>,
    pub organizations: Vec<OrgInfo>,
    pub config_path: String,
}

/// Response for listing a specific organization.
#[derive(Serialize)]
pub struct ConfigListOrgResponse {
    pub organization: String,
    pub token: String,
    pub is_default: bool,
}

/// Response for the config unset command.
#[derive(Serialize)]
pub struct ConfigUnsetResponse {
    pub message: String,
    pub key: String,
}

/// Response for the config validate command.
#[derive(Serialize)]
pub struct ConfigValidateResponse {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

/// Information about a configured organization.
#[derive(Serialize)]
pub struct OrgInfo {
    pub name: String,
    pub token: String,
    pub is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// A single validation issue in JSON-serializable format.
#[derive(Serialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub field: Option<String>,
    pub message: String,
}

impl HumanDisplay for ConfigSetResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

impl HumanDisplay for ConfigGetResponse {
    fn human_fmt(&self) -> String {
        let mut lines = Vec::new();

        if let Some(org) = &self.organization {
            lines.push(format!("{}: {}", "Organization".dimmed(), org));
        }

        lines.push(format!("{}: {}", self.key, self.value));

        if let Some(source) = &self.source {
            lines.push(format!("{}: {}", "Source".dimmed(), source));
        }

        lines.join("\n")
    }
}

impl HumanDisplay for ConfigListResponse {
    fn human_fmt(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("{}: {}", "Config path".dimmed(), self.config_path));

        if self.organizations.is_empty() {
            lines.push(format!("\n{}", "No organizations configured.".dimmed()));
        } else {
            lines.push(format!("\n{}", "Organizations:".bold()));
            for org in &self.organizations {
                let default_marker = if org.is_default {
                    format!(" {}", "(default)".cyan())
                } else {
                    String::new()
                };
                let origin_marker = if let Some(ref origin) = org.origin {
                    format!(" {}", format!("({})", origin).dimmed())
                } else {
                    String::new()
                };
                lines.push(format!(
                    "  {} {}{}{}",
                    org.name.bold(),
                    format!("[{}]", org.token).dimmed(),
                    default_marker,
                    origin_marker
                ));
            }
        }

        if let Some(default) = &self.default_org {
            lines.push(format!("\n{}: {}", "Default org".dimmed(), default));
        }

        lines.join("\n")
    }
}

impl HumanDisplay for ConfigListOrgResponse {
    fn human_fmt(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "{}: {}",
            "Organization".dimmed(),
            self.organization
        ));
        lines.push(format!("{}: {}", "Token".dimmed(), self.token));
        lines.push(format!(
            "{}: {}",
            "Is default".dimmed(),
            if self.is_default { "yes" } else { "no" }
        ));

        lines.join("\n")
    }
}

impl HumanDisplay for ConfigUnsetResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

impl HumanDisplay for ConfigValidateResponse {
    fn human_fmt(&self) -> String {
        let mut lines = Vec::new();

        if self.is_valid && self.issues.is_empty() {
            lines.push(format!("{} Configuration is valid.", "OK".green().bold()));
        } else if self.is_valid {
            lines.push(format!(
                "{} Configuration is valid (with warnings).",
                "OK".yellow().bold()
            ));
        } else {
            lines.push(format!(
                "{} Configuration has errors.",
                "INVALID".red().bold()
            ));
        }

        if !self.issues.is_empty() {
            lines.push(format!("\n{}", "Issues found:".bold()));
            for issue in &self.issues {
                let severity_colored = match issue.severity.as_str() {
                    "error" => "ERROR".red().bold().to_string(),
                    "warning" => "WARNING".yellow().bold().to_string(),
                    _ => issue.severity.clone(),
                };
                let field_str = if let Some(field) = &issue.field {
                    format!(" [{}]", field)
                } else {
                    String::new()
                };
                lines.push(format!(
                    "  {}{}: {}",
                    severity_colored, field_str, issue.message
                ));
            }
        }

        lines.join("\n")
    }
}

/// Output a config response with the appropriate format.
fn output_config<T: Serialize + HumanDisplay>(data: &T, format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("{}", data.human_fmt());
        }
        OutputFormat::Json => {
            output_success(data);
        }
    }
}

/// Set a configuration value.
pub fn set_config(
    key: &str,
    value: &str,
    org: Option<&str>,
    scope: Option<ConfigScope>,
    format: OutputFormat,
) -> Result<()> {
    match key {
        "token" => {
            // Determine which config to load based on scope
            let mut config = match scope {
                Some(ConfigScope::Global) => Config::load_global()?,
                Some(ConfigScope::Local) => Config::load_local()?.unwrap_or_default(),
                None => {
                    // Auto-detect: use local if exists, otherwise global
                    if Config::find_local_config_path().is_some() {
                        Config::load_local()?.unwrap_or_default()
                    } else {
                        Config::load_global()?
                    }
                }
            };

            // Determine organization name
            let org_name = if let Some(org_name) = org {
                org_name.to_string()
            } else if config.organizations.is_empty() {
                "default".to_string()
            } else if let Some(default) = &config.default_org {
                default.clone()
            } else {
                return Err(crate::error::LinError::config(
                    "Multiple organizations exist. Please specify which one to update with --org <name>".to_string()
                ));
            };

            let is_first = config.organizations.is_empty();
            config.add_org(&org_name, value);
            config.save_with_scope(scope)?;

            // Get the path where we saved
            let config_path = match scope {
                Some(ConfigScope::Global) => Config::global_config_path(),
                Some(ConfigScope::Local) => std::path::PathBuf::from(".lin/config.json"),
                None => {
                    if Config::find_local_config_path().is_some() {
                        std::path::PathBuf::from(".lin/config.json")
                    } else {
                        Config::global_config_path()
                    }
                }
            };

            let response = ConfigSetResponse {
                message: format!(
                    "Token set for organization '{}'{}",
                    org_name,
                    if is_first { " and set as default" } else { "" }
                ),
                key: key.to_string(),
                value_or_org: org_name,
                config_path: config_path.display().to_string(),
            };

            output_config(&response, format);
            Ok(())
        }
        "default-org" => {
            // Determine which config to load based on scope
            let mut config = match scope {
                Some(ConfigScope::Global) => Config::load_global()?,
                Some(ConfigScope::Local) => Config::load_local()?.unwrap_or_default(),
                None => {
                    // Auto-detect: use local if exists, otherwise global
                    if Config::find_local_config_path().is_some() {
                        Config::load_local()?.unwrap_or_default()
                    } else {
                        Config::load_global()?
                    }
                }
            };

            config.set_default(value)?;
            config.save_with_scope(scope)?;

            // Get the path where we saved
            let config_path = match scope {
                Some(ConfigScope::Global) => Config::global_config_path(),
                Some(ConfigScope::Local) => std::path::PathBuf::from(".lin/config.json"),
                None => {
                    if Config::find_local_config_path().is_some() {
                        std::path::PathBuf::from(".lin/config.json")
                    } else {
                        Config::global_config_path()
                    }
                }
            };

            let response = ConfigSetResponse {
                message: format!("Default organization set to '{}'", value),
                key: key.to_string(),
                value_or_org: value.to_string(),
                config_path: config_path.display().to_string(),
            };

            output_config(&response, format);
            Ok(())
        }
        _ => Err(crate::error::LinError::config(format!(
            "Unknown configuration key '{}'. Valid keys: token, default-org",
            key
        ))),
    }
}

/// Get a configuration value.
pub fn get_config(key: &str, org: Option<&str>, format: OutputFormat) -> Result<()> {
    match key {
        "token" => {
            let config = Config::load()?;
            let token = config.get_token(org)?;

            let (masked_value, full_value) = match format {
                OutputFormat::Human => (Config::mask_token(&token), token.clone()),
                OutputFormat::Json => (token.clone(), token.clone()),
            };

            let response = ConfigGetResponse {
                key: key.to_string(),
                value: if matches!(format, OutputFormat::Human) {
                    masked_value
                } else {
                    full_value
                },
                source: Some("config".to_string()),
                organization: Some(
                    org.map(|s| s.to_string())
                        .or_else(|| config.default_org.clone())
                        .unwrap_or_else(|| "default".to_string()),
                ),
            };

            output_config(&response, format);
            Ok(())
        }
        "default-org" => {
            let config = Config::load()?;
            let default = config.default_org.ok_or_else(|| {
                crate::error::LinError::config("No default organization is set".to_string())
            })?;

            let response = ConfigGetResponse {
                key: key.to_string(),
                value: default,
                source: None,
                organization: None,
            };

            output_config(&response, format);
            Ok(())
        }
        _ => Err(crate::error::LinError::config(format!(
            "Unknown configuration key '{}'. Valid keys: token, default-org",
            key
        ))),
    }
}

/// List configuration values.
pub fn list_config(org: Option<&str>, show_origin: bool, format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
    let config_path = Config::config_path().display().to_string();

    if let Some(org_name) = org {
        // List specific organization
        let token = config.organizations.get(org_name).ok_or_else(|| {
            crate::error::LinError::config(format!(
                "Organization '{}' not found in configuration",
                org_name
            ))
        })?;

        let masked_token = Config::mask_token(token);
        let is_default = config.default_org.as_deref() == Some(org_name);

        let response = ConfigListOrgResponse {
            organization: org_name.to_string(),
            token: masked_token,
            is_default,
        };

        output_config(&response, format);
    } else {
        // List all configuration
        let organizations: Vec<OrgInfo> = if show_origin {
            // Load local and global configs separately to show origins
            let global_config = Config::load_global()?;
            let local_config = Config::load_local()?;

            let mut org_map: std::collections::HashMap<String, (String, String)> =
                std::collections::HashMap::new();

            // Add global orgs
            for (name, token) in &global_config.organizations {
                org_map.insert(name.clone(), (token.clone(), "global".to_string()));
            }

            // Override with local orgs
            if let Some(ref local) = local_config {
                for (name, token) in &local.organizations {
                    org_map.insert(name.clone(), (token.clone(), "local".to_string()));
                }
            }

            org_map
                .iter()
                .map(|(name, (token, origin))| OrgInfo {
                    name: name.clone(),
                    token: Config::mask_token(token),
                    is_default: config.default_org.as_deref() == Some(name.as_str()),
                    origin: Some(origin.clone()),
                })
                .collect()
        } else {
            config
                .organizations
                .iter()
                .map(|(name, token)| OrgInfo {
                    name: name.clone(),
                    token: Config::mask_token(token),
                    is_default: config.default_org.as_deref() == Some(name.as_str()),
                    origin: None,
                })
                .collect()
        };

        let response = ConfigListResponse {
            default_org: config.default_org.clone(),
            organizations,
            config_path,
        };

        output_config(&response, format);
    }

    Ok(())
}

/// Remove a configuration value.
pub fn unset_config(
    key: &str,
    org: Option<&str>,
    scope: Option<ConfigScope>,
    format: OutputFormat,
) -> Result<()> {
    match key {
        "token" => {
            // Determine which config to load based on scope
            let mut config = match scope {
                Some(ConfigScope::Global) => Config::load_global()?,
                Some(ConfigScope::Local) => Config::load_local()?.ok_or_else(|| {
                    crate::error::LinError::config("No local config found".to_string())
                })?,
                None => {
                    // Auto-detect: use local if exists, otherwise global
                    if Config::find_local_config_path().is_some() {
                        Config::load_local()?.unwrap_or_default()
                    } else {
                        Config::load_global()?
                    }
                }
            };

            // Determine organization name
            let org_name = if let Some(org_name) = org {
                org_name.to_string()
            } else if let Some(default) = &config.default_org {
                default.clone()
            } else if config.organizations.len() == 1 {
                config.organizations.keys().next().unwrap().clone()
            } else {
                return Err(crate::error::LinError::config(
                    "Multiple organizations exist. Please specify which one to remove with --org <name>".to_string()
                ));
            };

            // Check if this is the only or default org and prompt for confirmation
            let is_only = config.organizations.len() == 1;
            let is_default = config.default_org.as_deref() == Some(&org_name);

            if is_only || is_default {
                eprint!("This will remove ");
                if is_only {
                    eprint!("the only organization");
                } else {
                    eprint!("the default organization");
                }
                eprint!(". Continue? [y/N] ");
                io::stderr().flush()?;

                let mut response = String::new();
                io::stdin().lock().read_line(&mut response)?;

                if !response.trim().eq_ignore_ascii_case("y") {
                    return Err(crate::error::LinError::config(
                        "Operation cancelled".to_string(),
                    ));
                }
            }

            config.remove_org(&org_name)?;
            config.save_with_scope(scope)?;

            let response = ConfigUnsetResponse {
                message: format!("Token removed for organization '{}'", org_name),
                key: key.to_string(),
            };

            output_config(&response, format);
            Ok(())
        }
        "default-org" => {
            // Determine which config to load based on scope
            let mut config = match scope {
                Some(ConfigScope::Global) => Config::load_global()?,
                Some(ConfigScope::Local) => Config::load_local()?.ok_or_else(|| {
                    crate::error::LinError::config("No local config found".to_string())
                })?,
                None => {
                    // Auto-detect: use local if exists, otherwise global
                    if Config::find_local_config_path().is_some() {
                        Config::load_local()?.unwrap_or_default()
                    } else {
                        Config::load_global()?
                    }
                }
            };

            config.default_org = None;
            config.save_with_scope(scope)?;

            let response = ConfigUnsetResponse {
                message: "Default organization cleared".to_string(),
                key: key.to_string(),
            };

            output_config(&response, format);
            Ok(())
        }
        _ => Err(crate::error::LinError::config(format!(
            "Unknown configuration key '{}'. Valid keys: token, default-org",
            key
        ))),
    }
}

/// Validate the configuration file.
pub fn validate_config(format: OutputFormat) -> Result<()> {
    let (config, mut file_issues) = Config::load_and_validate_file()?;

    let validation_result = config.validate();

    // Combine file issues with validation issues
    let mut all_issues: Vec<ValidationIssue> = file_issues
        .drain(..)
        .map(|issue| issue_to_response(&issue))
        .collect();

    all_issues.extend(validation_result.issues.iter().map(issue_to_response));

    let is_valid = validation_result.is_valid
        && !file_issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error);

    let response = ConfigValidateResponse {
        is_valid,
        issues: all_issues,
    };

    output_config(&response, format);
    Ok(())
}

/// Convert a ConfigValidationIssue to a ValidationIssue.
fn issue_to_response(issue: &ConfigValidationIssue) -> ValidationIssue {
    ValidationIssue {
        severity: match issue.severity {
            ValidationSeverity::Error => "error".to_string(),
            ValidationSeverity::Warning => "warning".to_string(),
        },
        field: Some(issue.field.clone()),
        message: issue.message.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_set_response_serialization() {
        let response = ConfigSetResponse {
            message: "Token set".to_string(),
            key: "token".to_string(),
            value_or_org: "my-org".to_string(),
            config_path: "/path/to/config".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"key\":\"token\""));
        assert!(json.contains("\"value_or_org\":\"my-org\""));
    }

    #[test]
    fn test_config_get_response_serialization() {
        let response = ConfigGetResponse {
            key: "token".to_string(),
            value: "lin_api_abc123...".to_string(),
            source: Some("config".to_string()),
            organization: Some("my-org".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"key\":\"token\""));
        assert!(json.contains("\"organization\":\"my-org\""));
    }

    #[test]
    fn test_validation_issue_serialization() {
        let issue = ValidationIssue {
            severity: "error".to_string(),
            field: Some("token".to_string()),
            message: "Invalid token".to_string(),
        };
        let json = serde_json::to_string(&issue).unwrap();
        assert!(json.contains("\"severity\":\"error\""));
        assert!(json.contains("\"field\":\"token\""));
    }
}
