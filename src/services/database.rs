use crate::models::package::*;
use crate::schema::{package_files, package_versions, packages};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::{info, warn};
use std::path::Path;
use std::time::Duration;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// SQLite connection customizer to enable WAL mode and set pragmas for better concurrency
#[derive(Debug)]
pub struct SqliteConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::sql_query;

        // Set busy timeout first (before WAL mode) - this one is critical
        sql_query("PRAGMA busy_timeout = 60000") // 60 seconds
            .execute(conn)
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;

        // Enable WAL mode for better concurrency - critical for avoiding locks
        // Retry WAL mode setup since it's important for concurrency
        let mut wal_attempts = 0;
        let max_wal_attempts = 3;
        loop {
            match sql_query("PRAGMA journal_mode = WAL").execute(conn) {
                Ok(_) => break,
                Err(e) => {
                    wal_attempts += 1;
                    if wal_attempts >= max_wal_attempts {
                        warn!(
                            "Failed to enable WAL mode after {} attempts: {}",
                            max_wal_attempts, e
                        );
                        break;
                    }
                    // Short delay before retry
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        // Enable foreign key constraints - important but not critical
        if let Err(e) = sql_query("PRAGMA foreign_keys = ON").execute(conn) {
            warn!("Failed to enable foreign keys: {}", e);
        }

        // Optimize for concurrent access - use NORMAL instead of FULL for better performance
        if let Err(e) = sql_query("PRAGMA synchronous = NORMAL").execute(conn) {
            warn!("Failed to set synchronous mode: {}", e);
        }

        // Set cache size (negative value means KB) - performance optimization
        if let Err(e) = sql_query("PRAGMA cache_size = -32000").execute(conn) {
            warn!("Failed to set cache size: {}", e);
        }

        // Set WAL autocheckpoint for better performance - performance optimization
        if let Err(e) = sql_query("PRAGMA wal_autocheckpoint = 1000").execute(conn) {
            warn!("Failed to set WAL autocheckpoint: {}", e);
        }

        // Set temp store to memory for better performance - performance optimization
        if let Err(e) = sql_query("PRAGMA temp_store = MEMORY").execute(conn) {
            warn!("Failed to set temp store: {}", e);
        }

        // Set mmap size for better I/O performance - performance optimization
        if let Err(e) = sql_query("PRAGMA mmap_size = 268435456").execute(conn) {
            warn!("Failed to set mmap size: {}", e);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DatabaseService {
    pub pool: DbPool,
}

impl DatabaseService {
    pub fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure the database directory exists
        if let Some(parent) = Path::new(database_url).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(20) // Increase pool size for better concurrency
            .min_idle(Some(2)) // Keep some connections ready
            .connection_timeout(Duration::from_secs(60)) // Increase timeout
            .idle_timeout(Some(Duration::from_secs(300))) // 5 minutes idle timeout
            .max_lifetime(Some(Duration::from_secs(1800))) // 30 minutes max lifetime
            .connection_customizer(Box::new(SqliteConnectionCustomizer))
            .build(manager)?;

        // Run migrations
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        info!("Database initialized successfully with WAL mode and optimized settings");

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> Result<DbConnection, diesel::r2d2::Error> {
        // Retry connection acquisition with exponential backoff
        let mut attempts = 0;
        let max_attempts = 5;

        loop {
            match self.pool.get() {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(diesel::r2d2::Error::ConnectionError(
                            diesel::ConnectionError::BadConnection(format!(
                                "Failed to get connection after {} attempts: {}",
                                max_attempts, e
                            )),
                        ));
                    }

                    // Exponential backoff: 10ms, 20ms, 40ms, 80ms
                    let delay = Duration::from_millis(10 * (1 << (attempts - 1)));
                    std::thread::sleep(delay);
                }
            }
        }
    }

    // Package operations
    pub fn create_or_get_package(
        &self,
        name: &str,
        description: Option<String>,
        author_id: Option<i32>,
    ) -> Result<Package, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to get existing package first
        if let Some(existing_package) = packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
            .optional()?
        {
            return Ok(existing_package);
        }

        // Create new package
        let new_package = NewPackage::new(name.to_string(), description, author_id);

        diesel::insert_into(packages::table)
            .values(&new_package)
            .execute(&mut conn)?;

        packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
    }

    pub fn get_package_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Package>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
            .optional()
    }

    // Package version operations
    pub fn create_or_get_package_version(
        &self,
        package_id: i32,
        version: &str,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to get existing version first
        if let Some(existing_version) = package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
            .optional()?
        {
            return Ok(existing_version);
        }

        // Create new version
        let new_version = NewPackageVersion::new(package_id, version.to_string());

        diesel::insert_into(package_versions::table)
            .values(&new_version)
            .execute(&mut conn)?;

        package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
    }

    pub fn create_or_get_package_version_with_metadata(
        &self,
        package_id: i32,
        version: &str,
        package_json: &serde_json::Value,
    ) -> Result<PackageVersion, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to get existing version first
        if let Some(existing_version) = package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
            .optional()?
        {
            // If version exists but has no metadata, update it with metadata
            if existing_version.description.is_none()
                && existing_version.scripts.is_none()
                && existing_version.dependencies.is_none()
                && existing_version.dev_dependencies.is_none()
            {
                // Continue to extract and update metadata
            } else {
                return Ok(existing_version);
            }
        }

        // Extract metadata from package.json
        let description = package_json["description"].as_str().map(|s| s.to_string());
        let main_file = package_json["main"].as_str().map(|s| s.to_string());

        // Serialize complex fields to JSON strings
        let scripts = package_json["scripts"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let dependencies = package_json["dependencies"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let dev_dependencies = package_json["devDependencies"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let peer_dependencies = package_json["peerDependencies"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let engines = package_json["engines"]
            .as_object()
            .map(|obj| serde_json::to_string(obj).unwrap_or_default());

        let shasum = package_json
            .get("dist")
            .and_then(|dist| dist.get("shasum"))
            .and_then(|shasum| shasum.as_str())
            .map(|s| s.to_string());

        // Create new version with metadata
        let new_version = NewPackageVersion::with_metadata(
            package_id,
            version.to_string(),
            description,
            main_file,
            scripts,
            dependencies,
            dev_dependencies,
            peer_dependencies,
            engines,
            shasum,
        );

        // Check if we need to update existing version or insert new one
        let existing_count: i64 = package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .count()
            .get_result(&mut conn)?;

        if existing_count > 0 {
            // Update existing version
            diesel::update(
                package_versions::table
                    .filter(package_versions::package_id.eq(package_id))
                    .filter(package_versions::version.eq(version)),
            )
            .set((
                package_versions::description.eq(&new_version.description),
                package_versions::main_file.eq(&new_version.main_file),
                package_versions::scripts.eq(&new_version.scripts),
                package_versions::dependencies.eq(&new_version.dependencies),
                package_versions::dev_dependencies.eq(&new_version.dev_dependencies),
                package_versions::peer_dependencies.eq(&new_version.peer_dependencies),
                package_versions::engines.eq(&new_version.engines),
                package_versions::shasum.eq(&new_version.shasum),
                package_versions::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)?;
        } else {
            // Insert new version
            diesel::insert_into(package_versions::table)
                .values(&new_version)
                .execute(&mut conn)?;
        }

        package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .filter(package_versions::version.eq(version))
            .first::<PackageVersion>(&mut conn)
    }

    pub fn get_package_versions(
        &self,
        package_id: i32,
    ) -> Result<Vec<PackageVersion>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        package_versions::table
            .filter(package_versions::package_id.eq(package_id))
            .order(package_versions::created_at.desc())
            .load::<PackageVersion>(&mut conn)
    }

    // Package file operations
    pub fn create_or_update_package_file(
        &self,
        package_version_id: i32,
        filename: &str,
        size_bytes: i64,
        upstream_url: &str,
        file_path: &str,
        etag: Option<String>,
        content_type: Option<String>,
    ) -> Result<PackageFile, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Try to get existing file first
        if let Some(existing_file) = package_files::table
            .filter(package_files::package_version_id.eq(package_version_id))
            .filter(package_files::filename.eq(filename))
            .first::<PackageFile>(&mut conn)
            .optional()?
        {
            // Update access info
            self.update_file_access_info(existing_file.id)?;
            return Ok(existing_file);
        }

        // Create new file
        let mut new_file = NewPackageFile::new(
            package_version_id,
            filename.to_string(),
            size_bytes,
            upstream_url.to_string(),
            file_path.to_string(),
        );
        new_file.etag = etag;
        new_file.content_type = content_type;

        diesel::insert_into(package_files::table)
            .values(&new_file)
            .execute(&mut conn)?;

        package_files::table
            .filter(package_files::package_version_id.eq(package_version_id))
            .filter(package_files::filename.eq(filename))
            .first::<PackageFile>(&mut conn)
    }

    pub fn get_package_file(
        &self,
        package_name: &str,
        filename: &str,
    ) -> Result<Option<(Package, PackageVersion, PackageFile)>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .inner_join(package_versions::table.inner_join(package_files::table))
            .filter(packages::name.eq(package_name))
            .filter(package_files::filename.eq(filename))
            .first::<(Package, (PackageVersion, PackageFile))>(&mut conn)
            .optional()
            .map(|opt| opt.map(|(pkg, (ver, file))| (pkg, ver, file)))
    }

    pub fn update_file_access_info(&self, file_id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let update = UpdatePackageFile {
            last_accessed: Some(Utc::now().naive_utc()),
            access_count: None, // We'll increment this in SQL
            etag: None,
        };

        diesel::update(package_files::table.find(file_id))
            .set((
                &update,
                package_files::access_count.eq(package_files::access_count + 1),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    // Helper method to create a complete package entry (package + version + file)
    pub fn create_complete_package_entry(
        &self,
        name: &str,
        version: &str,
        filename: &str,
        size_bytes: i64,
        upstream_url: &str,
        file_path: &str,
        etag: Option<String>,
        content_type: Option<String>,
        author_id: Option<i32>,
        description: Option<String>,
    ) -> Result<(Package, PackageVersion, PackageFile), diesel::result::Error> {
        // Create or get package
        let package = self.create_or_get_package(name, description, author_id)?;

        // Create or get version
        let package_version = self.create_or_get_package_version(package.id, version)?;

        // Create or update file
        let package_file = self.create_or_update_package_file(
            package_version.id,
            filename,
            size_bytes,
            upstream_url,
            file_path,
            etag,
            content_type,
        )?;

        Ok((package, package_version, package_file))
    }

    // Analytics and API methods
    pub fn get_package_with_versions(
        &self,
        name: &str,
    ) -> Result<Option<PackageWithVersions>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get the package
        let package = match packages::table
            .filter(packages::name.eq(name))
            .first::<Package>(&mut conn)
            .optional()?
        {
            Some(pkg) => pkg,
            None => return Ok(None),
        };

        // Get all versions with their files (use LEFT JOIN to include versions without files)
        let version_files: Vec<(PackageVersion, Option<PackageFile>)> = package_versions::table
            .left_join(package_files::table)
            .filter(package_versions::package_id.eq(package.id))
            .order(package_versions::created_at.desc())
            .load::<(PackageVersion, Option<PackageFile>)>(&mut conn)?;

        // Group files by version
        let mut versions_map: std::collections::HashMap<i32, (PackageVersion, Vec<PackageFile>)> =
            std::collections::HashMap::new();

        for (version, file_opt) in version_files {
            let entry = versions_map
                .entry(version.id)
                .or_insert((version.clone(), Vec::new()));

            // Only add file if it exists (LEFT JOIN can return None)
            if let Some(file) = file_opt {
                entry.1.push(file);
            }
        }

        let versions: Vec<PackageVersionWithFiles> = versions_map
            .into_values()
            .map(|(version, files)| PackageVersionWithFiles { version, files })
            .collect();

        Ok(Some(PackageWithVersions { package, versions }))
    }

    pub fn get_popular_packages(
        &self,
        limit: i64,
    ) -> Result<Vec<PopularPackage>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let results: Vec<(Package, PackageVersion, PackageFile)> = packages::table
            .inner_join(package_versions::table.inner_join(package_files::table))
            .order(package_files::access_count.desc())
            .load::<(Package, (PackageVersion, PackageFile))>(&mut conn)?
            .into_iter()
            .map(|(pkg, (ver, file))| (pkg, ver, file))
            .collect();

        let mut package_stats: std::collections::HashMap<String, (i64, i64, i64)> =
            std::collections::HashMap::new();

        for (pkg, _ver, file) in results {
            let entry = package_stats.entry(pkg.name).or_insert((0, 0, 0));
            entry.0 += file.access_count as i64; // total downloads
            entry.1 += 1; // unique versions
            entry.2 += file.size_bytes; // total size
        }

        let mut popular_packages: Vec<PopularPackage> = package_stats
            .into_iter()
            .map(
                |(name, (total_downloads, unique_versions, total_size_bytes))| PopularPackage {
                    name,
                    total_downloads,
                    unique_versions,
                    total_size_bytes,
                },
            )
            .collect();

        popular_packages.sort_by(|a, b| b.total_downloads.cmp(&a.total_downloads));
        popular_packages.truncate(limit as usize);

        Ok(popular_packages)
    }

    pub fn get_all_packages_with_versions(
        &self,
    ) -> Result<Vec<PackageWithVersions>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get all packages
        let all_packages = packages::table.load::<Package>(&mut conn)?;

        let mut result = Vec::new();

        for package in all_packages {
            // Get versions and files for this package
            let version_files: Vec<(PackageVersion, PackageFile)> = package_versions::table
                .inner_join(package_files::table)
                .filter(package_versions::package_id.eq(package.id))
                .order(package_versions::created_at.desc())
                .load::<(PackageVersion, PackageFile)>(&mut conn)?;

            // Group files by version
            let mut versions_map: std::collections::HashMap<
                i32,
                (PackageVersion, Vec<PackageFile>),
            > = std::collections::HashMap::new();

            for (version, file) in version_files {
                let entry = versions_map
                    .entry(version.id)
                    .or_insert((version.clone(), Vec::new()));
                entry.1.push(file);
            }

            let versions: Vec<PackageVersionWithFiles> = versions_map
                .into_values()
                .map(|(version, files)| PackageVersionWithFiles { version, files })
                .collect();

            result.push(PackageWithVersions { package, versions });
        }

        Ok(result)
    }

    pub fn get_recent_packages(
        &self,
        limit: i64,
    ) -> Result<Vec<PackageWithVersions>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get recent packages by their creation date
        let recent_packages = packages::table
            .order(packages::created_at.desc())
            .limit(limit)
            .load::<Package>(&mut conn)?;

        let mut result = Vec::new();

        for package in recent_packages {
            // Get versions and files for this package
            let version_files: Vec<(PackageVersion, PackageFile)> = package_versions::table
                .inner_join(package_files::table)
                .filter(package_versions::package_id.eq(package.id))
                .order(package_versions::created_at.desc())
                .load::<(PackageVersion, PackageFile)>(&mut conn)?;

            // Group files by version
            let mut versions_map: std::collections::HashMap<
                i32,
                (PackageVersion, Vec<PackageFile>),
            > = std::collections::HashMap::new();

            for (version, file) in version_files {
                let entry = versions_map
                    .entry(version.id)
                    .or_insert((version.clone(), Vec::new()));
                entry.1.push(file);
            }

            let versions: Vec<PackageVersionWithFiles> = versions_map
                .into_values()
                .map(|(version, files)| PackageVersionWithFiles { version, files })
                .collect();

            result.push(PackageWithVersions { package, versions });
        }

        Ok(result)
    }

    pub fn get_cache_stats(&self) -> Result<(usize, i64), diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total_packages: i64 = packages::table.count().get_result(&mut conn)?;

        // Get total size by loading all files and summing in Rust to avoid SQL type issues
        let all_files: Vec<PackageFile> = package_files::table.load(&mut conn)?;
        let total_size_bytes: i64 = all_files.iter().map(|f| f.size_bytes).sum();

        Ok((total_packages as usize, total_size_bytes))
    }
}
