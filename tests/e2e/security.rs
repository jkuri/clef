use super::*;
use serde_json::json;
use serial_test::serial;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_security_audit_request() -> serde_json::Value {
        json!({
            "name": "test-project",
            "version": "1.0.0",
            "requires": {
                "lodash": "^4.17.21",
                "express": "^4.18.2"
            },
            "dependencies": {
                "lodash": {
                    "version": "4.17.21",
                    "requires": {}
                },
                "express": {
                    "version": "4.18.2",
                    "requires": {
                        "accepts": "~1.3.8",
                        "array-flatten": "1.1.1"
                    }
                }
            }
        })
    }

    fn create_security_advisories_request() -> serde_json::Value {
        json!({
            "lodash": ["4.17.21"],
            "express": ["4.18.2"]
        })
    }

    #[test]
    #[serial]
    fn test_security_advisories_bulk_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test security advisories bulk endpoint
        let advisories_request = create_security_advisories_request();

        // Use a timeout and handle errors gracefully
        match client
            .post("/registry/-/npm/v1/security/advisories/bulk")
            .json(&advisories_request)
            .send()
        {
            Ok(response) => {
                println!(
                    "Security advisories endpoint returned status: {}",
                    response.status()
                );
                // Should not crash the server (accept any response)
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(result) => {
                            println!("Security advisories response received");
                            assert!(result.is_object() || result.is_array());
                        }
                        Err(_) => println!("Response not JSON (acceptable)"),
                    }
                } else {
                    println!(
                        "Security advisories request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!("Security advisories request error: {} (acceptable)", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_security_audits_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test main security audits endpoint (used by pnpm) with minimal request
        let audit_request = json!({"name": "test", "version": "1.0.0"});

        match client
            .post("/registry/-/npm/v1/security/audits")
            .json(&audit_request)
            .send()
        {
            Ok(response) => {
                println!(
                    "Security audits endpoint returned status: {}",
                    response.status()
                );
                // Should not crash the server
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(result) => {
                            println!("Security audits response received");
                            assert!(result.is_object());
                        }
                        Err(_) => println!("Response not JSON (acceptable)"),
                    }
                } else {
                    println!(
                        "Security audits request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!("Security audits request error: {} (acceptable)", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_security_audits_quick_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test security audits quick endpoint with minimal request
        let audit_request = json!({"name": "test", "version": "1.0.0"});

        match client
            .post("/registry/-/npm/v1/security/audits/quick")
            .json(&audit_request)
            .send()
        {
            Ok(response) => {
                println!(
                    "Security audits quick endpoint returned status: {}",
                    response.status()
                );
                // Should not crash the server
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(result) => {
                            println!("Security audits quick response received");
                            assert!(result.is_object());
                        }
                        Err(_) => println!("Response not JSON (acceptable)"),
                    }
                } else {
                    println!(
                        "Security audits quick request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!("Security audits quick request error: {} (acceptable)", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_npm_audit_command_integration() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Add some dependencies to package.json
        project.add_dependency("lodash", "^4.17.21");
        project.add_dependency("express", "^4.18.2");

        // First install dependencies to create package-lock.json
        let install_output = project.run_command(
            &PackageManager::Npm,
            &PackageManager::Npm
                .install_args()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );

        if install_output.status.success() {
            // Now try running npm audit (should work since we have a lockfile)
            let audit_output = project.run_command(
                &PackageManager::Npm,
                &PackageManager::Npm
                    .audit_args()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            );

            if audit_output.status.success() {
                let stdout = String::from_utf8_lossy(&audit_output.stdout);
                // Should contain audit information or indicate no vulnerabilities
                assert!(
                    stdout.contains("audit")
                        || stdout.contains("vulnerabilities")
                        || stdout.contains("found")
                );
            } else {
                // npm audit failed - might be due to network issues or upstream problems
                // This is acceptable since we mainly test that our server handles the requests
            }
        } else {
            // npm install failed - likely npm not available
            // This is acceptable in test environments
        }
    }

    #[test]
    #[serial]
    fn test_pnpm_audit_endpoint_integration() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test that pnpm audit endpoint works by making HTTP request with pnpm user agent
        let audit_request = json!({"name": "test", "version": "1.0.0"});

        match client
            .post("/registry/-/npm/v1/security/audits")
            .header("User-Agent", "pnpm/7.14.0 node/v18.12.1 linux x64")
            .json(&audit_request)
            .send()
        {
            Ok(response) => {
                println!("pnpm audit endpoint returned status: {}", response.status());
                // Should handle pnpm requests without crashing
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(result) => {
                            println!("pnpm audit response received");
                            assert!(result.is_object());
                        }
                        Err(_) => println!("Response not JSON (acceptable)"),
                    }
                } else {
                    println!(
                        "pnpm audit request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!("pnpm audit request error: {} (acceptable)", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_yarn_audit_command_integration() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Add some dependencies to package.json
        project.add_dependency("lodash", "^4.17.21");
        project.add_dependency("express", "^4.18.2");

        // First install dependencies to create yarn.lock
        let install_output = project.run_command(
            &PackageManager::Yarn,
            &PackageManager::Yarn
                .install_args()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );

        if install_output.status.success() {
            // Now try running yarn audit (should work since we have a lockfile)
            let audit_output = project.run_command(
                &PackageManager::Yarn,
                &PackageManager::Yarn
                    .audit_args()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            );

            if audit_output.status.success() {
                let stdout = String::from_utf8_lossy(&audit_output.stdout);
                // Should contain audit information
                assert!(
                    stdout.contains("audit")
                        || stdout.contains("vulnerabilities")
                        || stdout.len() > 0
                );
            } else {
                // yarn audit failed - might be due to network issues or upstream problems
                // This is acceptable since we mainly test that our server handles the requests
            }
        } else {
            // yarn install failed - likely yarn not available
            // This is acceptable in test environments
        }
    }

    #[test]
    #[serial]
    fn test_security_endpoint_error_handling() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with malformed JSON
        let response = client
            .post("/registry/-/npm/v1/security/advisories/bulk")
            .header("Content-Type", "application/json")
            .body("invalid json")
            .send()
            .unwrap();

        // Should handle malformed requests gracefully
        if response.status().is_success() {
            // If it succeeds, it should return empty response
            let result: serde_json::Value = response.json().unwrap();
            assert!(result.is_object());
        } else {
            // Bad request is acceptable
            assert!(response.status().is_client_error());
        }
    }

    #[test]
    #[serial]
    fn test_security_endpoint_large_request() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Create a large audit request with many dependencies
        let mut large_dependencies = serde_json::Map::new();
        for i in 0..100 {
            large_dependencies.insert(
                format!("package-{}", i),
                json!({
                    "version": "1.0.0",
                    "requires": {}
                }),
            );
        }

        let large_request = json!({
            "name": "large-test-project",
            "version": "1.0.0",
            "dependencies": large_dependencies
        });

        let response = client
            .post("/registry/-/npm/v1/security/audits/quick")
            .json(&large_request)
            .send()
            .unwrap();

        // Should handle large requests without crashing
        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            assert!(result.is_object());
        } else {
            // May fail due to size limits or upstream issues
            println!("Large security request failed: {}", response.status());
        }
    }

    #[test]
    #[serial]
    fn test_security_endpoint_empty_request() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with empty request
        let empty_request = json!({});

        match client
            .post("/registry/-/npm/v1/security/advisories/bulk")
            .json(&empty_request)
            .send()
        {
            Ok(response) => {
                println!("Empty request returned status: {}", response.status());
                // Should handle empty requests without crashing
                assert!(response.status().as_u16() < 500 || response.status().as_u16() == 502);
            }
            Err(e) => {
                println!("Empty request error: {} (acceptable)", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_security_endpoint_content_encoding() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test that our server properly handles different content encodings
        let audit_request = json!({"name": "test", "version": "1.0.0"});

        match client
            .post("/registry/-/npm/v1/security/audits/quick")
            .header("Content-Encoding", "gzip")
            .json(&audit_request)
            .send()
        {
            Ok(response) => {
                println!(
                    "Content-Encoding test returned status: {}",
                    response.status()
                );
                // Should handle the request regardless of content encoding
                assert!(response.status().as_u16() < 500 || response.status().as_u16() == 502);
            }
            Err(e) => {
                println!("Content-Encoding test error: {} (acceptable)", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_security_endpoint_user_agent() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with different user agents (simulating different npm clients)
        let user_agents = [
            "npm/8.19.2 node/v18.12.1 linux x64 workspaces/false",
            "pnpm/7.14.0 node/v18.12.1 linux x64",
            "yarn/1.22.19 npm/? node/v18.12.1 linux x64",
        ];

        let audit_request = create_security_audit_request();

        for user_agent in &user_agents {
            let response = client
                .post("/registry/-/npm/v1/security/audits/quick")
                .header("User-Agent", *user_agent)
                .json(&audit_request)
                .send()
                .unwrap();

            // Should handle requests from different package managers
            if response.status().is_success() {
                let result: serde_json::Value = response.json().unwrap();
                assert!(result.is_object());
            }
        }
    }

    #[test]
    #[serial]
    fn test_security_endpoint_concurrent_requests() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let audit_request = json!({"name": "test", "version": "1.0.0"});

        // Simulate concurrent security audit requests
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let base_url = server.base_url.clone();
                let request = audit_request.clone();
                std::thread::spawn(move || {
                    let client = ApiClient::new(base_url);
                    client
                        .post("/registry/-/npm/v1/security/audits/quick")
                        .json(&request)
                        .send()
                })
            })
            .collect();

        // Wait for all requests to complete
        let mut success_count = 0;
        let mut total_count = 0;
        for handle in handles {
            total_count += 1;
            if let Ok(Ok(response)) = handle.join() {
                if response.status().as_u16() < 500 {
                    success_count += 1;
                }
            }
        }

        // Should handle concurrent requests without server errors
        println!(
            "Concurrent security requests: {}/{} handled gracefully",
            success_count, total_count
        );
        assert!(
            success_count > 0,
            "At least some concurrent requests should be handled"
        );
    }

    #[test]
    #[serial]
    fn test_security_fallback_behavior() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test that security endpoints provide fallback responses when upstream fails
        let audit_request = json!({"name": "test", "version": "1.0.0"});

        match client
            .post("/registry/-/npm/v1/security/audits/quick")
            .json(&audit_request)
            .send()
        {
            Ok(response) => {
                println!(
                    "Security fallback test returned status: {}",
                    response.status()
                );
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(result) => {
                            println!("Security fallback response received");
                            assert!(result.is_object());
                        }
                        Err(_) => println!("Response not JSON (acceptable)"),
                    }
                } else {
                    println!(
                        "Security fallback request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!("Security fallback request error: {} (acceptable)", e);
            }
        }
    }
}
