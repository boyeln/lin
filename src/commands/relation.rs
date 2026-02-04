//! Issue relation management commands.
//!
//! Commands for listing, adding, and removing relations between Linear issues.
//! Supports parent/child, blocks/blocked by, and related relationships.

use crate::api::{queries, GraphQLClient};
use crate::error::LinError;
use crate::models::{
    IssueRelationCreateResponse, IssueRelationDeleteResponse, IssueRelationsResponse,
    IssueSetParentResponse, IssuesResponse, NormalizedRelation,
};
use crate::output::{output, OutputFormat};
use crate::Result;

use super::issue::{is_uuid, parse_identifier};

/// Relation types supported by Linear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// Parent/child relationship (this issue becomes a child)
    Parent,
    /// Parent/child relationship (this issue becomes a parent, i.e., make target a sub-issue)
    Sub,
    /// This issue blocks the target issue
    Blocks,
    /// This issue is blocked by the target issue
    BlockedBy,
    /// General related issue
    Related,
    /// Duplicate issue
    Duplicate,
}

impl RelationType {
    /// Parse a relation type from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "parent" => Some(RelationType::Parent),
            "sub" | "child" => Some(RelationType::Sub),
            "blocks" => Some(RelationType::Blocks),
            "blocked_by" | "blockedby" | "blocked-by" => Some(RelationType::BlockedBy),
            "related" => Some(RelationType::Related),
            "duplicate" | "dup" => Some(RelationType::Duplicate),
            _ => None,
        }
    }

    /// Get the Linear API relation type string.
    /// Note: For parent/sub relations, we use the issue update mutation with parentId.
    /// For inverse relations (blocked_by), we swap the source and target.
    pub fn to_api_type(&self) -> Option<&'static str> {
        match self {
            RelationType::Blocks => Some("blocks"),
            RelationType::BlockedBy => Some("blocks"), // Swapped direction
            RelationType::Related => Some("related"),
            RelationType::Duplicate => Some("duplicate"),
            RelationType::Parent | RelationType::Sub => None, // Use parentId mutation
        }
    }

    /// Check if this relation type requires swapping source and target.
    pub fn requires_swap(&self) -> bool {
        matches!(self, RelationType::BlockedBy)
    }
}

/// List all relations for an issue.
///
/// Fetches parent, children, blocks, blocked by, and related relations.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
pub fn list_relations(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    // First, resolve the issue ID if given an identifier
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({
        "id": issue_id
    });

    let response: IssueRelationsResponse =
        client.query(queries::ISSUE_RELATIONS_QUERY, variables)?;

    // Normalize all relations into a unified list for display
    let mut relations: Vec<NormalizedRelation> = Vec::new();

    // Add parent relation
    if let Some(parent) = response.issue.parent {
        relations.push(NormalizedRelation {
            id: format!("parent:{}", parent.id),
            relation_type: "parent".to_string(),
            related_issue: parent,
        });
    }

    // Add children as relations
    for child in response.issue.children.nodes {
        relations.push(NormalizedRelation {
            id: format!("child:{}", child.id),
            relation_type: "child".to_string(),
            related_issue: child,
        });
    }

    // Add direct relations (blocks, related, duplicate)
    for rel in response.issue.relations.nodes {
        if let Some(related_issue) = rel.related_issue {
            relations.push(NormalizedRelation {
                id: rel.id,
                relation_type: rel.type_.clone(),
                related_issue,
            });
        }
    }

    // Add inverse relations (blocked_by from the perspective of this issue)
    for rel in response.issue.inverse_relations.nodes {
        if let Some(source_issue) = rel.issue {
            // Inverse "blocks" means this issue is blocked by the source
            let display_type = match rel.type_.as_str() {
                "blocks" => "blocked_by".to_string(),
                other => format!("{}_inverse", other),
            };
            relations.push(NormalizedRelation {
                id: rel.id,
                relation_type: display_type,
                related_issue: source_issue,
            });
        }
    }

    output(&relations, format);
    Ok(())
}

/// Add a relation between two issues.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `source_id_or_identifier` - The source issue's UUID or identifier
/// * `target_id_or_identifier` - The target issue's UUID or identifier
/// * `relation_type` - The type of relation to create
/// * `format` - The output format (Human or Json)
pub fn add_relation(
    client: &GraphQLClient,
    source_id_or_identifier: &str,
    target_id_or_identifier: &str,
    relation_type: RelationType,
    format: OutputFormat,
) -> Result<()> {
    // Resolve both issue IDs
    let source_id = resolve_issue_id(client, source_id_or_identifier)?;
    let target_id = resolve_issue_id(client, target_id_or_identifier)?;

    match relation_type {
        RelationType::Parent => {
            // Set target as parent of source (source becomes sub-issue)
            set_parent(client, &source_id, Some(&target_id), format)
        }
        RelationType::Sub => {
            // Set source as parent of target (target becomes sub-issue)
            set_parent(client, &target_id, Some(&source_id), format)
        }
        RelationType::Blocks
        | RelationType::BlockedBy
        | RelationType::Related
        | RelationType::Duplicate => {
            // Use issueRelationCreate for these types
            let api_type = relation_type
                .to_api_type()
                .expect("Non-parent relations should have API type");

            // Swap source and target for inverse relations
            let (issue_id, related_issue_id) = if relation_type.requires_swap() {
                (&target_id, &source_id)
            } else {
                (&source_id, &target_id)
            };

            let variables = serde_json::json!({
                "input": {
                    "issueId": issue_id,
                    "relatedIssueId": related_issue_id,
                    "type": api_type
                }
            });

            let response: IssueRelationCreateResponse =
                client.query(queries::ISSUE_RELATION_CREATE_MUTATION, variables)?;

            if !response.issue_relation_create.success {
                return Err(LinError::api("Failed to create relation"));
            }

            match response.issue_relation_create.issue_relation {
                Some(relation) => {
                    output(&relation, format);
                    Ok(())
                }
                None => Err(LinError::api(
                    "Relation creation succeeded but no relation returned",
                )),
            }
        }
    }
}

/// Remove a relation by its ID.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `relation_id` - The relation's unique identifier
/// * `format` - The output format (Human or Json)
pub fn remove_relation(
    client: &GraphQLClient,
    relation_id: &str,
    format: OutputFormat,
) -> Result<()> {
    // Check if this is a parent/child relation (prefixed IDs)
    if relation_id.starts_with("parent:") || relation_id.starts_with("child:") {
        let issue_id = relation_id
            .split(':')
            .nth(1)
            .ok_or_else(|| LinError::api("Invalid relation ID format"))?;

        if relation_id.starts_with("parent:") {
            // To remove parent relation, we clear the parent from the current issue
            // But we don't have the child issue ID here, so we need a different approach
            return Err(LinError::api(
                "To remove a parent relation, use 'lin issue update <child-issue> --parent none'",
            ));
        } else {
            // To remove a child, clear the child's parent
            set_parent(client, issue_id, None, format)?;
            return Ok(());
        }
    }

    // Regular relation deletion
    let variables = serde_json::json!({
        "id": relation_id
    });

    let response: IssueRelationDeleteResponse =
        client.query(queries::ISSUE_RELATION_DELETE_MUTATION, variables)?;

    if !response.issue_relation_delete.success {
        return Err(LinError::api("Failed to delete relation"));
    }

    // Output a simple success message
    #[derive(serde::Serialize)]
    struct DeleteSuccess {
        message: String,
        relation_id: String,
    }

    let success = DeleteSuccess {
        message: "Relation deleted successfully".to_string(),
        relation_id: relation_id.to_string(),
    };

    // For JSON output, serialize the success struct
    // For human output, print a simple message
    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "success": true,
                "data": success
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json).expect("Failed to serialize")
            );
        }
        OutputFormat::Human => {
            println!("Relation {} deleted successfully.", relation_id);
        }
    }

    Ok(())
}

/// Set the parent of an issue.
fn set_parent(
    client: &GraphQLClient,
    child_id: &str,
    parent_id: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let variables = serde_json::json!({
        "id": child_id,
        "input": {
            "parentId": parent_id
        }
    });

    let response: IssueSetParentResponse =
        client.query(queries::ISSUE_SET_PARENT_MUTATION, variables)?;

    if !response.issue_update.success {
        return Err(LinError::api("Failed to set parent"));
    }

    match response.issue_update.issue {
        Some(issue) => {
            // Create a normalized relation for output
            if let Some(parent) = issue.parent {
                let relation = NormalizedRelation {
                    id: format!("parent:{}", parent.id),
                    relation_type: "parent".to_string(),
                    related_issue: parent,
                };
                output(&relation, format);
            } else {
                // Parent was removed
                match format {
                    OutputFormat::Json => {
                        let json = serde_json::json!({
                            "success": true,
                            "data": {
                                "message": "Parent removed",
                                "issue": {
                                    "id": issue.id,
                                    "identifier": issue.identifier
                                }
                            }
                        });
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json).expect("Failed to serialize")
                        );
                    }
                    OutputFormat::Human => {
                        println!(
                            "Parent removed from issue {} ({}).",
                            issue.identifier, issue.id
                        );
                    }
                }
            }
            Ok(())
        }
        None => Err(LinError::api(
            "Parent update succeeded but no issue returned",
        )),
    }
}

/// Resolve an issue ID from either a UUID or an identifier like "ENG-123".
fn resolve_issue_id(client: &GraphQLClient, id_or_identifier: &str) -> Result<String> {
    if is_uuid(id_or_identifier) {
        Ok(id_or_identifier.to_string())
    } else {
        // Parse the identifier and look up the issue to get its UUID
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        let lookup_variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let lookup_response: IssuesResponse =
            client.query(queries::ISSUE_BY_IDENTIFIER_QUERY, lookup_variables)?;

        if lookup_response.issues.nodes.is_empty() {
            return Err(LinError::api(format!(
                "Issue '{}' not found",
                id_or_identifier
            )));
        }

        Ok(lookup_response.issues.nodes[0].id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

    #[test]
    fn test_relation_type_parse() {
        assert_eq!(RelationType::parse("parent"), Some(RelationType::Parent));
        assert_eq!(RelationType::parse("sub"), Some(RelationType::Sub));
        assert_eq!(RelationType::parse("child"), Some(RelationType::Sub));
        assert_eq!(RelationType::parse("blocks"), Some(RelationType::Blocks));
        assert_eq!(
            RelationType::parse("blocked_by"),
            Some(RelationType::BlockedBy)
        );
        assert_eq!(
            RelationType::parse("blocked-by"),
            Some(RelationType::BlockedBy)
        );
        assert_eq!(RelationType::parse("related"), Some(RelationType::Related));
        assert_eq!(
            RelationType::parse("duplicate"),
            Some(RelationType::Duplicate)
        );
        assert_eq!(RelationType::parse("dup"), Some(RelationType::Duplicate));
        assert_eq!(RelationType::parse("invalid"), None);
    }

    #[test]
    fn test_relation_type_to_api_type() {
        assert_eq!(RelationType::Blocks.to_api_type(), Some("blocks"));
        assert_eq!(RelationType::BlockedBy.to_api_type(), Some("blocks"));
        assert_eq!(RelationType::Related.to_api_type(), Some("related"));
        assert_eq!(RelationType::Duplicate.to_api_type(), Some("duplicate"));
        assert_eq!(RelationType::Parent.to_api_type(), None);
        assert_eq!(RelationType::Sub.to_api_type(), None);
    }

    #[test]
    fn test_relation_type_requires_swap() {
        assert!(!RelationType::Blocks.requires_swap());
        assert!(RelationType::BlockedBy.requires_swap());
        assert!(!RelationType::Related.requires_swap());
        assert!(!RelationType::Parent.requires_swap());
    }

    #[test]
    fn test_list_relations_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issue": {
                            "id": "issue-123",
                            "identifier": "ENG-123",
                            "relations": {
                                "nodes": [
                                    {
                                        "id": "rel-1",
                                        "type": "blocks",
                                        "relatedIssue": {
                                            "id": "issue-456",
                                            "identifier": "ENG-456",
                                            "title": "Blocked issue"
                                        }
                                    }
                                ]
                            },
                            "inverseRelations": {
                                "nodes": []
                            },
                            "parent": null,
                            "children": {
                                "nodes": []
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = list_relations(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_relations_with_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
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
                                    "id": "issue-uuid-123",
                                    "identifier": "ENG-123",
                                    "title": "Test Issue",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Second mock: get relations
        let relations_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issue": {
                            "id": "issue-uuid-123",
                            "identifier": "ENG-123",
                            "relations": {
                                "nodes": []
                            },
                            "inverseRelations": {
                                "nodes": []
                            },
                            "parent": {
                                "id": "parent-123",
                                "identifier": "ENG-100",
                                "title": "Parent Issue"
                            },
                            "children": {
                                "nodes": [
                                    {
                                        "id": "child-1",
                                        "identifier": "ENG-124",
                                        "title": "Child Issue"
                                    }
                                ]
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = list_relations(&client, "ENG-123", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        relations_mock.assert();
    }

    #[test]
    fn test_add_relation_blocks() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueRelationCreate": {
                            "success": true,
                            "issueRelation": {
                                "id": "rel-new",
                                "type": "blocks",
                                "issue": {
                                    "id": "issue-1",
                                    "identifier": "ENG-1",
                                    "title": "Source"
                                },
                                "relatedIssue": {
                                    "id": "issue-2",
                                    "identifier": "ENG-2",
                                    "title": "Target"
                                }
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = add_relation(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001",
            RelationType::Blocks,
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_add_relation_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueRelationCreate": {
                            "success": false,
                            "issueRelation": null
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = add_relation(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            "550e8400-e29b-41d4-a716-446655440001",
            RelationType::Related,
            OutputFormat::Human,
        );

        assert!(result.is_err());
        mock.assert();
    }

    #[test]
    fn test_remove_relation_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueRelationDelete": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = remove_relation(&client, "rel-123", OutputFormat::Human);

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_remove_relation_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueRelationDelete": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = remove_relation(&client, "rel-123", OutputFormat::Human);

        assert!(result.is_err());
        mock.assert();
    }

    #[test]
    fn test_add_parent_relation() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueUpdate": {
                            "success": true,
                            "issue": {
                                "id": "child-id",
                                "identifier": "ENG-123",
                                "title": "Child Issue",
                                "parent": {
                                    "id": "parent-id",
                                    "identifier": "ENG-100",
                                    "title": "Parent Issue"
                                }
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = add_relation(
            &client,
            "550e8400-e29b-41d4-a716-446655440000", // child
            "550e8400-e29b-41d4-a716-446655440001", // parent
            RelationType::Parent,
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }
}
