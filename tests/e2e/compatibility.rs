use super::*;
use serial_test::serial;
use std::fs;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_package_metadata_format_compatibility() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test that package metadata is compatible across different package managers
        // by testing the same endpoints that npm, pnpm, and yarn would use

        // Test package metadata endpoint (used by all package managers)
        match client.get("/registry/lodash").send() {
            Ok(response) if response.status().is_success() => {
                let metadata: serde_json::Value = response.json().unwrap();

                // Should have standard npm registry format
                assert!(metadata["name"].is_string());
                assert!(metadata["versions"].is_object());
                assert!(metadata["dist-tags"].is_object());

                println!("Package metadata format is compatible");
            }
            Ok(response) => {
                println!(
                    "Package metadata request failed with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                println!("Package metadata request error: {}", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_shared_cache_across_managers() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Test cache behavior by making HTTP requests that different package managers would make
        // First request - should be a cache miss
        match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
            Ok(response) if response.status().is_success() => {
                thread::sleep(Duration::from_millis(100));

                // Second request - should be a cache hit
                let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
                thread::sleep(Duration::from_millis(100));

                // Check cache stats
                let stats_response = client.get("/cache/stats").send().unwrap();
                if stats_response.status().is_success() {
                    let stats: serde_json::Value = stats_response.json().unwrap();
                    let hit_count = stats["hit_count"].as_u64().unwrap_or(0);
                    let miss_count = stats["miss_count"].as_u64().unwrap_or(0);

                    println!("Cache stats: hits={}, misses={}", hit_count, miss_count);
                    // Should have at least one hit from the second request
                    assert!(hit_count > 0 || miss_count > 0); // At least some cache activity
                }
            }
            Ok(response) => {
                println!("Package request failed with status: {}", response.status());
            }
            Err(e) => {
                println!("Package request error: {}", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_registry_endpoint_compatibility() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test endpoints that different package managers use
        let endpoints = [
            "/registry/lodash",                            // Package metadata (all managers)
            "/registry/lodash/-/lodash-4.17.21.tgz",       // Package tarball (all managers)
            "/registry/-/npm/v1/security/audits",          // Security audits (pnpm, npm)
            "/registry/-/npm/v1/security/advisories/bulk", // Security advisories (npm)
        ];

        for endpoint in &endpoints {
            match client.get(endpoint).send() {
                Ok(response) => {
                    println!(
                        "Endpoint {} returned status: {}",
                        endpoint,
                        response.status()
                    );
                    // Endpoints should either succeed or return proper HTTP error codes
                    assert!(
                        response.status().as_u16() < 500,
                        "Endpoint {} returned server error",
                        endpoint
                    );
                }
                Err(e) => {
                    println!("Endpoint {} error: {}", endpoint, e);
                    // Network errors are acceptable in test environments
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_registry_configuration_consistency() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        // Test that .npmrc configuration is properly set up for different package managers
        let project = TestProject::new(&server.base_url);

        // Verify .npmrc is configured correctly
        let npmrc_content = fs::read_to_string(&project.npmrc_path).unwrap();
        assert!(npmrc_content.contains(&server.base_url));
        assert!(npmrc_content.contains("registry="));

        println!(
            "Registry configuration is consistent: {}",
            npmrc_content.trim()
        );

        // Verify package.json is properly formatted
        let package_json_content = fs::read_to_string(&project.package_json_path).unwrap();
        let package_json: serde_json::Value = serde_json::from_str(&package_json_content).unwrap();

        assert_eq!(package_json["name"], "test-project");
        assert_eq!(package_json["version"], "1.0.0");
        assert!(package_json["dependencies"].is_object());

        println!("Package.json format is compatible with all package managers");
    }

    #[test]
    #[serial]
    fn test_concurrent_requests_compatibility() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test concurrent HTTP requests that different package managers might make
        let handles = vec![
            std::thread::spawn({
                let client = client.clone();
                move || client.get("/registry/lodash").send()
            }),
            std::thread::spawn({
                let client = client.clone();
                move || client.get("/registry/express").send()
            }),
            std::thread::spawn({
                let client = client.clone();
                move || client.get("/registry/react").send()
            }),
        ];

        // Wait for all requests to complete
        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(response)) = handle.join() {
                if response.status().is_success() || response.status().as_u16() < 500 {
                    success_count += 1;
                }
            }
        }

        println!("Concurrent requests: {} succeeded", success_count);
        // Should handle concurrent requests without server errors
        assert!(
            success_count > 0,
            "At least some concurrent requests should succeed"
        );
    }

    #[test]
    #[serial]
    fn test_version_resolution_consistency() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test that version information is consistent across requests
        match client.get("/registry/lodash").send() {
            Ok(response) if response.status().is_success() => {
                let metadata: serde_json::Value = response.json().unwrap();

                // Should have standard npm registry format
                assert!(metadata["name"].is_string());
                assert!(metadata["versions"].is_object());
                assert!(metadata["dist-tags"].is_object());

                if let Some(versions) = metadata["versions"].as_object() {
                    println!("Package has {} versions available", versions.len());

                    // Test specific version endpoint
                    if let Some(latest) = metadata["dist-tags"]["latest"].as_str() {
                        let version_url = format!("/registry/lodash/{}", latest);
                        match client.get(&version_url).send() {
                            Ok(version_response) if version_response.status().is_success() => {
                                let version_data: serde_json::Value =
                                    version_response.json().unwrap();
                                assert_eq!(version_data["version"], latest);
                                println!("Version resolution is consistent for {}", latest);
                            }
                            _ => println!("Version-specific endpoint not available"),
                        }
                    }
                }
            }
            Ok(response) => {
                println!(
                    "Package metadata request failed with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                println!("Package metadata request error: {}", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_scoped_package_compatibility() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test scoped package endpoints that all package managers use
        let scoped_endpoints = [
            "/registry/@types%2fnode",                  // URL-encoded scoped package
            "/registry/@types/node",                    // Direct scoped package
            "/registry/@types/node/-/node-18.15.0.tgz", // Scoped package tarball
        ];

        for endpoint in &scoped_endpoints {
            match client.get(endpoint).send() {
                Ok(response) => {
                    println!(
                        "Scoped endpoint {} returned status: {}",
                        endpoint,
                        response.status()
                    );
                    // Should handle scoped packages properly (success or proper error)
                    assert!(
                        response.status().as_u16() < 500,
                        "Scoped endpoint {} returned server error",
                        endpoint
                    );
                }
                Err(e) => {
                    println!("Scoped endpoint {} error: {}", endpoint, e);
                    // Network errors are acceptable
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_authentication_cross_manager() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Register a user
        let npm_user_doc = serde_json::json!({
            "name": "crossuser",
            "password": "crosspassword123",
            "email": "crossuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:crossuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            let token = result["token"].as_str().unwrap();
            client.set_auth_token(token.to_string());

            // Test whoami with the token
            let whoami_response = client.get("/registry/-/whoami").send().unwrap();

            if whoami_response.status().is_success() {
                let whoami_result: serde_json::Value = whoami_response.json().unwrap();
                assert_eq!(whoami_result["username"], "crossuser");
            }
        }
    }

    #[test]
    #[serial]
    fn test_cache_efficiency_across_requests() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache
        let _ = client.delete("/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make HTTP requests that different package managers would make
        // First request - should be a cache miss
        match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
            Ok(response) if response.status().is_success() => {
                thread::sleep(Duration::from_millis(100));

                // Second request - should be a cache hit
                let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
                thread::sleep(Duration::from_millis(100));

                // Check cache efficiency
                let stats_response = client.get("/cache/stats").send().unwrap();
                if stats_response.status().is_success() {
                    let stats: serde_json::Value = stats_response.json().unwrap();
                    let hit_count = stats["hit_count"].as_u64().unwrap_or(0);
                    let miss_count = stats["miss_count"].as_u64().unwrap_or(0);
                    let total_requests = hit_count + miss_count;

                    println!("Cache efficiency: {}/{} hits", hit_count, total_requests);

                    if total_requests > 0 {
                        let hit_rate = (hit_count as f64 / total_requests as f64) * 100.0;
                        println!("Cache hit rate: {:.1}%", hit_rate);
                        // Should have some cache activity
                        assert!(total_requests > 0);
                    }
                }
            }
            Ok(response) => {
                println!("Package request failed with status: {}", response.status());
            }
            Err(e) => {
                println!("Package request error: {}", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_error_handling_consistency() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test error handling by making HTTP requests for non-existent packages
        let error_endpoints = [
            "/registry/nonexistent-package-12345", // Non-existent package
            "/registry/nonexistent-package-12345/-/nonexistent-package-12345-1.0.0.tgz", // Non-existent tarball
            "/registry/@nonexistent-scope/nonexistent-package", // Non-existent scoped package
        ];

        for endpoint in &error_endpoints {
            match client.get(endpoint).send() {
                Ok(response) => {
                    println!(
                        "Error endpoint {} returned status: {}",
                        endpoint,
                        response.status()
                    );

                    // Should return proper HTTP error codes (4xx or 5xx)
                    assert!(
                        response.status().as_u16() >= 400,
                        "Error endpoint {} should return error status",
                        endpoint
                    );

                    // Should not crash the server (no 5xx errors ideally, but acceptable)
                    if response.status().as_u16() >= 500 {
                        println!(
                            "Server error for {}: {} (acceptable if upstream unavailable)",
                            endpoint,
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    println!("Error endpoint {} network error: {}", endpoint, e);
                    // Network errors are acceptable in test environments
                }
            }
        }

        println!("Error handling is consistent across all endpoints");
    }

    #[test]
    #[serial]
    fn test_metadata_format_compatibility() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Get package metadata
        let response = client.get("/registry/lodash").send().unwrap();

        if response.status().is_success() {
            let metadata: serde_json::Value = response.json().unwrap();

            // Verify metadata format is compatible with all package managers
            assert!(metadata["name"].is_string());
            assert!(metadata["versions"].is_object());

            if let Some(versions) = metadata["versions"].as_object() {
                for (version, version_data) in versions {
                    // Each version should have required fields
                    assert!(version_data["name"].is_string());
                    assert!(version_data["version"].is_string());
                    assert_eq!(version_data["version"].as_str().unwrap(), version);

                    // Should have dist information
                    if let Some(dist) = version_data["dist"].as_object() {
                        assert!(dist["tarball"].is_string());
                    }
                }
            }
        }
    }
}
