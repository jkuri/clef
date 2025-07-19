-- Drop indexes
DROP INDEX IF EXISTS idx_user_tokens_expires_at;
DROP INDEX IF EXISTS idx_user_tokens_user_id;
DROP INDEX IF EXISTS idx_user_tokens_token;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_username;

-- Drop tables
DROP TABLE IF EXISTS user_tokens;
DROP TABLE IF EXISTS users;
