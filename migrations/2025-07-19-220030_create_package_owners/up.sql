-- Create package_owners table for ownership management
CREATE TABLE package_owners (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    package_name TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    permission_level TEXT NOT NULL DEFAULT 'write', -- 'read', 'write', 'admin'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(package_name, user_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX idx_package_owners_package_name ON package_owners(package_name);
CREATE INDEX idx_package_owners_user_id ON package_owners(user_id);
