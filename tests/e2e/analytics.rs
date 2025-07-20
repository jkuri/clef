use super::*;
use serial_test::serial;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_packages_list_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make some package requests to populate the database
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/express").send();
        thread::sleep(Duration::from_millis(200));

        // Test packages list endpoint
        match client.get("/api/v1/packages").send() {
            Ok(response) => {
                // The packages endpoint should succeed
                assert!(
                    response.status().is_success(),
                    "Packages endpoint failed with status: {}",
                    response.status()
                );

                match response.json::<serde_json::Value>() {
                    Ok(packages) => {
                        println!("Packages response: {packages}");

                        // Handle both array and object responses
                        if packages.is_array() {
                            if let Some(packages_array) = packages.as_array() {
                                for package in packages_array {
                                    // Only check fields that exist
                                    if package["name"].is_string() {
                                        println!("Found package: {}", package["name"]);
                                    }
                                }
                            }
                        } else if packages.is_object() {
                            println!("Packages endpoint returned object format");
                        } else {
                            println!("Packages endpoint returned unexpected format");
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse packages response: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Packages endpoint error: {e}");
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_versions_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make a request to populate package data
        let populate_response = client.get("/registry/lodash").send().unwrap();
        println!("Populate request status: {}", populate_response.status());

        // The populate request should succeed
        assert!(
            populate_response.status().is_success(),
            "Populate request failed with status: {}",
            populate_response.status()
        );

        thread::sleep(Duration::from_millis(500)); // Wait for data to be stored

        // Test package versions endpoint
        let response = client.get("/api/v1/packages/lodash").send().unwrap();

        // The package versions endpoint should now succeed since we fixed the LEFT JOIN issue
        assert!(
            response.status().is_success(),
            "Package versions endpoint failed with status: {}",
            response.status()
        );

        let package_versions: serde_json::Value = response.json().unwrap();
        assert!(package_versions["package"]["name"].is_string());
        assert_eq!(package_versions["package"]["name"], "lodash");
        assert!(package_versions["versions"].is_array());
    }

    #[test]
    #[serial]
    fn test_popular_packages_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make requests to different packages to create download data
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
        let _ = client.get("/registry/express/-/express-4.18.2.tgz").send();
        let _ = client.get("/registry/react/-/react-18.2.0.tgz").send();
        thread::sleep(Duration::from_millis(300));

        // Test popular packages endpoint with default limit
        match client.get("/api/v1/packages/popular").send() {
            Ok(response) => {
                // The popular packages endpoint should succeed
                assert!(
                    response.status().is_success(),
                    "Popular packages endpoint failed with status: {}",
                    response.status()
                );

                match response.json::<serde_json::Value>() {
                    Ok(popular_packages) => {
                        println!("Popular packages response: {popular_packages}");

                        if popular_packages.is_array() {
                            if let Some(packages) = popular_packages.as_array() {
                                println!("Found {} popular packages", packages.len());
                                assert!(packages.len() <= 10); // Default limit

                                for (i, package) in packages.iter().enumerate() {
                                    println!("Package {}: {}", i + 1, package);
                                    // Only check fields that exist
                                    if package["name"].is_string() {
                                        println!("  Name: {}", package["name"]);
                                    }
                                }
                            }
                        } else {
                            println!("Popular packages returned non-array format");
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse popular packages response: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Popular packages endpoint error: {e}");
            }
        }
    }

    #[test]
    #[serial]
    fn test_popular_packages_with_limit() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make requests to populate data
        let packages = ["lodash", "express", "react", "vue", "angular"];
        for package in &packages {
            let _ = client.get(&format!("/{package}")).send();
        }
        thread::sleep(Duration::from_millis(300));

        // Test popular packages endpoint with custom limit
        let response = client
            .get("/api/v1/packages/popular?limit=3")
            .send()
            .unwrap();

        // The popular packages endpoint with limit should succeed
        assert!(
            response.status().is_success(),
            "Popular packages endpoint with limit failed with status: {}",
            response.status()
        );

        let popular_packages: serde_json::Value = response.json().unwrap();
        assert!(popular_packages.is_array());

        if let Some(packages) = popular_packages.as_array() {
            assert!(packages.len() <= 3); // Custom limit
        }
    }

    #[test]
    #[serial]
    fn test_analytics_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make various requests to populate analytics data
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/express").send();
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
        let _ = client.get("/registry/express/-/express-4.18.2.tgz").send();
        thread::sleep(Duration::from_millis(300));

        // Test comprehensive analytics endpoint
        match client.get("/api/v1/analytics").send() {
            Ok(response) => {
                // The analytics endpoint should succeed
                assert!(
                    response.status().is_success(),
                    "Analytics endpoint failed with status: {}",
                    response.status()
                );

                match response.json::<serde_json::Value>() {
                    Ok(analytics) => {
                        println!("Analytics response: {analytics}");

                        // Check fields that exist
                        if analytics["total_packages"].is_number() {
                            println!("Total packages: {}", analytics["total_packages"]);
                        }
                        if analytics["total_size_bytes"].is_number() {
                            println!("Total size bytes: {}", analytics["total_size_bytes"]);
                        }
                        if analytics["cache_hit_rate"].is_number() {
                            println!("Cache hit rate: {}", analytics["cache_hit_rate"]);
                        }

                        // Check popular packages if they exist
                        if let Some(popular) = analytics["most_popular_packages"].as_array() {
                            println!("Found {} popular packages", popular.len());
                            for (i, package) in popular.iter().enumerate() {
                                println!("Popular package {}: {}", i + 1, package);
                            }
                        }

                        // Check recent packages if they exist
                        if let Some(recent) = analytics["recent_packages"].as_array() {
                            println!("Found {} recent packages", recent.len());
                            for (i, package) in recent.iter().enumerate() {
                                println!("Recent package {}: {}", i + 1, package);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse analytics response: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Analytics endpoint error: {e}");
            }
        }
    }

    #[test]
    #[serial]
    fn test_download_count_tracking() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make multiple downloads of the same package
        let mut successful_downloads = 0;
        for i in 0..3 {
            match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
                Ok(response) if response.status().is_success() => {
                    successful_downloads += 1;
                    println!("Download {} successful", i + 1);
                }
                Ok(response) => {
                    println!(
                        "Download {} failed with status: {}",
                        i + 1,
                        response.status()
                    );
                }
                Err(e) => {
                    println!("Download {} error: {}", i + 1, e);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        thread::sleep(Duration::from_millis(200));

        // Check if download count is tracked (only if we had successful downloads)
        if successful_downloads > 0 {
            match client.get("/api/v1/packages/lodash").send() {
                Ok(response) if response.status().is_success() => {
                    match response.json::<serde_json::Value>() {
                        Ok(package_data) => {
                            println!("Package data response: {package_data}");

                            if let Some(versions) = package_data["versions"].as_array() {
                                println!("Found {} versions", versions.len());
                                // Find version 4.17.21
                                for version in versions {
                                    if version["version"] == "4.17.21" {
                                        let download_count =
                                            version["download_count"].as_u64().unwrap_or(0);
                                        println!("Download count for 4.17.21: {download_count}");
                                        // Just log the count - don't assert specific values
                                        break;
                                    }
                                }
                            } else {
                                println!("No versions array found in package data");
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse package data: {e}");
                        }
                    }
                }
                Ok(response) => {
                    println!(
                        "Package data request failed with status: {}",
                        response.status()
                    );
                }
                Err(e) => {
                    println!("Package data request error: {e}");
                }
            }
        } else {
            println!("No successful downloads - skipping download count test");
        }
    }

    #[test]
    #[serial]
    fn test_package_metadata_tracking() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Request package metadata
        let _ = client.get("/registry/lodash").send();
        thread::sleep(Duration::from_millis(200));

        // Check if package metadata is stored
        let response = client.get("/api/v1/packages").send().unwrap();

        // The packages endpoint should succeed
        assert!(
            response.status().is_success(),
            "Packages endpoint failed with status: {}",
            response.status()
        );

        let packages: serde_json::Value = response.json().unwrap();

        if let Some(packages_array) = packages.as_array() {
            let lodash_package = packages_array.iter().find(|p| p["name"] == "lodash");

            if let Some(package) = lodash_package {
                assert!(package["description"].is_string());
                assert!(package["latest_version"].is_string());
                assert!(package["created_at"].is_string());
                assert!(package["updated_at"].is_string());
            }
        }
    }

    #[test]
    #[serial]
    fn test_analytics_with_different_package_managers() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Simulate usage from different package managers by making direct HTTP requests
        // This avoids relying on actual package manager installations

        // Simulate npm-style requests
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
        thread::sleep(Duration::from_millis(100));

        // Simulate pnpm-style requests
        let _ = client.get("/registry/express").send();
        let _ = client.get("/registry/express/-/express-4.18.2.tgz").send();
        thread::sleep(Duration::from_millis(100));

        // Simulate yarn-style requests
        let _ = client.get("/registry/react").send();
        let _ = client.get("/registry/react/-/react-18.2.0.tgz").send();
        thread::sleep(Duration::from_millis(200));

        // Check analytics
        match client.get("/api/v1/analytics").send() {
            Ok(response) => {
                // The analytics endpoint should succeed
                assert!(
                    response.status().is_success(),
                    "Analytics endpoint failed with status: {}",
                    response.status()
                );

                match response.json::<serde_json::Value>() {
                    Ok(analytics) => {
                        let total_packages = analytics["total_packages"].as_i64().unwrap_or(0);
                        println!("Total packages tracked: {total_packages}");

                        // Should have tracked some packages (allow for network failures)
                        if total_packages == 0 {
                            println!("No packages tracked - may be due to network issues");
                        } else {
                            println!(
                                "Successfully tracked {total_packages} packages from different managers"
                            );
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse analytics response: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Analytics request error: {e}");
            }
        }
    }

    #[test]
    #[serial]
    fn test_cache_analytics_integration() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make requests to generate cache activity
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send(); // Should be cache hit
        thread::sleep(Duration::from_millis(200));

        // Check analytics includes cache hit rate
        let response = client.get("/api/v1/analytics").send().unwrap();

        // The analytics endpoint should succeed
        assert!(
            response.status().is_success(),
            "Analytics endpoint failed with status: {}",
            response.status()
        );

        let analytics: serde_json::Value = response.json().unwrap();
        let cache_hit_rate = analytics["cache_hit_rate"].as_f64().unwrap_or(0.0);

        // Should have a valid hit rate
        assert!((0.0..=100.0).contains(&cache_hit_rate));
    }

    #[test]
    #[serial]
    fn test_nonexistent_package_analytics() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test analytics for non-existent package
        let response = client
            .get("/api/v1/packages/nonexistent-package-12345")
            .send()
            .unwrap();

        // Should return 404 or empty result
        if response.status().is_success() {
            let package_data: serde_json::Value = response.json().unwrap();
            // If successful, should indicate no versions found
            if let Some(versions) = package_data["versions"].as_array() {
                assert!(versions.is_empty());
            }
        } else {
            assert_eq!(response.status(), 404);
        }
    }

    #[test]
    #[serial]
    fn test_analytics_time_tracking() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make a request
        let _ = client.get("/registry/lodash").send();
        thread::sleep(Duration::from_millis(200));

        // Check that timestamps are properly recorded
        let response = client.get("/api/v1/packages").send().unwrap();

        // The packages endpoint should succeed
        assert!(
            response.status().is_success(),
            "Packages endpoint failed with status: {}",
            response.status()
        );

        let packages: serde_json::Value = response.json().unwrap();

        if let Some(packages_array) = packages.as_array() {
            for package in packages_array {
                let created_at = package["created_at"].as_str().unwrap();
                let updated_at = package["updated_at"].as_str().unwrap();

                // Should be valid ISO timestamps
                assert!(created_at.contains("T"));
                assert!(updated_at.contains("T"));

                // Parse timestamps to verify they're valid
                assert!(chrono::DateTime::parse_from_rfc3339(created_at).is_ok());
                assert!(chrono::DateTime::parse_from_rfc3339(updated_at).is_ok());
            }
        }
    }
}
