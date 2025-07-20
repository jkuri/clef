use crate::config::AppConfig;
use crate::database::files::CompletePackageParams;
use crate::models::{CacheEntry, CacheStats};
use crate::services::DatabaseService;
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
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

    /// Initialize cache service with persistent stats from database
    pub fn new_with_database(
        config: AppConfig,
        database: Option<&DatabaseService>,
    ) -> Result<Self, std::io::Error> {
        if config.cache_enabled {
            // Create cache directory if it doesn't exist
            fs::create_dir_all(&config.cache_dir)?;
            info!("Cache initialized at: {}", config.cache_dir);
        }

        // Try to restore counters from database
        let (initial_hit_count, initial_miss_count) = if let Some(db) = database {
            match db.get_persistent_cache_stats() {
                Ok(Some(stats)) => {
                    info!(
                        "Restored cache stats from database: hits={}, misses={}",
                        stats.hit_count, stats.miss_count
                    );
                    (stats.hit_count as u64, stats.miss_count as u64)
                }
                Ok(None) => {
                    info!("No existing cache stats found in database, starting fresh");
                    (0, 0)
                }
                Err(e) => {
                    warn!("Failed to restore cache stats from database: {e}, starting fresh");
                    (0, 0)
                }
            }
        } else {
            (0, 0)
        };

        Ok(Self {
            config,
            hit_count: std::sync::atomic::AtomicU64::new(initial_hit_count),
            miss_count: std::sync::atomic::AtomicU64::new(initial_miss_count),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.config.cache_enabled
    }

    // Database is now passed as parameter to methods that need it

    fn extract_version_from_filename(&self, package: &str, filename: &str) -> Option<String> {
        // Extract version from filename like "package-1.2.3.tgz"
        // For scoped packages like "@angular/animations", the filename is "animations-17.3.12.tgz"

        // First try the full package name (for non-scoped packages)
        let name_prefix = format!("{package}-");
        if let Some(version_part) = filename.strip_prefix(&name_prefix) {
            if let Some(version) = version_part.strip_suffix(".tgz") {
                return Some(version.to_string());
            }
        }

        // For scoped packages, try using just the package name part after the slash
        if package.contains('/') {
            if let Some(package_name) = package.split('/').next_back() {
                let scoped_prefix = format!("{package_name}-");
                if let Some(version_part) = filename.strip_prefix(&scoped_prefix) {
                    if let Some(version) = version_part.strip_suffix(".tgz") {
                        return Some(version.to_string());
                    }
                }
            }
        }

        None
    }

    pub fn get_cache_key(&self, package: &str, filename: &str) -> String {
        format!("{package}/{filename}")
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
        let meta_filename = format!("{filename}.meta");
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

    pub async fn get(
        &self,
        package: &str,
        filename: &str,
        database: Option<&DatabaseService>,
    ) -> Option<CacheEntry> {
        if !self.config.cache_enabled {
            return None;
        }

        let cache_key = self.get_cache_key(package, filename);

        debug!("Checking cache for key: {cache_key}");

        // First check if we have this package file in the database
        let file_path = if let Some(database) = database {
            // Check database for the package file
            if let Ok(Some((_package, _version, file))) =
                database.get_package_file(package, filename)
            {
                // Use the file path from the database
                std::path::PathBuf::from(&file.file_path)
            } else {
                // Fall back to the default cache path
                self.get_cache_path(package, filename)
            }
        } else {
            // No database, use default cache path
            self.get_cache_path(package, filename)
        };

        // Check if file exists on disk
        if !file_path.exists() {
            self.miss_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Cache miss for key: {cache_key} - file not found at {file_path:?}");
            return None;
        }

        // Read cache entry (no TTL check - packages are kept forever)
        match fs::read(&file_path) {
            Ok(data) => {
                let size = data.len() as u64;
                let created_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Try to read metadata (etag, etc.)
                let meta_path = self.get_metadata_path(package, filename);
                let etag = fs::read_to_string(&meta_path).ok();

                self.hit_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Cache hit for key: {cache_key} (size: {size} bytes)");

                // Persist hit count to database if available
                if let Some(database) = database {
                    let _ = database.increment_cache_hit_count();
                }

                // Update access info in database if available
                if let Some(database) = database {
                    if let Ok(Some((_package, _version, file))) =
                        database.get_package_file(package, filename)
                    {
                        let _ = database.update_file_access_info(file.id);
                    }
                }

                Some(CacheEntry {
                    data,
                    created_at,
                    size,
                    etag,
                })
            }
            Err(e) => {
                warn!("Failed to read cache entry {cache_key}: {e}");
                self.miss_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // Persist miss count to database if available
                if let Some(database) = database {
                    let _ = database.increment_cache_miss_count();
                }

                None
            }
        }
    }

    pub async fn get_metadata(&self, package: &str) -> Option<CacheEntry> {
        self.get_metadata_with_database(package, None).await
    }

    pub async fn get_metadata_with_database(
        &self,
        package: &str,
        database: Option<&DatabaseService>,
    ) -> Option<CacheEntry> {
        if !self.config.cache_enabled {
            return None;
        }

        let cache_key = format!("{package}.metadata");
        let cache_path = self.get_metadata_cache_path(package);

        debug!("Checking metadata cache for key: {cache_key}");

        if !cache_path.exists() {
            self.miss_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("Metadata cache miss for key: {cache_key} - file not found");

            // Persist miss count to database if available
            if let Some(database) = database {
                let _ = database.increment_cache_miss_count();
            }

            return None;
        }

        // Check if metadata is stale (TTL for upstream packages, never expire for published packages)
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default();
                let ttl_seconds = self.config.cache_ttl_hours * 3600;

                // Only apply TTL to upstream packages (check if this is a published package by looking for author_id in cached metadata)
                if age.as_secs() > ttl_seconds {
                    if let Ok(data) = fs::read_to_string(&cache_path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                            // If it doesn't have published versions (no author_id), it's upstream and should expire
                            if !self.has_published_versions(&json) {
                                debug!("Metadata cache expired for upstream package: {cache_key}");
                                self.miss_count
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                                // Persist miss count to database if available
                                if let Some(database) = database {
                                    let _ = database.increment_cache_miss_count();
                                }

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

                self.hit_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!("Metadata cache hit for key: {cache_key} (size: {size} bytes)");

                // Persist hit count and update access info in database if available
                if let Some(database) = database {
                    let _ = database.increment_cache_hit_count();
                    let _ = database.update_metadata_access_info(package);
                }

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
                warn!("Failed to read metadata cache entry {cache_key}: {e}");
                self.miss_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // Persist miss count to database if available
                if let Some(database) = database {
                    let _ = database.increment_cache_miss_count();
                }

                None
            }
        }
    }

    pub async fn put(
        &self,
        package: &str,
        filename: &str,
        data: &[u8],
        etag: Option<&str>,
        _upstream_url: &str,
        database: Option<&DatabaseService>,
    ) -> Result<(), std::io::Error> {
        if !self.config.cache_enabled {
            return Ok(());
        }

        let cache_key = self.get_cache_key(package, filename);
        let cache_path = self.get_cache_path(package, filename);
        let meta_path = self.get_metadata_path(package, filename);

        debug!(
            "Storing in cache key: {} (size: {} bytes)",
            cache_key,
            data.len()
        );

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

        // Store metadata in database if available and version is known
        if let Some(db) = database {
            if let Some(version) = self.extract_version_from_filename(package, filename) {
                let params = CompletePackageParams {
                    name: package.to_string(),
                    version,
                    filename: filename.to_string(),
                    size_bytes: data.len() as i64,
                    upstream_url: _upstream_url.to_string(),
                    file_path: cache_path.to_string_lossy().to_string(),
                    etag: etag.map(|s| s.to_string()),
                    content_type: Some("application/octet-stream".to_string()),
                    author_id: None, // cached packages don't have authors
                    description: None,
                };
                if let Err(e) = db.create_complete_package_entry(&params) {
                    warn!("Failed to store package metadata in database: {e}");
                } else {
                    debug!("Stored package metadata in database for {package}/{filename}");
                }
            } else {
                debug!(
                    "Skipping database storage for {package}/{filename} - version could not be extracted"
                );
            }
        }

        info!(
            "Cached tarball for {}/{} (size: {} bytes) - PERMANENT STORAGE",
            package,
            filename,
            data.len()
        );
        Ok(())
    }

    pub async fn put_metadata(
        &self,
        package: &str,
        metadata_json: &str,
    ) -> Result<(), std::io::Error> {
        self.put_metadata_with_etag(package, metadata_json, None)
            .await
    }

    pub async fn put_metadata_with_etag(
        &self,
        package: &str,
        metadata_json: &str,
        etag: Option<&str>,
    ) -> Result<(), std::io::Error> {
        self.put_metadata_with_etag_and_database(package, metadata_json, etag, None)
            .await
    }

    pub async fn put_metadata_with_etag_and_database(
        &self,
        package: &str,
        metadata_json: &str,
        etag: Option<&str>,
        database: Option<&DatabaseService>,
    ) -> Result<(), std::io::Error> {
        if !self.config.cache_enabled {
            return Ok(());
        }

        let cache_key = format!("{package}.metadata");
        let cache_path = self.get_metadata_cache_path(package);
        let etag_path = self.get_metadata_etag_path(package);

        debug!(
            "Storing metadata in cache key: {} (size: {} bytes)",
            cache_key,
            metadata_json.len()
        );

        // Create package directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write metadata to cache
        fs::write(&cache_path, metadata_json)?;

        // Write ETag if provided
        if let Some(etag_value) = etag {
            fs::write(&etag_path, etag_value)?;
            debug!("Stored ETag for metadata cache: {package} -> {etag_value}");
        } else if etag_path.exists() {
            // Remove old ETag file if no new ETag provided
            let _ = fs::remove_file(&etag_path);
        }

        // Store metadata information in database if available
        if let Some(db) = database {
            if let Err(e) = db.upsert_metadata_cache_entry(
                package,
                metadata_json.len() as i64,
                &cache_path.to_string_lossy(),
                etag,
            ) {
                warn!("Failed to store metadata cache info in database: {e}");
            } else {
                debug!("Stored metadata cache info in database for {package}");
            }
        }

        info!(
            "Cached metadata for {package} (size: {} bytes)",
            metadata_json.len()
        );
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
            info!("Invalidated metadata cache for package: {package}");
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

    fn collect_cache_entries(
        dir: &Path,
        entries: &mut Vec<(PathBuf, SystemTime, u64)>,
    ) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively check package directories
                Self::collect_cache_entries(&path, entries)?;
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                // Collect both .tgz files (tarballs) and .json files (metadata)
                if ext == "tgz" || ext == "json" {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(created) = metadata.created() {
                            entries.push((path, created, metadata.len()));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Re-process existing cached files and add them to the database
    /// This is useful when the version extraction logic is fixed and we need to
    /// populate the database with existing cached files
    pub async fn reprocess_cached_files(
        &self,
        database: &DatabaseService,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if !self.config.cache_enabled {
            return Ok(0);
        }

        let cache_dir = Path::new(&self.config.cache_dir);
        if !cache_dir.exists() {
            return Ok(0);
        }

        let mut processed_count = 0;
        self.reprocess_directory(cache_dir, database, &mut processed_count)?;

        info!("Re-processed {processed_count} cached files and added them to database");
        Ok(processed_count)
    }

    fn reprocess_directory(
        &self,
        dir: &Path,
        database: &DatabaseService,
        processed_count: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively process subdirectories
                self.reprocess_directory(&path, database, processed_count)?;
            } else if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename == "metadata.json" {
                    // Handle metadata.json files
                    if let Some(package_name) = self.extract_package_name_from_path(&path) {
                        // Check if this metadata is already in the database
                        if let Ok(Some(_)) = database.get_metadata_cache_entry(&package_name) {
                            debug!("Metadata already in database: {package_name}");
                            continue;
                        }

                        // Read the file to get its size
                        if let Ok(data) = fs::read(&path) {
                            // Try to read etag if it exists
                            let etag_path = self.get_metadata_etag_path(&package_name);
                            let etag = if etag_path.exists() {
                                fs::read_to_string(&etag_path).ok()
                            } else {
                                None
                            };

                            match database.upsert_metadata_cache_entry(
                                &package_name,
                                data.len() as i64,
                                &path.to_string_lossy(),
                                etag.as_deref(),
                            ) {
                                Ok(_) => {
                                    *processed_count += 1;
                                    info!(
                                        "Re-processed and added metadata to database: {package_name}"
                                    );
                                }
                                Err(e) => {
                                    warn!("Failed to add metadata {package_name} to database: {e}");
                                }
                            }
                        }
                    }
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "tgz" {
                        // Extract package name and filename from path
                        if let Some(package_name) = self.extract_package_name_from_path(&path) {
                            // Check if this file is already in the database
                            if let Ok(Some(_)) = database.get_package_file(&package_name, filename)
                            {
                                debug!("File already in database: {package_name}/{filename}");
                                continue;
                            }

                            // Try to extract version and add to database
                            if let Some(version) =
                                self.extract_version_from_filename(&package_name, filename)
                            {
                                // Read the file to get its size
                                if let Ok(data) = fs::read(&path) {
                                    let params = CompletePackageParams {
                                        name: package_name.clone(),
                                        version,
                                        filename: filename.to_string(),
                                        size_bytes: data.len() as i64,
                                        upstream_url: format!(
                                            "reprocessed://{package_name}/{filename}"
                                        ),
                                        file_path: path.to_string_lossy().to_string(),
                                        etag: None,
                                        content_type: Some("application/octet-stream".to_string()),
                                        author_id: None,
                                        description: None,
                                    };

                                    match database.create_complete_package_entry(&params) {
                                        Ok(_) => {
                                            *processed_count += 1;
                                            info!(
                                                "Re-processed and added to database: {package_name}/{filename}"
                                            );
                                        }
                                        Err(e) => {
                                            warn!("Failed to add {filename} to database: {e}");
                                        }
                                    }
                                }
                            } else {
                                debug!("Could not extract version from {package_name}/{filename}");
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_package_name_from_path(&self, path: &Path) -> Option<String> {
        // Extract package name from cache path structure
        // Expected structure: cache_dir/packages/package_name/file.tgz or cache_dir/packages/@scope/package_name/file.tgz
        let cache_dir = Path::new(&self.config.cache_dir);

        if let Ok(relative_path) = path.strip_prefix(cache_dir) {
            let components: Vec<&str> = relative_path
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();

            // Skip the "packages" directory component
            if components.len() >= 3 && components[0] == "packages" {
                // Check if it's a scoped package (starts with @)
                if components[1].starts_with('@') && components.len() >= 4 {
                    // Scoped package: packages/@scope/package_name/file.tgz
                    return Some(format!("{}/{}", components[1], components[2]));
                } else {
                    // Regular package: packages/package_name/file.tgz
                    return Some(components[1].to_string());
                }
            }
        }

        None
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
        Self::collect_cache_entries(&packages_dir, &mut entries)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_extract_version_from_filename() {
        let config = AppConfig::default();
        let cache = CacheService::new(config).unwrap();

        // Test regular packages
        assert_eq!(
            cache.extract_version_from_filename("lodash", "lodash-4.17.21.tgz"),
            Some("4.17.21".to_string())
        );

        // Test scoped packages
        assert_eq!(
            cache.extract_version_from_filename("@angular/animations", "animations-17.3.12.tgz"),
            Some("17.3.12".to_string())
        );

        assert_eq!(
            cache.extract_version_from_filename("@types/node", "node-20.5.0.tgz"),
            Some("20.5.0".to_string())
        );

        // Test cases that should fail
        assert_eq!(
            cache.extract_version_from_filename("lodash", "express-4.17.21.tgz"),
            None
        );

        assert_eq!(
            cache.extract_version_from_filename("@angular/animations", "common-17.3.12.tgz"),
            None
        );

        // Test non-tgz files
        assert_eq!(
            cache.extract_version_from_filename("lodash", "lodash-4.17.21.zip"),
            None
        );
    }

    #[test]
    fn test_extract_package_name_from_path() {
        let mut config = AppConfig::default();
        config.cache_dir = "data".to_string();
        let cache = CacheService::new(config).unwrap();

        // Test regular packages
        let path = Path::new("data/packages/lodash/lodash-4.17.21.tgz");
        assert_eq!(
            cache.extract_package_name_from_path(path),
            Some("lodash".to_string())
        );

        // Test scoped packages
        let path = Path::new("data/packages/@angular/animations/animations-17.3.12.tgz");
        assert_eq!(
            cache.extract_package_name_from_path(path),
            Some("@angular/animations".to_string())
        );

        let path = Path::new("data/packages/@types/node/node-20.5.0.tgz");
        assert_eq!(
            cache.extract_package_name_from_path(path),
            Some("@types/node".to_string())
        );

        // Test invalid paths
        let path = Path::new("invalid/path.tgz");
        assert_eq!(cache.extract_package_name_from_path(path), None);

        let path = Path::new("data/packages/file.tgz");
        assert_eq!(cache.extract_package_name_from_path(path), None);
    }
}
