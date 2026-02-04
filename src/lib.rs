//! lin - A command-line interface for Linear
//!
//! This crate provides a CLI tool for interacting with Linear's API,
//! allowing users to manage issues, teams, and organizations from the terminal.
//!
//! # Features
//!
//! - JSON output for all commands (scriptability)
//! - Nested command structure (e.g., `lin issue list`)
//! - Cross-platform support
//! - Multi-organization support with configuration file
//!
//! # Modules
//!
//! - [`error`] - Custom error types for the CLI
//! - [`output`] - JSON output utilities
//! - [`config`] - Configuration management
//! - [`auth`] - Authentication and token resolution
//! - [`commands`] - Command implementations

pub mod auth;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;

// Re-export commonly used types
pub use error::LinError;

/// Standard result type for lin operations.
pub type Result<T> = std::result::Result<T, LinError>;
