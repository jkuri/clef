use crate::models::package::*;
use crate::schema::{package_files, package_versions, packages};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::info;
use std::path::Path;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

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
        let pool = Pool::builder().max_size(10).build(manager)?;

        // Run migrations
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        info!("Database initialized successfully");

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> Result<DbConnection, diesel::r2d2::Error> {
        self.pool.get().map_err(|e| {
            diesel::r2d2::Error::ConnectionError(diesel::ConnectionError::BadConnection(
                e.to_string(),
            ))
        })
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
        package_json: Option<String>,
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
        let new_version = NewPackageVersion::new(package_id, version.to_string(), package_json);

        diesel::insert_into(package_versions::table)
            .values(&new_version)
            .execute(&mut conn)?;

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
        package_json: Option<String>,
        author_id: Option<i32>,
        description: Option<String>,
    ) -> Result<(Package, PackageVersion, PackageFile), diesel::result::Error> {
        // Create or get package
        let package = self.create_or_get_package(name, description, author_id)?;

        // Create or get version
        let package_version =
            self.create_or_get_package_version(package.id, version, package_json)?;

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
