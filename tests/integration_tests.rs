use rocket::local::blocking::Client;
use rocket::http::Status;
use std::env;
use std::sync::Arc;
use pnrs::{AppConfig, AppState, CacheService, DatabaseService};
use rocket::Config;
use rocket_cors::{AllowedOrigins, CorsOptions};

fn create_test_rocket() -> rocket::Rocket<rocket::Build> {
    // Clear any environment variables that might interfere with tests
    unsafe {
        env::remove_var("PNRS_UPSTREAM_REGISTRY");
        env::remove_var("PNRS_PORT");
        env::remove_var("PNRS_HOST");
        env::remove_var("PNRS_CACHE_DIR");
    }

    // Create unique cache directory for this test
    let test_id = std::thread::current().id();
    let cache_dir = format!("./test_cache_{:?}", test_id);
    unsafe {
        env::set_var("PNRS_CACHE_DIR", &cache_dir);
    }

    // Load configuration from environment
    let config = AppConfig::from_env();

    // Create HTTP client
    let client = reqwest::Client::new();

    // Initialize cache service
    let cache = Arc::new(CacheService::new(config.clone()).expect("Failed to initialize cache"));

    // Initialize database service with unique database file
    let database_url = format!("{}/test_cache_{:?}.db", config.cache_dir, test_id);
    let database = Arc::new(DatabaseService::new(&database_url).expect("Failed to initialize database"));

    // Create app state
    let state = AppState {
        config: config.clone(),
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
        .attach(pnrs::RequestLogger)
        .mount("/", pnrs::routes::get_routes())
}

#[test]
fn test_health_check() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("PNRS - Private NPM Registry Server is running!".into()));
}

#[test]
fn test_package_metadata_success() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/lodash").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"name\":\"lodash\""));
    assert!(body.contains("\"description\""));
}

#[test]
fn test_package_metadata_not_found() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/nonexistent-package-12345").dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("Upstream error: 404"));
}

#[test]
fn test_package_version_metadata_success() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/lodash/4.17.21").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"name\":\"lodash\""));
    assert!(body.contains("\"version\":\"4.17.21\""));
}

#[test]
fn test_package_version_metadata_not_found() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/lodash/999.999.999").dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("Upstream error: 404"));
}

#[test]
fn test_tarball_head_success() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.head("/lodash/-/lodash-4.17.21.tgz").dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_tarball_head_not_found() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.head("/nonexistent/-/nonexistent-1.0.0.tgz").dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_tarball_download_success() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/lodash/-/lodash-4.17.21.tgz").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_bytes().expect("valid response body");
    assert!(body.len() > 1000); // Should be a substantial tarball

    // Check that it's a gzipped tarball (starts with gzip magic bytes)
    assert_eq!(&body[0..2], &[0x1f, 0x8b]);
}

#[test]
fn test_tarball_download_not_found() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/nonexistent/-/nonexistent-1.0.0.tgz").dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("Upstream error: 404"));
}

#[cfg(test)]
mod config_tests {
    use std::env;
    use pnrs::AppConfig;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.upstream_registry, "https://registry.npmjs.org");
        assert_eq!(config.port, 8000);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn test_config_env_parsing() {
        // Test port parsing
        assert_eq!("8080".parse::<u16>().unwrap_or(8000), 8080);
        assert_eq!("invalid".parse::<u16>().unwrap_or(8000), 8000);

        // Test default values
        let default_registry = env::var("NONEXISTENT_VAR")
            .unwrap_or_else(|_| "https://registry.npmjs.org".to_string());
        assert_eq!(default_registry, "https://registry.npmjs.org");
    }
}

#[test]
fn test_cache_stats() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/cache/stats").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"enabled\":true"));
    assert!(body.contains("\"cache_dir\":\"./test_cache_"));
    assert!(body.contains("\"max_size_mb\":1024"));
}

#[test]
fn test_cache_health() {
    let client = Client::tracked(create_test_rocket()).expect("valid rocket instance");
    let response = client.get("/cache/health").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"enabled\":true"));
    assert!(body.contains("\"status\":"));
}
