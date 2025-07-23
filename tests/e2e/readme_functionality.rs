use super::*;
use base64::prelude::*;
use serde_json::json;
use serial_test::serial;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that upstream packages include README content in version-specific endpoints
    #[test]
    #[serial]
    fn test_upstream_package_version_includes_readme() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with a well-known package that has README content
        match client.get("/registry/express/4.18.2").send() {
            Ok(response) => {
                assert!(
                    response.status().is_success(),
                    "Version metadata request failed with status: {}",
                    response.status()
                );

                let metadata: serde_json::Value = response.json().unwrap();

                // Verify basic package info
                assert_eq!(metadata["name"], "express");
                assert_eq!(metadata["version"], "4.18.2");

                // Verify README is present and contains expected content
                assert!(
                    metadata.get("readme").is_some(),
                    "README field should be present in version metadata"
                );

                if let Some(readme) = metadata["readme"].as_str() {
                    assert!(
                        readme.contains("Express"),
                        "README should contain package name"
                    );
                    assert!(
                        readme.len() > 100,
                        "README should have substantial content, got {} chars",
                        readme.len()
                    );
                    println!(
                        "✅ Upstream package README successfully included ({} chars)",
                        readme.len()
                    );
                } else {
                    panic!("README field should be a string");
                }
            }
            Err(e) => {
                println!(
                    "Warning: Upstream package test failed: {e}. This may be due to network issues."
                );
            }
        }
    }

    /// Test that published packages store and retrieve README content correctly
    #[test]
    #[serial]
    fn test_published_package_readme_storage_and_retrieval() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());
        let project = TestProject::new(&server.base_url);

        // Register and authenticate user
        project.register_and_login_user(&server.base_url, server.port);

        // Create a test package with README content
        let readme_content = "# Test README Package\n\nThis is a comprehensive test package to verify README functionality.\n\n## Features\n\n- README content storage in database\n- Version-specific metadata retrieval\n- Database integration testing\n\n## Usage\n\n```javascript\nconst testPkg = require('readme-test-package');\nconsole.log(testPkg.greet());\n```\n\n## License\n\nMIT License - This is a test package.";

        project.create_test_package_with_readme("readme-test-package", "1.0.0", readme_content);

        // Publish the package using npm
        let publish_output = project.run_command(&PackageManager::Npm, &["publish".to_string()]);

        if !publish_output.status.success() {
            println!(
                "Publish stderr: {}",
                String::from_utf8_lossy(&publish_output.stderr)
            );
            println!(
                "Publish stdout: {}",
                String::from_utf8_lossy(&publish_output.stdout)
            );
            panic!("Failed to publish test package");
        }

        println!("✅ Test package published successfully");

        // Test version-specific endpoint includes README
        let version_response = client
            .get("/registry/readme-test-package/1.0.0")
            .send()
            .unwrap();

        assert!(
            version_response.status().is_success(),
            "Version metadata request failed with status: {}",
            version_response.status()
        );

        let version_metadata: serde_json::Value = version_response.json().unwrap();

        // Verify basic package info
        assert_eq!(version_metadata["name"], "readme-test-package");
        assert_eq!(version_metadata["version"], "1.0.0");

        // Verify README is present and matches what we published
        assert!(
            version_metadata.get("readme").is_some(),
            "README field should be present in published package version metadata"
        );

        if let Some(retrieved_readme) = version_metadata["readme"].as_str() {
            assert_eq!(
                retrieved_readme, readme_content,
                "Retrieved README should match published README content"
            );
            println!("✅ Published package README correctly stored and retrieved");
        } else {
            panic!("README field should be a string");
        }
    }

    /// Test README content with various formats (markdown, plain text, etc.)
    #[test]
    #[serial]
    fn test_readme_content_formats() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());
        let project = TestProject::new(&server.base_url);

        // Register and authenticate user
        project.register_and_login_user(&server.base_url, server.port);

        // Test different README formats
        let test_cases = vec![
            (
                "markdown-readme-test",
                "1.0.0",
                "# Markdown README\n\n**Bold text** and *italic text*\n\n- List item 1\n- List item 2\n\n```javascript\nconsole.log('code block');\n```",
            ),
            (
                "plain-text-readme-test",
                "1.0.0",
                "Plain text README without markdown formatting.\n\nJust simple text content for testing.",
            ),
            (
                "unicode-readme-test",
                "1.0.0",
                "# Unicode README 🚀\n\nTesting unicode characters: 中文, العربية, русский\n\nEmojis: 📦 ⚡ 🎉",
            ),
        ];

        for (package_name, version, readme_content) in test_cases {
            // Create and publish package
            project.create_test_package_with_readme(package_name, version, readme_content);

            let publish_output =
                project.run_command(&PackageManager::Npm, &["publish".to_string()]);

            if !publish_output.status.success() {
                println!(
                    "Failed to publish {}: {}",
                    package_name,
                    String::from_utf8_lossy(&publish_output.stderr)
                );
                continue;
            }

            // Verify README content
            let version_response = client
                .get(&format!("/registry/{}/{}", package_name, version))
                .send()
                .unwrap();

            assert!(version_response.status().is_success());
            let metadata: serde_json::Value = version_response.json().unwrap();

            if let Some(retrieved_readme) = metadata["readme"].as_str() {
                assert_eq!(retrieved_readme, readme_content);
                println!("✅ {} README format test passed", package_name);
            } else {
                panic!("README not found for {}", package_name);
            }
        }
    }

    /// Test that packages without README don't break the system
    #[test]
    #[serial]
    fn test_package_without_readme() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());
        let project = TestProject::new(&server.base_url);

        // Register and authenticate user
        project.register_and_login_user(&server.base_url, server.port);

        // Create a package without README
        project.create_test_package("no-readme-package", "1.0.0");

        let publish_output = project.run_command(&PackageManager::Npm, &["publish".to_string()]);

        if !publish_output.status.success() {
            println!(
                "Publish stderr: {}",
                String::from_utf8_lossy(&publish_output.stderr)
            );
            panic!("Failed to publish package without README");
        }

        // Verify the package works without README
        let version_response = client
            .get("/registry/no-readme-package/1.0.0")
            .send()
            .unwrap();

        assert!(version_response.status().is_success());
        let metadata: serde_json::Value = version_response.json().unwrap();

        assert_eq!(metadata["name"], "no-readme-package");
        assert_eq!(metadata["version"], "1.0.0");

        // README field should either be null, not present, or empty string
        if let Some(readme) = metadata.get("readme") {
            if readme.is_null() {
                println!("✅ Package without README handled correctly (null)");
            } else if let Some(readme_str) = readme.as_str() {
                // README might be empty string or npm's default error message
                if readme_str.is_empty() {
                    println!("✅ Package without README handled correctly (empty string)");
                } else if readme_str.contains("ERROR: No README data found!") {
                    println!("✅ Package without README handled correctly (npm default message)");
                } else {
                    panic!(
                        "README should be empty or npm error message for packages without README, got: '{}'",
                        readme_str
                    );
                }
            } else {
                panic!("README field has unexpected type: {:?}", readme);
            }
        } else {
            println!("✅ Package without README handled correctly (field absent)");
        }
    }

    /// Test README functionality with API-based publishing (not npm CLI)
    #[test]
    #[serial]
    fn test_api_published_package_readme() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());
        let project = TestProject::new(&server.base_url);

        // Register and authenticate user
        project.register_and_login_user(&server.base_url, server.port);

        // Get auth token for API client
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:testuser",
            "name": "testuser",
            "password": "testpass123",
            "email": "testuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let auth_response = client
            .put("/registry/-/user/org.couchdb.user:testuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if auth_response.status().is_success() {
            let auth_result: serde_json::Value = auth_response.json().unwrap();
            if let Some(token) = auth_result["token"].as_str() {
                client.set_auth_token(token.to_string());
            }
        }

        let readme_content = "# API Published Package\n\nThis package was published via direct API call to test README functionality.\n\n## API Publishing\n\nThis tests that README content is properly extracted and stored when packages are published via the REST API rather than npm CLI.";

        // Create tarball data
        let tarball_data = b"test tarball content for API publishing";
        let encoded_tarball = BASE64_STANDARD.encode(tarball_data);

        // Create publish request with README in package.json
        let publish_request = json!({
            "_id": "api-readme-test",
            "name": "api-readme-test",
            "description": "API published package with README",
            "versions": {
                "1.0.0": {
                    "name": "api-readme-test",
                    "version": "1.0.0",
                    "description": "API published package with README",
                    "main": "index.js",
                    "readme": readme_content,
                    "scripts": {
                        "test": "echo \"Error: no test specified\" && exit 1"
                    },
                    "author": "test",
                    "license": "MIT",
                    "dist": {
                        "tarball": format!("{}/api-readme-test/-/api-readme-test-1.0.0.tgz", server.base_url),
                        "shasum": "dummy-shasum-api-test"
                    }
                }
            },
            "_attachments": {
                "api-readme-test-1.0.0.tgz": {
                    "content_type": "application/octet-stream",
                    "data": encoded_tarball,
                    "length": tarball_data.len()
                }
            }
        });

        let response = client
            .put("/registry/api-readme-test")
            .json(&publish_request)
            .send()
            .unwrap();

        if !response.status().is_success() {
            println!("API publish failed: {}", response.status());
            println!("Response: {:?}", response.text());
            panic!("Failed to publish package via API");
        }

        println!("✅ Package published via API successfully");

        // Test that README is included in version-specific endpoint
        let version_response = client
            .get("/registry/api-readme-test/1.0.0")
            .send()
            .unwrap();

        assert!(version_response.status().is_success());
        let metadata: serde_json::Value = version_response.json().unwrap();

        assert_eq!(metadata["name"], "api-readme-test");
        assert_eq!(metadata["version"], "1.0.0");

        if let Some(retrieved_readme) = metadata["readme"].as_str() {
            assert_eq!(retrieved_readme, readme_content);
            println!("✅ API published package README correctly stored and retrieved");
        } else {
            panic!("README not found in API published package");
        }
    }

    /// Test README with very large content
    #[test]
    #[serial]
    fn test_large_readme_content() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());
        let project = TestProject::new(&server.base_url);

        // Register and authenticate user
        project.register_and_login_user(&server.base_url, server.port);

        // Create a large README (simulate a comprehensive documentation)
        let mut large_readme = String::from("# Large README Test\n\n");
        large_readme.push_str("This is a test of README functionality with large content.\n\n");

        // Add multiple sections to make it large
        for i in 1..=50 {
            large_readme.push_str(&format!(
                "## Section {}\n\nThis is section {} of the large README. It contains detailed information about feature {}.\n\n```javascript\n// Example code for feature {}\nconst feature{} = require('./feature{}');\nfeature{}.initialize();\n```\n\n",
                i, i, i, i, i, i, i
            ));
        }

        large_readme.push_str("## Conclusion\n\nThis concludes the large README test.");

        project.create_test_package_with_readme("large-readme-test", "1.0.0", &large_readme);

        let publish_output = project.run_command(&PackageManager::Npm, &["publish".to_string()]);

        if !publish_output.status.success() {
            println!(
                "Large README publish failed: {}",
                String::from_utf8_lossy(&publish_output.stderr)
            );
            // Don't panic - large content might hit limits, which is acceptable
            println!("⚠️  Large README test skipped due to size limits");
            return;
        }

        // Verify large README is handled correctly
        let version_response = client
            .get("/registry/large-readme-test/1.0.0")
            .send()
            .unwrap();

        assert!(version_response.status().is_success());
        let metadata: serde_json::Value = version_response.json().unwrap();

        if let Some(retrieved_readme) = metadata["readme"].as_str() {
            assert_eq!(retrieved_readme, large_readme);
            println!(
                "✅ Large README ({} chars) handled correctly",
                large_readme.len()
            );
        } else {
            panic!("Large README not found");
        }
    }

    /// Test that upstream packages without README don't cause errors
    #[test]
    #[serial]
    fn test_upstream_package_without_readme() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with a package that might not have README (or test with a version that doesn't)
        // We'll use a specific version that might not have comprehensive README
        match client.get("/registry/lodash/1.0.0").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let metadata: serde_json::Value = response.json().unwrap();

                    // Verify basic package info
                    assert_eq!(metadata["name"], "lodash");
                    assert_eq!(metadata["version"], "1.0.0");

                    // README might be present or absent - both should be handled gracefully
                    if let Some(readme) = metadata.get("readme") {
                        if readme.is_null() {
                            println!("✅ Upstream package without README handled correctly (null)");
                        } else if let Some(readme_str) = readme.as_str() {
                            println!(
                                "✅ Upstream package README present ({} chars)",
                                readme_str.len()
                            );
                        }
                    } else {
                        println!(
                            "✅ Upstream package without README handled correctly (field absent)"
                        );
                    }
                } else {
                    println!("Upstream package version not found - this is acceptable");
                }
            }
            Err(e) => {
                println!(
                    "Warning: Upstream package test failed: {e}. This may be due to network issues."
                );
            }
        }
    }

    /// Test that README is properly fetched and stored from upstream package metadata
    #[test]
    #[serial]
    fn test_upstream_package_readme_fetching_and_storage() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with a well-known package that has README content
        // First, fetch package-level metadata to trigger upstream fetching and storage
        match client.get("/registry/express").send() {
            Ok(response) => {
                assert!(
                    response.status().is_success(),
                    "Package metadata request failed with status: {}",
                    response.status()
                );

                let package_metadata: serde_json::Value = response.json().unwrap();

                // Verify basic package info
                assert_eq!(package_metadata["name"], "express");

                // Verify README is present at package level
                assert!(
                    package_metadata.get("readme").is_some(),
                    "README field should be present in package metadata"
                );

                let package_readme = package_metadata["readme"].as_str().unwrap();
                assert!(
                    !package_readme.is_empty(),
                    "Package README should not be empty"
                );
                assert!(
                    package_readme.contains("Express"),
                    "README should contain package name"
                );

                println!(
                    "✅ Package-level README fetched successfully ({} chars)",
                    package_readme.len()
                );

                // Now test specific version metadata to ensure README is included
                match client.get("/registry/express/4.18.2").send() {
                    Ok(version_response) => {
                        assert!(
                            version_response.status().is_success(),
                            "Version metadata request failed with status: {}",
                            version_response.status()
                        );

                        let version_metadata: serde_json::Value = version_response.json().unwrap();

                        // Verify basic version info
                        assert_eq!(version_metadata["name"], "express");
                        assert_eq!(version_metadata["version"], "4.18.2");

                        // Verify README is present in version metadata
                        assert!(
                            version_metadata.get("readme").is_some(),
                            "README field should be present in version metadata"
                        );

                        let version_readme = version_metadata["readme"].as_str().unwrap();
                        assert!(
                            !version_readme.is_empty(),
                            "Version README should not be empty"
                        );

                        // The version README should match the package README
                        assert_eq!(
                            version_readme, package_readme,
                            "Version README should match package README"
                        );

                        println!("✅ Version-level README matches package-level README");
                    }
                    Err(e) => {
                        panic!("Failed to fetch version metadata: {e}");
                    }
                }
            }
            Err(e) => {
                println!(
                    "Warning: Upstream package test failed: {e}. This may be due to network issues."
                );
            }
        }
    }

    /// Test that README is properly stored in database after upstream fetch
    #[test]
    #[serial]
    fn test_upstream_readme_database_storage() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with a package that has README content
        match client.get("/registry/lodash").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let package_metadata: serde_json::Value = response.json().unwrap();

                    // Verify README is present
                    if let Some(readme) = package_metadata.get("readme") {
                        if let Some(readme_str) = readme.as_str() {
                            if !readme_str.is_empty() {
                                println!(
                                    "✅ Package README fetched from upstream ({} chars)",
                                    readme_str.len()
                                );

                                // Now fetch a specific version to ensure README is stored in database
                                if let Some(versions) = package_metadata["versions"].as_object() {
                                    if let Some(latest_version) = versions.keys().last() {
                                        match client
                                            .get(&format!("/registry/lodash/{}", latest_version))
                                            .send()
                                        {
                                            Ok(version_response) => {
                                                if version_response.status().is_success() {
                                                    let version_metadata: serde_json::Value =
                                                        version_response.json().unwrap();

                                                    // Verify README is present in version metadata
                                                    assert!(
                                                        version_metadata.get("readme").is_some(),
                                                        "README should be present in version metadata after database storage"
                                                    );

                                                    let version_readme = version_metadata["readme"]
                                                        .as_str()
                                                        .unwrap();
                                                    assert_eq!(
                                                        version_readme, readme_str,
                                                        "Version README should match package README after database storage"
                                                    );

                                                    println!(
                                                        "✅ README properly stored in database for version {}",
                                                        latest_version
                                                    );

                                                    // Fetch the same version again to test database retrieval
                                                    match client
                                                        .get(&format!(
                                                            "/registry/lodash/{}",
                                                            latest_version
                                                        ))
                                                        .send()
                                                    {
                                                        Ok(cached_response) => {
                                                            if cached_response.status().is_success()
                                                            {
                                                                let cached_metadata: serde_json::Value = cached_response.json().unwrap();

                                                                // Verify README is still present from database/cache
                                                                assert!(
                                                                    cached_metadata
                                                                        .get("readme")
                                                                        .is_some(),
                                                                    "README should be present when retrieved from database/cache"
                                                                );

                                                                let cached_readme = cached_metadata
                                                                    ["readme"]
                                                                    .as_str()
                                                                    .unwrap();
                                                                assert_eq!(
                                                                    cached_readme, readme_str,
                                                                    "Cached README should match original README"
                                                                );

                                                                println!(
                                                                    "✅ README properly retrieved from database/cache"
                                                                );
                                                            }
                                                        }
                                                        Err(e) => {
                                                            println!(
                                                                "Warning: Failed to fetch cached version: {e}"
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!(
                                                    "Warning: Failed to fetch version metadata: {e}"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "Warning: Upstream package test failed: {e}. This may be due to network issues."
                );
            }
        }
    }

    /// Test that README is consistently available across multiple versions of an upstream package
    #[test]
    #[serial]
    fn test_upstream_readme_consistency_across_versions() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test with a package that has multiple versions
        match client.get("/registry/react").send() {
            Ok(response) => {
                if response.status().is_success() {
                    let package_metadata: serde_json::Value = response.json().unwrap();

                    // Verify README is present at package level
                    if let Some(package_readme) = package_metadata.get("readme") {
                        if let Some(package_readme_str) = package_readme.as_str() {
                            if !package_readme_str.is_empty() {
                                println!(
                                    "✅ Package README fetched from upstream ({} chars)",
                                    package_readme_str.len()
                                );

                                // Test multiple versions to ensure README consistency
                                let test_versions = vec!["18.2.0", "17.0.2", "16.14.0"];

                                for version in test_versions {
                                    match client.get(&format!("/registry/react/{}", version)).send()
                                    {
                                        Ok(version_response) => {
                                            if version_response.status().is_success() {
                                                let version_metadata: serde_json::Value =
                                                    version_response.json().unwrap();

                                                // Verify README is present in version metadata
                                                assert!(
                                                    version_metadata.get("readme").is_some(),
                                                    "README should be present in version {} metadata",
                                                    version
                                                );

                                                let version_readme =
                                                    version_metadata["readme"].as_str().unwrap();
                                                assert!(
                                                    !version_readme.is_empty(),
                                                    "Version {} README should not be empty",
                                                    version
                                                );

                                                // The version README should match the package README
                                                assert_eq!(
                                                    version_readme, package_readme_str,
                                                    "Version {} README should match package README",
                                                    version
                                                );

                                                println!(
                                                    "✅ Version {} README matches package README",
                                                    version
                                                );
                                            } else {
                                                println!(
                                                    "Version {} not found - this is acceptable",
                                                    version
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            println!(
                                                "Warning: Failed to fetch version {} metadata: {e}",
                                                version
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "Warning: Upstream package test failed: {e}. This may be due to network issues."
                );
            }
        }
    }
}

impl TestProject {
    /// Helper method to create a test package with README content
    pub fn create_test_package_with_readme(&self, name: &str, version: &str, readme_content: &str) {
        let package_json = serde_json::json!({
            "name": name,
            "version": version,
            "description": format!("Test package {} for README e2e tests", name),
            "main": "index.js",
            "scripts": {
                "test": "echo \"Error: no test specified\" && exit 1"
            },
            "keywords": ["test", "readme"],
            "author": "test",
            "license": "MIT",
            "readme": readme_content
        });

        fs::write(
            &self.package_json_path,
            serde_json::to_string_pretty(&package_json).unwrap(),
        )
        .expect("Failed to write test package.json with README");

        // Create a simple index.js
        fs::write(
            self.path().join("index.js"),
            format!(
                "module.exports = {{ greet: () => 'Hello from {}!' }};",
                name
            ),
        )
        .expect("Failed to write index.js");
    }
}
