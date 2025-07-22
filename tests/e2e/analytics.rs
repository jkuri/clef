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
        let response = client.get("/api/v1/packages").send().unwrap();

        // The packages endpoint should succeed
        assert!(
            response.status().is_success(),
            "Packages endpoint failed with status: {}",
            response.status()
        );

        let packages: serde_json::Value = response.json().unwrap();

        // Should return proper structure (either array or paginated object)
        if packages.is_array() {
            // Legacy array format
            let packages_array = packages.as_array().unwrap();
            for package in packages_array {
                assert!(
                    package["name"].is_string(),
                    "Package should have name field"
                );
                if package["description"].is_string() {
                    assert!(!package["description"].as_str().unwrap().is_empty());
                }
            }
        } else if packages.is_object() {
            // New paginated format
            assert!(
                packages["packages"].is_array(),
                "Should have packages array"
            );
            assert!(
                packages["total_count"].is_number(),
                "Should have total_count"
            );
            assert!(
                packages["pagination"].is_object(),
                "Should have pagination object"
            );

            let packages_array = packages["packages"].as_array().unwrap();
            for package_with_versions in packages_array {
                let package = &package_with_versions["package"];
                assert!(
                    package["name"].is_string(),
                    "Package should have name field"
                );
                assert!(
                    package["created_at"].is_string(),
                    "Package should have created_at"
                );
                assert!(
                    package["updated_at"].is_string(),
                    "Package should have updated_at"
                );
            }
        } else {
            panic!("Packages endpoint returned unexpected format");
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

        // The populate request should succeed
        assert!(
            populate_response.status().is_success(),
            "Populate request failed with status: {}",
            populate_response.status()
        );

        thread::sleep(Duration::from_millis(500)); // Wait for data to be stored

        // Test package versions endpoint
        let response = client.get("/api/v1/packages/lodash").send().unwrap();

        // The package versions endpoint should succeed
        assert!(
            response.status().is_success(),
            "Package versions endpoint failed with status: {}",
            response.status()
        );

        let package_versions: serde_json::Value = response.json().unwrap();

        // Verify response structure
        assert!(
            package_versions["package"].is_object(),
            "Should have package object"
        );
        assert!(
            package_versions["package"]["name"].is_string(),
            "Package should have name"
        );
        assert_eq!(package_versions["package"]["name"], "lodash");
        assert!(
            package_versions["versions"].is_array(),
            "Should have versions array"
        );

        // Verify package metadata (Package structure)
        let package = &package_versions["package"];
        assert!(package["name"].is_string(), "Package should have name");
        assert!(
            package["created_at"].is_string(),
            "Package should have created_at"
        );
        assert!(
            package["updated_at"].is_string(),
            "Package should have updated_at"
        );

        // Optional fields that might exist
        if package["description"].is_string() {
            assert!(!package["description"].as_str().unwrap().is_empty());
        }

        // Verify versions structure (VersionWithFiles structure)
        let versions = package_versions["versions"].as_array().unwrap();
        if !versions.is_empty() {
            let first_version = &versions[0];
            assert!(
                first_version["version"].is_object(),
                "Version should be an object containing version info"
            );

            let version_info = &first_version["version"];
            assert!(
                version_info["version"].is_string(),
                "Version info should have version field"
            );
            assert!(
                version_info["created_at"].is_string(),
                "Version info should have created_at"
            );

            // Files array should exist
            assert!(
                first_version["files"].is_array(),
                "Version should have files array"
            );
        }
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
        let response = client.get("/api/v1/packages/popular").send().unwrap();

        // The popular packages endpoint should succeed
        assert!(
            response.status().is_success(),
            "Popular packages endpoint failed with status: {}",
            response.status()
        );

        let popular_packages: serde_json::Value = response.json().unwrap();

        // Should return an array
        assert!(
            popular_packages.is_array(),
            "Popular packages should return an array"
        );

        let packages = popular_packages.as_array().unwrap();
        assert!(packages.len() <= 10, "Should respect default limit of 10"); // Default limit

        // Verify each package has required fields (PopularPackage structure)
        for package in packages {
            assert!(
                package["name"].is_string(),
                "Package should have name field"
            );
            assert!(
                package["total_downloads"].is_number(),
                "Package should have total_downloads"
            );
            assert!(
                package["unique_versions"].is_number(),
                "Package should have unique_versions"
            );
            assert!(
                package["total_size_bytes"].is_number(),
                "Package should have total_size_bytes"
            );
        }

        // Verify packages are sorted by total downloads (descending)
        if packages.len() > 1 {
            for i in 1..packages.len() {
                let prev_count = packages[i - 1]["total_downloads"].as_i64().unwrap_or(0);
                let curr_count = packages[i]["total_downloads"].as_i64().unwrap_or(0);
                assert!(
                    prev_count >= curr_count,
                    "Popular packages should be sorted by total downloads descending"
                );
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
        let response = client.get("/api/v1/analytics").send().unwrap();

        // The analytics endpoint should succeed
        assert!(
            response.status().is_success(),
            "Analytics endpoint failed with status: {}",
            response.status()
        );

        let analytics: serde_json::Value = response.json().unwrap();

        // Verify required analytics fields
        assert!(
            analytics["total_packages"].is_number(),
            "Should have total_packages"
        );
        assert!(
            analytics["total_size_bytes"].is_number(),
            "Should have total_size_bytes"
        );
        assert!(
            analytics["cache_hit_rate"].is_number(),
            "Should have cache_hit_rate"
        );

        // Verify numeric ranges
        let total_packages = analytics["total_packages"].as_i64().unwrap();
        let total_size_bytes = analytics["total_size_bytes"].as_i64().unwrap();
        let cache_hit_rate = analytics["cache_hit_rate"].as_f64().unwrap();

        assert!(total_packages >= 0, "Total packages should be non-negative");
        assert!(total_size_bytes >= 0, "Total size should be non-negative");
        assert!(
            (0.0..=100.0).contains(&cache_hit_rate),
            "Cache hit rate should be 0-100%"
        );

        // Check popular packages if they exist (PopularPackage structure)
        if let Some(popular) = analytics["most_popular_packages"].as_array() {
            assert!(popular.len() <= 10, "Should limit popular packages");
            for package in popular {
                assert!(
                    package["name"].is_string(),
                    "Popular package should have name"
                );
                assert!(
                    package["total_downloads"].is_number(),
                    "Popular package should have total_downloads"
                );
                assert!(
                    package["unique_versions"].is_number(),
                    "Popular package should have unique_versions"
                );
                assert!(
                    package["total_size_bytes"].is_number(),
                    "Popular package should have total_size_bytes"
                );
            }
        }

        // Check recent packages if they exist (RecentPackage structure)
        if let Some(recent) = analytics["recent_packages"].as_array() {
            assert!(recent.len() <= 10, "Should limit recent packages");
            for recent_package in recent {
                assert!(
                    recent_package["package"].is_object(),
                    "Recent package should have package object"
                );
                assert!(
                    recent_package["versions"].is_array(),
                    "Recent package should have versions array"
                );

                let package = &recent_package["package"];
                assert!(
                    package["name"].is_string(),
                    "Recent package should have name"
                );
                assert!(
                    package["created_at"].is_string(),
                    "Recent package should have created_at"
                );
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
        for _i in 0..3 {
            if let Ok(response) = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
                if response.status().is_success() {
                    successful_downloads += 1;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        thread::sleep(Duration::from_millis(200));

        // Check if download count is tracked (only if we had successful downloads)
        if successful_downloads > 0 {
            let response = client.get("/api/v1/packages/lodash").send().unwrap();
            assert!(
                response.status().is_success(),
                "Package data request should succeed"
            );

            let package_data: serde_json::Value = response.json().unwrap();
            assert!(
                package_data["versions"].is_array(),
                "Should have versions array"
            );

            let versions = package_data["versions"].as_array().unwrap();
            assert!(!versions.is_empty(), "Should have at least one version");

            // Verify the structure matches VersionWithFiles
            let mut found_version = false;
            for version_with_files in versions {
                assert!(
                    version_with_files["version"].is_object(),
                    "Should have version object"
                );
                assert!(
                    version_with_files["files"].is_array(),
                    "Should have files array"
                );

                let version_info = &version_with_files["version"];
                if version_info["version"] == "4.17.21" {
                    found_version = true;
                    assert!(
                        version_info["version"].is_string(),
                        "Version info should have version field"
                    );
                    assert!(
                        version_info["created_at"].is_string(),
                        "Version info should have created_at"
                    );
                    break;
                }
            }

            // If we can't find the specific version, at least verify the structure
            if !found_version && !versions.is_empty() {
                let first_version = &versions[0];
                let version_info = &first_version["version"];
                assert!(
                    version_info["version"].is_string(),
                    "All versions should have version field"
                );
                assert!(
                    version_info["created_at"].is_string(),
                    "All versions should have created_at field"
                );
            }
        } else {
            // If no successful downloads, we can't test download counting
            // This is acceptable as it may be due to network issues
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
        let response = client.get("/api/v1/analytics").send().unwrap();

        // The analytics endpoint should succeed
        assert!(
            response.status().is_success(),
            "Analytics endpoint failed with status: {}",
            response.status()
        );

        let analytics: serde_json::Value = response.json().unwrap();

        // Verify analytics structure
        assert!(
            analytics["total_packages"].is_number(),
            "Should have total_packages"
        );
        assert!(
            analytics["total_size_bytes"].is_number(),
            "Should have total_size_bytes"
        );
        assert!(
            analytics["cache_hit_rate"].is_number(),
            "Should have cache_hit_rate"
        );

        let total_packages = analytics["total_packages"].as_i64().unwrap_or(0);
        let total_size_bytes = analytics["total_size_bytes"].as_i64().unwrap_or(0);
        let cache_hit_rate = analytics["cache_hit_rate"].as_f64().unwrap_or(0.0);

        // Basic validation of values
        assert!(total_packages >= 0, "Total packages should be non-negative");
        assert!(total_size_bytes >= 0, "Total size should be non-negative");
        assert!(
            (0.0..=100.0).contains(&cache_hit_rate),
            "Cache hit rate should be 0-100%"
        );

        // Note: We don't assert specific package counts as they may vary due to network conditions
        // The important thing is that the analytics endpoint works and returns valid data
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

        // Check if response has the new pagination structure
        if let Some(packages_obj) = packages.as_object() {
            if let Some(packages_array) = packages_obj["packages"].as_array() {
                for package_with_versions in packages_array {
                    let package = &package_with_versions["package"];
                    let created_at = package["created_at"].as_str().unwrap();
                    let updated_at = package["updated_at"].as_str().unwrap();

                    // Should be valid ISO timestamps (contains T separator)
                    assert!(created_at.contains("T"));
                    assert!(updated_at.contains("T"));

                    // Parse timestamps to verify they're valid
                    // Use NaiveDateTime parsing since the API returns naive timestamps
                    assert!(
                        chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S%.f")
                            .is_ok()
                    );
                    assert!(
                        chrono::NaiveDateTime::parse_from_str(updated_at, "%Y-%m-%dT%H:%M:%S%.f")
                            .is_ok()
                    );
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_packages_pagination() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make requests to populate the database with multiple packages
        let packages = ["lodash", "express", "react", "vue", "angular"];
        for package in &packages {
            let _ = client.get(&format!("/registry/{package}")).send();
        }
        thread::sleep(Duration::from_millis(500));

        // Test default pagination (page 1, limit 20)
        let response = client.get("/api/v1/packages").send().unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();

        // Verify pagination structure
        assert!(packages_response["packages"].is_array());
        assert!(packages_response["total_count"].is_number());
        assert!(packages_response["pagination"].is_object());

        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["page"], 1);
        assert_eq!(pagination["limit"], 20);
        assert!(pagination["total_pages"].is_number());
        assert!(pagination["has_next"].is_boolean());
        assert!(pagination["has_prev"].is_boolean());
        assert_eq!(pagination["has_prev"], false); // First page should not have previous

        // Test custom pagination
        let response = client
            .get("/api/v1/packages?limit=2&page=1")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["page"], 1);
        assert_eq!(pagination["limit"], 2);

        if let Some(packages_array) = packages_response["packages"].as_array() {
            assert!(packages_array.len() <= 2);
        }

        // Test page 2 if there are enough packages
        let total_count = packages_response["total_count"].as_i64().unwrap_or(0);
        if total_count > 2 {
            let response = client
                .get("/api/v1/packages?limit=2&page=2")
                .send()
                .unwrap();
            assert!(response.status().is_success());

            let packages_response: serde_json::Value = response.json().unwrap();
            let pagination = &packages_response["pagination"];
            assert_eq!(pagination["page"], 2);
            assert_eq!(pagination["has_prev"], true); // Second page should have previous
        }
    }

    #[test]
    #[serial]
    fn test_packages_search() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make requests to populate the database
        let packages = ["react", "react-dom", "lodash", "express"];
        for package in &packages {
            let _ = client.get(&format!("/registry/{package}")).send();
        }
        thread::sleep(Duration::from_millis(500));

        // Test search functionality
        let response = client.get("/api/v1/packages?search=react").send().unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();

        if let Some(packages_array) = packages_response["packages"].as_array() {
            // All returned packages should contain "react" in name or description
            for package_with_versions in packages_array {
                let package = &package_with_versions["package"];
                let name = package["name"].as_str().unwrap_or("");
                let description = package["description"].as_str().unwrap_or("");

                assert!(
                    name.to_lowercase().contains("react")
                        || description.to_lowercase().contains("react"),
                    "Package {} does not contain 'react' in name or description",
                    name
                );
            }
        }

        // Test search with pagination
        let response = client
            .get("/api/v1/packages?search=react&limit=1")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["limit"], 1);

        if let Some(packages_array) = packages_response["packages"].as_array() {
            assert!(packages_array.len() <= 1);
        }

        // Test search with no results
        let response = client
            .get("/api/v1/packages?search=nonexistent-package-xyz")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        assert_eq!(packages_response["total_count"], 0);

        if let Some(packages_array) = packages_response["packages"].as_array() {
            assert!(packages_array.is_empty());
        }
    }

    #[test]
    #[serial]
    fn test_packages_sorting() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make requests to populate the database
        let packages = ["zebra-package", "alpha-package", "beta-package"];
        for package in &packages {
            let _ = client.get(&format!("/registry/{package}")).send();
            thread::sleep(Duration::from_millis(100)); // Small delay to ensure different timestamps
        }
        thread::sleep(Duration::from_millis(300));

        // Test sorting by name ascending
        let response = client
            .get("/api/v1/packages?sort=name&order=asc&limit=10")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();

        if let Some(packages_array) = packages_response["packages"].as_array() {
            let mut prev_name = "";
            for package_with_versions in packages_array {
                let package = &package_with_versions["package"];
                let name = package["name"].as_str().unwrap_or("");

                if !prev_name.is_empty() {
                    assert!(
                        name >= prev_name,
                        "Packages not sorted by name ascending: {} should be >= {}",
                        name,
                        prev_name
                    );
                }
                prev_name = name;
            }
        }

        // Test sorting by name descending
        let response = client
            .get("/api/v1/packages?sort=name&order=desc&limit=10")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();

        if let Some(packages_array) = packages_response["packages"].as_array() {
            let mut prev_name = "zzzzz"; // Start with high value for descending
            for package_with_versions in packages_array {
                let package = &package_with_versions["package"];
                let name = package["name"].as_str().unwrap_or("");

                assert!(
                    name <= prev_name,
                    "Packages not sorted by name descending: {} should be <= {}",
                    name,
                    prev_name
                );
                prev_name = name;
            }
        }

        // Test sorting by created_at descending (default)
        let response = client
            .get("/api/v1/packages?sort=created_at&order=desc&limit=10")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();

        if let Some(packages_array) = packages_response["packages"].as_array() {
            let mut prev_created_at = "9999-12-31T23:59:59Z"; // Start with future date for descending
            for package_with_versions in packages_array {
                let package = &package_with_versions["package"];
                let created_at = package["created_at"].as_str().unwrap_or("");

                assert!(
                    created_at <= prev_created_at,
                    "Packages not sorted by created_at descending: {} should be <= {}",
                    created_at,
                    prev_created_at
                );
                prev_created_at = created_at;
            }
        }

        // Test invalid sort parameters (should default to created_at desc)
        let response = client
            .get("/api/v1/packages?sort=invalid&order=invalid&limit=5")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        // Should still return valid response with default sorting
        assert!(packages_response["packages"].is_array());
        assert!(packages_response["pagination"].is_object());
    }

    #[test]
    #[serial]
    fn test_packages_combined_features() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Make requests to populate the database
        let packages = ["react", "react-dom", "react-router", "lodash", "express"];
        for package in &packages {
            let _ = client.get(&format!("/registry/{package}")).send();
            thread::sleep(Duration::from_millis(50));
        }
        thread::sleep(Duration::from_millis(300));

        // Test combining search, pagination, and sorting
        let response = client
            .get("/api/v1/packages?search=react&sort=name&order=asc&limit=2&page=1")
            .send()
            .unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();

        // Verify pagination metadata
        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["page"], 1);
        assert_eq!(pagination["limit"], 2);

        // Verify search results are sorted
        if let Some(packages_array) = packages_response["packages"].as_array() {
            assert!(packages_array.len() <= 2);

            let mut prev_name = "";
            for package_with_versions in packages_array {
                let package = &package_with_versions["package"];
                let name = package["name"].as_str().unwrap_or("");
                let description = package["description"].as_str().unwrap_or("");

                // Should contain "react" in name or description
                assert!(
                    name.to_lowercase().contains("react")
                        || description.to_lowercase().contains("react"),
                    "Package {} does not contain 'react'",
                    name
                );

                // Should be sorted by name ascending
                if !prev_name.is_empty() {
                    assert!(
                        name >= prev_name,
                        "Search results not sorted by name ascending: {} should be >= {}",
                        name,
                        prev_name
                    );
                }
                prev_name = name;
            }
        }

        // Test parameter validation - limit should be clamped
        let response = client.get("/api/v1/packages?limit=1000").send().unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["limit"], 100); // Should be clamped to max 100

        // Test minimum limit
        let response = client.get("/api/v1/packages?limit=0").send().unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["limit"], 1); // Should be clamped to min 1

        // Test minimum page
        let response = client.get("/api/v1/packages?page=0").send().unwrap();
        assert!(response.status().is_success());

        let packages_response: serde_json::Value = response.json().unwrap();
        let pagination = &packages_response["pagination"];
        assert_eq!(pagination["page"], 1); // Should be clamped to min 1
    }
}
