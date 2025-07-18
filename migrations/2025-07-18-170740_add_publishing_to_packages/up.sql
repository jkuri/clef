-- Add publishing-related columns to packages table
ALTER TABLE packages ADD COLUMN author_id INTEGER;
ALTER TABLE packages ADD COLUMN description TEXT;
ALTER TABLE packages ADD COLUMN package_json TEXT; -- Store the full package.json as JSON
ALTER TABLE packages ADD COLUMN is_published BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE packages ADD COLUMN is_private BOOLEAN NOT NULL DEFAULT FALSE;

-- Create package_owners table for ownership management
CREATE TABLE IF NOT EXISTS package_owners (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    package_name TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    permission_level TEXT NOT NULL DEFAULT 'write', -- 'read', 'write', 'admin'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(package_name, user_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_packages_author_id ON packages(author_id);
CREATE INDEX IF NOT EXISTS idx_packages_is_published ON packages(is_published);
CREATE INDEX IF NOT EXISTS idx_package_owners_package_name ON package_owners(package_name);
CREATE INDEX IF NOT EXISTS idx_package_owners_user_id ON package_owners(user_id);
