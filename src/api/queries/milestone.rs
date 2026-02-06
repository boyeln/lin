//! Project milestone-related GraphQL queries.

/// Query to list milestones for a specific project.
///
/// Variables:
/// - `projectId` (String!): The project's unique identifier
/// - `first` (Int, optional): Number of milestones to fetch (default: 50)
///
/// Returns: `ProjectMilestonesResponse`
pub const PROJECT_MILESTONES_QUERY: &str = r#"
query ProjectMilestones($projectId: String!, $first: Int) {
    projectMilestones(filter: { project: { id: { eq: $projectId } } }, first: $first) {
        nodes {
            id
            name
            description
            targetDate
            sortOrder
            status
            progress
            createdAt
            updatedAt
        }
    }
}
"#;

/// Query to get a single milestone by ID.
///
/// Variables:
/// - `id` (String!): The milestone's unique identifier
///
/// Returns: `ProjectMilestoneResponse`
pub const PROJECT_MILESTONE_QUERY: &str = r#"
query ProjectMilestone($id: String!) {
    projectMilestone(id: $id) {
        id
        name
        description
        targetDate
        sortOrder
        status
        progress
        createdAt
        updatedAt
    }
}
"#;

/// Mutation to create a new project milestone.
///
/// Variables:
/// - `input` (ProjectMilestoneCreateInput!): The milestone creation input
///
/// Returns: `ProjectMilestonePayload`
pub const PROJECT_MILESTONE_CREATE_MUTATION: &str = r#"
mutation ProjectMilestoneCreate($input: ProjectMilestoneCreateInput!) {
    projectMilestoneCreate(input: $input) {
        success
        projectMilestone {
            id
            name
            description
            targetDate
            sortOrder
            status
            progress
            createdAt
            updatedAt
        }
    }
}
"#;

/// Mutation to update an existing project milestone.
///
/// Variables:
/// - `id` (String!): The milestone's unique identifier
/// - `input` (ProjectMilestoneUpdateInput!): The milestone update input
///
/// Returns: `ProjectMilestonePayload`
pub const PROJECT_MILESTONE_UPDATE_MUTATION: &str = r#"
mutation ProjectMilestoneUpdate($id: String!, $input: ProjectMilestoneUpdateInput!) {
    projectMilestoneUpdate(id: $id, input: $input) {
        success
        projectMilestone {
            id
            name
            description
            targetDate
            sortOrder
            status
            progress
            createdAt
            updatedAt
        }
    }
}
"#;

/// Mutation to delete a project milestone.
///
/// Variables:
/// - `id` (String!): The milestone's unique identifier
///
/// Returns: JSON with success field
pub const PROJECT_MILESTONE_DELETE_MUTATION: &str = r#"
mutation ProjectMilestoneDelete($id: String!) {
    projectMilestoneDelete(id: $id) {
        success
    }
}
"#;
