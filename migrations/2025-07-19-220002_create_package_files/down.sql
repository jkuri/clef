-- Drop indexes
DROP INDEX IF EXISTS idx_package_files_access_count;
DROP INDEX IF EXISTS idx_package_files_last_accessed;
DROP INDEX IF EXISTS idx_package_files_created_at;
DROP INDEX IF EXISTS idx_package_files_filename;
DROP INDEX IF EXISTS idx_package_files_package_version_id;

-- Drop table
DROP TABLE IF EXISTS package_files;
