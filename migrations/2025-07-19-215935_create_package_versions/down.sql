-- Drop indexes
DROP INDEX IF EXISTS idx_package_versions_package_version;
DROP INDEX IF EXISTS idx_package_versions_created_at;
DROP INDEX IF EXISTS idx_package_versions_version;
DROP INDEX IF EXISTS idx_package_versions_package_id;

-- Drop table
DROP TABLE IF EXISTS package_versions;
