use crate::schema::{organization_members, organizations};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};

// Organization model
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = organizations)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = organizations)]
pub struct NewOrganization {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = organizations)]
pub struct UpdateOrganization {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

// Organization member model
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = organization_members)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OrganizationMember {
    pub id: i32,
    pub user_id: i32,
    pub organization_id: i32,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = organization_members)]
pub struct NewOrganizationMember {
    pub user_id: i32,
    pub organization_id: i32,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = organization_members)]
pub struct UpdateOrganizationMember {
    pub role: Option<String>,
}

// Combined models for complex queries
#[derive(Serialize, Debug)]
pub struct OrganizationWithMembers {
    pub organization: Organization,
    pub members: Vec<OrganizationMemberWithUser>,
}

#[derive(Serialize, Debug)]
pub struct OrganizationMemberWithUser {
    pub member: OrganizationMember,
    pub username: String,
    pub email: String,
}

// Request/Response models for API
#[derive(Deserialize, Debug)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateOrganizationRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddMemberRequest {
    pub username: String,
    pub role: String, // "owner", "admin", "member"
}

#[derive(Deserialize, Debug)]
pub struct UpdateMemberRequest {
    pub role: String,
}

#[derive(Serialize, Debug)]
pub struct OrganizationResponse {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub member_count: i64,
    pub package_count: i64,
}

// Role validation
#[derive(Debug, PartialEq)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Member,
}

impl OrganizationRole {
    pub fn from_role_str(role: &str) -> Option<Self> {
        match role.to_lowercase().as_str() {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            _ => None,
        }
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_publish_packages(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member)
    }

    pub fn can_delete_organization(&self) -> bool {
        matches!(self, Self::Owner)
    }
}

impl std::fmt::Display for OrganizationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Owner => write!(f, "owner"),
            Self::Admin => write!(f, "admin"),
            Self::Member => write!(f, "member"),
        }
    }
}

impl NewOrganization {
    pub fn new(name: String, display_name: Option<String>, description: Option<String>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            name,
            display_name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

impl NewOrganizationMember {
    pub fn new(user_id: i32, organization_id: i32, role: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            user_id,
            organization_id,
            role,
            created_at: now,
        }
    }
}

// Validation functions
pub fn validate_organization_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Organization name cannot be empty".to_string());
    }

    if name.len() > 50 {
        return Err("Organization name cannot be longer than 50 characters".to_string());
    }

    // Organization names should follow npm scope naming rules
    // - Must start with a letter or underscore
    // - Can contain letters, numbers, underscores, hyphens, and dots
    // - Cannot start with a dot or hyphen
    if !name.chars().next().unwrap().is_ascii_alphabetic() && !name.starts_with('_') {
        return Err("Organization name must start with a letter or underscore".to_string());
    }

    for char in name.chars() {
        if !char.is_ascii_alphanumeric() && char != '_' && char != '-' && char != '.' {
            return Err("Organization name can only contain letters, numbers, underscores, hyphens, and dots".to_string());
        }
    }

    if name.starts_with('.') || name.starts_with('-') {
        return Err("Organization name cannot start with a dot or hyphen".to_string());
    }

    Ok(())
}

pub fn validate_role(role: &str) -> Result<OrganizationRole, String> {
    OrganizationRole::from_role_str(role)
        .ok_or_else(|| "Invalid role. Must be 'owner', 'admin', or 'member'".to_string())
}
