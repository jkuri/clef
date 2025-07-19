-- Restore the package_json field to package_versions table
-- This undoes the removal of the package_json column

-- SQLite doesn't support ADD COLUMN with constraints, so we need to recreate the table
-- 1. Create new table with package_json column
CREATE TABLE package_versions_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    main_file TEXT,
    scripts TEXT,
    dependencies TEXT,
    dev_dependencies TEXT,
    peer_dependencies TEXT,
    engines TEXT,
    package_json TEXT,
    shasum TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(package_id, version),
    FOREIGN KEY (package_id) REFERENCES packages (id) ON DELETE CASCADE
);

-- 2. Copy data from old table (package_json will be NULL)
INSERT INTO package_versions_new (
    id, package_id, version, description, main_file, scripts,
    dependencies, dev_dependencies, peer_dependencies, engines,
    package_json, shasum, created_at, updated_at
)
SELECT
    id, package_id, version, description, main_file, scripts,
    dependencies, dev_dependencies, peer_dependencies, engines,
    NULL as package_json, shasum, created_at, updated_at
FROM package_versions;

-- 3. Drop old table and rename new one
DROP TABLE package_versions;
ALTER TABLE package_versions_new RENAME TO package_versions;

-- 4. Recreate indexes
CREATE INDEX idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX idx_package_versions_version ON package_versions(version);
CREATE INDEX idx_package_versions_created_at ON package_versions(created_at);

-- 5. Recreate the packages_legacy view with package_json column
DROP VIEW IF EXISTS packages_legacy;
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
FROM packages p
JOIN package_versions pv ON p.id = pv.package_id
JOIN package_files pf ON pv.id = pf.package_version_id;
