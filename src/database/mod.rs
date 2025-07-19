//! Database module providing organized access to all database operations
//!
//! This module is organized into several sub-modules:
//! - `connection`: Database connection management and pool configuration
//! - `packages`: Package-related database operations
//! - `versions`: Package version-related database operations  
//! - `files`: Package file-related database operations
//! - `analytics`: Analytics and statistics operations
//! - `service`: Main DatabaseService that provides a unified interface

pub mod analytics;
pub mod connection;
pub mod files;
pub mod packages;
pub mod service;
pub mod versions;

// Re-export the main types and service for easy access
pub use connection::{DbConnection, DbPool, MIGRATIONS};
pub use service::DatabaseService;

// Re-export operation structs for advanced usage
pub use analytics::AnalyticsOperations;
pub use files::FileOperations;
pub use packages::PackageOperations;
pub use versions::VersionOperations;
