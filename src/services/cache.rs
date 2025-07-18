use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, warn, debug};
use crate::config::AppConfig;
use crate::services::DatabaseService;
use crate::models::{NewPackage, CacheEntry, CacheStats};
// Arc removed - database passed as parameter

#[derive(Debug)]
pub struct CacheService {
    config: AppConfig,
    hit_count: std::sync::atomic::AtomicU64,
    miss_count: std::sync::atomic::AtomicU64,
}

impl CacheService {
    pub fn new(config: AppConfig) -> Result<Self, std::io::Error> {
        if config.cache_enabled {
            // Create cache directory if it doesn't exist
            fs::create_dir_all(&config.cache_dir)?;
            info!("Cache initialized at: {}", config.cache_dir);
        }

        Ok(Self {
            config,
            hit_count: std::sync::atomic::AtomicU64::new(0),
            miss_count: std::sync::atomic::AtomicU64::new(0),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.config.cache_enabled
    }

    // Database is now passed as parameter to methods that need it

    fn extract_version_from_filename(&self, package: &str, filename: &str) -> Option<String> {
        // Extract version from filename like "package-1.2.3.tgz"
        let name_prefix = format!("{}-", package);
        if let Some(version_part) = filename.strip_prefix(&name_prefix) {
            if let Some(version) = version_part.strip_suffix(".tgz") {
                return Some(version.to_string());
            }
        }
        None
    }

    pub fn get_cache_key(&self, package: &str, filename: &str) -> String {
        format!("{}/{}", package, filename)
    }

    pub fn get_cache_path(&self, package: &str, filename: &str) -> PathBuf {
        // Scoped packages like @jkuri/test-scoped-package are stored as @jkuri/test-scoped-package/
        let packages_dir = Path::new(&self.config.cache_dir).join("packages");
        let package_dir = packages_dir.join(package);
        package_dir.join(filename)
    }

    pub fn get_metadata_path(&self, package: &str, filename: &str) -> PathBuf {
        // Scoped packages like @jkuri/test-scoped-package are stored as @jkuri/test-scoped-package/
        let packages_dir = Path::new(&self.config.cache_dir).join("packages");
        let package_dir = packages_dir.join(package);
        let meta_filename = format!("{}.meta", filename);
        package_dir.join(meta_filename)
    }

    pub fn get_metadata_cache_path(&self, package: &str) -> PathBuf {
        // Metadata cache files are stored as {package}.metadata.json
        let packages_dir = Path::new(&self.config.cache_dir).join("packages");
        let package_dir = packages_dir.join(package);
        package_dir.join("metadata.json")
    }

    pub fn get_metadata_etag_path(&self, package: &str) -> PathBuf {
        let packages_dir = Path::new(&self.config.cache_dir).join("packages");
        let package_dir = packages_dir.join(package);
        package_dir.join("metadata.etag")
    }

    fn has_published_versions(&self, metadata: &serde_json::Value) -> bool {
        // Check if metadata contains published versions by looking for versions with our server's tarball URLs
        if let Some(versions) = metadata.get("versions").and_then(|v| v.as_object()) {
            for version_data in versions.values() {
                if let Some(dist) = version_data.get("dist") {
                    if let Some(tarball) = dist.get("tarball").and_then(|t| t.as_str()) {
                        // If tarball URL points to our server, it's a published package
                        if tarball.contains(&format!("{}:{}", self.config.host, self.config.port)) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub async fn get(&self, package: &str, filename: &str) -> Option<CacheEntry> {
        if !self.config.cache_enabled {
            return None;
        }

        let cache_key = self.get_cache_key(package, filename);
        let cache_path = self.get_cache_path(package, filename);

        debug!("Checking cache for key: {}", cache_key);

        // Always check if file exists on disk (never delete packages)
        if !cache_path.exists() {
            self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Cache miss for key: {} - file not found", cache_key);
            return None;
        }

        // Read cache entry (no TTL check - packages are kept forever)
        match fs::read(&cache_path) {
            Ok(data) => {
                let size = data.len() as u64;
                let created_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Try to read metadata (etag, etc.)
                let meta_path = self.get_metadata_path(package, filename);
                let etag = fs::read_to_string(&meta_path).ok();

                self.hit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Cache hit for key: {} (size: {} bytes)", cache_key, size);

                Some(CacheEntry {
                    data,
                    created_at,
                    size,
                    etag,
                })
            }
            Err(e) => {
                warn!("Failed to read cache entry {}: {}", cache_key, e);
                self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        }
    }

    pub async fn get_metadata(&self, package: &str) -> Option<CacheEntry> {
        if !self.config.cache_enabled {
            return None;
        }

        let cache_key = format!("{}.metadata", package);
        let cache_path = self.get_metadata_cache_path(package);

        debug!("Checking metadata cache for key: {}", cache_key);

        if !cache_path.exists() {
            self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Metadata cache miss for key: {} - file not found", cache_key);
            return None;
        }

        // Check if metadata is stale (TTL for upstream packages, never expire for published packages)
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now().duration_since(modified).unwrap_or_default();
                let ttl_seconds = self.config.cache_ttl_hours * 3600;

                // Only apply TTL to upstream packages (check if this is a published package by looking for author_id in cached metadata)
                if age.as_secs() > ttl_seconds as u64 {
                    if let Ok(data) = fs::read_to_string(&cache_path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                            // If it doesn't have published versions (no author_id), it's upstream and should expire
                            if !self.has_published_versions(&json) {
                                debug!("Metadata cache expired for upstream package: {}", cache_key);
                                self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                return None;
                            }
                        }
                    }
                }
            }
        }

        match fs::read(&cache_path) {
            Ok(data) => {
                let size = data.len() as u64;
                let created_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                self.hit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Metadata cache hit for key: {} (size: {} bytes)", cache_key, size);

                // Try to read ETag from metadata file
                let etag_path = self.get_metadata_etag_path(package);
                let etag = fs::read_to_string(&etag_path).ok();

                Some(CacheEntry {
                    data,
                    created_at,
                    size,
                    etag,
                })
            }
            Err(e) => {
                warn!("Failed to read metadata cache entry {}: {}", cache_key, e);
                self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        }
    }

    pub async fn put(&self, package: &str, filename: &str, data: &[u8], etag: Option<&str>, _upstream_url: &str, database: Option<&DatabaseService>) -> Result<(), std::io::Error> {
        if !self.config.cache_enabled {
            return Ok(());
        }

        let cache_key = self.get_cache_key(package, filename);
        let cache_path = self.get_cache_path(package, filename);
        let meta_path = self.get_metadata_path(package, filename);

        debug!("Storing in cache key: {} (size: {} bytes)", cache_key, data.len());

        // Create package directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write data to cache (never delete - keep forever)
        fs::write(&cache_path, data)?;

        // Write metadata if available
        if let Some(etag_value) = etag {
            fs::write(&meta_path, etag_value)?;
        }

        // Store metadata in database if available
        if let Some(db) = database {
            let version = self.extract_version_from_filename(package, filename);
            let new_package = NewPackage::new(
                package.to_string(),
                version.unwrap_or_else(|| "unknown".to_string()),
                filename.to_string(),
                data.len() as i64,
                etag.map(|s| s.to_string()),
                Some("application/octet-stream".to_string()),
                _upstream_url.to_string(),
                cache_path.to_string_lossy().to_string(),
            );

            if let Err(e) = db.insert_package(new_package) {
                warn!("Failed to store package metadata in database: {}", e);
            } else {
                debug!("Stored package metadata in database for {}/{}", package, filename);
            }
        }

        info!("Cached tarball for {}/{} (size: {} bytes) - PERMANENT STORAGE", package, filename, data.len());
        Ok(())
    }

    pub async fn put_metadata(&self, package: &str, metadata_json: &str) -> Result<(), std::io::Error> {
        self.put_metadata_with_etag(package, metadata_json, None).await
    }

    pub async fn put_metadata_with_etag(&self, package: &str, metadata_json: &str, etag: Option<&str>) -> Result<(), std::io::Error> {
        if !self.config.cache_enabled {
            return Ok(());
        }

        let cache_key = format!("{}.metadata", package);
        let cache_path = self.get_metadata_cache_path(package);
        let etag_path = self.get_metadata_etag_path(package);

        debug!("Storing metadata in cache key: {} (size: {} bytes)", cache_key, metadata_json.len());

        // Create package directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write metadata to cache
        fs::write(&cache_path, metadata_json)?;

        // Write ETag if provided
        if let Some(etag_value) = etag {
            fs::write(&etag_path, etag_value)?;
            debug!("Stored ETag for metadata cache: {} -> {}", package, etag_value);
        } else if etag_path.exists() {
            // Remove old ETag file if no new ETag provided
            let _ = fs::remove_file(&etag_path);
        }

        info!("Cached metadata for {} (size: {} bytes)", package, metadata_json.len());
        Ok(())
    }

    pub async fn invalidate_metadata(&self, package: &str) -> Result<(), std::io::Error> {
        if !self.config.cache_enabled {
            return Ok(());
        }

        let cache_path = self.get_metadata_cache_path(package);
        let etag_path = self.get_metadata_etag_path(package);

        let mut removed_files = 0;

        if cache_path.exists() {
            fs::remove_file(&cache_path)?;
            removed_files += 1;
        }

        if etag_path.exists() {
            fs::remove_file(&etag_path)?;
            removed_files += 1;
        }

        if removed_files > 0 {
            info!("Invalidated metadata cache for package: {}", package);
        }

        Ok(())
    }

    // PERMANENT STORAGE: Packages are never deleted from cache
    // This ensures fast access to all previously downloaded packages
    pub async fn get_cache_info(&self) -> Result<String, std::io::Error> {
        let stats = self.get_stats().await?;
        Ok(format!(
            "PERMANENT CACHE: {} packages, {:.2} MB total - packages are kept forever for fast access",
            stats.total_entries,
            stats.total_size_bytes as f64 / 1024.0 / 1024.0
        ))
    }

    fn collect_cache_entries(&self, dir: &Path, entries: &mut Vec<(PathBuf, SystemTime, u64)>) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively check package directories
                self.collect_cache_entries(&path, entries)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("tgz") {
                // Only collect .tgz files (actual tarballs)
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        entries.push((path, created, metadata.len()));
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<CacheStats, std::io::Error> {
        let packages_dir = Path::new(&self.config.cache_dir).join("packages");

        if !packages_dir.exists() {
            return Ok(CacheStats {
                total_entries: 0,
                total_size_bytes: 0,
                hit_count: self.hit_count.load(std::sync::atomic::Ordering::Relaxed),
                miss_count: self.miss_count.load(std::sync::atomic::Ordering::Relaxed),
            });
        }

        let mut entries = Vec::new();
        self.collect_cache_entries(&packages_dir, &mut entries)?;

        let total_entries = entries.len();
        let total_size_bytes = entries.iter().map(|(_, _, size)| *size).sum();

        Ok(CacheStats {
            total_entries,
            total_size_bytes,
            hit_count: self.hit_count.load(std::sync::atomic::Ordering::Relaxed),
            miss_count: self.miss_count.load(std::sync::atomic::Ordering::Relaxed),
        })
    }

    pub fn get_hit_count(&self) -> u64 {
        self.hit_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_miss_count(&self) -> u64 {
        self.miss_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_hit_rate(&self) -> f64 {
        let hits = self.get_hit_count();
        let misses = self.get_miss_count();
        let total = hits + misses;

        if total > 0 {
            hits as f64 / total as f64 * 100.0
        } else {
            0.0
        }
    }

    pub async fn clear(&self) -> Result<(), std::io::Error> {
        let cache_dir = Path::new(&self.config.cache_dir);

        if !cache_dir.exists() {
            return Ok(());
        }

        warn!("CLEARING PERMANENT CACHE - This will remove all cached packages!");

        // Remove all package directories and their contents
        for entry in fs::read_dir(cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else if path.is_file() {
                fs::remove_file(path)?;
            }
        }

        info!("Permanent cache cleared - all packages removed");
        Ok(())
    }
}
