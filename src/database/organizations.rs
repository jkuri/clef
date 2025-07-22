use super::connection::{DbPool, get_connection_with_retry};
use crate::models::organization::*;
use crate::models::user::User;
use crate::schema::{organization_members, organizations, packages, users};
use diesel::prelude::*;

/// Organization-related database operations
pub struct OrganizationOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> OrganizationOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Creates a new organization
    pub fn create_organization(
        &self,
        name: &str,
        display_name: Option<String>,
        description: Option<String>,
        creator_user_id: i32,
    ) -> Result<Organization, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Validate organization name
        if let Err(e) = validate_organization_name(name) {
            return Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::CheckViolation,
                Box::new(e),
            ));
        }

        conn.transaction(|conn| {
            // Create the organization
            let new_org = NewOrganization::new(name.to_string(), display_name, description);

            diesel::insert_into(organizations::table)
                .values(&new_org)
                .execute(conn)?;

            let organization = organizations::table
                .filter(organizations::name.eq(name))
                .first::<Organization>(conn)?;

            // Add the creator as an owner
            let new_member = NewOrganizationMember::new(
                creator_user_id,
                organization.id,
                OrganizationRole::Owner.to_string(),
            );

            diesel::insert_into(organization_members::table)
                .values(&new_member)
                .execute(conn)?;

            Ok(organization)
        })
    }

    /// Gets an organization by name
    pub fn get_organization_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Organization>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        organizations::table
            .filter(organizations::name.eq(name))
            .first::<Organization>(&mut conn)
            .optional()
    }

    /// Gets an organization by ID
    pub fn get_organization_by_id(
        &self,
        id: i32,
    ) -> Result<Option<Organization>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        organizations::table
            .find(id)
            .first::<Organization>(&mut conn)
            .optional()
    }

    /// Updates an organization
    pub fn update_organization(
        &self,
        id: i32,
        display_name: Option<String>,
        description: Option<String>,
    ) -> Result<Organization, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let update_org = UpdateOrganization {
            display_name,
            description,
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        diesel::update(organizations::table.find(id))
            .set(&update_org)
            .execute(&mut conn)?;

        organizations::table
            .find(id)
            .first::<Organization>(&mut conn)
    }

    /// Deletes an organization (only if no packages are associated)
    pub fn delete_organization(&self, id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        conn.transaction(|conn| {
            // Check if organization has any packages
            let package_count: i64 = packages::table
                .filter(packages::organization_id.eq(id))
                .count()
                .get_result(conn)?;

            if package_count > 0 {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                    Box::new("Cannot delete organization with associated packages".to_string()),
                ));
            }

            // Delete all members first (due to foreign key constraints)
            diesel::delete(
                organization_members::table.filter(organization_members::organization_id.eq(id)),
            )
            .execute(conn)?;

            // Delete the organization
            diesel::delete(organizations::table.find(id)).execute(conn)?;

            Ok(())
        })
    }

    /// Adds a member to an organization
    pub fn add_member(
        &self,
        organization_id: i32,
        user_id: i32,
        role: &str,
    ) -> Result<OrganizationMember, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Validate role
        if validate_role(role).is_err() {
            return Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::CheckViolation,
                Box::new("Invalid role".to_string()),
            ));
        }

        let new_member = NewOrganizationMember::new(user_id, organization_id, role.to_string());

        diesel::insert_into(organization_members::table)
            .values(&new_member)
            .execute(&mut conn)?;

        organization_members::table
            .filter(organization_members::user_id.eq(user_id))
            .filter(organization_members::organization_id.eq(organization_id))
            .first::<OrganizationMember>(&mut conn)
    }

    /// Updates a member's role
    pub fn update_member_role(
        &self,
        organization_id: i32,
        user_id: i32,
        new_role: &str,
    ) -> Result<OrganizationMember, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Validate role
        if validate_role(new_role).is_err() {
            return Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::CheckViolation,
                Box::new("Invalid role".to_string()),
            ));
        }

        let update_member = UpdateOrganizationMember {
            role: Some(new_role.to_string()),
        };

        diesel::update(
            organization_members::table
                .filter(organization_members::organization_id.eq(organization_id))
                .filter(organization_members::user_id.eq(user_id)),
        )
        .set(&update_member)
        .execute(&mut conn)?;

        organization_members::table
            .filter(organization_members::user_id.eq(user_id))
            .filter(organization_members::organization_id.eq(organization_id))
            .first::<OrganizationMember>(&mut conn)
    }

    /// Removes a member from an organization
    pub fn remove_member(
        &self,
        organization_id: i32,
        user_id: i32,
    ) -> Result<(), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Check if this is the last owner
        let owner_count: i64 = organization_members::table
            .filter(organization_members::organization_id.eq(organization_id))
            .filter(organization_members::role.eq("owner"))
            .count()
            .get_result(&mut conn)?;

        if owner_count <= 1 {
            // Check if the user being removed is an owner
            let is_owner = organization_members::table
                .filter(organization_members::organization_id.eq(organization_id))
                .filter(organization_members::user_id.eq(user_id))
                .filter(organization_members::role.eq("owner"))
                .first::<OrganizationMember>(&mut conn)
                .optional()?
                .is_some();

            if is_owner {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    Box::new("Cannot remove the last owner from an organization".to_string()),
                ));
            }
        }

        diesel::delete(
            organization_members::table
                .filter(organization_members::organization_id.eq(organization_id))
                .filter(organization_members::user_id.eq(user_id)),
        )
        .execute(&mut conn)?;

        Ok(())
    }

    /// Gets all members of an organization
    pub fn get_organization_members(
        &self,
        organization_id: i32,
    ) -> Result<Vec<OrganizationMemberWithUser>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let results: Vec<(OrganizationMember, User)> = organization_members::table
            .inner_join(users::table)
            .filter(organization_members::organization_id.eq(organization_id))
            .load::<(OrganizationMember, User)>(&mut conn)?;

        Ok(results
            .into_iter()
            .map(|(member, user)| OrganizationMemberWithUser {
                member,
                username: user.username,
                email: user.email,
            })
            .collect())
    }

    /// Checks if a user is a member of an organization with a specific role or higher
    pub fn check_user_permission(
        &self,
        organization_id: i32,
        user_id: i32,
        required_role: OrganizationRole,
    ) -> Result<bool, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let member = organization_members::table
            .filter(organization_members::organization_id.eq(organization_id))
            .filter(organization_members::user_id.eq(user_id))
            .first::<OrganizationMember>(&mut conn)
            .optional()?;

        match member {
            Some(member) => {
                let user_role = OrganizationRole::from_role_str(&member.role)
                    .unwrap_or(OrganizationRole::Member);
                Ok(self.role_has_permission(&user_role, &required_role))
            }
            None => Ok(false),
        }
    }

    /// Helper function to check if a role has the required permission
    fn role_has_permission(
        &self,
        user_role: &OrganizationRole,
        required_role: &OrganizationRole,
    ) -> bool {
        match required_role {
            OrganizationRole::Member => true, // All roles can do member-level actions
            OrganizationRole::Admin => {
                matches!(user_role, OrganizationRole::Owner | OrganizationRole::Admin)
            }
            OrganizationRole::Owner => matches!(user_role, OrganizationRole::Owner),
        }
    }
}
