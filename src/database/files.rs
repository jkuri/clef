use crate::models::package::*;
use crate::schema::{package_files, packages, package_versions};
use chrono::Utc;
use diesel::prelude::*;
use super::connection::{DbPool, get_connection_with_retry};

/// Package file-related database operations
pub struct FileOperations<'a> {
    pool: &'a DbPool,
}

impl<'a> FileOperations<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Creates a new package file or updates existing one
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
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
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

    /// Gets a package file by package name and filename
    pub fn get_package_file(
        &self,
        package_name: &str,
        filename: &str,
    ) -> Result<Option<(Package, PackageVersion, PackageFile)>, diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
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

    /// Updates file access information (last accessed time and access count)
    pub fn update_file_access_info(&self, file_id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = get_connection_with_retry(self.pool).map_err(|e| {
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

    /// Helper method to create a complete package entry (package + version + file)
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
        use super::packages::PackageOperations;
        use super::versions::VersionOperations;

        let package_ops = PackageOperations::new(self.pool);
        let version_ops = VersionOperations::new(self.pool);

        // Create or get package
        let package = package_ops.create_or_get_package(name, description, author_id)?;

        // Create or get version
        let package_version = version_ops.create_or_get_package_version(package.id, version)?;

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
}
