//! Issue management commands.
//!
//! Commands for listing and viewing issue information from Linear.

mod create;
mod delete;
mod read;
mod update;

// Re-export all public items from submodules
pub use create::create_issue;
pub use delete::{archive_issue, delete_issue, unarchive_issue};
pub use read::{get_issue, get_issue_with_comments, list_issues};
pub use update::update_issue;

use crate::api::queries::issue::ISSUE_BY_IDENTIFIER_QUERY;
use crate::api::GraphQLClient;
use crate::error::LinError;
use crate::models::IssuesResponse;
use crate::output::HumanDisplay;
use crate::Result;
use serde::Serialize;

/// Sort field for issue list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IssueSortField {
    /// Sort by priority (default: ascending - urgent first).
    Priority,
    /// Sort by creation date (default: descending - newest first).
    #[default]
    Created,
    /// Sort by last update date (default: descending - most recent first).
    Updated,
    /// Sort by title (default: ascending - alphabetical).
    Title,
}

impl IssueSortField {
    /// Parse a sort field from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "priority" => Some(Self::Priority),
            "created" => Some(Self::Created),
            "updated" => Some(Self::Updated),
            "title" => Some(Self::Title),
            _ => None,
        }
    }

    /// Get the GraphQL field name for this sort field.
    pub fn to_graphql_field(&self) -> &'static str {
        match self {
            Self::Priority => "priority",
            Self::Created => "createdAt",
            Self::Updated => "updatedAt",
            Self::Title => "title",
        }
    }

    /// Get the default sort order for this field.
    pub fn default_order(&self) -> SortOrder {
        match self {
            Self::Priority => SortOrder::Asc,
            Self::Created => SortOrder::Desc,
            Self::Updated => SortOrder::Desc,
            Self::Title => SortOrder::Asc,
        }
    }
}

/// Sort order direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

impl SortOrder {
    /// Parse a sort order from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "asc" => Some(Self::Asc),
            "desc" => Some(Self::Desc),
            _ => None,
        }
    }
}

/// Priority level for filtering issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityFilter {
    /// No priority (0).
    None,
    /// Urgent priority (1).
    Urgent,
    /// High priority (2).
    High,
    /// Normal/Medium priority (3).
    Normal,
    /// Low priority (4).
    Low,
}

impl PriorityFilter {
    /// Parse a priority filter from a string.
    ///
    /// Accepts both numeric values (0-4) and named values (none, urgent, high, normal, low).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "0" | "none" => Some(Self::None),
            "1" | "urgent" => Some(Self::Urgent),
            "2" | "high" => Some(Self::High),
            "3" | "normal" | "medium" => Some(Self::Normal),
            "4" | "low" => Some(Self::Low),
            _ => None,
        }
    }

    /// Get the numeric value for this priority.
    pub fn to_value(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::Urgent => 1,
            Self::High => 2,
            Self::Normal => 3,
            Self::Low => 4,
        }
    }
}

/// Options for listing issues.
#[derive(Debug, Clone, Default)]
pub struct IssueListOptions {
    /// Filter by team key (e.g., "ENG").
    pub team: Option<String>,
    /// Filter by assignee ID or "me".
    pub assignee: Option<String>,
    /// Filter by state name.
    pub state: Option<String>,
    /// Filter by project ID.
    pub project: Option<String>,
    /// Filter by cycle ID.
    pub cycle: Option<String>,
    /// Filter by label ID.
    pub label: Option<String>,
    /// Filter by priority level.
    pub priority: Option<PriorityFilter>,
    /// Maximum number of issues to return (default 50).
    pub limit: Option<i32>,
    /// Filter issues created after this date (YYYY-MM-DD format).
    pub created_after: Option<String>,
    /// Filter issues created before this date (YYYY-MM-DD format).
    pub created_before: Option<String>,
    /// Filter issues updated after this date (YYYY-MM-DD format).
    pub updated_after: Option<String>,
    /// Filter issues updated before this date (YYYY-MM-DD format).
    pub updated_before: Option<String>,
    /// Sort field (priority, created, updated, title).
    pub sort_by: Option<IssueSortField>,
    /// Sort direction (asc, desc). Uses field-specific default if not specified.
    pub sort_order: Option<SortOrder>,
}

/// Options for creating a new issue.
#[derive(Debug, Clone)]
pub struct IssueCreateOptions {
    /// Issue title (required).
    pub title: String,
    /// Team ID (required).
    pub team_id: String,
    /// Issue description.
    pub description: Option<String>,
    /// Assignee user ID.
    pub assignee_id: Option<String>,
    /// Initial workflow state ID.
    pub state_id: Option<String>,
    /// Priority level (0=none, 1=urgent, 2=high, 3=normal, 4=low).
    pub priority: Option<i32>,
    /// Label IDs to add to the issue.
    pub label_ids: Option<Vec<String>>,
}

/// Options for updating an existing issue.
#[derive(Debug, Clone, Default)]
pub struct IssueUpdateOptions {
    /// New title.
    pub title: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New assignee user ID.
    pub assignee_id: Option<String>,
    /// New workflow state ID.
    pub state_id: Option<String>,
    /// New priority level (0=none, 1=urgent, 2=high, 3=normal, 4=low).
    pub priority: Option<i32>,
    /// Label IDs to set on the issue (replaces existing labels).
    pub label_ids: Option<Vec<String>>,
}

/// Check if a string looks like a UUID.
///
/// UUIDs are typically 36 characters with hyphens (8-4-4-4-12 format)
/// or 32 hex characters without hyphens.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string looks like a UUID, `false` otherwise.
///
/// # Example
///
/// ```
/// use lin::commands::issue::is_uuid;
///
/// assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
/// assert!(is_uuid("550e8400e29b41d4a716446655440000"));
/// assert!(!is_uuid("ENG-123"));
/// assert!(!is_uuid("ABC"));
/// ```
pub fn is_uuid(s: &str) -> bool {
    // Standard UUID format: 8-4-4-4-12 (36 chars with hyphens)
    if s.len() == 36 {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 5 {
            return parts[0].len() == 8
                && parts[1].len() == 4
                && parts[2].len() == 4
                && parts[3].len() == 4
                && parts[4].len() == 12
                && s.chars()
                    .filter(|c| *c != '-')
                    .all(|c| c.is_ascii_hexdigit());
        }
    }

    // UUID without hyphens: 32 hex characters
    if s.len() == 32 {
        return s.chars().all(|c| c.is_ascii_hexdigit());
    }

    false
}

/// Parse an issue identifier in the format "TEAM-NUMBER" (e.g., "ENG-123").
///
/// # Arguments
///
/// * `s` - The identifier string to parse
///
/// # Returns
///
/// A tuple of (team_key, issue_number) on success.
///
/// # Errors
///
/// Returns `LinError::Parse` if the string is not a valid identifier format.
///
/// # Example
///
/// ```
/// use lin::commands::issue::parse_identifier;
///
/// let (team, num) = parse_identifier("ENG-123").unwrap();
/// assert_eq!(team, "ENG");
/// assert_eq!(num, 123);
/// ```
pub fn parse_identifier(s: &str) -> Result<(String, i32)> {
    // Find the last hyphen (team keys might have hyphens in the future, though typically don't)
    let parts: Vec<&str> = s.split('-').collect();

    if parts.len() < 2 {
        return Err(LinError::parse(format!(
            "Invalid identifier format '{}': expected TEAM-NUMBER (e.g., ENG-123)",
            s
        )));
    }

    // Team key is everything before the last hyphen
    let team_parts = &parts[..parts.len() - 1];
    let team_key = team_parts.join("-");
    let number_str = parts[parts.len() - 1];

    // Validate team key: should be uppercase letters (and possibly hyphens between letters)
    // Don't allow empty team key, leading/trailing hyphens, or consecutive hyphens
    if team_key.is_empty()
        || team_key.starts_with('-')
        || team_key.ends_with('-')
        || team_key.contains("--")
        || !team_key.chars().all(|c| c.is_ascii_uppercase() || c == '-')
    {
        return Err(LinError::parse(format!(
            "Invalid team key '{}': expected uppercase letters (e.g., ENG, ABC-DEF)",
            team_key
        )));
    }

    // Parse the number
    let number: i32 = number_str.parse().map_err(|_| {
        LinError::parse(format!(
            "Invalid issue number '{}': expected an integer",
            number_str
        ))
    })?;

    if number <= 0 {
        return Err(LinError::parse(format!(
            "Invalid issue number '{}': must be positive",
            number
        )));
    }

    Ok((team_key, number))
}

/// Simple message response for delete/archive operations.
#[derive(Debug, Serialize)]
pub(crate) struct MessageResponse {
    pub(crate) message: String,
}

impl HumanDisplay for MessageResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

/// Resolve an issue identifier to its UUID.
///
/// If the input is already a UUID, returns it as-is.
/// Otherwise, parses the identifier and looks up the issue.
pub(crate) fn resolve_issue_id(client: &GraphQLClient, id_or_identifier: &str) -> Result<String> {
    if is_uuid(id_or_identifier) {
        Ok(id_or_identifier.to_string())
    } else {
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        let lookup_variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let lookup_response: IssuesResponse =
            client.query(ISSUE_BY_IDENTIFIER_QUERY, lookup_variables)?;

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

    // =============================================================================
    // is_uuid tests
    // =============================================================================

    #[test]
    fn test_is_uuid_standard_format() {
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(is_uuid("123e4567-e89b-12d3-a456-426614174000"));
        assert!(is_uuid("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"));
    }

    #[test]
    fn test_is_uuid_without_hyphens() {
        assert!(is_uuid("550e8400e29b41d4a716446655440000"));
        assert!(is_uuid("123e4567e89b12d3a456426614174000"));
        assert!(is_uuid("aaaaaaaabbbbccccddddeeeeeeeeeeee"));
    }

    #[test]
    fn test_is_uuid_lowercase_and_uppercase() {
        assert!(is_uuid("550E8400-E29B-41D4-A716-446655440000"));
        assert!(is_uuid("550e8400-E29B-41D4-a716-446655440000"));
    }

    #[test]
    fn test_is_uuid_not_uuid_identifier() {
        assert!(!is_uuid("ENG-123"));
        assert!(!is_uuid("ABC-1"));
        assert!(!is_uuid("TEAM-999"));
    }

    #[test]
    fn test_is_uuid_not_uuid_other() {
        assert!(!is_uuid("ABC"));
        assert!(!is_uuid("123"));
        assert!(!is_uuid(""));
        assert!(!is_uuid("not-a-uuid-string-at-all"));
        assert!(!is_uuid("550e8400-e29b-41d4-a716")); // Too short
        assert!(!is_uuid("550e8400-e29b-41d4-a716-446655440000-extra")); // Too long
        assert!(!is_uuid("550e8400-e29b-41d4-a716-44665544000g")); // Invalid char 'g'
    }

    #[test]
    fn test_is_uuid_wrong_hyphen_positions() {
        assert!(!is_uuid("5-50e8400-e29b-41d4-a716-46655440000"));
        assert!(!is_uuid("550e8400e29b-41d4-a716-446655440000"));
    }

    // =============================================================================
    // parse_identifier tests
    // =============================================================================

    #[test]
    fn test_parse_identifier_simple() {
        let (team, num) = parse_identifier("ENG-123").unwrap();
        assert_eq!(team, "ENG");
        assert_eq!(num, 123);
    }

    #[test]
    fn test_parse_identifier_single_letter_team() {
        let (team, num) = parse_identifier("A-1").unwrap();
        assert_eq!(team, "A");
        assert_eq!(num, 1);
    }

    #[test]
    fn test_parse_identifier_long_team() {
        let (team, num) = parse_identifier("ENGINEERING-999").unwrap();
        assert_eq!(team, "ENGINEERING");
        assert_eq!(num, 999);
    }

    #[test]
    fn test_parse_identifier_large_number() {
        let (team, num) = parse_identifier("ENG-123456").unwrap();
        assert_eq!(team, "ENG");
        assert_eq!(num, 123456);
    }

    #[test]
    fn test_parse_identifier_no_hyphen() {
        let result = parse_identifier("ENG123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid identifier format"));
    }

    #[test]
    fn test_parse_identifier_lowercase_team() {
        let result = parse_identifier("eng-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    #[test]
    fn test_parse_identifier_mixed_case_team() {
        let result = parse_identifier("Eng-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    #[test]
    fn test_parse_identifier_invalid_number() {
        let result = parse_identifier("ENG-abc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid issue number"));
    }

    #[test]
    fn test_parse_identifier_negative_number() {
        let result = parse_identifier("ENG--5");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_zero() {
        let result = parse_identifier("ENG-0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_parse_identifier_empty_string() {
        let result = parse_identifier("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_just_hyphen() {
        let result = parse_identifier("-");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_only_number() {
        let result = parse_identifier("-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    // =============================================================================
    // IssueSortField tests
    // =============================================================================

    #[test]
    fn test_issue_sort_field_parse() {
        assert_eq!(
            IssueSortField::parse("priority"),
            Some(IssueSortField::Priority)
        );
        assert_eq!(
            IssueSortField::parse("created"),
            Some(IssueSortField::Created)
        );
        assert_eq!(
            IssueSortField::parse("updated"),
            Some(IssueSortField::Updated)
        );
        assert_eq!(IssueSortField::parse("title"), Some(IssueSortField::Title));
        // Case insensitive
        assert_eq!(
            IssueSortField::parse("PRIORITY"),
            Some(IssueSortField::Priority)
        );
        assert_eq!(
            IssueSortField::parse("Created"),
            Some(IssueSortField::Created)
        );
        // Invalid values
        assert_eq!(IssueSortField::parse("invalid"), None);
        assert_eq!(IssueSortField::parse(""), None);
    }

    #[test]
    fn test_issue_sort_field_to_graphql_field() {
        assert_eq!(IssueSortField::Priority.to_graphql_field(), "priority");
        assert_eq!(IssueSortField::Created.to_graphql_field(), "createdAt");
        assert_eq!(IssueSortField::Updated.to_graphql_field(), "updatedAt");
        assert_eq!(IssueSortField::Title.to_graphql_field(), "title");
    }

    #[test]
    fn test_issue_sort_field_default_order() {
        assert_eq!(IssueSortField::Priority.default_order(), SortOrder::Asc);
        assert_eq!(IssueSortField::Created.default_order(), SortOrder::Desc);
        assert_eq!(IssueSortField::Updated.default_order(), SortOrder::Desc);
        assert_eq!(IssueSortField::Title.default_order(), SortOrder::Asc);
    }

    // =============================================================================
    // SortOrder tests
    // =============================================================================

    #[test]
    fn test_sort_order_parse() {
        assert_eq!(SortOrder::parse("asc"), Some(SortOrder::Asc));
        assert_eq!(SortOrder::parse("desc"), Some(SortOrder::Desc));
        // Case insensitive
        assert_eq!(SortOrder::parse("ASC"), Some(SortOrder::Asc));
        assert_eq!(SortOrder::parse("DESC"), Some(SortOrder::Desc));
        assert_eq!(SortOrder::parse("Asc"), Some(SortOrder::Asc));
        // Invalid values
        assert_eq!(SortOrder::parse("invalid"), None);
        assert_eq!(SortOrder::parse(""), None);
        assert_eq!(SortOrder::parse("ascending"), None);
    }

    // =============================================================================
    // PriorityFilter tests
    // =============================================================================

    #[test]
    fn test_priority_filter_parse_numeric() {
        assert_eq!(PriorityFilter::parse("0"), Some(PriorityFilter::None));
        assert_eq!(PriorityFilter::parse("1"), Some(PriorityFilter::Urgent));
        assert_eq!(PriorityFilter::parse("2"), Some(PriorityFilter::High));
        assert_eq!(PriorityFilter::parse("3"), Some(PriorityFilter::Normal));
        assert_eq!(PriorityFilter::parse("4"), Some(PriorityFilter::Low));
    }

    #[test]
    fn test_priority_filter_parse_named() {
        assert_eq!(PriorityFilter::parse("none"), Some(PriorityFilter::None));
        assert_eq!(
            PriorityFilter::parse("urgent"),
            Some(PriorityFilter::Urgent)
        );
        assert_eq!(PriorityFilter::parse("high"), Some(PriorityFilter::High));
        assert_eq!(
            PriorityFilter::parse("normal"),
            Some(PriorityFilter::Normal)
        );
        assert_eq!(
            PriorityFilter::parse("medium"),
            Some(PriorityFilter::Normal)
        );
        assert_eq!(PriorityFilter::parse("low"), Some(PriorityFilter::Low));
    }

    #[test]
    fn test_priority_filter_parse_case_insensitive() {
        assert_eq!(
            PriorityFilter::parse("URGENT"),
            Some(PriorityFilter::Urgent)
        );
        assert_eq!(PriorityFilter::parse("High"), Some(PriorityFilter::High));
        assert_eq!(
            PriorityFilter::parse("NORMAL"),
            Some(PriorityFilter::Normal)
        );
        assert_eq!(PriorityFilter::parse("Low"), Some(PriorityFilter::Low));
    }

    #[test]
    fn test_priority_filter_parse_invalid() {
        assert_eq!(PriorityFilter::parse("5"), None);
        assert_eq!(PriorityFilter::parse("-1"), None);
        assert_eq!(PriorityFilter::parse("critical"), None);
        assert_eq!(PriorityFilter::parse(""), None);
        assert_eq!(PriorityFilter::parse("very high"), None);
    }

    #[test]
    fn test_priority_filter_to_value() {
        assert_eq!(PriorityFilter::None.to_value(), 0);
        assert_eq!(PriorityFilter::Urgent.to_value(), 1);
        assert_eq!(PriorityFilter::High.to_value(), 2);
        assert_eq!(PriorityFilter::Normal.to_value(), 3);
        assert_eq!(PriorityFilter::Low.to_value(), 4);
    }
}
