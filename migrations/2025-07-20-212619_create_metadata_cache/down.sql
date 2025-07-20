-- Drop metadata_cache table
DROP INDEX IF EXISTS idx_metadata_cache_last_accessed;
DROP INDEX IF EXISTS idx_metadata_cache_package_name;
DROP TABLE IF EXISTS metadata_cache;
