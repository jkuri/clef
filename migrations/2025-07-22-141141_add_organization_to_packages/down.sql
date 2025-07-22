-- Drop index
DROP INDEX IF EXISTS idx_packages_organization_id;

-- Remove organization_id column from packages table
ALTER TABLE packages DROP COLUMN organization_id;
