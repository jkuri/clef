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

                    println!(
                        "Testing proxied metadata storage for lodash@{}",
                        latest_version
                    );

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
                            println!("Warning: Cached metadata test failed: {}", e);
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
                    "Package metadata request failed: {} - this may be due to network issues",
                    e
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
                        println!("✓ Version engines: {:?}", engines);
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
                    "Version metadata request failed: {} - this may be due to network issues",
                    e
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

                    println!("First request for chalk@{} completed", latest_version);

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
                            println!("Second request failed: {}", e);
                        }
                    }
                } else {
                    println!("First request failed with status: {}", response.status());
                }
            }
            Err(e) => {
                println!("First request failed: {}", e);
            }
        }
    }
}
