use crate::models::package::{Package, PopularPackage};
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
    pub recent_packages: Vec<Package>,
    pub cache_hit_rate: f64,
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

#[derive(Serialize)]
pub struct PackageListResponse {
    pub packages: Vec<Package>,
    pub total_count: usize,
    pub total_size_bytes: i64,
    pub total_size_mb: f64,
}
