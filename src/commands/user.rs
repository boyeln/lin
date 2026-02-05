//! User management commands.
//!
//! Commands for getting user information from Linear.

use crate::api::{queries, GraphQLClient};
use crate::models::{UsersResponse, ViewerResponse};
use crate::output::{output, OutputFormat};
use crate::Result;

/// Get the current authenticated user's information.
///
/// Fetches the viewer (authenticated user) from the Linear API and outputs
/// the user data.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::user::me;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// me(&client, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn me(client: &GraphQLClient, format: OutputFormat) -> Result<()> {
    let response: ViewerResponse = client.query(queries::VIEWER_QUERY, serde_json::json!({}))?;
    output(&response.viewer, format);
    Ok(())
}

/// List all users in the organization.
///
/// Fetches users from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::user::list_users;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// list_users(&client, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_users(client: &GraphQLClient, format: OutputFormat) -> Result<()> {
    let response: UsersResponse = client.query(queries::USERS_QUERY, serde_json::json!({}))?;
    output(&response.users.nodes, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    // =============================================================================
    // me command tests
    // =============================================================================

    #[test]
    fn test_me_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "viewer": {
                            "id": "user-123",
                            "name": "Test User",
                            "email": "test@example.com",
                            "displayName": "TU",
                            "active": true
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = me(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_me_with_null_display_name() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with null display name
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "viewer": {
                            "id": "user-456",
                            "name": "Another User",
                            "email": "another@example.com",
                            "displayName": null,
                            "active": true
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = me(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_me_inactive_user() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response for inactive user
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "viewer": {
                            "id": "user-789",
                            "name": "Inactive User",
                            "email": "inactive@example.com",
                            "displayName": "IU",
                            "active": false
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = me(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_me_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result = me(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_me_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP error
        let mock = server
            .mock("POST", "/")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result = me(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_me_http_500_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP 500
        let mock = server
            .mock("POST", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = me(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }

    // =============================================================================
    // list_users command tests
    // =============================================================================

    #[test]
    fn test_list_users_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "users": {
                            "nodes": [
                                {
                                    "id": "user-1",
                                    "name": "Alice",
                                    "email": "alice@example.com",
                                    "displayName": "A",
                                    "active": true
                                },
                                {
                                    "id": "user-2",
                                    "name": "Bob",
                                    "email": "bob@example.com",
                                    "displayName": "B",
                                    "active": true
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty users
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "users": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_with_mixed_active_status() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with mixed active status
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "users": {
                            "nodes": [
                                {
                                    "id": "user-1",
                                    "name": "Active User",
                                    "email": "active@example.com",
                                    "displayName": "AU",
                                    "active": true
                                },
                                {
                                    "id": "user-2",
                                    "name": "Inactive User",
                                    "email": "inactive@example.com",
                                    "displayName": null,
                                    "active": false
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP error
        let mock = server
            .mock("POST", "/")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_http_500_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP 500
        let mock = server
            .mock("POST", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_single_user() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with single user
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "users": {
                            "nodes": [
                                {
                                    "id": "user-solo",
                                    "name": "Solo User",
                                    "email": "solo@example.com",
                                    "displayName": "Solo",
                                    "active": true
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_users_many_users() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with many users
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "users": {
                            "nodes": [
                                {"id": "user-1", "name": "User 1", "email": "u1@example.com", "displayName": "U1", "active": true},
                                {"id": "user-2", "name": "User 2", "email": "u2@example.com", "displayName": "U2", "active": true},
                                {"id": "user-3", "name": "User 3", "email": "u3@example.com", "displayName": "U3", "active": true},
                                {"id": "user-4", "name": "User 4", "email": "u4@example.com", "displayName": "U4", "active": true},
                                {"id": "user-5", "name": "User 5", "email": "u5@example.com", "displayName": "U5", "active": false}
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_users(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }
}
