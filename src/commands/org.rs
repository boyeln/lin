//! Organization management commands.
//!
//! Commands for managing organization configurations including
//! adding, removing, and listing organizations and their API tokens.

use std::io::{self, BufRead, Write};

use serde::Serialize;

use crate::config::Config;
use crate::output::{output_success, HumanDisplay, OutputFormat};
use crate::Result;

/// Response for the org add command.
#[derive(Serialize)]
pub struct OrgAddResponse {
    pub message: String,
    pub organization: String,
    pub is_default: bool,
}

/// Response for the org remove command.
#[derive(Serialize)]
pub struct OrgRemoveResponse {
    pub message: String,
    pub organization: String,
}

/// Response for the org list command.
#[derive(Serialize)]
pub struct OrgListResponse {
    pub organizations: Vec<OrgInfo>,
    pub default_org: Option<String>,
}

/// Information about a configured organization.
#[derive(Serialize)]
pub struct OrgInfo {
    pub name: String,
    pub is_default: bool,
}

/// Response for the org set-default command.
#[derive(Serialize)]
pub struct OrgSetDefaultResponse {
    pub message: String,
    pub organization: String,
}

impl HumanDisplay for OrgAddResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

impl HumanDisplay for OrgRemoveResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

impl HumanDisplay for OrgListResponse {
    fn human_fmt(&self) -> String {
        if self.organizations.is_empty() {
            "No organizations configured.".to_string()
        } else {
            self.organizations
                .iter()
                .map(|org| {
                    if org.is_default {
                        format!("* {} (default)", org.name)
                    } else {
                        format!("  {}", org.name)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

impl HumanDisplay for OrgSetDefaultResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

/// Output an org response with the appropriate format.
fn output_org<T: Serialize + HumanDisplay>(data: &T, format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("{}", data.human_fmt());
        }
        OutputFormat::Json => {
            output_success(data);
        }
    }
}

/// Add an organization to the configuration.
///
/// Reads the API token from stdin to allow piping tokens securely.
///
/// # Arguments
///
/// * `name` - The name to identify this organization
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```bash
/// echo "lin_api_xxxxx" | lin org add my-company
/// ```
pub fn add_org(name: &str, format: OutputFormat) -> Result<()> {
    // Read token from stdin
    let token = read_token_from_stdin()?;

    let mut config = Config::load()?;
    let is_first = config.organizations.is_empty();
    config.add_org(name, &token);
    config.save()?;

    let response = OrgAddResponse {
        message: format!(
            "Organization '{}' added successfully{}",
            name,
            if is_first { " and set as default" } else { "" }
        ),
        organization: name.to_string(),
        is_default: is_first || config.default_org.as_deref() == Some(name),
    };

    output_org(&response, format);
    Ok(())
}

/// Remove an organization from the configuration.
///
/// # Arguments
///
/// * `name` - The name of the organization to remove
/// * `format` - The output format (Human or Json)
pub fn remove_org(name: &str, format: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;
    config.remove_org(name)?;
    config.save()?;

    let response = OrgRemoveResponse {
        message: format!("Organization '{}' removed successfully", name),
        organization: name.to_string(),
    };

    output_org(&response, format);
    Ok(())
}

/// List all configured organizations.
///
/// # Arguments
///
/// * `format` - The output format (Human or Json)
pub fn list_orgs(format: OutputFormat) -> Result<()> {
    let config = Config::load()?;

    let organizations: Vec<OrgInfo> = config
        .list_orgs()
        .into_iter()
        .map(|name| OrgInfo {
            name: name.to_string(),
            is_default: config.default_org.as_deref() == Some(name),
        })
        .collect();

    let response = OrgListResponse {
        organizations,
        default_org: config.default_org.clone(),
    };

    output_org(&response, format);
    Ok(())
}

/// Set the default organization.
///
/// # Arguments
///
/// * `name` - The name of the organization to set as default
/// * `format` - The output format (Human or Json)
pub fn set_default_org(name: &str, format: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;
    config.set_default(name)?;
    config.save()?;

    let response = OrgSetDefaultResponse {
        message: format!("Default organization set to '{}'", name),
        organization: name.to_string(),
    };

    output_org(&response, format);
    Ok(())
}

/// Read the API token from stdin.
///
/// Supports both piped input and interactive terminal input.
fn read_token_from_stdin() -> Result<String> {
    let stdin = io::stdin();

    // Check if stdin is a TTY (interactive terminal)
    if atty::is(atty::Stream::Stdin) {
        // Interactive mode: prompt user
        eprint!("Enter API token: ");
        io::stderr().flush()?;
    }

    let mut token = String::new();
    stdin.lock().read_line(&mut token)?;

    let token = token.trim().to_string();

    if token.is_empty() {
        return Err(crate::error::LinError::config(
            "API token cannot be empty. Please provide a valid Linear API token.",
        ));
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_info_serialization() {
        let info = OrgInfo {
            name: "test-org".to_string(),
            is_default: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"test-org\""));
        assert!(json.contains("\"is_default\":true"));
    }

    #[test]
    fn test_org_list_response_serialization() {
        let response = OrgListResponse {
            organizations: vec![
                OrgInfo {
                    name: "org1".to_string(),
                    is_default: true,
                },
                OrgInfo {
                    name: "org2".to_string(),
                    is_default: false,
                },
            ],
            default_org: Some("org1".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("org1"));
        assert!(json.contains("org2"));
        assert!(json.contains("\"default_org\":\"org1\""));
    }

    #[test]
    fn test_org_add_response_serialization() {
        let response = OrgAddResponse {
            message: "Organization added".to_string(),
            organization: "my-org".to_string(),
            is_default: true,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"organization\":\"my-org\""));
        assert!(json.contains("\"is_default\":true"));
    }
}
