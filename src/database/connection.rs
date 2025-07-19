use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::{info, warn};
use std::path::Path;
use std::time::Duration;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// SQLite connection customizer to enable WAL mode and set pragmas for better concurrency
#[derive(Debug)]
pub struct SqliteConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::sql_query;

        // Set busy timeout first (before WAL mode) - this one is critical
        sql_query("PRAGMA busy_timeout = 60000") // 60 seconds
            .execute(conn)
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;

        // Enable WAL mode for better concurrency - critical for avoiding locks
        // Retry WAL mode setup since it's important for concurrency
        let mut wal_attempts = 0;
        let max_wal_attempts = 3;
        loop {
            match sql_query("PRAGMA journal_mode = WAL").execute(conn) {
                Ok(_) => break,
                Err(e) => {
                    wal_attempts += 1;
                    if wal_attempts >= max_wal_attempts {
                        warn!(
                            "Failed to enable WAL mode after {} attempts: {}",
                            max_wal_attempts, e
                        );
                        break;
                    }
                    // Short delay before retry
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        // Enable foreign key constraints - important but not critical
        if let Err(e) = sql_query("PRAGMA foreign_keys = ON").execute(conn) {
            warn!("Failed to enable foreign keys: {}", e);
        }

        // Optimize for concurrent access - use NORMAL instead of FULL for better performance
        if let Err(e) = sql_query("PRAGMA synchronous = NORMAL").execute(conn) {
            warn!("Failed to set synchronous mode: {}", e);
        }

        // Set cache size (negative value means KB) - performance optimization
        if let Err(e) = sql_query("PRAGMA cache_size = -32000").execute(conn) {
            warn!("Failed to set cache size: {}", e);
        }

        // Set WAL autocheckpoint for better performance - performance optimization
        if let Err(e) = sql_query("PRAGMA wal_autocheckpoint = 1000").execute(conn) {
            warn!("Failed to set WAL autocheckpoint: {}", e);
        }

        // Set temp store to memory for better performance - performance optimization
        if let Err(e) = sql_query("PRAGMA temp_store = MEMORY").execute(conn) {
            warn!("Failed to set temp store: {}", e);
        }

        // Set mmap size for better I/O performance - performance optimization
        if let Err(e) = sql_query("PRAGMA mmap_size = 268435456").execute(conn) {
            warn!("Failed to set mmap size: {}", e);
        }

        Ok(())
    }
}

/// Creates a new database connection pool with optimized settings
pub fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    // Ensure the database directory exists
    if let Some(parent) = Path::new(database_url).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(20) // Increase pool size for better concurrency
        .min_idle(Some(2)) // Keep some connections ready
        .connection_timeout(Duration::from_secs(60)) // Increase timeout
        .idle_timeout(Some(Duration::from_secs(300))) // 5 minutes idle timeout
        .max_lifetime(Some(Duration::from_secs(1800))) // 30 minutes max lifetime
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)?;

    // Run migrations
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Failed to run migrations: {}", e))?;

    info!("Database initialized successfully with WAL mode and optimized settings");

    Ok(pool)
}

/// Gets a connection from the pool with retry logic and exponential backoff
pub fn get_connection_with_retry(pool: &DbPool) -> Result<DbConnection, diesel::r2d2::Error> {
    // Retry connection acquisition with exponential backoff
    let mut attempts = 0;
    let max_attempts = 5;

    loop {
        match pool.get() {
            Ok(conn) => return Ok(conn),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(diesel::r2d2::Error::ConnectionError(
                        diesel::ConnectionError::BadConnection(format!(
                            "Failed to get connection after {} attempts: {}",
                            max_attempts, e
                        )),
                    ));
                }

                // Exponential backoff: 10ms, 20ms, 40ms, 80ms
                let delay = Duration::from_millis(10 * (1 << (attempts - 1)));
                std::thread::sleep(delay);
            }
        }
    }
}
