DROP TABLE IF EXISTS packages;

CREATE TABLE packages (
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
    UNIQUE(name, filename)
);

CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_packages_created_at ON packages(created_at);
CREATE INDEX idx_packages_last_accessed ON packages(last_accessed);
CREATE INDEX idx_packages_access_count ON packages(access_count);
