//! Issue relation types for the Linear API.
//!
//! This module contains types for representing relationships between
//! Linear issues, including parent/child, blocking, and duplicate relations.

use serde::{Deserialize, Serialize};

/// A simplified issue reference for relations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedIssue {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Issue title.
    pub title: String,
}

/// An issue relation between two issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelation {
    /// Unique identifier for the relation.
    pub id: String,
    /// The type of relation (blocks, duplicate, related).
    #[serde(rename = "type")]
    pub type_: String,
    /// The related issue (target of the relation).
    pub related_issue: Option<RelatedIssue>,
}

/// An inverse issue relation (from the perspective of the related issue).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InverseIssueRelation {
    /// Unique identifier for the relation.
    pub id: String,
    /// The type of relation (blocks, duplicate, related).
    #[serde(rename = "type")]
    pub type_: String,
    /// The source issue of the relation.
    pub issue: Option<RelatedIssue>,
}

/// A full issue relation with both source and target.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullIssueRelation {
    /// Unique identifier for the relation.
    pub id: String,
    /// The type of relation (blocks, duplicate, related).
    #[serde(rename = "type")]
    pub type_: String,
    /// The source issue.
    pub issue: Option<RelatedIssue>,
    /// The related issue (target of the relation).
    pub related_issue: Option<RelatedIssue>,
}

/// A paginated list of issue relations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationConnection {
    /// List of issue relations.
    pub nodes: Vec<IssueRelation>,
}

/// A paginated list of inverse issue relations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InverseIssueRelationConnection {
    /// List of inverse issue relations.
    pub nodes: Vec<InverseIssueRelation>,
}

/// A paginated list of related issues (children).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedIssueConnection {
    /// List of related issues.
    pub nodes: Vec<RelatedIssue>,
}

/// Issue with all its relations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithRelations {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Outgoing relations from this issue.
    pub relations: IssueRelationConnection,
    /// Incoming relations to this issue.
    pub inverse_relations: InverseIssueRelationConnection,
    /// Parent issue (if this is a sub-issue).
    pub parent: Option<RelatedIssue>,
    /// Child issues (sub-issues).
    pub children: RelatedIssueConnection,
}

/// Response wrapper for issue relations query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationsResponse {
    /// The issue with its relations.
    pub issue: IssueWithRelations,
}

/// Response for issue relation creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created relation.
    pub issue_relation: Option<FullIssueRelation>,
}

/// Response wrapper for issue relation creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationCreateResponse {
    /// The mutation payload.
    pub issue_relation_create: IssueRelationCreatePayload,
}

/// Response for issue relation deletion mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationDeletePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue relation deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationDeleteResponse {
    /// The mutation payload.
    pub issue_relation_delete: IssueRelationDeletePayload,
}

/// Issue with parent after setting parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithParent {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Issue title.
    pub title: String,
    /// Parent issue (if set).
    pub parent: Option<RelatedIssue>,
}

/// Response for issue set parent mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueSetParentPayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The updated issue.
    pub issue: Option<IssueWithParent>,
}

/// Response wrapper for issue set parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueSetParentResponse {
    /// The mutation payload.
    pub issue_update: IssueSetParentPayload,
}

/// Normalized relation type for display.
/// This combines both direct and inverse relations into a unified view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedRelation {
    /// The relation ID.
    pub id: String,
    /// The normalized relation type for display (e.g., "parent", "child", "blocks", "blocked_by").
    pub relation_type: String,
    /// The related issue.
    pub related_issue: RelatedIssue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_relations_response_deserialization() {
        let json = r#"{
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
                    "nodes": [
                        {
                            "id": "rel-2",
                            "type": "blocks",
                            "issue": {
                                "id": "issue-789",
                                "identifier": "ENG-789",
                                "title": "Blocking issue"
                            }
                        }
                    ]
                },
                "parent": {
                    "id": "issue-parent",
                    "identifier": "ENG-100",
                    "title": "Parent issue"
                },
                "children": {
                    "nodes": [
                        {
                            "id": "issue-child",
                            "identifier": "ENG-124",
                            "title": "Child issue"
                        }
                    ]
                }
            }
        }"#;
        let response: IssueRelationsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue.id, "issue-123");
        assert_eq!(response.issue.identifier, "ENG-123");
        assert_eq!(response.issue.relations.nodes.len(), 1);
        assert_eq!(response.issue.relations.nodes[0].type_, "blocks");
        assert_eq!(response.issue.inverse_relations.nodes.len(), 1);
        assert!(response.issue.parent.is_some());
        assert_eq!(
            response.issue.parent.as_ref().unwrap().identifier,
            "ENG-100"
        );
        assert_eq!(response.issue.children.nodes.len(), 1);
    }

    #[test]
    fn test_issue_relation_create_response_deserialization() {
        let json = r#"{
            "issueRelationCreate": {
                "success": true,
                "issueRelation": {
                    "id": "rel-new",
                    "type": "blocks",
                    "issue": {
                        "id": "issue-1",
                        "identifier": "ENG-1",
                        "title": "Source issue"
                    },
                    "relatedIssue": {
                        "id": "issue-2",
                        "identifier": "ENG-2",
                        "title": "Target issue"
                    }
                }
            }
        }"#;
        let response: IssueRelationCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.issue_relation_create.success);
        assert!(response.issue_relation_create.issue_relation.is_some());
        let relation = response.issue_relation_create.issue_relation.unwrap();
        assert_eq!(relation.id, "rel-new");
        assert_eq!(relation.type_, "blocks");
    }

    #[test]
    fn test_issue_relation_delete_response_deserialization() {
        let json = r#"{
            "issueRelationDelete": {
                "success": true
            }
        }"#;
        let response: IssueRelationDeleteResponse = serde_json::from_str(json).unwrap();
        assert!(response.issue_relation_delete.success);
    }

    #[test]
    fn test_normalized_relation_serialization() {
        let relation = NormalizedRelation {
            id: "rel-123".to_string(),
            relation_type: "blocks".to_string(),
            related_issue: RelatedIssue {
                id: "issue-456".to_string(),
                identifier: "ENG-456".to_string(),
                title: "Blocked issue".to_string(),
            },
        };
        let json = serde_json::to_string(&relation).unwrap();
        assert!(json.contains("\"id\":\"rel-123\""));
        assert!(json.contains("\"relationType\":\"blocks\""));
        assert!(json.contains("\"identifier\":\"ENG-456\""));
    }
}
