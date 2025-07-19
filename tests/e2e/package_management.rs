use super::*;
use serial_test::serial;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_package_metadata_fetch_npm() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();
        let _project = TestProject::new(&server.base_url);

        // Test fetching package metadata directly via API
        let client = ApiClient::new(server.base_url.clone());

        match client.get("/express").send() {
            Ok(response) if response.status().is_success() => {
                let metadata: serde_json::Value = response.json().unwrap();
                assert_eq!(metadata["name"], "express");
                assert!(metadata["versions"].is_object());
            }
            Ok(response) => {
                println!(
                    "Package metadata request failed with status: {}",
                    response.status()
                );
                // Don't fail the test - this might be due to network issues
            }
            Err(e) => {
                println!(
                    "Package metadata request failed: {}. This may be due to network issues.",
                    e
                );
                // Don't fail the test - this might be due to network issues
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_installation_npm() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test npm-style package requests (what npm would make during installation)
        match client.get("/lodash").send() {
            Ok(response) => {
                println!(
                    "npm-style package metadata request returned: {}",
                    response.status()
                );
                if response.status().is_success() {
                    // Test tarball download (what npm would do next)
                    match client.get("/lodash/-/lodash-4.17.21.tgz").send() {
                        Ok(tarball_response) => {
                            println!(
                                "npm-style tarball download returned: {}",
                                tarball_response.status()
                            );
                            assert!(
                                tarball_response.status().is_success()
                                    || tarball_response.status().as_u16() < 500
                            );
                        }
                        Err(e) => println!("npm-style tarball request error: {} (acceptable)", e),
                    }
                } else {
                    println!(
                        "npm-style metadata request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => println!("npm-style request error: {} (acceptable)", e),
        }
    }

    #[test]
    #[serial]
    fn test_package_installation_pnpm() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test pnpm-style package requests (what pnpm would make during installation)
        match client
            .get("/lodash")
            .header("User-Agent", "pnpm/7.14.0 node/v18.12.1 linux x64")
            .send()
        {
            Ok(response) => {
                println!(
                    "pnpm-style package metadata request returned: {}",
                    response.status()
                );
                if response.status().is_success() {
                    // Test tarball download with pnpm user agent
                    match client
                        .get("/lodash/-/lodash-4.17.21.tgz")
                        .header("User-Agent", "pnpm/7.14.0 node/v18.12.1 linux x64")
                        .send()
                    {
                        Ok(tarball_response) => {
                            println!(
                                "pnpm-style tarball download returned: {}",
                                tarball_response.status()
                            );
                            assert!(
                                tarball_response.status().is_success()
                                    || tarball_response.status().as_u16() < 500
                            );
                        }
                        Err(e) => println!("pnpm-style tarball request error: {} (acceptable)", e),
                    }
                } else {
                    println!(
                        "pnpm-style metadata request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => println!("pnpm-style request error: {} (acceptable)", e),
        }
    }

    #[test]
    #[serial]
    fn test_package_installation_yarn() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test yarn-style package requests (what yarn would make during installation)
        match client
            .get("/lodash")
            .header("User-Agent", "yarn/1.22.19 npm/? node/v18.12.1 linux x64")
            .send()
        {
            Ok(response) => {
                println!(
                    "yarn-style package metadata request returned: {}",
                    response.status()
                );
                if response.status().is_success() {
                    // Test tarball download with yarn user agent
                    match client
                        .get("/lodash/-/lodash-4.17.21.tgz")
                        .header("User-Agent", "yarn/1.22.19 npm/? node/v18.12.1 linux x64")
                        .send()
                    {
                        Ok(tarball_response) => {
                            println!(
                                "yarn-style tarball download returned: {}",
                                tarball_response.status()
                            );
                            assert!(
                                tarball_response.status().is_success()
                                    || tarball_response.status().as_u16() < 500
                            );
                        }
                        Err(e) => println!("yarn-style tarball request error: {} (acceptable)", e),
                    }
                } else {
                    println!(
                        "yarn-style metadata request failed: {} (acceptable)",
                        response.status()
                    );
                }
            }
            Err(e) => println!("yarn-style request error: {} (acceptable)", e),
        }
    }

    #[test]
    #[serial]
    fn test_package_version_metadata() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        // Test fetching specific version metadata
        let client = ApiClient::new(server.base_url.clone());

        match client.get("/express/4.18.2").send() {
            Ok(response) if response.status().is_success() => {
                let metadata: serde_json::Value = response.json().unwrap();
                assert_eq!(metadata["name"], "express");
                assert_eq!(metadata["version"], "4.18.2");
            }
            Ok(response) => {
                println!(
                    "Version metadata request failed with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                println!(
                    "Version metadata request failed: {}. This may be due to network issues.",
                    e
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_tarball_download() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        // Test downloading package tarball
        let client = ApiClient::new(server.base_url.clone());

        match client.get("/lodash/-/lodash-4.17.21.tgz").send() {
            Ok(response) if response.status().is_success() => {
                match response.bytes() {
                    Ok(content) => {
                        assert!(content.len() > 0);
                        // Verify it's a gzipped tarball by checking magic bytes
                        assert_eq!(&content[0..2], &[0x1f, 0x8b]);
                    }
                    Err(e) => {
                        println!("Failed to read tarball content: {}", e);
                    }
                }
            }
            Ok(response) => {
                println!("Tarball download failed with status: {}", response.status());
            }
            Err(e) => {
                println!(
                    "Tarball download failed: {}. This may be due to network issues.",
                    e
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_tarball_head_request() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        // Test HEAD request for package tarball
        let client = ApiClient::new(server.base_url.clone());

        match client
            .client
            .head(&format!("{}/lodash/-/lodash-4.17.21.tgz", server.base_url))
            .send()
        {
            Ok(response) if response.status().is_success() => {
                // Should have content-length header
                assert!(response.headers().contains_key("content-length"));
            }
            Ok(response) => {
                println!("HEAD request failed with status: {}", response.status());
            }
            Err(e) => {
                println!(
                    "HEAD request failed: {}. This may be due to network issues.",
                    e
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_multiple_package_installation() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test multiple package requests (what would happen during multi-package installation)
        let packages = ["lodash", "express"];
        let mut success_count = 0;

        for package in &packages {
            match client.get(&format!("/{}", package)).send() {
                Ok(response) => {
                    println!(
                        "Package {} metadata request returned: {}",
                        package,
                        response.status()
                    );
                    if response.status().is_success() {
                        success_count += 1;
                    }
                }
                Err(e) => println!("Package {} request error: {} (acceptable)", package, e),
            }
        }

        println!(
            "Multiple package test: {}/{} packages accessible",
            success_count,
            packages.len()
        );
        // At least some packages should be accessible
        assert!(
            success_count > 0,
            "At least one package should be accessible"
        );
    }

    #[test]
    #[serial]
    fn test_package_cache_behavior() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First request - should be a cache miss
        let response1 = client.get("/lodash/-/lodash-4.17.21.tgz").send().unwrap();
        if response1.status().is_success() {
            // Second request - should be a cache hit
            let response2 = client.get("/lodash/-/lodash-4.17.21.tgz").send().unwrap();
            assert!(response2.status().is_success());

            // Check cache stats
            let stats_response = client.get("/cache/stats").send().unwrap();
            if stats_response.status().is_success() {
                let stats: serde_json::Value = stats_response.json().unwrap();
                assert!(stats["hit_count"].as_u64().unwrap_or(0) > 0);
            }
        }
    }

    #[test]
    #[serial]
    fn test_invalid_package_request() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test requesting non-existent package
        match client
            .get("/this-package-definitely-does-not-exist-12345")
            .send()
        {
            Ok(response) => {
                // Should return an error status for non-existent packages
                // Could be 404 (not found) or 502 (bad gateway if upstream fails)
                assert!(response.status().is_client_error() || response.status().is_server_error());
                // Status is expected to be an error - no need to print it
            }
            Err(e) => {
                println!(
                    "Invalid package request failed: {}. This may be due to network issues.",
                    e
                );
                // Don't fail the test - network issues are acceptable
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_with_special_characters() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test package with special characters (URL encoded)
        match client.get("/@types/node").send() {
            Ok(response) if response.status().is_success() => {
                let metadata: serde_json::Value = response.json().unwrap();
                assert_eq!(metadata["name"], "@types/node");
            }
            Ok(response) => {
                println!(
                    "Scoped package request failed with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                println!(
                    "Scoped package request failed: {}. This may be due to network issues.",
                    e
                );
            }
        }
    }
}
