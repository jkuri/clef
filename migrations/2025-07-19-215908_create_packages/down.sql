-- Drop indexes
DROP INDEX IF EXISTS idx_packages_is_private;
DROP INDEX IF EXISTS idx_packages_created_at;
DROP INDEX IF EXISTS idx_packages_author_id;
DROP INDEX IF EXISTS idx_packages_name;

-- Drop table
DROP TABLE IF EXISTS packages;
