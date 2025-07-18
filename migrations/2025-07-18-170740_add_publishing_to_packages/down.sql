-- Remove publishing-related additions
DROP INDEX IF EXISTS idx_package_owners_user_id;
DROP INDEX IF EXISTS idx_package_owners_package_name;
DROP INDEX IF EXISTS idx_packages_is_published;
DROP INDEX IF EXISTS idx_packages_author_id;
DROP TABLE IF EXISTS package_owners;

-- Remove columns from packages table (SQLite doesn't support DROP COLUMN directly)
-- We would need to recreate the table, but for now we'll leave the columns
