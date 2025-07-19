-- Drop indexes
DROP INDEX IF EXISTS idx_package_owners_user_id;
DROP INDEX IF EXISTS idx_package_owners_package_name;

-- Drop table
DROP TABLE IF EXISTS package_owners;
