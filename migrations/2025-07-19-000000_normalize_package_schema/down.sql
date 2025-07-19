-- Revert the normalized package schema back to the original flat structure

-- 1. Recreate the original packages table
CREATE TABLE packages_old (
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
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(name, filename)
);

-- 2. Migrate data back to the flat structure
INSERT INTO packages_old (
    name, version, filename, size_bytes, etag, content_type, upstream_url, file_path,
    created_at, last_accessed, access_count, author_id, description, package_json, is_private
)
SELECT 
    p.name,
    pv.version,
    pf.filename,
    pf.size_bytes,
    pf.etag,
    pf.content_type,
    pf.upstream_url,
    pf.file_path,
    pf.created_at,
    pf.last_accessed,
    pf.access_count,
    p.author_id,
    COALESCE(pv.description, p.description) as description,
    pv.package_json,
    p.is_private
FROM packages p
JOIN package_versions pv ON p.id = pv.package_id
JOIN package_files pf ON pv.id = pf.package_version_id;

-- 3. Drop the normalized tables
DROP VIEW IF EXISTS packages_legacy;
DROP TABLE package_files;
DROP TABLE package_versions;
DROP TABLE packages;

-- 4. Rename the old table back
ALTER TABLE packages_old RENAME TO packages;

-- 5. Recreate the original indexes
CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_packages_created_at ON packages(created_at);
CREATE INDEX idx_packages_last_accessed ON packages(last_accessed);
CREATE INDEX idx_packages_access_count ON packages(access_count);
CREATE INDEX idx_packages_author_id ON packages(author_id);
CREATE INDEX idx_packages_is_private ON packages(is_private);
