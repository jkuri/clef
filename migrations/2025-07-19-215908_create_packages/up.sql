-- Create packages table - stores package-level metadata
CREATE TABLE packages (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    author_id INTEGER,
    homepage TEXT,
    repository_url TEXT,
    license TEXT,
    keywords TEXT, -- JSON array as text
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users (id) ON DELETE SET NULL
);

-- Create indexes for performance
CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_packages_author_id ON packages(author_id);
CREATE INDEX idx_packages_created_at ON packages(created_at);
