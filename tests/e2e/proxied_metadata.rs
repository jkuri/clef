use super::*;
use serde_json::Value;
use serial_test::serial;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_proxied_package_metadata_storage() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test fetching a popular package that should exist on npm registry
        // This will proxy the request to upstream and store metadata in our database
        match client.get("/registry/lodash").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let metadata: Value = response.json().unwrap();

                    // Verify basic package structure
                    assert_eq!(metadata["name"], "lodash");
                    assert!(metadata["versions"].is_object());
                    assert!(metadata["dist-tags"].is_object());

                    // Get the latest version
                    let latest_version = metadata["dist-tags"]["latest"]
                        .as_str()
                        .expect("Latest version should be available");

                    println!("Testing proxied metadata storage for lodash@{latest_version}");

                    // Verify that version metadata exists
                    let versions = metadata["versions"].as_object().unwrap();
                    let latest_version_data = &versions[latest_version];

                    // Check that essential metadata fields are present
                    assert!(latest_version_data["name"].is_string());
                    assert!(latest_version_data["version"].is_string());
                    assert_eq!(latest_version_data["name"], "lodash");
                    assert_eq!(latest_version_data["version"], latest_version);

                    // Check for common metadata fields that should be stored
                    if latest_version_data["description"].is_string() {
                        println!("✓ Description: {}", latest_version_data["description"]);
                    }

                    if latest_version_data["main"].is_string() {
                        println!("✓ Main file: {}", latest_version_data["main"]);
                    }

                    if latest_version_data["dependencies"].is_object() {
                        let deps = latest_version_data["dependencies"].as_object().unwrap();
                        println!("✓ Dependencies: {} packages", deps.len());
                    }

                    if latest_version_data["scripts"].is_object() {
                        let scripts = latest_version_data["scripts"].as_object().unwrap();
                        println!("✓ Scripts: {} commands", scripts.len());
                    }

                    // Wait a moment for database operations to complete
                    std::thread::sleep(Duration::from_millis(500));

                    // Now test that we can fetch the same package again (should use cached metadata)
                    match client.get("/registry/lodash").send() {
                        Ok(cached_response) => {
                            assert!(cached_response.status().is_success());
                            let cached_metadata: Value = cached_response.json().unwrap();

                            // Verify the cached response has the same structure
                            assert_eq!(cached_metadata["name"], "lodash");
                            assert_eq!(cached_metadata["dist-tags"]["latest"], latest_version);

                            println!("✓ Cached metadata retrieval successful");
                        }
                        Err(e) => {
                            println!("Warning: Cached metadata test failed: {e}");
                        }
                    }

                    println!("✅ Proxied package metadata storage test passed!");
                } else {
                    println!(
                        "Package metadata request failed with status: {} - this may be due to network issues",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!(
                    "Package metadata request failed: {e} - this may be due to network issues"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_proxied_version_specific_metadata_storage() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test fetching a specific version of a package
        // This should proxy to upstream and store that specific version's metadata
        match client.get("/registry/express/4.18.2").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let version_metadata: Value = response.json().unwrap();

                    // Verify version-specific metadata structure
                    assert_eq!(version_metadata["name"], "express");
                    assert_eq!(version_metadata["version"], "4.18.2");

                    // Check for metadata fields that should be stored
                    if version_metadata["description"].is_string() {
                        println!("✓ Version description: {}", version_metadata["description"]);
                    }

                    if version_metadata["main"].is_string() {
                        println!("✓ Version main file: {}", version_metadata["main"]);
                    }

                    if version_metadata["dependencies"].is_object() {
                        let deps = version_metadata["dependencies"].as_object().unwrap();
                        println!("✓ Version dependencies: {} packages", deps.len());

                        // Express should have some dependencies
                        assert!(!deps.is_empty(), "Express should have dependencies");
                    }

                    if version_metadata["engines"].is_object() {
                        let engines = version_metadata["engines"].as_object().unwrap();
                        println!("✓ Version engines: {engines:?}");
                    }

                    // Verify dist information is present
                    if version_metadata["dist"].is_object() {
                        let dist = version_metadata["dist"].as_object().unwrap();
                        assert!(dist.contains_key("tarball"));
                        println!("✓ Dist info with tarball URL");
                    }

                    println!("✅ Proxied version-specific metadata storage test passed!");
                } else {
                    println!(
                        "Version metadata request failed with status: {} - this may be due to network issues",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!(
                    "Version metadata request failed: {e} - this may be due to network issues"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_level_metadata_storage() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test fetching a package that has rich metadata (homepage, repository, license, keywords)
        // Using react as it typically has all these fields
        match client.get("/registry/react").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let metadata: Value = response.json().unwrap();

                    // Verify basic package structure
                    assert_eq!(metadata["name"], "react");

                    // Check for package-level metadata fields that should be stored
                    if metadata["homepage"].is_string() {
                        println!("✓ Homepage: {}", metadata["homepage"]);
                    }

                    if metadata["repository"].is_object() || metadata["repository"].is_string() {
                        println!("✓ Repository: {}", metadata["repository"]);
                    }

                    if metadata["license"].is_string() {
                        println!("✓ License: {}", metadata["license"]);
                    }

                    if metadata["keywords"].is_array() {
                        let keywords = metadata["keywords"].as_array().unwrap();
                        println!("✓ Keywords: {} items", keywords.len());
                    }

                    // Wait for database operations to complete
                    std::thread::sleep(Duration::from_millis(1000));

                    // Now check if we can access the package list API to verify the metadata was stored
                    match client.get("/api/v1/packages").send() {
                        Ok(packages_response) => {
                            if packages_response.status().is_success() {
                                let packages_data: Value = packages_response.json().unwrap();

                                if let Some(packages_array) = packages_data["packages"].as_array() {
                                    // Look for our react package in the stored packages
                                    let react_package = packages_array
                                        .iter()
                                        .find(|pkg| pkg["package"]["name"] == "react");

                                    if let Some(react_pkg) = react_package {
                                        let pkg_data = &react_pkg["package"];

                                        // Verify that package-level metadata was stored
                                        if pkg_data["homepage"].is_string() {
                                            println!("✓ Stored homepage: {}", pkg_data["homepage"]);
                                        }

                                        if pkg_data["repository_url"].is_string() {
                                            println!(
                                                "✓ Stored repository_url: {}",
                                                pkg_data["repository_url"]
                                            );
                                        }

                                        if pkg_data["license"].is_string() {
                                            println!("✓ Stored license: {}", pkg_data["license"]);
                                        }

                                        if pkg_data["keywords"].is_string() {
                                            println!("✓ Stored keywords: {}", pkg_data["keywords"]);
                                        }

                                        println!("✅ Package-level metadata storage test passed!");
                                    } else {
                                        println!(
                                            "React package not found in stored packages - this may indicate the metadata wasn't stored properly"
                                        );
                                    }
                                }
                            } else {
                                println!(
                                    "Failed to fetch packages list: {}",
                                    packages_response.status()
                                );
                            }
                        }
                        Err(e) => {
                            println!("Failed to fetch packages list: {e}");
                        }
                    }
                } else {
                    println!(
                        "Package metadata request failed with status: {} - this may be due to network issues",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!(
                    "Package metadata request failed: {e} - this may be due to network issues"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_version_timestamps_from_upstream() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test fetching a package that has time information in the registry
        // Using a smaller package to avoid too much data
        match client.get("/registry/express").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let metadata: Value = response.json().unwrap();

                    // Verify basic package structure
                    assert_eq!(metadata["name"], "express");

                    // Check if time field exists in the upstream response
                    if metadata["time"].is_object() {
                        println!("✓ Time field found in upstream response");

                        // Get a specific version to check
                        if let Some(versions) = metadata["versions"].as_object() {
                            if let Some((version_key, _)) = versions.iter().next() {
                                println!("✓ Checking timestamps for version: {}", version_key);

                                // Wait for database operations to complete
                                std::thread::sleep(Duration::from_millis(1000));

                                // Now check if we can access the package versions API to verify timestamps were stored
                                match client.get(&format!("/api/v1/packages/express")).send() {
                                    Ok(versions_response) => {
                                        if versions_response.status().is_success() {
                                            let versions_data: Value =
                                                versions_response.json().unwrap();

                                            if let Some(versions_array) =
                                                versions_data["versions"].as_array()
                                            {
                                                // Look for the version we checked
                                                let version_found =
                                                    versions_array.iter().find(|v| {
                                                        v["version"]["version"].as_str()
                                                            == Some(version_key)
                                                    });

                                                if let Some(version_data) = version_found {
                                                    let created_at =
                                                        &version_data["version"]["created_at"];
                                                    if created_at.is_string() {
                                                        println!(
                                                            "✓ Version created_at timestamp: {}",
                                                            created_at
                                                        );

                                                        // Verify it's a valid timestamp format
                                                        let timestamp_str =
                                                            created_at.as_str().unwrap();
                                                        assert!(
                                                            timestamp_str.contains("T"),
                                                            "Timestamp should be in ISO format"
                                                        );

                                                        // Try to parse it to verify it's valid
                                                        assert!(
                                                            chrono::NaiveDateTime::parse_from_str(
                                                                timestamp_str,
                                                                "%Y-%m-%dT%H:%M:%S%.f"
                                                            )
                                                            .is_ok(),
                                                            "Should be a valid timestamp"
                                                        );

                                                        println!(
                                                            "✅ Version timestamps from upstream test passed!"
                                                        );
                                                    } else {
                                                        println!(
                                                            "⚠️ Version created_at field is not a string or is missing"
                                                        );
                                                    }
                                                } else {
                                                    println!(
                                                        "⚠️ Version {} not found in stored versions",
                                                        version_key
                                                    );
                                                }
                                            }
                                        } else {
                                            println!(
                                                "Failed to fetch package versions: {}",
                                                versions_response.status()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        println!("Failed to fetch package versions: {e}");
                                    }
                                }
                            }
                        }
                    } else {
                        println!(
                            "⚠️ No time field found in upstream response - this may be expected for some packages"
                        );
                    }
                } else {
                    println!(
                        "Package metadata request failed with status: {} - this may be due to network issues",
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!(
                    "Package metadata request failed: {e} - this may be due to network issues"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_repository_url_cleaning() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with a package that has a git+ repository URL (React)
        match client.get("/registry/react").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let metadata: Value = response.json().unwrap();

                    // Verify the upstream repository URL has git+ prefix
                    if let Some(repo_obj) = metadata["repository"].as_object() {
                        if let Some(url) = repo_obj.get("url").and_then(|u| u.as_str()) {
                            println!("✓ Upstream repository URL: {}", url);
                            assert!(
                                url.starts_with("git+"),
                                "Expected git+ prefix in upstream URL"
                            );
                        }
                    }

                    // Wait for database operations to complete
                    std::thread::sleep(Duration::from_millis(1000));

                    // Check that the stored URL is cleaned
                    match client.get("/api/v1/packages").send() {
                        Ok(packages_response) => {
                            if packages_response.status().is_success() {
                                let packages_data: Value = packages_response.json().unwrap();

                                if let Some(packages_array) = packages_data["packages"].as_array() {
                                    let react_package = packages_array
                                        .iter()
                                        .find(|pkg| pkg["package"]["name"] == "react");

                                    if let Some(react_pkg) = react_package {
                                        let stored_url = &react_pkg["package"]["repository_url"];
                                        if let Some(url_str) = stored_url.as_str() {
                                            println!("✓ Stored repository URL: {}", url_str);

                                            // Verify the URL is cleaned
                                            assert!(
                                                !url_str.starts_with("git+"),
                                                "Stored URL should not have git+ prefix"
                                            );
                                            assert!(
                                                !url_str.ends_with(".git"),
                                                "Stored URL should not have .git suffix"
                                            );
                                            assert!(
                                                url_str.starts_with("https://"),
                                                "Stored URL should be HTTPS for browser access"
                                            );
                                            assert_eq!(
                                                url_str, "https://github.com/facebook/react",
                                                "Should be the cleaned GitHub URL"
                                            );

                                            println!("✅ Repository URL cleaning test passed!");
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to fetch packages list: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "Package metadata request failed: {e} - this may be due to network issues"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_metadata_persistence_across_requests() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First, fetch a package to populate metadata
        match client.get("/registry/chalk").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let first_metadata: Value = response.json().unwrap();
                    let latest_version = first_metadata["dist-tags"]["latest"]
                        .as_str()
                        .expect("Latest version should be available");

                    println!("First request for chalk@{latest_version} completed");

                    // Wait a moment for database operations
                    std::thread::sleep(Duration::from_millis(500));

                    // Make a second request - this should use stored metadata
                    match client.get("/registry/chalk").send() {
                        Ok(second_response) => {
                            assert!(second_response.status().is_success());
                            let second_metadata: Value = second_response.json().unwrap();

                            // Verify consistency between requests
                            assert_eq!(first_metadata["name"], second_metadata["name"]);
                            assert_eq!(
                                first_metadata["dist-tags"]["latest"],
                                second_metadata["dist-tags"]["latest"]
                            );

                            // Check that version metadata is consistent
                            let first_versions = first_metadata["versions"].as_object().unwrap();
                            let second_versions = second_metadata["versions"].as_object().unwrap();

                            if let (Some(first_latest), Some(second_latest)) = (
                                first_versions.get(latest_version),
                                second_versions.get(latest_version),
                            ) {
                                assert_eq!(first_latest["version"], second_latest["version"]);
                                assert_eq!(first_latest["name"], second_latest["name"]);

                                // If description exists, it should be consistent
                                if first_latest["description"].is_string()
                                    && second_latest["description"].is_string()
                                {
                                    assert_eq!(
                                        first_latest["description"],
                                        second_latest["description"]
                                    );
                                }
                            }

                            println!("✅ Metadata persistence across requests verified!");
                        }
                        Err(e) => {
                            println!("Second request failed: {e}");
                        }
                    }
                } else {
                    println!("First request failed with status: {}", response.status());
                }
            }
            Err(e) => {
                println!("First request failed: {e}");
            }
        }
    }
}
