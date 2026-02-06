//! Domain models for Linear API responses.
//!
//! This module contains all the data structures used to represent
//! Linear entities like users, teams, issues, and their relationships.

pub mod attachment;
pub mod comment;
pub mod common;
pub mod cycle;
pub mod issue;
pub mod label;
pub mod milestone;
pub mod project;
pub mod relation;
pub mod team;
pub mod user;
pub mod workflow;

// Re-export all types for convenience since they're used throughout the codebase.
pub use attachment::*;
pub use comment::*;
pub use cycle::*;
pub use issue::*;
pub use label::*;
pub use milestone::*;
pub use project::*;
pub use relation::*;
pub use team::*;
pub use user::*;
pub use workflow::*;
