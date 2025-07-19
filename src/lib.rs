pub mod config;
pub mod routes;
pub mod fairings;
pub mod state;
pub mod error;
pub mod services;
pub mod models;
pub mod schema;

use rocket::Config;
use rocket_cors::{AllowedOrigins, CorsOptions};
use std::sync::Arc;

pub use config::AppConfig;
pub use state::AppState;
pub use fairings::RequestLogger;
pub use services::{CacheService, DatabaseService};

pub fn create_rocket() -> rocket::Rocket<rocket::Build> {
    // Load configuration from environment
    let config = AppConfig::from_env();

    // Create HTTP client
    let client = reqwest::Client::new();

    // Initialize cache service
    let cache = Arc::new(CacheService::new(config.clone()).expect("Failed to initialize cache"));

    // Initialize database service
    let database = Arc::new(DatabaseService::new(&config.database_url).expect("Failed to initialize database"));

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
