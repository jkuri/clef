-- Drop indexes
DROP INDEX IF EXISTS idx_organization_members_role;
DROP INDEX IF EXISTS idx_organization_members_organization_id;
DROP INDEX IF EXISTS idx_organization_members_user_id;
DROP INDEX IF EXISTS idx_organizations_name;

-- Drop tables (order matters due to foreign keys)
DROP TABLE IF EXISTS organization_members;
DROP TABLE IF EXISTS organizations;
