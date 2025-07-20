use crate::database::connection::{DbPool, get_connection_with_retry};
use crate::models::metadata_cache::{
    MetadataCacheRecord, MetadataCacheStats, NewMetadataCacheRecord, UpdateMetadataCacheRecord,
};
use crate::schema::metadata_cache;
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use log::{debug, warn};

#[derive(QueryableByName)]
struct SumResult {
    #[diesel(sql_type = BigInt)]
    total: i64,
}

pub struct MetadataCacheOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> MetadataCacheOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Get metadata cache entry by package name
    pub fn get_metadata_cache_entry(
        &self,
        package_name: &str,
    ) -> Result<Option<MetadataCacheRecord>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        metadata_cache::table
            .filter(metadata_cache::package_name.eq(package_name))
            .first::<MetadataCacheRecord>(&mut conn)
            .optional()
    }

    /// Create or update metadata cache entry
    pub fn upsert_metadata_cache_entry(
        &self,
        package_name: &str,
        size_bytes: i64,
        file_path: &str,
        etag: Option<&str>,
    ) -> Result<MetadataCacheRecord, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now().naive_utc();

        // Try to update existing record first
        let update_result = diesel::update(
            metadata_cache::table.filter(metadata_cache::package_name.eq(package_name)),
        )
        .set(UpdateMetadataCacheRecord {
            size_bytes: Some(size_bytes),
            file_path: Some(file_path.to_string()),
            etag: etag.map(|s| s.to_string()),
            updated_at: Some(now),
            last_accessed: Some(now),
            access_count: None, // Don't reset access count on update
        })
        .get_result::<MetadataCacheRecord>(&mut conn);

        match update_result {
            Ok(record) => {
                debug!("Updated metadata cache entry for package: {package_name}");
                Ok(record)
            }
            Err(diesel::result::Error::NotFound) => {
                // No record exists, create one
                let new_record = NewMetadataCacheRecord {
                    package_name: package_name.to_string(),
                    size_bytes,
                    file_path: file_path.to_string(),
                    etag: etag.map(|s| s.to_string()),
                    created_at: now,
                    updated_at: now,
                    last_accessed: now,
                    access_count: 0,
                };

                let result = diesel::insert_into(metadata_cache::table)
                    .values(&new_record)
                    .get_result::<MetadataCacheRecord>(&mut conn);

                match result {
                    Ok(record) => {
                        debug!("Created metadata cache entry for package: {package_name}");
                        Ok(record)
                    }
                    Err(e) => {
                        warn!("Failed to create metadata cache entry for {package_name}: {e}");
                        Err(e)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to update metadata cache entry for {package_name}: {e}");
                Err(e)
            }
        }
    }

    /// Update access info for metadata cache entry
    pub fn update_metadata_access_info(
        &self,
        package_name: &str,
    ) -> Result<(), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now().naive_utc();

        diesel::update(metadata_cache::table.filter(metadata_cache::package_name.eq(package_name)))
            .set((
                metadata_cache::last_accessed.eq(now),
                metadata_cache::access_count.eq(metadata_cache::access_count + 1),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Get metadata cache statistics
    pub fn get_metadata_cache_stats(&self) -> Result<MetadataCacheStats, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        // Get count
        let total_entries: i64 = metadata_cache::table
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        // Get sum of sizes - use a raw query to handle the sum properly
        let total_size_bytes: i64 =
            diesel::sql_query("SELECT COALESCE(SUM(size_bytes), 0) as total FROM metadata_cache")
                .get_result::<SumResult>(&mut conn)
                .map(|result| result.total)
                .unwrap_or(0);

        Ok(MetadataCacheStats {
            total_entries,
            total_size_bytes,
            total_size_mb: total_size_bytes as f64 / 1024.0 / 1024.0,
        })
    }

    /// Delete metadata cache entry
    pub fn delete_metadata_cache_entry(
        &self,
        package_name: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(metadata_cache::table.filter(metadata_cache::package_name.eq(package_name)))
            .execute(&mut conn)
    }

    /// Clear all metadata cache entries
    pub fn clear_metadata_cache(&self) -> Result<usize, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(metadata_cache::table).execute(&mut conn)
    }
}
