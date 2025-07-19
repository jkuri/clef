use crate::schema::{package_files, package_owners, package_versions, packages};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};

// Package model - stores package-level metadata
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = packages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub author_id: Option<i32>,
    pub homepage: Option<String>,
    pub repository_url: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<String>, // JSON array as text
    pub is_private: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = packages)]
pub struct NewPackage {
    pub name: String,
    pub description: Option<String>,
    pub author_id: Option<i32>,
    pub homepage: Option<String>,
    pub repository_url: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<String>,
    pub is_private: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = packages)]
pub struct UpdatePackage {
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository_url: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

// Package version model - stores version-specific metadata
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = package_versions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PackageVersion {
    pub id: i32,
    pub package_id: i32,
    pub version: String,
    pub description: Option<String>,
    pub main_file: Option<String>,
    pub scripts: Option<String>,           // JSON object as text
    pub dependencies: Option<String>,      // JSON object as text
    pub dev_dependencies: Option<String>,  // JSON object as text
    pub peer_dependencies: Option<String>, // JSON object as text
    pub engines: Option<String>,           // JSON object as text
    pub shasum: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = package_versions)]
pub struct NewPackageVersion {
    pub package_id: i32,
    pub version: String,
    pub description: Option<String>,
    pub main_file: Option<String>,
    pub scripts: Option<String>,
    pub dependencies: Option<String>,
    pub dev_dependencies: Option<String>,
    pub peer_dependencies: Option<String>,
    pub engines: Option<String>,
    pub shasum: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = package_versions)]
pub struct UpdatePackageVersion {
    pub description: Option<String>,
    pub main_file: Option<String>,
    pub scripts: Option<String>,
    pub dependencies: Option<String>,
    pub dev_dependencies: Option<String>,
    pub peer_dependencies: Option<String>,
    pub engines: Option<String>,
    pub shasum: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

// Package file model - stores file-specific metadata and cache info
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = package_files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PackageFile {
    pub id: i32,
    pub package_version_id: i32,
    pub filename: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub upstream_url: String,
    pub file_path: String,
    pub created_at: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
    pub access_count: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = package_files)]
pub struct NewPackageFile {
    pub package_version_id: i32,
    pub filename: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub upstream_url: String,
    pub file_path: String,
    pub created_at: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
    pub access_count: i32,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = package_files)]
pub struct UpdatePackageFile {
    pub last_accessed: Option<NaiveDateTime>,
    pub access_count: Option<i32>,
    pub etag: Option<String>,
}

// Combined models for complex queries
#[derive(Serialize, Debug)]
pub struct PackageWithVersions {
    pub package: Package,
    pub versions: Vec<PackageVersionWithFiles>,
}

#[derive(Serialize, Debug)]
pub struct PackageVersionWithFiles {
    pub version: PackageVersion,
    pub files: Vec<PackageFile>,
}

#[derive(Serialize, Debug)]
pub struct PopularPackage {
    pub name: String,
    pub total_downloads: i64,
    pub unique_versions: i64,
    pub total_size_bytes: i64,
}

// Analytics and API response structs
#[derive(Serialize, Debug)]
pub struct PackageListResponse {
    pub packages: Vec<PackageWithVersions>,
    pub total_count: usize,
    pub total_size_bytes: i64,
    pub total_size_mb: f64,
}

#[derive(Serialize, Debug)]
pub struct PackageVersionsResponse {
    pub package: Package,
    pub versions: Vec<PackageVersionWithFiles>,
    pub total_size_bytes: i64,
}

// Package ownership models (unchanged)
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

// Implementation methods
impl NewPackage {
    pub fn new(name: String, description: Option<String>, author_id: Option<i32>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            name,
            description,
            author_id,
            homepage: None,
            repository_url: None,
            license: None,
            keywords: None,
            is_private: false,
            created_at: now,
            updated_at: now,
        }
    }
}

impl NewPackageVersion {
    pub fn new(package_id: i32, version: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            package_id,
            version,
            description: None,
            main_file: None,
            scripts: None,
            dependencies: None,
            dev_dependencies: None,
            peer_dependencies: None,
            engines: None,
            shasum: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl NewPackageFile {
    pub fn new(
        package_version_id: i32,
        filename: String,
        size_bytes: i64,
        upstream_url: String,
        file_path: String,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            package_version_id,
            filename,
            size_bytes,
            content_type: None,
            etag: None,
            upstream_url,
            file_path,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }
}

impl NewPackageOwner {
    pub fn new(package_name: String, user_id: i32, permission_level: String) -> Self {
        Self {
            package_name,
            user_id,
            permission_level,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
