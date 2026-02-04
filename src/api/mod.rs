//! API module for interacting with Linear's GraphQL API.
//!
//! This module provides:
//! - [`client::GraphQLClient`] - HTTP client for making GraphQL requests
//! - [`queries`] - GraphQL query and mutation strings

pub mod client;
pub mod queries;

pub use client::GraphQLClient;
