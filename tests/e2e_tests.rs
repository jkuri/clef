mod e2e;

use e2e::*;
use serial_test::serial;

// Import all test modules
#[path = "e2e/analytics.rs"]
mod analytics;
#[path = "e2e/authentication.rs"]
mod authentication;
#[path = "e2e/cache_management.rs"]
mod cache_management;
#[path = "e2e/compatibility.rs"]
mod compatibility;
#[path = "e2e/package_management.rs"]
mod package_management;
#[path = "e2e/package_ownership.rs"]
mod package_ownership;
#[path = "e2e/performance.rs"]
mod performance;
#[path = "e2e/private_packages.rs"]
mod private_packages;
#[path = "e2e/proxied_metadata.rs"]
mod proxied_metadata;
#[path = "e2e/publishing.rs"]
mod publishing;
#[path = "e2e/scoped_packages.rs"]
mod scoped_packages;
#[path = "e2e/security.rs"]
mod security;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_server_startup() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        // Test basic health check
        let client = ApiClient::new(server.base_url.clone());
        let response = client.get("/api/v1/health").send().unwrap();
        assert!(response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_package_managers_available() {
        // Check if npm, pnpm, and yarn are available for testing
        let managers = [
            PackageManager::Npm,
            PackageManager::Pnpm,
            PackageManager::Yarn,
        ];

        for manager in &managers {
            let output = std::process::Command::new(manager.command())
                .arg("--version")
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    println!(
                        "{} version: {}",
                        manager.command(),
                        String::from_utf8_lossy(&output.stdout)
                    );
                }
                _ => {
                    println!(
                        "Warning: {} is not available for testing",
                        manager.command()
                    );
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_project_setup() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Verify project structure
        assert!(project.package_json_path.exists());
        assert!(project.npmrc_path.exists());

        // Verify .npmrc content
        let npmrc_content = std::fs::read_to_string(&project.npmrc_path).unwrap();
        assert!(npmrc_content.contains(&server.base_url));
    }
}
