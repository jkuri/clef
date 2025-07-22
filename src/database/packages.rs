use super::connection::{DbPool, get_connection_with_retry};
use crate::models::package::*;
use crate::schema::{organizations, packages};
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
        self.create_or_get_package_with_update(name, description, author_id, false)
    }

    /// Creates a new package or returns existing one, with option to update description
    pub fn create_or_get_package_with_update(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
        update_description: bool,
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
            // If we should update description and it's provided, update the existing package
            if update_description
                && description.is_some()
                && description != existing_package.description
            {
                let update_package = UpdatePackage {
                    description: description.clone(),
                    author_id: Some(author_id), // Update author_id when publishing
                    homepage: None,
                    repository_url: None,
                    license: None,
                    keywords: None,
                    updated_at: Some(chrono::Utc::now().naive_utc()),
                };

                diesel::update(packages::table.find(existing_package.id))
                    .set(&update_package)
                    .execute(&mut conn)?;

                // Return the updated package
                return packages::table
                    .find(existing_package.id)
                    .first::<Package>(&mut conn);
            }

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
            author_id: None,   // Don't update author_id here
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

    /// Creates a package with organization link for scoped packages
    pub fn create_or_get_package_with_organization(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
        organization_id: Option<i32>,
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
            // Update the package with new information
            let update_package = UpdatePackage {
                description: description.clone(),
                author_id: Some(author_id), // Update author_id when publishing
                homepage: None,
                repository_url: None,
                license: None,
                keywords: None,
                updated_at: Some(chrono::Utc::now().naive_utc()),
            };

            diesel::update(packages::table.find(existing_package.id))
                .set(&update_package)
                .execute(&mut conn)?;

            // If organization_id is provided and different, update it
            if let Some(org_id) = organization_id {
                if existing_package.organization_id != Some(org_id) {
                    diesel::update(packages::table.find(existing_package.id))
                        .set(packages::organization_id.eq(org_id))
                        .execute(&mut conn)?;
                }
            }

            return packages::table
                .find(existing_package.id)
                .first::<Package>(&mut conn);
        }

        // Create new package with organization
        let new_package = NewPackage::new_with_organization(
            name.to_string(),
            description,
            author_id,
            organization_id,
        );

        diesel::insert_into(packages::table)
            .values(&new_package)
            .execute(&mut conn)?;

        packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
    }

    /// Extracts organization name from scoped package name
    /// Returns None for non-scoped packages
    pub fn extract_organization_name(package_name: &str) -> Option<String> {
        if package_name.starts_with('@') {
            if let Some(slash_pos) = package_name.find('/') {
                // Extract the scope name without the @ symbol
                let scope = &package_name[1..slash_pos];
                return Some(scope.to_string());
            }
        }
        None
    }

    /// Gets or creates organization for a scoped package
    pub fn get_or_create_organization_for_package(
        &self,
        package_name: &str,
        creator_user_id: Option<i32>,
    ) -> Result<Option<i32>, diesel::result::Error> {
        if let Some(org_name) = Self::extract_organization_name(package_name) {
            let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(e.to_string()),
                )
            })?;

            // Try to find existing organization
            if let Some(org) = organizations::table
                .filter(organizations::name.eq(&org_name))
                .first::<crate::models::organization::Organization>(&mut conn)
                .optional()?
            {
                return Ok(Some(org.id));
            }

            // Create organization if it doesn't exist and we have a creator
            if let Some(user_id) = creator_user_id {
                use crate::models::organization::{
                    NewOrganization, NewOrganizationMember, OrganizationRole,
                };
                use crate::schema::organization_members;

                return conn.transaction(|conn| {
                    // Create the organization
                    let new_org = NewOrganization::new(org_name.clone(), None, None);

                    diesel::insert_into(organizations::table)
                        .values(&new_org)
                        .execute(conn)?;

                    let organization = organizations::table
                        .filter(organizations::name.eq(&org_name))
                        .first::<crate::models::organization::Organization>(conn)?;

                    // Add the creator as an owner
                    let new_member = NewOrganizationMember::new(
                        user_id,
                        organization.id,
                        OrganizationRole::Owner.to_string(),
                    );

                    diesel::insert_into(organization_members::table)
                        .values(&new_member)
                        .execute(conn)?;

                    Ok(Some(organization.id))
                });
            }
        }

        Ok(None)
    }

    /// Updates package to link with organization
    pub fn link_package_to_organization(
        &self,
        package_id: i32,
        organization_id: i32,
    ) -> Result<Package, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(packages::table.find(package_id))
            .set(packages::organization_id.eq(organization_id))
            .execute(&mut conn)?;

        packages::table.find(package_id).first::<Package>(&mut conn)
    }

    /// Gets all packages for an organization
    pub fn get_packages_by_organization(
        &self,
        organization_id: i32,
    ) -> Result<Vec<Package>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .filter(packages::organization_id.eq(organization_id))
            .load::<Package>(&mut conn)
    }
}
