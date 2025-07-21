use super::connection::{DbPool, get_connection_with_retry};
use crate::models::package::*;
use crate::schema::{package_owners, packages};
use diesel::prelude::*;

/// Package ownership-related database operations
pub struct PackageOwnerOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> PackageOwnerOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Checks if a user has write permission for a package
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

        Ok(owner.is_some())
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
