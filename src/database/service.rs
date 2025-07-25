use super::analytics::AnalyticsOperations;
use super::cache_stats::CacheStatsOperations;
use super::connection::{DbConnection, DbPool, create_pool, get_connection_with_retry};
use super::files::{CompletePackageParams, FileOperations, PackageFileParams};
use super::metadata_cache::MetadataCacheOperations;
use super::organizations::OrganizationOperations;
use super::package_owners::PackageOwnerOperations;
use super::packages::PackageOperations;
use super::versions::VersionOperations;
use crate::models::metadata_cache::{MetadataCacheRecord, MetadataCacheStats};
use crate::models::organization::*;
use crate::models::package::*;
use crate::models::user::User;
use crate::schema::users;
use diesel::prelude::*;

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

    pub fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

        let mut conn = get_connection_with_retry(&self.pool)?;
        match conn.run_pending_migrations(MIGRATIONS) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Migration error: {e}").into()),
        }
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

    pub fn create_or_get_package_with_update(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
        update_description: bool,
    ) -> Result<Package, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.create_or_get_package_with_update(name, description, author_id, update_description)
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

    pub fn update_package_metadata(
        &self,
        package_id: i32,
        homepage: Option<String>,
        repository_url: Option<String>,
        license: Option<String>,
        keywords: Option<String>,
    ) -> Result<Package, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.update_package_metadata(package_id, homepage, repository_url, license, keywords)
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

    pub fn create_or_get_package_version_with_metadata_and_update(
        &self,
        package_id: i32,
        version: &str,
        package_json: &serde_json::Value,
        force_update: bool,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let ops = VersionOperations::new(&self.pool);
        ops.create_or_get_package_version_with_metadata_and_update(
            package_id,
            version,
            package_json,
            force_update,
        )
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

    // Metadata cache operations
    pub fn get_metadata_cache_entry(
        &self,
        package_name: &str,
    ) -> Result<Option<MetadataCacheRecord>, diesel::result::Error> {
        let ops = MetadataCacheOperations::new(&self.pool);
        ops.get_metadata_cache_entry(package_name)
    }

    pub fn upsert_metadata_cache_entry(
        &self,
        package_name: &str,
        size_bytes: i64,
        file_path: &str,
        etag: Option<&str>,
    ) -> Result<MetadataCacheRecord, diesel::result::Error> {
        let ops = MetadataCacheOperations::new(&self.pool);
        ops.upsert_metadata_cache_entry(package_name, size_bytes, file_path, etag)
    }

    pub fn update_metadata_access_info(
        &self,
        package_name: &str,
    ) -> Result<(), diesel::result::Error> {
        let ops = MetadataCacheOperations::new(&self.pool);
        ops.update_metadata_access_info(package_name)
    }

    pub fn get_metadata_cache_stats(&self) -> Result<MetadataCacheStats, diesel::result::Error> {
        let ops = MetadataCacheOperations::new(&self.pool);
        ops.get_metadata_cache_stats()
    }

    pub fn clear_metadata_cache(&self) -> Result<usize, diesel::result::Error> {
        let ops = MetadataCacheOperations::new(&self.pool);
        ops.clear_metadata_cache()
    }

    // Package ownership operations
    pub fn has_read_permission(
        &self,
        package_name: &str,
        user_id: Option<i32>,
    ) -> Result<bool, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.has_read_permission(package_name, user_id)
    }

    pub fn has_write_permission(
        &self,
        package_name: &str,
        user_id: i32,
    ) -> Result<bool, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.has_write_permission(package_name, user_id)
    }

    pub fn package_exists(&self, package_name: &str) -> Result<bool, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.package_exists(package_name)
    }

    pub fn package_published(&self, package_name: &str) -> Result<bool, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.package_published(package_name)
    }

    pub fn create_package_owner(
        &self,
        package_name: &str,
        user_id: i32,
        permission_level: &str,
    ) -> Result<PackageOwner, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.create_package_owner(package_name, user_id, permission_level)
    }

    pub fn get_package_owners(
        &self,
        package_name: &str,
    ) -> Result<Vec<PackageOwner>, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.get_package_owners(package_name)
    }

    pub fn add_package_owner(
        &self,
        package_name: &str,
        user_id: i32,
        permission_level: &str,
    ) -> Result<PackageOwner, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.add_package_owner(package_name, user_id, permission_level)
    }

    pub fn remove_package_owner(
        &self,
        package_name: &str,
        user_id: i32,
    ) -> Result<usize, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.remove_package_owner(package_name, user_id)
    }

    pub fn update_permission_level(
        &self,
        package_name: &str,
        user_id: i32,
        new_permission_level: &str,
    ) -> Result<PackageOwner, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.update_permission_level(package_name, user_id, new_permission_level)
    }

    pub fn can_publish_package(
        &self,
        package_name: &str,
        user_id: i32,
    ) -> Result<bool, diesel::result::Error> {
        let ops = PackageOwnerOperations::new(&self.pool);
        ops.can_publish_package(package_name, user_id)
    }

    // Organization operations
    pub fn create_organization(
        &self,
        name: &str,
        display_name: Option<String>,
        description: Option<String>,
        creator_user_id: i32,
    ) -> Result<Organization, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.create_organization(name, display_name, description, creator_user_id)
    }

    pub fn get_organization_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Organization>, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.get_organization_by_name(name)
    }

    pub fn get_organization_by_id(
        &self,
        id: i32,
    ) -> Result<Option<Organization>, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.get_organization_by_id(id)
    }

    pub fn update_organization(
        &self,
        id: i32,
        display_name: Option<String>,
        description: Option<String>,
    ) -> Result<Organization, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.update_organization(id, display_name, description)
    }

    pub fn delete_organization(&self, id: i32) -> Result<(), diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.delete_organization(id)
    }

    pub fn add_organization_member(
        &self,
        organization_id: i32,
        user_id: i32,
        role: &str,
    ) -> Result<OrganizationMember, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.add_member(organization_id, user_id, role)
    }

    pub fn update_organization_member_role(
        &self,
        organization_id: i32,
        user_id: i32,
        new_role: &str,
    ) -> Result<OrganizationMember, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.update_member_role(organization_id, user_id, new_role)
    }

    pub fn remove_organization_member(
        &self,
        organization_id: i32,
        user_id: i32,
    ) -> Result<(), diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.remove_member(organization_id, user_id)
    }

    pub fn get_organization_members(
        &self,
        organization_id: i32,
    ) -> Result<Vec<OrganizationMemberWithUser>, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.get_organization_members(organization_id)
    }

    pub fn check_organization_permission(
        &self,
        organization_id: i32,
        user_id: i32,
        required_role: OrganizationRole,
    ) -> Result<bool, diesel::result::Error> {
        let ops = OrganizationOperations::new(&self.pool);
        ops.check_user_permission(organization_id, user_id, required_role)
    }

    // Package-Organization operations
    pub fn create_or_get_package_with_organization(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
        organization_id: Option<i32>,
    ) -> Result<Package, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.create_or_get_package_with_organization(name, description, author_id, organization_id)
    }

    pub fn get_or_create_organization_for_package(
        &self,
        package_name: &str,
        creator_user_id: Option<i32>,
    ) -> Result<Option<i32>, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_or_create_organization_for_package(package_name, creator_user_id)
    }

    pub fn link_package_to_organization(
        &self,
        package_id: i32,
        organization_id: i32,
    ) -> Result<Package, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.link_package_to_organization(package_id, organization_id)
    }

    pub fn get_packages_by_organization(
        &self,
        organization_id: i32,
    ) -> Result<Vec<Package>, diesel::result::Error> {
        let ops = PackageOperations::new(&self.pool);
        ops.get_packages_by_organization(organization_id)
    }

    pub fn extract_organization_name(package_name: &str) -> Option<String> {
        PackageOperations::extract_organization_name(package_name)
    }

    // User operations
    pub fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(&self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        users::table
            .filter(users::username.eq(username))
            .filter(users::is_active.eq(true))
            .first::<User>(&mut conn)
            .optional()
    }
}
