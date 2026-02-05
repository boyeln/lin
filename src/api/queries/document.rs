//! Document-related GraphQL queries and mutations.

/// Query to list all documents in the organization.
///
/// Variables:
/// - `first` (Int, optional): Number of documents to fetch (default: 50)
/// - `filter` (DocumentFilter, optional): Filter criteria for documents
///
/// Returns: `DocumentsResponse`
pub const DOCUMENTS_QUERY: &str = r#"
query Documents($first: Int, $filter: DocumentFilter) {
    documents(first: $first, filter: $filter) {
        nodes {
            id
            title
            icon
            color
            createdAt
            updatedAt
            creator {
                id
                name
                email
                displayName
                active
            }
            project {
                id
                name
            }
        }
    }
}
"#;

/// Query to get a single document by ID (with content).
///
/// Variables:
/// - `id` (String!): The document's unique identifier
///
/// Returns: `DocumentResponse`
pub const DOCUMENT_QUERY: &str = r#"
query Document($id: String!) {
    document(id: $id) {
        id
        title
        content
        icon
        color
        createdAt
        updatedAt
        creator {
            id
            name
            email
            displayName
            active
        }
        project {
            id
            name
        }
    }
}
"#;

/// Mutation to create a new document.
///
/// Variables:
/// - `input` (DocumentCreateInput!): Document creation input containing:
///   - `title` (String!): Document title
///   - `content` (String, optional): Document content (markdown)
///   - `projectId` (String, optional): Project ID to associate with
///
/// Returns: `DocumentCreateResponse`
pub const DOCUMENT_CREATE_MUTATION: &str = r#"
mutation DocumentCreate($input: DocumentCreateInput!) {
    documentCreate(input: $input) {
        success
        document {
            id
            title
            content
            icon
            color
            createdAt
            updatedAt
            creator {
                id
                name
                email
                displayName
                active
            }
            project {
                id
                name
            }
        }
    }
}
"#;

/// Mutation to delete a document.
///
/// Variables:
/// - `id` (String!): The document's unique identifier
///
/// Returns: `DocumentDeleteResponse`
pub const DOCUMENT_DELETE_MUTATION: &str = r#"
mutation DocumentDelete($id: String!) {
    documentDelete(id: $id) {
        success
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documents_query_is_valid() {
        assert!(DOCUMENTS_QUERY.contains("query Documents"));
        assert!(DOCUMENTS_QUERY.contains("$first: Int"));
        assert!(DOCUMENTS_QUERY.contains("$filter: DocumentFilter"));
        assert!(DOCUMENTS_QUERY.contains("documents"));
        assert!(DOCUMENTS_QUERY.contains("nodes"));
        assert!(DOCUMENTS_QUERY.contains("title"));
        assert!(DOCUMENTS_QUERY.contains("creator"));
    }

    #[test]
    fn test_document_query_is_valid() {
        assert!(DOCUMENT_QUERY.contains("query Document"));
        assert!(DOCUMENT_QUERY.contains("$id: String!"));
        assert!(DOCUMENT_QUERY.contains("document(id: $id)"));
        assert!(DOCUMENT_QUERY.contains("title"));
        assert!(DOCUMENT_QUERY.contains("content"));
        assert!(DOCUMENT_QUERY.contains("creator"));
    }

    #[test]
    fn test_document_create_mutation_is_valid() {
        assert!(DOCUMENT_CREATE_MUTATION.contains("mutation DocumentCreate"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("$input: DocumentCreateInput!"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("documentCreate"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("success"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("document"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("content"));
    }

    #[test]
    fn test_document_delete_mutation_is_valid() {
        assert!(DOCUMENT_DELETE_MUTATION.contains("mutation DocumentDelete"));
        assert!(DOCUMENT_DELETE_MUTATION.contains("$id: String!"));
        assert!(DOCUMENT_DELETE_MUTATION.contains("documentDelete"));
        assert!(DOCUMENT_DELETE_MUTATION.contains("success"));
    }
}
