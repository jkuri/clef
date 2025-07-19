-- Normalize the package schema into separate tables for packages, versions, and files
-- This migration creates the new normalized structure

-- 1. Create the new normalized tables

-- Packages table - stores package-level metadata
CREATE TABLE packages_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    author_id INTEGER,
    homepage TEXT,
    repository_url TEXT,
    license TEXT,
    keywords TEXT, -- JSON array as text
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users (id) ON DELETE SET NULL
);

-- Package versions table - stores version-specific metadata
CREATE TABLE package_versions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL,
    version TEXT NOT NULL,
    description TEXT, -- Version-specific description (can override package description)
    main_file TEXT,
    scripts TEXT, -- JSON object as text
    dependencies TEXT, -- JSON object as text
    dev_dependencies TEXT, -- JSON object as text
    peer_dependencies TEXT, -- JSON object as text
    engines TEXT, -- JSON object as text
    package_json TEXT, -- Full package.json for this version
    shasum TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(package_id, version),
    FOREIGN KEY (package_id) REFERENCES packages_new (id) ON DELETE CASCADE
);

-- Package files table - stores file-specific metadata and cache info
CREATE TABLE package_files (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    package_version_id INTEGER NOT NULL,
    filename TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    content_type TEXT,
    etag TEXT,
    upstream_url TEXT NOT NULL,
    file_path TEXT NOT NULL, -- Local cache file path
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 1,
    UNIQUE(package_version_id, filename),
    FOREIGN KEY (package_version_id) REFERENCES package_versions (id) ON DELETE CASCADE
);

-- 2. Create indexes for performance
CREATE INDEX idx_packages_new_name ON packages_new(name);
CREATE INDEX idx_packages_new_author_id ON packages_new(author_id);
CREATE INDEX idx_packages_new_created_at ON packages_new(created_at);
CREATE INDEX idx_packages_new_is_private ON packages_new(is_private);

CREATE INDEX idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX idx_package_versions_version ON package_versions(version);
CREATE INDEX idx_package_versions_created_at ON package_versions(created_at);
CREATE INDEX idx_package_versions_package_version ON package_versions(package_id, version);

CREATE INDEX idx_package_files_package_version_id ON package_files(package_version_id);
CREATE INDEX idx_package_files_filename ON package_files(filename);
CREATE INDEX idx_package_files_last_accessed ON package_files(last_accessed);
CREATE INDEX idx_package_files_access_count ON package_files(access_count);

-- 3. Create a view for backward compatibility (optional)
CREATE VIEW packages_legacy AS
SELECT
    pf.id,
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
FROM packages_new p
JOIN package_versions pv ON p.id = pv.package_id
JOIN package_files pf ON pv.id = pf.package_version_id;

-- 4. Migrate existing data
-- First, create packages from unique names in the old table (handle conflicting metadata by taking the most recent)
INSERT INTO packages_new (name, description, author_id, is_private, created_at, updated_at)
SELECT
    name,
    description,
    author_id,
    is_private,
    MIN(created_at) as created_at,
    MAX(created_at) as updated_at
FROM packages
WHERE name IS NOT NULL
GROUP BY name;

-- Then create package versions (handle duplicates by taking the first occurrence)
INSERT INTO package_versions (package_id, version, description, package_json, created_at, updated_at)
SELECT
    p_new.id,
    p_old.version,
    p_old.description,
    p_old.package_json,
    MIN(p_old.created_at) as created_at,
    MIN(p_old.created_at) as updated_at
FROM packages p_old
JOIN packages_new p_new ON p_old.name = p_new.name
WHERE p_old.version IS NOT NULL
GROUP BY p_new.id, p_old.version;

-- Finally, create package files
INSERT INTO package_files (package_version_id, filename, size_bytes, content_type, etag, upstream_url, file_path, created_at, last_accessed, access_count)
SELECT
    pv.id,
    p_old.filename,
    p_old.size_bytes,
    p_old.content_type,
    p_old.etag,
    p_old.upstream_url,
    p_old.file_path,
    p_old.created_at,
    p_old.last_accessed,
    p_old.access_count
FROM packages p_old
JOIN packages_new p_new ON p_old.name = p_new.name
JOIN package_versions pv ON p_new.id = pv.package_id AND p_old.version = pv.version
WHERE p_old.filename IS NOT NULL;

-- 5. Drop the old table and rename the new one
DROP TABLE packages;
ALTER TABLE packages_new RENAME TO packages;

-- 6. Update the package_owners table to reference the new packages table structure
-- (The foreign key relationship should still work since we preserved the package names)
