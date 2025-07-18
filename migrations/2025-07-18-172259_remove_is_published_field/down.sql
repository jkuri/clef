-- Restore the is_published field
-- Create packages table with is_published field
CREATE TABLE packages_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    filename TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    etag TEXT,
    content_type TEXT,
    upstream_url TEXT NOT NULL,
    file_path TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 1,
    author_id INTEGER,
    description TEXT,
    package_json TEXT,
    is_published BOOLEAN NOT NULL DEFAULT FALSE,
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(name, filename)
);

-- Copy data back (setting is_published based on whether author_id exists)
INSERT INTO packages_new (id, name, version, filename, size_bytes, etag, content_type, upstream_url, file_path, created_at, last_accessed, access_count, author_id, description, package_json, is_published, is_private)
SELECT id, name, version, filename, size_bytes, etag, content_type, upstream_url, file_path, created_at, last_accessed, access_count, author_id, description, package_json,
       CASE WHEN author_id IS NOT NULL THEN TRUE ELSE FALSE END as is_published,
       is_private
FROM packages;

-- Drop old table and indexes
DROP INDEX IF EXISTS idx_packages_author_id;
DROP INDEX IF EXISTS idx_packages_access_count;
DROP INDEX IF EXISTS idx_packages_last_accessed;
DROP INDEX IF EXISTS idx_packages_created_at;
DROP INDEX IF EXISTS idx_packages_name;
DROP TABLE packages;

-- Rename new table
ALTER TABLE packages_new RENAME TO packages;

-- Recreate indexes including is_published
CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_packages_created_at ON packages(created_at);
CREATE INDEX idx_packages_last_accessed ON packages(last_accessed);
CREATE INDEX idx_packages_access_count ON packages(access_count);
CREATE INDEX idx_packages_author_id ON packages(author_id);
CREATE INDEX idx_packages_is_published ON packages(is_published);
