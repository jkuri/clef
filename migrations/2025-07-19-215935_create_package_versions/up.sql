-- Create package_versions table - stores version-specific metadata
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
    shasum TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(package_id, version),
    FOREIGN KEY (package_id) REFERENCES packages (id) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX idx_package_versions_version ON package_versions(version);
CREATE INDEX idx_package_versions_created_at ON package_versions(created_at);
CREATE INDEX idx_package_versions_package_version ON package_versions(package_id, version);
