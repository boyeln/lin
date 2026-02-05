//! List operations for issues.

use crate::api::queries::issue::ISSUES_QUERY;
use crate::api::GraphQLClient;
use crate::error::LinError;
use crate::models::IssuesResponse;
use crate::output::{output, OutputFormat};
use crate::Result;

use super::IssueListOptions;

/// List issues with optional filters.
///
/// Fetches issues from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `viewer_id` - The current user's ID (used if assignee is "me")
/// * `options` - Filter options for the query
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::list::list_issues;
/// use lin::commands::issue::IssueListOptions;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = IssueListOptions {
///     team: Some("ENG".to_string()),
///     assignee: None,
///     state: None,
///     limit: Some(10),
///     ..Default::default()
/// };
/// list_issues(&client, None, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_issues(
    client: &GraphQLClient,
    viewer_id: Option<&str>,
    options: IssueListOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build the filter object
    let mut filter = serde_json::Map::new();

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
                    LinError::config(
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

    // Add project filter if specified
    if let Some(project_id) = &options.project {
        filter.insert(
            "project".to_string(),
            serde_json::json!({ "id": { "eq": project_id } }),
        );
    }

    // Add cycle filter if specified
    if let Some(cycle_id) = &options.cycle {
        filter.insert(
            "cycle".to_string(),
            serde_json::json!({ "id": { "eq": cycle_id } }),
        );
    }

    // Add label filter if specified
    if let Some(label_id) = &options.label {
        filter.insert(
            "labels".to_string(),
            serde_json::json!({ "id": { "eq": label_id } }),
        );
    }

    // Add priority filter if specified
    if let Some(priority) = &options.priority {
        filter.insert(
            "priority".to_string(),
            serde_json::json!({ "eq": priority.to_value() }),
        );
    }

    // Add createdAt date filters
    let mut created_at_filter = serde_json::Map::new();
    if let Some(created_after) = &options.created_after {
        created_at_filter.insert("gt".to_string(), serde_json::json!(created_after));
    }
    if let Some(created_before) = &options.created_before {
        created_at_filter.insert("lt".to_string(), serde_json::json!(created_before));
    }
    if !created_at_filter.is_empty() {
        filter.insert(
            "createdAt".to_string(),
            serde_json::Value::Object(created_at_filter),
        );
    }

    // Add updatedAt date filters
    let mut updated_at_filter = serde_json::Map::new();
    if let Some(updated_after) = &options.updated_after {
        updated_at_filter.insert("gt".to_string(), serde_json::json!(updated_after));
    }
    if let Some(updated_before) = &options.updated_before {
        updated_at_filter.insert("lt".to_string(), serde_json::json!(updated_before));
    }
    if !updated_at_filter.is_empty() {
        filter.insert(
            "updatedAt".to_string(),
            serde_json::Value::Object(updated_at_filter),
        );
    }

    // Build variables
    let mut variables = serde_json::Map::new();
    variables.insert(
        "first".to_string(),
        serde_json::json!(options.limit.unwrap_or(50)),
    );

    if !filter.is_empty() {
        variables.insert("filter".to_string(), serde_json::Value::Object(filter));
    }

    // Add sorting if specified
    if let Some(sort_field) = &options.sort_by {
        variables.insert(
            "orderBy".to_string(),
            serde_json::json!(sort_field.to_graphql_field()),
        );
    }

    let response: IssuesResponse =
        client.query(ISSUES_QUERY, serde_json::Value::Object(variables))?;

    // If we have a sort order that differs from the default, we need to reverse the results
    // because Linear's API doesn't support explicit sort direction via GraphQL
    let mut issues = response.issues.nodes;
    if let Some(sort_field) = &options.sort_by {
        let default_order = sort_field.default_order();
        let requested_order = options.sort_order.unwrap_or(default_order);
        if requested_order != default_order {
            issues.reverse();
        }
    }

    output(&issues, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::commands::issue::{IssueSortField, PriorityFilter, SortOrder};
    use crate::output::OutputFormat;

    #[test]
    fn test_list_issues_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-1",
                                    "identifier": "ENG-1",
                                    "title": "First issue",
                                    "description": "Description 1",
                                    "priority": 1,
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
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                },
                                {
                                    "id": "issue-2",
                                    "identifier": "ENG-2",
                                    "title": "Second issue",
                                    "description": null,
                                    "priority": 2,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-02T00:00:00.000Z",
                                    "updatedAt": "2024-01-02T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions::default();

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_team_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_assignee_me() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            assignee: Some("me".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, Some("user-123"), options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_assignee_me_no_viewer() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            assignee: Some("me".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Cannot use 'me'"));
    }

    #[test]
    fn test_list_issues_with_state_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            state: Some("In Progress".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_project_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            project: Some("project-123".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_limit() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            limit: Some(10),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_all_filters() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            assignee: Some("user-456".to_string()),
            state: Some("Done".to_string()),
            project: Some("project-789".to_string()),
            cycle: None,
            label: None,
            limit: Some(25),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_api_error() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let options = IssueListOptions::default();

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));
        mock.assert();
    }

    #[test]
    fn test_list_issues_empty() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions::default();

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    // =============================================================================
    // Date filter tests
    // =============================================================================

    #[test]
    fn test_list_issues_with_created_after_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            created_after: Some("2024-01-01".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_created_before_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            created_before: Some("2024-12-31".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_updated_after_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            updated_after: Some("2024-06-01".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_updated_before_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            updated_before: Some("2024-06-30".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_date_range_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            created_after: Some("2024-01-01".to_string()),
            created_before: Some("2024-06-30".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_all_date_filters() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            created_after: Some("2024-01-01".to_string()),
            created_before: Some("2024-12-31".to_string()),
            updated_after: Some("2024-06-01".to_string()),
            updated_before: Some("2024-06-30".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_date_and_other_filters() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            state: Some("In Progress".to_string()),
            created_after: Some("2024-01-01".to_string()),
            updated_before: Some("2024-12-31".to_string()),
            limit: Some(25),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    // =============================================================================
    // list_issues with sort tests
    // =============================================================================

    #[test]
    fn test_list_issues_with_sort_by_priority() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Priority),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_by_created() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Created),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_by_updated() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Updated),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_by_title() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Title),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_and_order_asc() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Priority),
            sort_order: Some(SortOrder::Asc),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_and_order_desc() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Created),
            sort_order: Some(SortOrder::Desc),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_and_filters() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            state: Some("In Progress".to_string()),
            sort_by: Some(IssueSortField::Updated),
            sort_order: Some(SortOrder::Desc),
            limit: Some(25),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_sort_returns_issues() {
        let mut server = mockito::Server::new();

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
                                    "identifier": "ENG-1",
                                    "title": "First issue",
                                    "description": null,
                                    "priority": 1,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                },
                                {
                                    "id": "issue-2",
                                    "identifier": "ENG-2",
                                    "title": "Second issue",
                                    "description": null,
                                    "priority": 2,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-02T00:00:00.000Z",
                                    "updatedAt": "2024-01-02T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            sort_by: Some(IssueSortField::Priority),
            sort_order: Some(SortOrder::Asc),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_priority_filter() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            priority: Some(PriorityFilter::Urgent),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_combined_filters() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        // Combine multiple filters: team + assignee + priority + state
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            assignee: Some("user-123".to_string()),
            state: Some("In Progress".to_string()),
            priority: Some(PriorityFilter::High),
            limit: Some(10),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_all_filters_including_priority() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        // Combine all available filters
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            assignee: Some("user-123".to_string()),
            state: Some("In Progress".to_string()),
            project: Some("project-456".to_string()),
            cycle: Some("cycle-789".to_string()),
            label: Some("label-abc".to_string()),
            priority: Some(PriorityFilter::Urgent),
            limit: Some(25),
            created_after: Some("2024-01-01".to_string()),
            created_before: Some("2024-12-31".to_string()),
            updated_after: Some("2024-06-01".to_string()),
            updated_before: Some("2024-06-30".to_string()),
            sort_by: Some(IssueSortField::Priority),
            sort_order: Some(SortOrder::Asc),
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }
}
