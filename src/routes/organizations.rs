use crate::error::ApiError;
use crate::models::auth::AuthenticatedUser;
use crate::models::organization::*;
use crate::state::AppState;
use rocket::serde::json::Json;
use rocket::{State, delete, get, post, put};

/// Create a new organization
#[post("/api/v1/organizations", data = "<request>")]
pub async fn create_organization(
    request: Json<CreateOrganizationRequest>,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<Organization>, ApiError> {
    // Validate organization name
    validate_organization_name(&request.name).map_err(ApiError::BadRequest)?;

    let organization = state
        .database
        .create_organization(
            &request.name,
            request.display_name.clone(),
            request.description.clone(),
            user.user_id,
        )
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => ApiError::Conflict(format!("Organization '{}' already exists", request.name)),
            _ => ApiError::InternalServerError(format!("Database error: {e}")),
        })?;

    Ok(Json(organization))
}

/// Get organization by name
#[get("/api/v1/organizations/<name>")]
pub async fn get_organization(
    name: &str,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<OrganizationWithMembers>, ApiError> {
    let organization = state
        .database
        .get_organization_by_name(name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Organization '{name}' not found")))?;

    // Check if user is a member of the organization
    let is_member = state
        .database
        .check_organization_permission(organization.id, user.user_id, OrganizationRole::Member)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    if !is_member {
        return Err(ApiError::Forbidden(
            "You are not a member of this organization".to_string(),
        ));
    }

    let members = state
        .database
        .get_organization_members(organization.id)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    Ok(Json(OrganizationWithMembers {
        organization,
        members,
    }))
}

/// Update organization
#[put("/api/v1/organizations/<name>", data = "<request>")]
pub async fn update_organization(
    name: &str,
    request: Json<UpdateOrganizationRequest>,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<Organization>, ApiError> {
    let organization = state
        .database
        .get_organization_by_name(name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Organization '{name}' not found")))?;

    // Check if user has admin permission
    let has_permission = state
        .database
        .check_organization_permission(organization.id, user.user_id, OrganizationRole::Admin)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    if !has_permission {
        return Err(ApiError::Forbidden(
            "You don't have permission to update this organization".to_string(),
        ));
    }

    let updated_organization = state
        .database
        .update_organization(
            organization.id,
            request.display_name.clone(),
            request.description.clone(),
        )
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    Ok(Json(updated_organization))
}

/// Delete organization
#[delete("/api/v1/organizations/<name>")]
pub async fn delete_organization(
    name: &str,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let organization = state
        .database
        .get_organization_by_name(name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Organization '{name}' not found")))?;

    // Check if user has owner permission
    let has_permission = state
        .database
        .check_organization_permission(organization.id, user.user_id, OrganizationRole::Owner)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    if !has_permission {
        return Err(ApiError::Forbidden(
            "You don't have permission to delete this organization".to_string(),
        ));
    }

    state
        .database
        .delete_organization(organization.id)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => ApiError::BadRequest(
                "Cannot delete organization with associated packages".to_string(),
            ),
            _ => ApiError::InternalServerError(format!("Database error: {e}")),
        })?;

    Ok(Json(serde_json::json!({
        "message": format!("Organization '{}' deleted successfully", name)
    })))
}

/// Add member to organization
#[post("/api/v1/organizations/<name>/members", data = "<request>")]
pub async fn add_member(
    name: &str,
    request: Json<AddMemberRequest>,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<OrganizationMember>, ApiError> {
    let organization = state
        .database
        .get_organization_by_name(name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Organization '{name}' not found")))?;

    // Check if user has admin permission
    let has_permission = state
        .database
        .check_organization_permission(organization.id, user.user_id, OrganizationRole::Admin)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    if !has_permission {
        return Err(ApiError::Forbidden(
            "You don't have permission to add members to this organization".to_string(),
        ));
    }

    // Validate role
    validate_role(&request.role).map_err(ApiError::BadRequest)?;

    // Find user by username
    let target_user = state
        .database
        .get_user_by_username(&request.username)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("User '{}' not found", request.username)))?;

    let member = state
        .database
        .add_organization_member(organization.id, target_user.id, &request.role)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => ApiError::Conflict("User is already a member of this organization".to_string()),
            _ => ApiError::InternalServerError(format!("Database error: {e}")),
        })?;

    Ok(Json(member))
}

/// Update member role
#[put("/api/v1/organizations/<name>/members/<username>", data = "<request>")]
pub async fn update_member_role(
    name: &str,
    username: &str,
    request: Json<UpdateMemberRequest>,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<OrganizationMember>, ApiError> {
    let organization = state
        .database
        .get_organization_by_name(name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Organization '{name}' not found")))?;

    // Check if user has admin permission
    let has_permission = state
        .database
        .check_organization_permission(organization.id, user.user_id, OrganizationRole::Admin)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    if !has_permission {
        return Err(ApiError::Forbidden(
            "You don't have permission to update member roles in this organization".to_string(),
        ));
    }

    // Validate role
    validate_role(&request.role).map_err(ApiError::BadRequest)?;

    // Find user by username
    let target_user = state
        .database
        .get_user_by_username(username)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("User '{username}' not found")))?;

    let updated_member = state
        .database
        .update_organization_member_role(organization.id, target_user.id, &request.role)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    Ok(Json(updated_member))
}

/// Remove member from organization
#[delete("/api/v1/organizations/<name>/members/<username>")]
pub async fn remove_member(
    name: &str,
    username: &str,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let organization = state
        .database
        .get_organization_by_name(name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Organization '{name}' not found")))?;

    // Find user by username
    let target_user = state
        .database
        .get_user_by_username(username)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("User '{username}' not found")))?;

    // Check if user has admin permission OR is removing themselves
    let has_admin_permission = state
        .database
        .check_organization_permission(organization.id, user.user_id, OrganizationRole::Admin)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    let is_self_removal = user.user_id == target_user.id;

    if !has_admin_permission && !is_self_removal {
        return Err(ApiError::Forbidden(
            "You don't have permission to remove this member".to_string(),
        ));
    }

    state
        .database
        .remove_organization_member(organization.id, target_user.id)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::CheckViolation,
                _,
            ) => ApiError::BadRequest(
                "Cannot remove the last owner from an organization".to_string(),
            ),
            _ => ApiError::InternalServerError(format!("Database error: {e}")),
        })?;

    Ok(Json(serde_json::json!({
        "message": format!("User '{}' removed from organization '{}'", username, name)
    })))
}
