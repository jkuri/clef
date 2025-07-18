use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::{packages, package_owners};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = packages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub filename: String,
    pub size_bytes: i64,
    pub etag: Option<String>,
    pub content_type: Option<String>,
    pub upstream_url: String,
    pub file_path: String,
    pub created_at: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
    pub access_count: i32,
    pub author_id: Option<i32>,
    pub description: Option<String>,
    pub package_json: Option<String>,
    pub is_private: bool,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = packages)]
pub struct NewPackage {
    pub name: String,
    pub version: String,
    pub filename: String,
    pub size_bytes: i64,
    pub etag: Option<String>,
    pub content_type: Option<String>,
    pub upstream_url: String,
    pub file_path: String,
    pub created_at: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
    pub access_count: i32,
    pub author_id: Option<i32>,
    pub description: Option<String>,
    pub package_json: Option<String>,
    pub is_private: bool,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = packages)]
pub struct UpdatePackage {
    pub last_accessed: Option<NaiveDateTime>,
    pub access_count: Option<i32>,
}

#[derive(Serialize, Debug)]
pub struct PopularPackage {
    pub name: String,
    pub total_downloads: i64,
    pub unique_versions: i64,
    pub total_size_bytes: i64,
}

#[derive(Serialize, Debug)]
pub struct PackageVersions {
    pub package_name: String,
    pub versions: Vec<Package>,
    pub total_size_bytes: i64,
}

// Package ownership models
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = package_owners)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PackageOwner {
    pub id: i32,
    pub package_name: String,
    pub user_id: i32,
    pub permission_level: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = package_owners)]
pub struct NewPackageOwner {
    pub package_name: String,
    pub user_id: i32,
    pub permission_level: String,
    pub created_at: NaiveDateTime,
}

impl NewPackage {
    pub fn new(
        name: String,
        version: String,
        filename: String,
        size_bytes: i64,
        etag: Option<String>,
        content_type: Option<String>,
        upstream_url: String,
        file_path: String,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            name,
            version,
            filename,
            size_bytes,
            etag,
            content_type,
            upstream_url,
            file_path,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            author_id: None,
            description: None,
            package_json: None,
            is_private: false,
        }
    }
}

impl Package {
    pub fn extract_version_from_filename(&self) -> Option<String> {
        // Extract version from filename like "package-1.2.3.tgz"
        let name_prefix = format!("{}-", self.name);
        if let Some(version_part) = self.filename.strip_prefix(&name_prefix) {
            if let Some(version) = version_part.strip_suffix(".tgz") {
                return Some(version.to_string());
            }
        }
        None
    }
}

impl NewPackageOwner {
    pub fn new(package_name: String, user_id: i32, permission_level: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            package_name,
            user_id,
            permission_level,
            created_at: now,
        }
    }
}
