-- Create metadata_cache table to track cached metadata.json files
CREATE TABLE metadata_cache (
    id INTEGER PRIMARY KEY NOT NULL,
    package_name TEXT NOT NULL UNIQUE,
    size_bytes BIGINT NOT NULL,
    file_path TEXT NOT NULL,
    etag TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 0
);

-- Create index for faster lookups
CREATE INDEX idx_metadata_cache_package_name ON metadata_cache(package_name);
CREATE INDEX idx_metadata_cache_last_accessed ON metadata_cache(last_accessed);
