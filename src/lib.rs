pub mod config;
pub mod database;
pub mod error;
pub mod fairings;
pub mod models;
pub mod routes;
pub mod schema;
pub mod services;
pub mod state;

use rocket::Config;
use rocket_cors::{AllowedOrigins, CorsOptions};
use std::sync::Arc;

pub use config::AppConfig;
pub use database::DatabaseService;
pub use fairings::RequestLogger;
pub use services::CacheService;
pub use state::AppState;

pub fn create_rocket() -> rocket::Rocket<rocket::Build> {
    // Load configuration from environment
    let config = AppConfig::from_env();

    // Create HTTP client
    let client = reqwest::Client::new();

    // Initialize database service first
    let database = Arc::new(
        DatabaseService::new(&config.database_url).expect("Failed to initialize database"),
    );

    // Initialize cache service with database for persistent stats
    let cache = Arc::new(
        CacheService::new_with_database(config.clone(), Some(&database))
            .expect("Failed to initialize cache"),
    );

    // Create app state
    let state = AppState {
        config,
        client,
        cache,
        database,
    };

    // Configure CORS
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Failed to create CORS configuration");

    // Configure Rocket with custom host and port
    let rocket_config = Config {
        port: state.config.port,
        address: state.config.host.parse().expect("Invalid host address"),
        ..Config::default()
    };

    rocket::custom(&rocket_config)
        .manage(state)
        .attach(cors)
        .attach(RequestLogger)
        .mount("/", routes::get_routes())
}
