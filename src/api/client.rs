//! GraphQL client for Linear's API.
//!
//! This module provides a blocking HTTP client for making GraphQL requests
//! to the Linear API.

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::LinError;
use crate::Result;

/// Default Linear API endpoint.
pub const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// GraphQL client for making requests to Linear's API.
///
/// The client handles authentication, request formatting, and response parsing
/// for GraphQL queries and mutations.
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct ViewerResponse {
///     viewer: User,
/// }
///
/// #[derive(Deserialize)]
/// struct User {
///     id: String,
///     name: String,
/// }
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let response: ViewerResponse = client.query(
///     "query { viewer { id name } }",
///     serde_json::json!({}),
/// )?;
/// ```
pub struct GraphQLClient {
    /// The API token for authentication.
    token: String,
    /// The GraphQL endpoint URL.
    base_url: String,
    /// The HTTP client.
    client: Client,
}

/// GraphQL request body.
#[derive(Serialize)]
struct GraphQLRequest<'a> {
    query: &'a str,
    variables: serde_json::Value,
}

/// Raw GraphQL response structure.
#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error structure.
#[derive(Deserialize, Debug)]
pub struct GraphQLError {
    /// Error message.
    pub message: String,
    /// Path to the field that caused the error.
    #[serde(default)]
    pub path: Vec<serde_json::Value>,
    /// Additional error extensions.
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
}

impl std::fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl GraphQLClient {
    /// Create a new GraphQL client with the given API token.
    ///
    /// Uses the default Linear API endpoint.
    ///
    /// # Arguments
    ///
    /// * `token` - Linear API token for authentication
    pub fn new(token: &str) -> Self {
        Self::with_url(token, LINEAR_API_URL)
    }

    /// Create a new GraphQL client with a custom URL.
    ///
    /// This is primarily useful for testing with a mock server.
    ///
    /// # Arguments
    ///
    /// * `token` - Linear API token for authentication
    /// * `url` - Custom GraphQL endpoint URL
    pub fn with_url(token: &str, url: &str) -> Self {
        let client = Client::new();
        Self {
            token: token.to_string(),
            base_url: url.to_string(),
            client,
        }
    }

    /// Execute a GraphQL query or mutation.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query or mutation string
    /// * `variables` - Variables to pass to the query
    ///
    /// # Returns
    ///
    /// Returns the deserialized response data on success.
    ///
    /// # Errors
    ///
    /// Returns `LinError::Api` if:
    /// - The HTTP request fails
    /// - The response contains GraphQL errors
    /// - The response cannot be parsed
    ///
    /// # Example
    ///
    /// ```ignore
    /// let variables = serde_json::json!({
    ///     "first": 10,
    ///     "teamId": "team-123"
    /// });
    /// let response: IssuesResponse = client.query(ISSUES_QUERY, variables)?;
    /// ```
    pub fn query<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        // Build headers
        let mut headers = HeaderMap::new();

        // Linear API expects just the token, not "Bearer token"
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.token)
                .map_err(|e| LinError::api(format!("Invalid token format: {}", e)))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Build request body
        let request_body = GraphQLRequest { query, variables };

        // Make the request
        let response = self
            .client
            .post(&self.base_url)
            .headers(headers)
            .json(&request_body)
            .send()
            .map_err(|e| LinError::api(format!("Request failed: {}", e)))?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(LinError::api(format!(
                "HTTP {} {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_text
            )));
        }

        // Parse the response
        let response_text = response
            .text()
            .map_err(|e| LinError::api(format!("Failed to read response: {}", e)))?;

        let graphql_response: GraphQLResponse<T> = serde_json::from_str(&response_text)
            .map_err(|e| LinError::parse(format!("Failed to parse response: {}", e)))?;

        // Check for GraphQL errors
        if let Some(errors) = graphql_response.errors {
            if !errors.is_empty() {
                let error_messages: Vec<String> =
                    errors.iter().map(|e| e.message.clone()).collect();
                return Err(LinError::api(format!(
                    "GraphQL errors: {}",
                    error_messages.join("; ")
                )));
            }
        }

        // Return the data
        graphql_response.data.ok_or_else(|| {
            LinError::api("GraphQL response contained no data".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_client_creation() {
        let client = GraphQLClient::new("test-token");
        assert_eq!(client.token, "test-token");
        assert_eq!(client.base_url, LINEAR_API_URL);
    }

    #[test]
    fn test_graphql_client_with_url() {
        let client = GraphQLClient::with_url("test-token", "http://localhost:8080/graphql");
        assert_eq!(client.token, "test-token");
        assert_eq!(client.base_url, "http://localhost:8080/graphql");
    }

    #[test]
    fn test_graphql_error_display() {
        let error = GraphQLError {
            message: "Test error message".to_string(),
            path: vec![],
            extensions: None,
        };
        assert_eq!(format!("{}", error), "Test error message");
    }

    #[test]
    fn test_graphql_request_serialization() {
        let request = GraphQLRequest {
            query: "query { viewer { id } }",
            variables: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"query\":\"query { viewer { id } }\""));
        assert!(json.contains("\"variables\""));
        assert!(json.contains("\"key\":\"value\""));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::models::ViewerResponse;

    #[test]
    fn test_successful_query_parsing() {
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
        let result: ViewerResponse = client
            .query("query { viewer { id name email displayName active } }", serde_json::json!({}))
            .expect("Query should succeed");

        // Verify response
        assert_eq!(result.viewer.id, "user-123");
        assert_eq!(result.viewer.name, "Test User");
        assert_eq!(result.viewer.email, "test@example.com");
        assert_eq!(result.viewer.display_name, Some("TU".to_string()));
        assert!(result.viewer.active);

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_graphql_error_handling() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL errors
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated",
                            "path": ["viewer"]
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("GraphQL errors"));
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_multiple_graphql_errors() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with multiple GraphQL errors
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {"message": "Error 1"},
                        {"message": "Error 2"}
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Verify error contains both messages
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Error 1"));
        assert!(err.to_string().contains("Error 2"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_http_error_handling() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP error
        let mock = server
            .mock("POST", "/")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));
        assert!(err.to_string().contains("Unauthorized"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_http_500_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP 500
        let mock = server
            .mock("POST", "/")
            .with_status(500)
            .with_header("content-type", "text/plain")
            .with_body("Internal Server Error")
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_invalid_json_response() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with invalid JSON
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not valid json")
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Verify error is a parse error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Parse error"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_empty_data_response() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with null data and no errors
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": null}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Verify error about no data
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("no data"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_authorization_header_is_token_only() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock that specifically checks the token is sent WITHOUT "Bearer" prefix
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "lin_api_xxxxx")  // Just the token, no "Bearer"
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "viewer": {
                            "id": "user-123",
                            "name": "Test",
                            "email": "test@example.com",
                            "displayName": null,
                            "active": true
                        }
                    }
                }"#,
            )
            .create();

        // Create client with a token that looks like a real Linear token
        let client = GraphQLClient::with_url("lin_api_xxxxx", &server.url());

        // Make request
        let result: Result<ViewerResponse> =
            client.query("query { viewer { id } }", serde_json::json!({}));

        // Should succeed if header matches
        assert!(result.is_ok());

        // Verify mock was called with correct auth header
        mock.assert();
    }

    #[test]
    fn test_query_with_variables() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
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
                            "active": false
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request with variables
        let variables = serde_json::json!({
            "first": 10,
            "filter": {"teamId": "team-123"}
        });
        let result: ViewerResponse = client
            .query("query($first: Int) { viewer { id name email displayName active } }", variables)
            .expect("Query should succeed");

        // Verify response
        assert_eq!(result.viewer.id, "user-456");
        assert!(!result.viewer.active);

        // Verify mock was called
        mock.assert();
    }
}
