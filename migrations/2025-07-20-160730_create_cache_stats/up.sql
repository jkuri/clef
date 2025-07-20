-- Create cache_stats table to persist cache hit/miss counters
CREATE TABLE cache_stats (
    id INTEGER PRIMARY KEY NOT NULL,
    hit_count BIGINT NOT NULL DEFAULT 0,
    miss_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial record
INSERT INTO cache_stats (hit_count, miss_count) VALUES (0, 0);
