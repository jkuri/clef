-- Add organization_id column to packages table
ALTER TABLE packages ADD COLUMN organization_id INTEGER;

-- Add foreign key constraint
-- Note: SQLite doesn't support adding foreign key constraints to existing tables,
-- so we'll handle this constraint in the application layer for now

-- Create index for performance
CREATE INDEX idx_packages_organization_id ON packages(organization_id);
