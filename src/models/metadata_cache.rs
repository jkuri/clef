use crate::schema::metadata_cache;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::Serialize;

#[derive(Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = metadata_cache)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MetadataCacheRecord {
    pub id: i32,
    pub package_name: String,
    pub size_bytes: i64,
    pub file_path: String,
    pub etag: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
    pub access_count: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = metadata_cache)]
pub struct NewMetadataCacheRecord {
    pub package_name: String,
    pub size_bytes: i64,
    pub file_path: String,
    pub etag: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
    pub access_count: i32,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = metadata_cache)]
pub struct UpdateMetadataCacheRecord {
    pub size_bytes: Option<i64>,
    pub file_path: Option<String>,
    pub etag: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub last_accessed: Option<NaiveDateTime>,
    pub access_count: Option<i32>,
}

#[derive(Serialize, Debug)]
pub struct MetadataCacheStats {
    pub total_entries: i64,
    pub total_size_bytes: i64,
    pub total_size_mb: f64,
}
