use super::connection::{DbPool, get_connection_with_retry};
use crate::models::package::*;
use crate::schema::package_versions;
use diesel::prelude::*;

/// Package version-related database operations
pub struct VersionOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> VersionOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Creates a new package version or returns existing one if it already exists
    pub fn create_or_get_package_version(
        &self,
        package_id: i32,
        version: &str,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to get existing version first
        if let Some(existing_version) = package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
            .optional()?
        {
            return Ok(existing_version);
        }

        // Create new version
        let new_version = NewPackageVersion::new(package_id, version.to_string());

        diesel::insert_into(package_versions::table)
            .values(&new_version)
            .execute(&mut conn)?;

        package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
    }

    /// Creates a new package version with metadata or updates existing one
    pub fn create_or_get_package_version_with_metadata(
        &self,
        package_id: i32,
        version: &str,
        package_json: &serde_json::Value,
    ) -> Result<PackageVersion, diesel::result::Error> {
        self.create_or_get_package_version_with_metadata_and_update(
            package_id,
            version,
            package_json,
            false,
        )
    }

    /// Creates a new package version with metadata or updates existing one, with option to force update
    pub fn create_or_get_package_version_with_metadata_and_update(
        &self,
        package_id: i32,
        version: &str,
        package_json: &serde_json::Value,
        force_update: bool,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut existing_version_id = None;

        // Try to get existing version first
        if let Some(existing_version) = package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
            .optional()?
        {
            if force_update {
                // Store the existing version ID for updating
                existing_version_id = Some(existing_version.id);
            } else {
                // If version exists but has no metadata or missing README, update it with metadata
                if existing_version.description.is_none()
                    && existing_version.scripts.is_none()
                    && existing_version.dependencies.is_none()
                    && existing_version.dev_dependencies.is_none()
                {
                    // Store the existing version ID for updating
                    existing_version_id = Some(existing_version.id);
                } else if existing_version.readme.is_none()
                    || existing_version
                        .readme
                        .as_ref()
                        .map_or(true, |r| r.is_empty())
                {
                    // If README is missing or empty, update the record to include README
                    existing_version_id = Some(existing_version.id);
                } else {
                    return Ok(existing_version);
                }
            }
        }

        // Extract metadata from package.json
        let description = package_json["description"].as_str().map(|s| s.to_string());
        let main_file = package_json["main"].as_str().map(|s| s.to_string());

        // Serialize complex fields to JSON strings
        let scripts = package_json["scripts"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let dependencies = package_json["dependencies"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let dev_dependencies = package_json["devDependencies"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let peer_dependencies = package_json["peerDependencies"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let engines = package_json["engines"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let shasum = package_json
            .get("dist")
            .and_then(|dist| dist.get("shasum"))
            .and_then(|shasum| shasum.as_str())
            .map(|s| s.to_string());

        // Extract README content if available
        let readme = package_json
            .get("readme")
            .and_then(|readme| readme.as_str())
            .map(|s| s.to_string());

        // Extract publication time if available
        let created_at = package_json
            .get("_published_time")
            .and_then(|time| time.as_str())
            .and_then(|time_str| {
                chrono::DateTime::parse_from_rfc3339(time_str)
                    .map(|dt| dt.naive_utc())
                    .ok()
            });

        // Create new version with metadata
        let metadata = PackageVersionMetadata {
            description,
            main_file,
            scripts,
            dependencies,
            dev_dependencies,
            peer_dependencies,
            engines,
            shasum,
            readme,
            created_at,
        };
        let new_version =
            NewPackageVersion::with_metadata(package_id, version.to_string(), metadata);

        // Check if we need to update existing version or insert new one
        if let Some(_version_id) = existing_version_id {
            // Update existing version
            diesel::update(
                package_versions::table
                    .filter(package_versions::package_id.eq(package_id))
                    .filter(package_versions::version.eq(version)),
            )
            .set((
                package_versions::description.eq(&new_version.description),
                package_versions::main_file.eq(&new_version.main_file),
                package_versions::scripts.eq(&new_version.scripts),
                package_versions::dependencies.eq(&new_version.dependencies),
                package_versions::dev_dependencies.eq(&new_version.dev_dependencies),
                package_versions::peer_dependencies.eq(&new_version.peer_dependencies),
                package_versions::engines.eq(&new_version.engines),
                package_versions::shasum.eq(&new_version.shasum),
                package_versions::readme.eq(&new_version.readme),
                package_versions::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
        } else {
            // Insert new version
            diesel::insert_into(package_versions::table)
                .values(&new_version)
                .execute(&mut conn)?;
        }

        package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
    }

    /// Gets all versions for a package
    pub fn get_package_versions(
        &self,
        package_id: i32,
    ) -> Result<Vec<PackageVersion>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .order(package_versions::created_at.desc())
            .load::<PackageVersion>(&mut conn)
    }
}
