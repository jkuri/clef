use crate::models::package::{PackageWithVersions, PopularPackage};
use crate::schema::cache_stats;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::serde::Serialize;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub created_at: u64,
    pub size: u64,
    pub etag: Option<String>,
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub hit_count: u64,
    pub miss_count: u64,
}

#[derive(Serialize, Debug)]
pub struct CacheAnalytics {
    pub total_packages: i64,
    pub total_size_bytes: i64,
    pub total_size_mb: f64,
    pub most_popular_packages: Vec<PopularPackage>,
    pub recent_packages: Vec<PackageWithVersions>,
    pub cache_hit_rate: f64,
}

// Database model for persistent cache stats
#[derive(Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = cache_stats)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CacheStatsRecord {
    pub id: i32,
    pub hit_count: i64,
    pub miss_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = cache_stats)]
pub struct NewCacheStatsRecord {
    pub hit_count: i64,
    pub miss_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = cache_stats)]
pub struct UpdateCacheStatsRecord {
    pub hit_count: Option<i64>,
    pub miss_count: Option<i64>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct CacheStatsResponse {
    pub enabled: bool,
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub total_size_mb: f64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub cache_dir: String,
    pub ttl_hours: u64,
}
