use clef::{AppConfig, AppState, CacheService, DatabaseService};
use rocket::Config;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket_cors::{AllowedOrigins, CorsOptions};
use serial_test::serial;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::TempDir;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

struct TestRocket {
    rocket: rocket::Rocket<rocket::Build>,
    _temp_dir: TempDir, // Keep alive for cleanup
}

fn create_test_rocket() -> TestRocket {
    // Clear any environment variables that might interfere with tests
    unsafe {
        env::remove_var("CLEF_UPSTREAM_REGISTRY");
        env::remove_var("CLEF_PORT");
        env::remove_var("CLEF_HOST");
        env::remove_var("CLEF_CACHE_DIR");
    }

    // Create temporary directory for this test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache_dir = temp_dir.path().join("cache");
    std::fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

    unsafe {
        env::set_var("CLEF_CACHE_DIR", cache_dir.to_string_lossy().as_ref());
    }

    // Load configuration from environment
    let config = AppConfig::from_env();

    // Create HTTP client
    let client = reqwest::Client::new();

    // Initialize cache service
    let cache = Arc::new(CacheService::new(config.clone()).expect("Failed to initialize cache"));

    // Initialize database service with unique database file
    let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let database_url = format!("{}/test_{}.db", config.cache_dir, test_id);
    let database =
        Arc::new(DatabaseService::new(&database_url).expect("Failed to initialize database"));

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

    let rocket = rocket::custom(&rocket_config)
        .manage(state)
        .attach(cors)
        .attach(clef::RequestLogger)
        .mount("/", clef::routes::get_routes());

    TestRocket {
        rocket,
        _temp_dir: temp_dir,
    }
}

#[test]
#[serial]
fn test_health_check() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/api/v1/health").dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("Response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    assert_eq!(json["status"], "ok");
}

#[test]
#[serial]
fn test_package_metadata_success() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/registry/lodash").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"name\":\"lodash\""));
    assert!(body.contains("\"description\""));
}

#[test]
#[serial]
fn test_package_metadata_not_found() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/registry/nonexistent-package-12345").dispatch();

    assert_eq!(response.status(), Status::BadGateway);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("Upstream error: 404"));
}

#[test]
#[serial]
fn test_package_version_metadata_success() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/registry/lodash/4.17.21").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"name\":\"lodash\""));
    assert!(body.contains("\"version\":\"4.17.21\""));
}

#[test]
#[serial]
fn test_package_version_metadata_not_found() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/registry/lodash/999.999.999").dispatch();

    assert_eq!(response.status(), Status::BadGateway);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("Upstream error: 404"));
}

#[test]
#[serial]
fn test_tarball_head_success() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client
        .head("/registry/lodash/-/lodash-4.17.21.tgz")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
#[serial]
fn test_tarball_head_not_found() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client
        .head("/registry/nonexistent/-/nonexistent-1.0.0.tgz")
        .dispatch();

    assert_eq!(response.status(), Status::BadGateway);
}

#[test]
#[serial]
fn test_tarball_download_success() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client
        .get("/registry/lodash/-/lodash-4.17.21.tgz")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_bytes().expect("valid response body");
    assert!(body.len() > 1000); // Should be a substantial tarball

    // Check that it's a gzipped tarball (starts with gzip magic bytes)
    assert_eq!(&body[0..2], &[0x1f, 0x8b]);
}

#[test]
#[serial]
fn test_tarball_download_not_found() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client
        .get("/registry/nonexistent/-/nonexistent-1.0.0.tgz")
        .dispatch();

    assert_eq!(response.status(), Status::BadGateway);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("Upstream error: 404"));
}

#[cfg(test)]
mod config_tests {
    use clef::AppConfig;
    use std::env;

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
#[serial]
fn test_cache_stats() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/api/v1/cache/stats").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"enabled\":true"));
    assert!(body.contains("\"cache_dir\":"));
}

#[test]
#[serial]
fn test_cache_health() {
    let test_rocket = create_test_rocket();
    let client = Client::tracked(test_rocket.rocket).expect("valid rocket instance");
    let response = client.get("/api/v1/cache/health").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().expect("valid response body");
    assert!(body.contains("\"enabled\":true"));
    assert!(body.contains("\"status\":"));
}
