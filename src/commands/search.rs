//! Search command for full-text issue search.
//!
//! Provides commands for searching issues using Linear's full-text search capability.

use crate::api::queries::search::ISSUE_SEARCH_QUERY;
use crate::api::GraphQLClient;
use crate::models::IssueSearchResponse;
use crate::output::{output, OutputFormat};
use crate::Result;

/// Options for searching issues.
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Filter by team key (e.g., "ENG").
    pub team: Option<String>,
    /// Filter by assignee ID or "me".
    pub assignee: Option<String>,
    /// Filter by state name.
    pub state: Option<String>,
    /// Maximum number of issues to return (default 50).
    pub limit: Option<i32>,
}

/// Search issues using full-text search.
///
/// Searches issues in the Linear workspace using Linear's full-text search capability.
/// The search query matches against issue titles, descriptions, and other text content.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `query` - The search query string
/// * `viewer_id` - The current user's ID (used if assignee is "me")
/// * `options` - Filter options for the search
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::search::{search_issues, SearchOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = SearchOptions {
///     team: Some("ENG".to_string()),
///     limit: Some(10),
///     ..Default::default()
/// };
/// search_issues(&client, "authentication bug", None, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn search_issues(
    client: &GraphQLClient,
    query: &str,
    viewer_id: Option<&str>,
    options: SearchOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build the filter object with searchableContent for text search
    let mut filter = serde_json::Map::new();

    // Add searchableContent filter for the query text
    filter.insert(
        "searchableContent".to_string(),
        serde_json::json!({ "contains": query }),
    );

    // Add team filter if specified
    if let Some(team_key) = &options.team {
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "key": { "eq": team_key } }),
        );
    }

    // Add assignee filter if specified
    if let Some(assignee) = &options.assignee {
        let assignee_id = if assignee.to_lowercase() == "me" {
            viewer_id
                .ok_or_else(|| {
                    crate::error::LinError::config(
                        "Cannot use 'me' as assignee without viewer ID. Please authenticate first.",
                    )
                })?
                .to_string()
        } else {
            assignee.clone()
        };
        filter.insert(
            "assignee".to_string(),
            serde_json::json!({ "id": { "eq": assignee_id } }),
        );
    }

    // Add state filter if specified
    if let Some(state_name) = &options.state {
        filter.insert(
            "state".to_string(),
            serde_json::json!({ "name": { "eq": state_name } }),
        );
    }

    // Build variables
    let mut variables = serde_json::Map::new();
    variables.insert(
        "first".to_string(),
        serde_json::json!(options.limit.unwrap_or(50)),
    );
    variables.insert("filter".to_string(), serde_json::Value::Object(filter));

    let response: IssueSearchResponse =
        client.query(ISSUE_SEARCH_QUERY, serde_json::Value::Object(variables))?;

    output(&response.issues.nodes, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn create_mock_client(server: &Server) -> GraphQLClient {
        GraphQLClient::with_url("test-token", &server.url())
    }

    #[test]
    fn test_search_issues_basic() {
        let mut server = Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-1",
                                    "identifier": "ENG-123",
                                    "title": "Fix authentication bug",
                                    "description": "Users cannot log in",
                                    "priority": 2,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-02T00:00:00.000Z",
                                    "state": {
                                        "id": "state-1",
                                        "name": "In Progress",
                                        "color": "#0066ff",
                                        "type": "started"
                                    },
                                    "team": {
                                        "id": "team-1",
                                        "key": "ENG",
                                        "name": "Engineering",
                                        "description": null
                                    },
                                    "assignee": null
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        let client = create_mock_client(&server);
        let options = SearchOptions::default();

        let result = search_issues(&client, "authentication", None, options, OutputFormat::Json);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_issues_with_team_filter() {
        let mut server = Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = create_mock_client(&server);
        let options = SearchOptions {
            team: Some("ENG".to_string()),
            ..Default::default()
        };

        let result = search_issues(&client, "bug", None, options, OutputFormat::Json);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_issues_with_state_filter() {
        let mut server = Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = create_mock_client(&server);
        let options = SearchOptions {
            state: Some("In Progress".to_string()),
            ..Default::default()
        };

        let result = search_issues(&client, "feature", None, options, OutputFormat::Json);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_issues_with_assignee_me_requires_viewer_id() {
        let server = Server::new();

        // No mock needed - should fail before making request
        let client = create_mock_client(&server);
        let options = SearchOptions {
            assignee: Some("me".to_string()),
            ..Default::default()
        };

        let result = search_issues(&client, "task", None, options, OutputFormat::Json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("viewer ID"));
    }

    #[test]
    fn test_search_issues_with_assignee_me_and_viewer_id() {
        let mut server = Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = create_mock_client(&server);
        let options = SearchOptions {
            assignee: Some("me".to_string()),
            ..Default::default()
        };

        let result = search_issues(
            &client,
            "my task",
            Some("user-123"),
            options,
            OutputFormat::Json,
        );

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_issues_with_limit() {
        let mut server = Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = create_mock_client(&server);
        let options = SearchOptions {
            limit: Some(10),
            ..Default::default()
        };

        let result = search_issues(&client, "urgent", None, options, OutputFormat::Json);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_issues_with_all_filters() {
        let mut server = Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-2",
                                    "identifier": "ENG-456",
                                    "title": "Urgent bug fix",
                                    "description": null,
                                    "priority": 1,
                                    "createdAt": "2024-01-03T00:00:00.000Z",
                                    "updatedAt": "2024-01-03T00:00:00.000Z",
                                    "state": {
                                        "id": "state-2",
                                        "name": "Todo",
                                        "color": "#cccccc",
                                        "type": "unstarted"
                                    },
                                    "team": {
                                        "id": "team-1",
                                        "key": "ENG",
                                        "name": "Engineering",
                                        "description": null
                                    },
                                    "assignee": {
                                        "id": "user-1",
                                        "name": "John Doe",
                                        "email": "john@example.com",
                                        "displayName": "JD",
                                        "active": true
                                    }
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        let client = create_mock_client(&server);
        let options = SearchOptions {
            team: Some("ENG".to_string()),
            assignee: Some("user-1".to_string()),
            state: Some("Todo".to_string()),
            limit: Some(25),
        };

        let result = search_issues(&client, "urgent", None, options, OutputFormat::Json);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert!(options.team.is_none());
        assert!(options.assignee.is_none());
        assert!(options.state.is_none());
        assert!(options.limit.is_none());
    }
}
