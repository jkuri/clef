use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::info;
use std::path::Path;

use crate::models::{NewPackage, Package, PackageVersions, PopularPackage, UpdatePackage};
use crate::schema::packages;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

#[derive(Debug)]
pub struct DatabaseService {
    pool: DbPool,
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

        info!("Database initialized at: {}", database_url);

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error>> {
        self.pool
            .get()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    pub fn insert_package(
        &self,
        new_package: NewPackage,
    ) -> Result<Package, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Check if package already exists
        if let Some(existing_package) =
            self.get_package(&new_package.name, &new_package.filename)?
        {
            // Package already exists, update access info and return it
            self.update_access_info(existing_package.id)?;
            return Ok(existing_package);
        }

        // Insert the package (only if it doesn't exist)
        diesel::insert_into(packages::table)
            .values(&new_package)
            .execute(&mut conn)?;

        // Get the inserted package by name and filename
        packages::table
            .filter(packages::name.eq(&new_package.name))
            .filter(packages::filename.eq(&new_package.filename))
            .first::<Package>(&mut conn)
    }

    /// Insert or update package using SQLite's INSERT OR REPLACE
    pub fn upsert_package(
        &self,
        new_package: NewPackage,
    ) -> Result<Package, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Use raw SQL for INSERT OR REPLACE to handle unique constraint gracefully
        diesel::sql_query(
            "INSERT OR REPLACE INTO packages (
                name, version, filename, size_bytes, etag, content_type,
                upstream_url, file_path, created_at, last_accessed, access_count,
                author_id, description, package_json, is_private
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind::<diesel::sql_types::Text, _>(&new_package.name)
        .bind::<diesel::sql_types::Text, _>(&new_package.version)
        .bind::<diesel::sql_types::Text, _>(&new_package.filename)
        .bind::<diesel::sql_types::BigInt, _>(&new_package.size_bytes)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_package.etag)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_package.content_type)
        .bind::<diesel::sql_types::Text, _>(&new_package.upstream_url)
        .bind::<diesel::sql_types::Text, _>(&new_package.file_path)
        .bind::<diesel::sql_types::Timestamp, _>(&new_package.created_at)
        .bind::<diesel::sql_types::Timestamp, _>(&new_package.last_accessed)
        .bind::<diesel::sql_types::Integer, _>(&new_package.access_count)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(&new_package.author_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_package.description)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_package.package_json)
        .bind::<diesel::sql_types::Bool, _>(&new_package.is_private)
        .execute(&mut conn)?;

        // Get the package by name and filename
        packages::table
            .filter(packages::name.eq(&new_package.name))
            .filter(packages::filename.eq(&new_package.filename))
            .first::<Package>(&mut conn)
    }

    pub fn get_package(
        &self,
        name: &str,
        filename: &str,
    ) -> Result<Option<Package>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .filter(packages::name.eq(name))
            .filter(packages::filename.eq(filename))
            .first::<Package>(&mut conn)
            .optional()
    }

    pub fn update_access_info(&self, package_id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let update = UpdatePackage {
            last_accessed: Some(Utc::now().naive_utc()),
            access_count: None, // We'll increment this in SQL
        };

        diesel::update(packages::table.find(package_id))
            .set((
                &update,
                packages::access_count.eq(packages::access_count + 1),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_package_versions(
        &self,
        name: &str,
    ) -> Result<PackageVersions, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let versions = packages::table
            .filter(packages::name.eq(name))
            .order(packages::created_at.desc())
            .load::<Package>(&mut conn)?;

        let total_size_bytes = versions.iter().map(|v| v.size_bytes).sum();

        Ok(PackageVersions {
            package_name: name.to_string(),
            versions,
            total_size_bytes,
        })
    }

    pub fn list_all_packages(&self) -> Result<Vec<Package>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .order(packages::created_at.desc())
            .load::<Package>(&mut conn)
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

        // For now, let's use a simpler approach - get all packages and group in Rust
        let all_packages = packages::table
            .order(packages::access_count.desc())
            .load::<Package>(&mut conn)?;

        let mut package_stats: std::collections::HashMap<String, (i64, i64, i64)> =
            std::collections::HashMap::new();

        for package in all_packages {
            let entry = package_stats
                .entry(package.name.clone())
                .or_insert((0, 0, 0));
            entry.0 += package.access_count as i64; // total_downloads
            entry.1 += 1; // unique_versions
            entry.2 += package.size_bytes; // total_size_bytes
        }

        let mut results: Vec<_> = package_stats
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

        results.sort_by(|a, b| b.total_downloads.cmp(&a.total_downloads));
        results.truncate(limit as usize);

        Ok(results)
    }

    pub fn get_recent_packages(&self, limit: i64) -> Result<Vec<Package>, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        packages::table
            .order(packages::created_at.desc())
            .limit(limit)
            .load::<Package>(&mut conn)
    }

    pub fn get_cache_stats(&self) -> Result<(i64, i64), diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let packages = packages::table.load::<Package>(&mut conn)?;
        let count = packages.len() as i64;
        let total_size = packages.iter().map(|p| p.size_bytes).sum::<i64>();

        Ok((count, total_size))
    }

    pub fn delete_package(
        &self,
        name: &str,
        filename: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(
            packages::table
                .filter(packages::name.eq(name))
                .filter(packages::filename.eq(filename)),
        )
        .execute(&mut conn)
    }

    pub fn clear_all(&self) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_connection().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(packages::table).execute(&mut conn)
    }
}
