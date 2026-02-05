//! Cycle/sprint-related GraphQL queries.

/// Query to list cycles for a team.
///
/// Variables:
/// - `teamId` (String!): The team's unique identifier
/// - `first` (Int, optional): Number of cycles to fetch (default: 50)
///
/// Returns: `CyclesResponse`
pub const CYCLES_QUERY: &str = r#"
query Cycles($teamId: String!, $first: Int) {
    team(id: $teamId) {
        id
        cycles(first: $first, orderBy: createdAt) {
            nodes {
                id
                number
                name
                description
                startsAt
                endsAt
                completedAt
                progress
                completedScopeHistory
                scopeHistory
            }
        }
    }
}
"#;

/// Query to get a single cycle by ID with its issues.
///
/// Variables:
/// - `id` (String!): The cycle's unique identifier
///
/// Returns: `CycleResponse`
pub const CYCLE_QUERY: &str = r#"
query Cycle($id: String!) {
    cycle(id: $id) {
        id
        number
        name
        description
        startsAt
        endsAt
        completedAt
        progress
        completedScopeHistory
        scopeHistory
        issues {
            nodes {
                id
                identifier
                title
                description
                priority
                createdAt
                updatedAt
                state {
                    id
                    name
                    color
                    type
                }
                team {
                    id
                    key
                    name
                    description
                }
                assignee {
                    id
                    name
                    email
                    displayName
                    active
                }
            }
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycles_query_is_valid() {
        assert!(CYCLES_QUERY.contains("query Cycles"));
        assert!(CYCLES_QUERY.contains("$teamId: String!"));
        assert!(CYCLES_QUERY.contains("team(id: $teamId)"));
        assert!(CYCLES_QUERY.contains("cycles"));
        assert!(CYCLES_QUERY.contains("nodes"));
        assert!(CYCLES_QUERY.contains("number"));
        assert!(CYCLES_QUERY.contains("progress"));
        assert!(CYCLES_QUERY.contains("startsAt"));
        assert!(CYCLES_QUERY.contains("endsAt"));
    }

    #[test]
    fn test_cycle_query_is_valid() {
        assert!(CYCLE_QUERY.contains("query Cycle"));
        assert!(CYCLE_QUERY.contains("$id: String!"));
        assert!(CYCLE_QUERY.contains("cycle(id: $id)"));
        assert!(CYCLE_QUERY.contains("issues"));
        assert!(CYCLE_QUERY.contains("nodes"));
        assert!(CYCLE_QUERY.contains("identifier"));
    }
}
