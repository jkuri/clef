use super::connection::{DbPool, get_connection_with_retry};
use crate::models::package::*;
use crate::schema::{package_files, package_versions, packages};
use diesel::prelude::*;
use log::{debug, info};

/// Analytics and statistics-related database operations
pub struct AnalyticsOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> AnalyticsOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Gets popular packages based on download counts
    pub fn get_popular_packages(
        &self,
        limit: i64,
    ) -> Result<Vec<PopularPackage>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
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

        debug!(
            "Found {} package files for popular packages calculation",
            results.len()
        );

        let mut package_stats: std::collections::HashMap<String, (i64, i64, i64)> =
            std::collections::HashMap::new();

        for (pkg, _ver, file) in results {
            debug!(
                "Processing package: {} with access_count: {}",
                pkg.name, file.access_count
            );
            let entry = package_stats.entry(pkg.name).or_insert((0, 0, 0));
            entry.0 += file.access_count as i64; // total downloads
            entry.1 += 1; // unique versions
            entry.2 += file.size_bytes; // total size
        }

        info!(
            "Aggregated stats for {} unique packages",
            package_stats.len()
        );

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

        info!(
            "Returning {} popular packages (limit: {})",
            popular_packages.len(),
            limit
        );
        for (i, pkg) in popular_packages.iter().enumerate() {
            debug!(
                "Popular package #{}: {} (downloads: {}, versions: {}, size: {} bytes)",
                i + 1,
                pkg.name,
                pkg.total_downloads,
                pkg.unique_versions,
                pkg.total_size_bytes
            );
        }

        Ok(popular_packages)
    }

    /// Gets cache statistics (total packages and total size)
    pub fn get_cache_stats(&self) -> Result<(usize, i64), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
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
