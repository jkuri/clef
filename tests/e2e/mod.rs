use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Once, OnceLock};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

static INIT: Once = Once::new();
static BINARY_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Build the binary once and cache the path
fn ensure_binary_built() -> PathBuf {
    BINARY_PATH
        .get_or_init(|| {
            println!("Building binary once for all E2E tests...");
            let build_output = Command::new("cargo")
                .args(["build"])
                .output()
                .expect("Failed to build project");

            if !build_output.status.success() {
                panic!(
                    "Failed to build project: {}",
                    String::from_utf8_lossy(&build_output.stderr)
                );
            }

            // Get the binary path
            let target_dir =
                std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
            let binary_path = PathBuf::from(target_dir).join("debug").join("pnrs");

            println!("Binary built successfully and cached");
            binary_path
        })
        .clone()
}

/// Test server configuration
pub struct TestServer {
    pub port: u16,
    pub base_url: String,
    _temp_dir: TempDir, // Keep alive for cleanup
    pub cache_dir: PathBuf,
    pub db_path: PathBuf,
}

impl TestServer {
    pub fn new() -> Self {
        let port = find_free_port();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cache_dir = temp_dir.path().join("cache");
        let db_path = temp_dir.path().join("test.db");

        fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

        Self {
            port,
            base_url: format!("http://localhost:{port}"),
            _temp_dir: temp_dir,
            cache_dir,
            db_path,
        }
    }

    pub fn start(&self) -> TestServerHandle {
        // Get the pre-built binary path (builds once if not already built)
        let binary_path = ensure_binary_built();

        let mut cmd = Command::new(&binary_path);
        cmd.env("PNRS_PORT", self.port.to_string())
            .env("PNRS_HOST", "127.0.0.1")
            .env("PNRS_CACHE_DIR", &self.cache_dir)
            .env("PNRS_DATABASE_URL", self.db_path.display().to_string())
            .env("PNRS_UPSTREAM_REGISTRY", "https://registry.npmjs.org") // Add upstream registry
            .env("PNRS_CACHE_ENABLED", "true")
            .env("PNRS_CACHE_TTL_HOURS", "24")
            .env("RUST_LOG", "-") // Enable info logging to see our custom logs
            .stdout(Stdio::inherit()) // Show stdout for debugging
            .stderr(Stdio::inherit()); // Show stderr for debugging

        let mut child = cmd.spawn().expect("Failed to start test server");

        // Wait for server to be ready with shorter timeout per attempt
        let base_url = self.base_url.clone();
        let mut server_ready = false;

        for _ in 0..60 {
            // 60 attempts = 30 seconds max
            // Check if child process is still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    panic!("Test server exited early with status: {status}");
                }
                Ok(None) => {
                    // Process is still running, continue
                }
                Err(e) => {
                    panic!("Failed to check server process status: {e}");
                }
            }

            // Try to connect to server
            match reqwest::blocking::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
                .unwrap()
                .get(format!("{base_url}/"))
                .send()
            {
                Ok(response) if response.status().is_success() => {
                    server_ready = true;
                    break;
                }
                Ok(_) | Err(_) => {
                    // Server not ready yet, continue waiting
                }
            }

            thread::sleep(Duration::from_millis(500));
        }

        if !server_ready {
            let _ = child.kill();
            panic!("Test server failed to start within 30 seconds");
        }

        TestServerHandle { child }
    }
}

pub struct TestServerHandle {
    child: std::process::Child,
}

impl Drop for TestServerHandle {
    fn drop(&mut self) {
        // Try to terminate the process
        let _ = self.child.kill();

        // Wait for the process to exit with a short timeout
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            match self.child.try_wait() {
                Ok(Some(_)) => return, // Process has exited
                Ok(None) => {
                    // Process still running, continue waiting
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => return, // Error checking status, assume it's dead
            }
        }

        // Final wait attempt
        let _ = self.child.wait();
    }
}

/// Package manager abstraction for testing
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
}

impl PackageManager {
    pub fn command(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
        }
    }

    pub fn install_args(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Npm => vec!["install"],
            PackageManager::Pnpm => vec!["install"],
            PackageManager::Yarn => vec!["install"],
        }
    }

    pub fn add_args(&self, package: &str) -> Vec<String> {
        match self {
            PackageManager::Npm => vec!["install".to_string(), package.to_string()],
            PackageManager::Pnpm => vec!["add".to_string(), package.to_string()],
            PackageManager::Yarn => vec!["add".to_string(), package.to_string()],
        }
    }

    #[allow(dead_code)]
    pub fn login_args(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Npm => vec!["login"],
            PackageManager::Pnpm => vec!["login"],
            PackageManager::Yarn => vec!["login"],
        }
    }

    #[allow(dead_code)]
    pub fn publish_args(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Npm => vec!["publish"],
            PackageManager::Pnpm => vec!["publish"],
            PackageManager::Yarn => vec!["publish"],
        }
    }

    #[allow(dead_code)]
    pub fn whoami_args(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Npm => vec!["whoami"],
            PackageManager::Pnpm => vec!["whoami"],
            PackageManager::Yarn => vec!["whoami"],
        }
    }

    pub fn audit_args(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Npm => vec!["audit"],
            PackageManager::Pnpm => vec!["audit"],
            PackageManager::Yarn => vec!["audit"],
        }
    }
}

/// Test project setup utilities
pub struct TestProject {
    pub dir: TempDir,
    pub package_json_path: PathBuf,
    pub npmrc_path: PathBuf,
}

impl TestProject {
    pub fn new(registry_url: &str) -> Self {
        let dir = TempDir::new().expect("Failed to create test project directory");
        let package_json_path = dir.path().join("package.json");
        let npmrc_path = dir.path().join(".npmrc");

        // Create basic package.json
        let package_json = serde_json::json!({
            "name": "test-project",
            "version": "1.0.0",
            "description": "Test project for e2e tests",
            "main": "index.js",
            "dependencies": {}
        });

        fs::write(
            &package_json_path,
            serde_json::to_string_pretty(&package_json).unwrap(),
        )
        .expect("Failed to write package.json");

        // Create .npmrc with registry configuration
        fs::write(&npmrc_path, format!("registry={registry_url}/registry\n"))
            .expect("Failed to write .npmrc");

        Self {
            dir,
            package_json_path,
            npmrc_path,
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn run_command(&self, pm: &PackageManager, args: &[String]) -> std::process::Output {
        Command::new(pm.command())
            .args(args)
            .current_dir(self.path())
            .output()
            .expect("Failed to run package manager command")
    }

    pub fn add_dependency(&self, name: &str, version: &str) {
        let mut package_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&self.package_json_path).unwrap()).unwrap();

        package_json["dependencies"][name] = serde_json::Value::String(version.to_string());

        fs::write(
            &self.package_json_path,
            serde_json::to_string_pretty(&package_json).unwrap(),
        )
        .expect("Failed to update package.json");
    }

    #[allow(dead_code)]
    pub fn create_test_package(&self, name: &str, version: &str) {
        let package_json = serde_json::json!({
            "name": name,
            "version": version,
            "description": "Test package for e2e tests",
            "main": "index.js",
            "scripts": {
                "test": "echo \"Error: no test specified\" && exit 1"
            },
            "keywords": ["test"],
            "author": "test",
            "license": "MIT"
        });

        fs::write(
            &self.package_json_path,
            serde_json::to_string_pretty(&package_json).unwrap(),
        )
        .expect("Failed to write test package.json");

        // Create a simple index.js
        fs::write(
            self.path().join("index.js"),
            "module.exports = 'Hello from test package';",
        )
        .expect("Failed to write index.js");
    }
}

/// Utility functions
fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a free port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

pub fn init_test_env() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

/// Helper function to handle network requests with proper error handling
#[allow(dead_code)]
pub fn safe_request<F, T>(operation: F, operation_name: &str) -> Option<T>
where
    F: FnOnce() -> Result<T, reqwest::Error>,
{
    match operation() {
        Ok(result) => Some(result),
        Err(e) => {
            println!(
                "Warning: {operation_name} failed: {e}. This may be due to network issues or missing upstream registry access."
            );
            None
        }
    }
}

/// HTTP client utilities for direct API testing
#[derive(Clone)]
pub struct ApiClient {
    pub client: reqwest::blocking::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            auth_token: None,
        }
    }

    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    pub fn get(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let mut req = self.client.get(format!("{}{}", self.base_url, path));
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }

    pub fn post(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let mut req = self.client.post(format!("{}{}", self.base_url, path));
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }

    pub fn put(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let mut req = self.client.put(format!("{}{}", self.base_url, path));
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }

    pub fn delete(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let mut req = self.client.delete(format!("{}{}", self.base_url, path));
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }
}
