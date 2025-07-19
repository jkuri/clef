//! Database module providing organized access to all database operations
//! 
//! This module is organized into several sub-modules:
//! - `connection`: Database connection management and pool configuration
//! - `packages`: Package-related database operations
//! - `versions`: Package version-related database operations  
//! - `files`: Package file-related database operations
//! - `analytics`: Analytics and statistics operations
//! - `service`: Main DatabaseService that provides a unified interface

pub mod connection;
pub mod packages;
pub mod versions;
pub mod files;
pub mod analytics;
pub mod service;

// Re-export the main types and service for easy access
pub use connection::{DbPool, DbConnection, MIGRATIONS};
pub use service::DatabaseService;

// Re-export operation structs for advanced usage
pub use packages::PackageOperations;
pub use versions::VersionOperations;
pub use files::FileOperations;
pub use analytics::AnalyticsOperations;
