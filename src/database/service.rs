use super::analytics::AnalyticsOperations;
use super::cache_stats::CacheStatsOperations;
use super::connection::{DbConnection, DbPool, create_pool, get_connection_with_retry};
use super::files::{CompletePackageParams, FileOperations, PackageFileParams};
use super::packages::PackageOperations;
use super::versions::VersionOperations;
use crate::models::package::*;

/// Main database service that provides a unified interface to all database operations
#[derive(Debug)]
pub struct DatabaseService {
    pub pool: DbPool,
}

impl DatabaseService {
    /// Creates a new DatabaseService with an initialized connection pool
    pub fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = create_pool(database_url)?;
        Ok(Self { pool })
    }

    /// Gets a connection from the pool with retry logic
    pub fn get_connection(&self) -> Result<DbConnection, diesel::r2d2::Error> {
        get_connection_with_retry(&self.pool)
    }

    // Package operations
    pub fn create_or_get_package(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
    ) -> Result<Package, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.create_or_get_package(name, description, author_id)
    }

    pub fn get_package_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Package>, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_package_by_name(name)
    }

    pub fn get_package_with_versions(
        &self,
        name: &str,
    ) -> Result<Option<PackageWithVersions>, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_package_with_versions(name)
    }

    pub fn get_all_packages_with_versions(
        &self,
    ) -> Result<Vec<PackageWithVersions>, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_all_packages_with_versions()
    }

    pub fn get_recent_packages(
        &self,
        limit: i64,
    ) -> Result<Vec<PackageWithVersions>, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_recent_packages(limit)
    }

    pub fn get_packages_paginated(
        &self,
        limit: i64,
        offset: i64,
        search_query: Option<&str>,
        sort_column: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<(Vec<PackageWithVersions>, i64), diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_packages_paginated(limit, offset, search_query, sort_column, sort_order)
    }

    // Package version operations
    pub fn create_or_get_package_version(
        &self,
        package_id: i32,
        version: &str,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let ops = VersionOperations::new(&self.pool);
        ops.create_or_get_package_version(package_id, version)
    }

    pub fn create_or_get_package_version_with_metadata(
        &self,
        package_id: i32,
        version: &str,
        package_json: &serde_json::Value,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let ops = VersionOperations::new(&self.pool);
        ops.create_or_get_package_version_with_metadata(package_id, version, package_json)
    }

    pub fn get_package_versions(
        &self,
        package_id: i32,
    ) -> Result<Vec<PackageVersion>, diesel::result::Error> {
        let ops = VersionOperations::new(&self.pool);
        ops.get_package_versions(package_id)
    }

    // Package file operations
    #[allow(clippy::too_many_arguments)]
    pub fn create_or_update_package_file(
        &self,
        package_version_id: i32,
        filename: &str,
        size_bytes: i64,
        upstream_url: &str,
        file_path: &str,
        etag: Option<String>,
        content_type: Option<String>,
    ) -> Result<PackageFile, diesel::result::Error> {
        let ops = FileOperations::new(&self.pool);
        let params = PackageFileParams {
            filename: filename.to_string(),
            size_bytes,
            upstream_url: upstream_url.to_string(),
            file_path: file_path.to_string(),
            etag,
            content_type,
        };
        ops.create_or_update_package_file(package_version_id, &params)
    }

    pub fn get_package_file(
        &self,
        package_name: &str,
        filename: &str,
    ) -> Result<Option<(Package, PackageVersion, PackageFile)>, diesel::result::Error> {
        let ops = FileOperations::new(&self.pool);
        ops.get_package_file(package_name, filename)
    }

    pub fn update_file_access_info(&self, file_id: i32) -> Result<(), diesel::result::Error> {
        let ops = FileOperations::new(&self.pool);
        ops.update_file_access_info(file_id)
    }

    pub fn create_complete_package_entry(
        &self,
        params: &CompletePackageParams,
    ) -> Result<(Package, PackageVersion, PackageFile), diesel::result::Error> {
        let ops = FileOperations::new(&self.pool);
        ops.create_complete_package_entry(params)
    }

    // Analytics operations
    pub fn get_popular_packages(
        &self,
        limit: i64,
    ) -> Result<Vec<PopularPackage>, diesel::result::Error> {
        let ops = AnalyticsOperations::new(&self.pool);
        ops.get_popular_packages(limit)
    }

    pub fn get_cache_stats(&self) -> Result<(usize, i64), diesel::result::Error> {
        let ops = AnalyticsOperations::new(&self.pool);
        ops.get_cache_stats()
    }

    // Cache stats operations
    pub fn get_persistent_cache_stats(
        &self,
    ) -> Result<Option<crate::models::cache::CacheStatsRecord>, diesel::result::Error> {
        let ops = CacheStatsOperations::new(&self.pool);
        ops.get_cache_stats()
    }

    pub fn update_persistent_cache_stats(
        &self,
        hit_count: u64,
        miss_count: u64,
    ) -> Result<crate::models::cache::CacheStatsRecord, diesel::result::Error> {
        let ops = CacheStatsOperations::new(&self.pool);
        ops.update_cache_stats(hit_count, miss_count)
    }

    pub fn increment_cache_hit_count(&self) -> Result<(), diesel::result::Error> {
        let ops = CacheStatsOperations::new(&self.pool);
        ops.increment_hit_count()
    }

    pub fn increment_cache_miss_count(&self) -> Result<(), diesel::result::Error> {
        let ops = CacheStatsOperations::new(&self.pool);
        ops.increment_miss_count()
    }
}
