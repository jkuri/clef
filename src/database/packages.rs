use super::connection::{DbPool, get_connection_with_retry};
use crate::models::package::*;
use crate::schema::packages;
use diesel::prelude::*;

/// Package-related database operations
pub struct PackageOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> PackageOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Creates a new package or returns existing one if it already exists
    pub fn create_or_get_package(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
    ) -> Result<Package, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to get existing package first
        if let Some(existing_package) = packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
            .optional()?
        {
            return Ok(existing_package);
        }

        // Create new package
        let new_package = NewPackage::new(name.to_string(), description, author_id);

        diesel::insert_into(packages::table)
            .values(&new_package)
            .execute(&mut conn)?;

        packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
    }

    /// Gets a package by name
    pub fn get_package_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Package>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
            .optional()
    }

    /// Updates package metadata (homepage, repository_url, license, keywords)
    pub fn update_package_metadata(
        &self,
        package_id: i32,
        homepage: Option<String>,
        repository_url: Option<String>,
        license: Option<String>,
        keywords: Option<String>,
    ) -> Result<Package, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let update_package = UpdatePackage {
            description: None, // Don't update description here
            homepage,
            repository_url,
            license,
            keywords,
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        diesel::update(packages::table.find(package_id))
            .set(&update_package)
            .execute(&mut conn)?;

        packages::table.find(package_id).first::<Package>(&mut conn)
    }

    /// Gets a package with all its versions and files
    pub fn get_package_with_versions(
        &self,
        name: &str,
    ) -> Result<Option<PackageWithVersions>, diesel::result::Error> {
        use crate::schema::{package_files, package_versions};

        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get the package
        let package = match packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
            .optional()?
        {
            Some(pkg) => pkg,
            None => return Ok(None),
        };

        // Get all versions with their files (use LEFT JOIN to include versions without files)
        let version_files: Vec<(PackageVersion, Option<PackageFile>)> = package_versions::table
            .left_join(package_files::table)
            .filter(package_versions::package_id.eq(package.id))
            .order(package_versions::created_at.desc())
            .load::<(PackageVersion, Option<PackageFile>)>(&mut conn)?;

        // Group files by version
        let mut versions_map: std::collections::HashMap<i32, (PackageVersion, Vec<PackageFile>)> =
            std::collections::HashMap::new();

        for (version, file_opt) in version_files {
            let entry = versions_map
                .entry(version.id)
                .or_insert((version.clone(), Vec::new()));

            // Only add file if it exists (LEFT JOIN can return None)
            if let Some(file) = file_opt {
                entry.1.push(file);
            }
        }

        let versions: Vec<PackageVersionWithFiles> = versions_map
            .into_values()
            .map(|(version, files)| PackageVersionWithFiles { version, files })
            .collect();

        Ok(Some(PackageWithVersions { package, versions }))
    }

    /// Gets all packages with their versions and files
    pub fn get_all_packages_with_versions(
        &self,
    ) -> Result<Vec<PackageWithVersions>, diesel::result::Error> {
        use crate::schema::{package_files, package_versions};

        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get all packages
        let all_packages = packages::table.load::<Package>(&mut conn)?;

        let mut result = Vec::new();

        for package in all_packages {
            // Get versions and files for this package
            let version_files: Vec<(PackageVersion, PackageFile)> = package_versions::table
                .inner_join(package_files::table)
                .filter(package_versions::package_id.eq(package.id))
                .order(package_versions::created_at.desc())
                .load::<(PackageVersion, PackageFile)>(&mut conn)?;

            // Group files by version
            let mut versions_map: std::collections::HashMap<
                i32,
                (PackageVersion, Vec<PackageFile>),
            > = std::collections::HashMap::new();

            for (version, file) in version_files {
                let entry = versions_map
                    .entry(version.id)
                    .or_insert((version.clone(), Vec::new()));
                entry.1.push(file);
            }

            let versions: Vec<PackageVersionWithFiles> = versions_map
                .into_values()
                .map(|(version, files)| PackageVersionWithFiles { version, files })
                .collect();

            result.push(PackageWithVersions { package, versions });
        }

        Ok(result)
    }

    /// Gets packages with pagination, optional search, and sorting
    pub fn get_packages_paginated(
        &self,
        limit: i64,
        offset: i64,
        search_query: Option<&str>,
        sort_column: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<(Vec<PackageWithVersions>, i64), diesel::result::Error> {
        use crate::schema::{package_files, package_versions};

        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get total count first
        let total_count: i64 = if let Some(search) = search_query {
            let search_pattern = format!("%{search}%");
            packages::table
                .filter(
                    packages::name
                        .like(&search_pattern)
                        .or(packages::description.like(&search_pattern)),
                )
                .count()
                .get_result(&mut conn)?
        } else {
            packages::table.count().get_result(&mut conn)?
        };

        // Apply sorting
        let sort_col = sort_column.unwrap_or("created_at");
        let sort_ord = sort_order.unwrap_or("desc");

        // Get paginated packages with search and sorting
        let paginated_packages = if let Some(search) = search_query {
            let search_pattern = format!("%{search}%");
            match (sort_col, sort_ord) {
                ("name", "asc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::name.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("name", "desc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::name.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("created_at", "asc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::created_at.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("created_at", "desc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::created_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("updated_at", "asc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::updated_at.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("updated_at", "desc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::updated_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("id", "asc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::id.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("id", "desc") => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::id.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                _ => packages::table
                    .filter(
                        packages::name
                            .like(&search_pattern)
                            .or(packages::description.like(&search_pattern)),
                    )
                    .order(packages::created_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
            }
        } else {
            match (sort_col, sort_ord) {
                ("name", "asc") => packages::table
                    .order(packages::name.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("name", "desc") => packages::table
                    .order(packages::name.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("created_at", "asc") => packages::table
                    .order(packages::created_at.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("created_at", "desc") => packages::table
                    .order(packages::created_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("updated_at", "asc") => packages::table
                    .order(packages::updated_at.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("updated_at", "desc") => packages::table
                    .order(packages::updated_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("id", "asc") => packages::table
                    .order(packages::id.asc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                ("id", "desc") => packages::table
                    .order(packages::id.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
                _ => packages::table
                    .order(packages::created_at.desc())
                    .limit(limit)
                    .offset(offset)
                    .load::<Package>(&mut conn)?,
            }
        };

        let mut result = Vec::new();

        // For each package, get its versions and files
        for package in paginated_packages {
            let version_files: Vec<(PackageVersion, Option<PackageFile>)> = package_versions::table
                .left_join(package_files::table)
                .filter(package_versions::package_id.eq(package.id))
                .order(package_versions::created_at.desc())
                .load::<(PackageVersion, Option<PackageFile>)>(&mut conn)?;

            // Group files by version
            let mut versions_map: std::collections::HashMap<
                i32,
                (PackageVersion, Vec<PackageFile>),
            > = std::collections::HashMap::new();

            for (version, file_opt) in version_files {
                let entry = versions_map
                    .entry(version.id)
                    .or_insert((version.clone(), Vec::new()));

                if let Some(file) = file_opt {
                    entry.1.push(file);
                }
            }

            let versions: Vec<PackageVersionWithFiles> = versions_map
                .into_values()
                .map(|(version, files)| PackageVersionWithFiles { version, files })
                .collect();

            result.push(PackageWithVersions { package, versions });
        }

        Ok((result, total_count))
    }

    /// Gets recent packages by creation date
    pub fn get_recent_packages(
        &self,
        limit: i64,
    ) -> Result<Vec<PackageWithVersions>, diesel::result::Error> {
        use crate::schema::{package_files, package_versions};

        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get recent packages by their creation date
        let recent_packages = packages::table
            .order(packages::created_at.desc())
            .limit(limit)
            .load::<Package>(&mut conn)?;

        let mut result = Vec::new();

        for package in recent_packages {
            // Get versions and files for this package
            let version_files: Vec<(PackageVersion, PackageFile)> = package_versions::table
                .inner_join(package_files::table)
                .filter(package_versions::package_id.eq(package.id))
                .order(package_versions::created_at.desc())
                .load::<(PackageVersion, PackageFile)>(&mut conn)?;

            // Group files by version
            let mut versions_map: std::collections::HashMap<
                i32,
                (PackageVersion, Vec<PackageFile>),
            > = std::collections::HashMap::new();

            for (version, file) in version_files {
                let entry = versions_map
                    .entry(version.id)
                    .or_insert((version.clone(), Vec::new()));
                entry.1.push(file);
            }

            let versions: Vec<PackageVersionWithFiles> = versions_map
                .into_values()
                .map(|(version, files)| PackageVersionWithFiles { version, files })
                .collect();

            result.push(PackageWithVersions { package, versions });
        }

        Ok(result)
    }
}
