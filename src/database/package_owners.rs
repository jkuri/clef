use super::connection::{DbPool, get_connection_with_retry};
use crate::models::organization::OrganizationRole;
use crate::models::package::*;
use crate::schema::{organization_members, package_owners, packages};
use diesel::prelude::*;

/// Package ownership-related database operations
pub struct PackageOwnerOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> PackageOwnerOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Checks if a user has read permission for a package
    /// For scoped packages, checks organization membership
    /// For regular packages, all are public by default
    pub fn has_read_permission(
        &self,
        package_name: &str,
        user_id: Option<i32>,
    ) -> Result<bool, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Check if package exists locally
        let package = packages::table
            .filter(packages::name.eq(package_name))
            .first::<Package>(&mut conn)
            .optional()?;

        match package {
            Some(pkg) => {
                // Package exists locally
                // If it's published locally (has author_id), it's public regardless of organization
                if pkg.author_id.is_some() {
                    Ok(true) // Published packages are public
                } else if let Some(org_id) = pkg.organization_id {
                    // Cached organization package - check organization membership
                    if let Some(uid) = user_id {
                        // Check if user is a member of the organization
                        let is_member = organization_members::table
                            .filter(organization_members::organization_id.eq(org_id))
                            .filter(organization_members::user_id.eq(uid))
                            .first::<crate::models::organization::OrganizationMember>(&mut conn)
                            .optional()?
                            .is_some();

                        Ok(is_member)
                    } else {
                        // No user provided, deny access to cached organization packages
                        Ok(false)
                    }
                } else {
                    // Regular cached package - all are public
                    Ok(true)
                }
            }
            None => Ok(true), // Package doesn't exist locally = allow access (will proxy to upstream)
        }
    }

    /// Checks if a user has write permission for a package
    /// For scoped packages, checks organization membership
    /// For regular packages, checks individual ownership
    pub fn has_write_permission(
        &self,
        package_name: &str,
        user_id: i32,
    ) -> Result<bool, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // First check individual ownership (for both scoped and regular packages)
        let owner = package_owners::table
            .filter(package_owners::package_name.eq(package_name))
            .filter(package_owners::user_id.eq(user_id))
            .filter(
                package_owners::permission_level
                    .eq("write")
                    .or(package_owners::permission_level.eq("admin")),
            )
            .first::<PackageOwner>(&mut conn)
            .optional()?;

        if owner.is_some() {
            return Ok(true);
        }

        // If no individual ownership, check organization membership for scoped packages
        let package = packages::table
            .filter(packages::name.eq(package_name))
            .first::<Package>(&mut conn)
            .optional()?;

        if let Some(pkg) = package {
            if let Some(org_id) = pkg.organization_id {
                // Check if user is a member of the organization with at least member role
                let member = organization_members::table
                    .filter(organization_members::organization_id.eq(org_id))
                    .filter(organization_members::user_id.eq(user_id))
                    .first::<crate::models::organization::OrganizationMember>(&mut conn)
                    .optional()?;

                if let Some(member) = member {
                    // All organization members can publish packages
                    let user_role = OrganizationRole::from_role_str(&member.role)
                        .unwrap_or(OrganizationRole::Member);
                    return Ok(user_role.can_publish_packages());
                }
            }
        }

        Ok(false)
    }

    /// Checks if a package exists (has any owners)
    pub fn package_exists(&self, package_name: &str) -> Result<bool, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let count: i64 = package_owners::table
            .filter(package_owners::package_name.eq(package_name))
            .count()
            .get_result(&mut conn)?;

        Ok(count > 0)
    }

    /// Checks if a package exists in the packages table (for published packages)
    pub fn package_published(&self, package_name: &str) -> Result<bool, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let package = packages::table
            .filter(packages::name.eq(package_name))
            .first::<Package>(&mut conn)
            .optional()?;

        Ok(package.is_some())
    }

    /// Creates a new package owner
    pub fn create_package_owner(
        &self,
        package_name: &str,
        user_id: i32,
        permission_level: &str,
    ) -> Result<PackageOwner, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let new_owner = NewPackageOwner::new(
            package_name.to_string(),
            user_id,
            permission_level.to_string(),
        );

        diesel::insert_into(package_owners::table)
            .values(&new_owner)
            .execute(&mut conn)?;

        package_owners::table
            .filter(package_owners::package_name.eq(package_name))
            .filter(package_owners::user_id.eq(user_id))
            .first::<PackageOwner>(&mut conn)
    }

    /// Gets all owners of a package
    pub fn get_package_owners(
        &self,
        package_name: &str,
    ) -> Result<Vec<PackageOwner>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        package_owners::table
            .filter(package_owners::package_name.eq(package_name))
            .load::<PackageOwner>(&mut conn)
    }

    /// Adds a user as an owner of a package
    pub fn add_package_owner(
        &self,
        package_name: &str,
        user_id: i32,
        permission_level: &str,
    ) -> Result<PackageOwner, diesel::result::Error> {
        self.create_package_owner(package_name, user_id, permission_level)
    }

    /// Removes a user as an owner of a package
    pub fn remove_package_owner(
        &self,
        package_name: &str,
        user_id: i32,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(
            package_owners::table
                .filter(package_owners::package_name.eq(package_name))
                .filter(package_owners::user_id.eq(user_id)),
        )
        .execute(&mut conn)
    }

    /// Updates a user's permission level for a package
    pub fn update_permission_level(
        &self,
        package_name: &str,
        user_id: i32,
        new_permission_level: &str,
    ) -> Result<PackageOwner, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(
            package_owners::table
                .filter(package_owners::package_name.eq(package_name))
                .filter(package_owners::user_id.eq(user_id)),
        )
        .set(package_owners::permission_level.eq(new_permission_level))
        .execute(&mut conn)?;

        package_owners::table
            .filter(package_owners::package_name.eq(package_name))
            .filter(package_owners::user_id.eq(user_id))
            .first::<PackageOwner>(&mut conn)
    }

    /// Checks if a user can publish to a package (either new package or has write permission)
    pub fn can_publish_package(
        &self,
        package_name: &str,
        user_id: i32,
    ) -> Result<bool, diesel::result::Error> {
        // If package doesn't exist, anyone can publish it (new package)
        if !self.package_exists(package_name)? {
            return Ok(true);
        }

        // If package exists, check if user has write permission
        self.has_write_permission(package_name, user_id)
    }
}
