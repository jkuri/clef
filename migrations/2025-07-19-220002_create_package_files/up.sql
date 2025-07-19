-- Create package_files table - stores file-specific metadata and cache info
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

-- Create indexes for performance
CREATE INDEX idx_package_files_package_version_id ON package_files(package_version_id);
CREATE INDEX idx_package_files_filename ON package_files(filename);
CREATE INDEX idx_package_files_created_at ON package_files(created_at);
CREATE INDEX idx_package_files_last_accessed ON package_files(last_accessed);
CREATE INDEX idx_package_files_access_count ON package_files(access_count);
