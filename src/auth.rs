//! Authentication utilities for the lin CLI.
//!
//! Handles API token resolution with the following priority:
//! 1. CLI --api-token flag
//! 2. LINEAR_API_TOKEN environment variable
//! 3. Configuration file (per-organization tokens)

use std::env;

use crate::Result;
use crate::config::Config;
use crate::error::LinError;

/// Environment variable name for the Linear API token.
pub const LINEAR_API_TOKEN_ENV: &str = "LINEAR_API_TOKEN";

/// Resolve the API token to use for Linear API requests.
///
/// Token resolution priority:
/// 1. CLI `--api-token` flag (if provided)
/// 2. `LINEAR_API_TOKEN` environment variable
/// 3. Configuration file token for the specified or default organization
///
/// # Arguments
///
/// * `cli_token` - Optional token provided via CLI flag
/// * `config` - The loaded configuration
/// * `org` - Optional organization name to get the token for
///
/// # Errors
///
/// Returns an error if no token can be resolved from any source.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::auth::get_api_token;
/// use lin::config::Config;
///
/// let config = Config::load()?;
/// let token = get_api_token(None, &config, None)?;
/// # Ok(())
/// # }
/// ```
pub fn get_api_token(
    cli_token: Option<&str>,
    config: &Config,
    org: Option<&str>,
) -> Result<String> {
    // Priority 1: CLI --api-token flag
    if let Some(token) = cli_token {
        return Ok(token.to_string());
    }

    // Priority 2: LINEAR_API_TOKEN environment variable
    if let Ok(token) = env::var(LINEAR_API_TOKEN_ENV) {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // Priority 3: Config file (organization-specific token)
    config.get_token(org)
}

/// Check if an API token is available from any source.
///
/// This is useful for commands that optionally need authentication.
pub fn has_api_token(cli_token: Option<&str>, config: &Config, org: Option<&str>) -> bool {
    get_api_token(cli_token, config, org).is_ok()
}

/// Require an API token, returning a user-friendly error if none is available.
///
/// This wraps `get_api_token` with a more descriptive error message that
/// explains all the ways to provide a token.
pub fn require_api_token(
    cli_token: Option<&str>,
    config: &Config,
    org: Option<&str>,
) -> Result<String> {
    get_api_token(cli_token, config, org).map_err(|_| {
        LinError::config(
            "No API token found. Provide a token using one of these methods:\n\
             1. Use --api-token flag: lin --api-token <token> <command>\n\
             2. Set LINEAR_API_TOKEN environment variable\n\
             3. Add a token: lin config set token <value>",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config() -> Config {
        let mut config = Config::default();
        config.add_org("test-org", "config-token");
        config
    }

    #[test]
    fn test_cli_token_has_priority() {
        let config = make_test_config();

        // Even with env var set (we can't easily set this in tests),
        // CLI token should take priority
        let token = get_api_token(Some("cli-token"), &config, None).unwrap();
        assert_eq!(token, "cli-token");
    }

    #[test]
    fn test_config_token_fallback() {
        let config = make_test_config();

        // Clear any env var that might be set
        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };

        let token = get_api_token(None, &config, None).unwrap();
        assert_eq!(token, "config-token");
    }

    #[test]
    fn test_specific_org_token() {
        let mut config = Config::default();
        config.add_org("org1", "token1");
        config.add_org("org2", "token2");

        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };

        let token = get_api_token(None, &config, Some("org2")).unwrap();
        assert_eq!(token, "token2");
    }

    #[test]
    fn test_no_token_error() {
        let config = Config::default();
        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };

        let result = get_api_token(None, &config, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_api_token() {
        let config = make_test_config();
        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };

        assert!(has_api_token(Some("token"), &config, None));
        assert!(has_api_token(None, &config, None));

        let empty_config = Config::default();
        assert!(!has_api_token(None, &empty_config, None));
    }

    #[test]
    fn test_require_api_token_error_message() {
        let config = Config::default();
        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };

        let result = require_api_token(None, &config, None);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("--api-token"));
        assert!(err_msg.contains("LINEAR_API_TOKEN"));
        assert!(err_msg.contains("lin config set token"));
    }

    #[test]
    fn test_env_token_priority_over_config() {
        let config = make_test_config();

        // Set env var
        unsafe { env::set_var(LINEAR_API_TOKEN_ENV, "env-token") };

        let token = get_api_token(None, &config, None).unwrap();
        assert_eq!(token, "env-token");

        // Clean up
        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };
    }

    #[test]
    fn test_empty_env_var_falls_through() {
        let config = make_test_config();

        // Set empty env var
        unsafe { env::set_var(LINEAR_API_TOKEN_ENV, "") };

        let token = get_api_token(None, &config, None).unwrap();
        assert_eq!(token, "config-token");

        // Clean up
        unsafe { env::remove_var(LINEAR_API_TOKEN_ENV) };
    }
}
