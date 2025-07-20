use super::connection::{DbPool, get_connection_with_retry};
use crate::models::cache::{CacheStatsRecord, NewCacheStatsRecord, UpdateCacheStatsRecord};
use crate::schema::cache_stats;
use chrono::Utc;
use diesel::prelude::*;

/// Cache statistics database operations
pub struct CacheStatsOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> CacheStatsOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Gets the current cache stats record (there should only be one)
    pub fn get_cache_stats(&self) -> Result<Option<CacheStatsRecord>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        cache_stats::table
            .order(cache_stats::id.desc())
            .first::<CacheStatsRecord>(&mut conn)
            .optional()
    }

    /// Updates the cache stats record
    pub fn update_cache_stats(
        &self,
        hit_count: u64,
        miss_count: u64,
    ) -> Result<CacheStatsRecord, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now().naive_utc();

        // Try to update existing record first
        let update_result = diesel::update(cache_stats::table)
            .set(UpdateCacheStatsRecord {
                hit_count: Some(hit_count as i64),
                miss_count: Some(miss_count as i64),
                updated_at: Some(now),
            })
            .get_result::<CacheStatsRecord>(&mut conn);

        match update_result {
            Ok(record) => Ok(record),
            Err(diesel::result::Error::NotFound) => {
                // No record exists, create one
                let new_record = NewCacheStatsRecord {
                    hit_count: hit_count as i64,
                    miss_count: miss_count as i64,
                    created_at: now,
                    updated_at: now,
                };

                diesel::insert_into(cache_stats::table)
                    .values(&new_record)
                    .get_result::<CacheStatsRecord>(&mut conn)
            }
            Err(e) => Err(e),
        }
    }

    /// Increments hit count
    pub fn increment_hit_count(&self) -> Result<(), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now().naive_utc();

        // Try to increment existing record
        let update_result = diesel::update(cache_stats::table)
            .set((
                cache_stats::hit_count.eq(cache_stats::hit_count + 1),
                cache_stats::updated_at.eq(now),
            ))
            .execute(&mut conn);

        match update_result {
            Ok(0) => {
                // No record exists, create one with hit_count = 1
                let new_record = NewCacheStatsRecord {
                    hit_count: 1,
                    miss_count: 0,
                    created_at: now,
                    updated_at: now,
                };

                diesel::insert_into(cache_stats::table)
                    .values(&new_record)
                    .execute(&mut conn)?;
            }
            Ok(_) => {} // Successfully updated
            Err(e) => return Err(e),
        }

        Ok(())
    }

    /// Increments miss count
    pub fn increment_miss_count(&self) -> Result<(), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now().naive_utc();

        // Try to increment existing record
        let update_result = diesel::update(cache_stats::table)
            .set((
                cache_stats::miss_count.eq(cache_stats::miss_count + 1),
                cache_stats::updated_at.eq(now),
            ))
            .execute(&mut conn);

        match update_result {
            Ok(0) => {
                // No record exists, create one with miss_count = 1
                let new_record = NewCacheStatsRecord {
                    hit_count: 0,
                    miss_count: 1,
                    created_at: now,
                    updated_at: now,
                };

                diesel::insert_into(cache_stats::table)
                    .values(&new_record)
                    .execute(&mut conn)?;
            }
            Ok(_) => {} // Successfully updated
            Err(e) => return Err(e),
        }

        Ok(())
    }
}
