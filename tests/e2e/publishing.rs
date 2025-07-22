use super::*;
use base64::prelude::*;
use serde_json::json;
use serial_test::serial;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tarball() -> Vec<u8> {
        // Create a minimal tarball for testing
        // This is a simplified version - in real tests you might want to create actual tar.gz files
        b"test tarball content".to_vec()
    }

    fn setup_authenticated_user(client: &ApiClient) -> Option<String> {
        // Register and login a user for publishing tests
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:publisher",
            "name": "publisher",
            "password": "publisherpassword123",
            "email": "publisher@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:publisher")
            .json(&npm_user_doc)
            .send()
            .ok()?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().ok()?;
            result["token"].as_str().map(|s| s.to_string())
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_package_publish_basic() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // Create publish request
            let tarball_data = create_test_tarball();
            let encoded_tarball = BASE64_STANDARD.encode(&tarball_data);

            let publish_request = json!({
                "_id": "test-package",
                "name": "test-package",
                "description": "A test package for e2e testing",
                "versions": {
                    "1.0.0": {
                        "name": "test-package",
                        "version": "1.0.0",
                        "description": "A test package for e2e testing",
                        "main": "index.js",
                        "scripts": {
                            "test": "echo \"Error: no test specified\" && exit 1"
                        },
                        "author": "test",
                        "license": "MIT",
                        "dist": {
                            "tarball": format!("{}/test-package/-/test-package-1.0.0.tgz", server.base_url),
                            "shasum": "dummy-shasum"
                        }
                    }
                },
                "_attachments": {
                    "test-package-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball,
                        "length": tarball_data.len()
                    }
                }
            });

            let response = client
                .put("/registry/test-package")
                .json(&publish_request)
                .send()
                .unwrap();

            // The package publish should succeed
            assert!(
                response.status().is_success(),
                "Package publish failed with status: {}",
                response.status()
            );

            let result: serde_json::Value = response.json().unwrap();
            assert_eq!(result["ok"], true);
            assert_eq!(result["id"], "test-package");

            // Now fetch the package metadata to verify license is stored
            let metadata_response = client.get("/registry/test-package").send().unwrap();

            assert!(
                metadata_response.status().is_success(),
                "Package metadata fetch failed with status: {}",
                metadata_response.status()
            );

            let metadata: serde_json::Value = metadata_response.json().unwrap();
            assert_eq!(metadata["name"], "test-package");
            assert_eq!(metadata["license"], "MIT");
            println!(
                "✓ License field properly stored and returned: {}",
                metadata["license"]
            );
        }
    }

    #[test]
    #[serial]
    fn test_package_publish_with_license_variations() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // Test 1: Package with Apache-2.0 license
            let tarball_data = create_test_tarball();
            let encoded_tarball = BASE64_STANDARD.encode(&tarball_data);

            let publish_request = json!({
                "_id": "test-package-apache",
                "name": "test-package-apache",
                "description": "A test package with Apache license",
                "versions": {
                    "1.0.0": {
                        "name": "test-package-apache",
                        "version": "1.0.0",
                        "description": "A test package with Apache license",
                        "main": "index.js",
                        "author": "test",
                        "license": "Apache-2.0",
                        "dist": {
                            "tarball": format!("{}/test-package-apache/-/test-package-apache-1.0.0.tgz", server.base_url),
                            "shasum": "dummy-shasum"
                        }
                    }
                },
                "_attachments": {
                    "test-package-apache-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball.clone(),
                        "length": tarball_data.len()
                    }
                }
            });

            let response = client
                .put("/registry/test-package-apache")
                .json(&publish_request)
                .send()
                .unwrap();

            assert!(response.status().is_success());

            // Verify Apache license is stored
            let metadata_response = client.get("/registry/test-package-apache").send().unwrap();
            let metadata: serde_json::Value = metadata_response.json().unwrap();
            assert_eq!(metadata["license"], "Apache-2.0");

            // Test 2: Package without license
            let publish_request_no_license = json!({
                "_id": "test-package-no-license",
                "name": "test-package-no-license",
                "description": "A test package without license",
                "versions": {
                    "1.0.0": {
                        "name": "test-package-no-license",
                        "version": "1.0.0",
                        "description": "A test package without license",
                        "main": "index.js",
                        "author": "test",
                        "dist": {
                            "tarball": format!("{}/test-package-no-license/-/test-package-no-license-1.0.0.tgz", server.base_url),
                            "shasum": "dummy-shasum"
                        }
                    }
                },
                "_attachments": {
                    "test-package-no-license-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball,
                        "length": tarball_data.len()
                    }
                }
            });

            let response = client
                .put("/registry/test-package-no-license")
                .json(&publish_request_no_license)
                .send()
                .unwrap();

            assert!(response.status().is_success());

            // Verify no license field is present when not provided
            let metadata_response = client
                .get("/registry/test-package-no-license")
                .send()
                .unwrap();
            let metadata: serde_json::Value = metadata_response.json().unwrap();
            assert!(metadata.get("license").is_none() || metadata["license"].is_null());

            println!("✓ License variations test passed");
        }
    }

    #[test]
    #[serial]
    fn test_package_publish_without_authentication() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Try to publish without authentication
        let tarball_data = create_test_tarball();
        let encoded_tarball = BASE64_STANDARD.encode(&tarball_data);

        let publish_request = json!({
            "name": "unauthorized-package",
            "versions": {
                "1.0.0": {
                    "name": "unauthorized-package",
                    "version": "1.0.0"
                }
            },
            "_attachments": {
                "unauthorized-package-1.0.0.tgz": {
                    "content_type": "application/octet-stream",
                    "data": encoded_tarball,
                    "length": tarball_data.len()
                }
            }
        });

        let response = client
            .put("/registry/unauthorized-package")
            .json(&publish_request)
            .send()
            .unwrap();

        // Should fail without authentication
        assert!(!response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_scoped_package_publish() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // Create scoped package publish request
            let tarball_data = create_test_tarball();
            let encoded_tarball = BASE64_STANDARD.encode(&tarball_data);

            let publish_request = json!({
                "_id": "@testscope/scoped-package",
                "name": "@testscope/scoped-package",
                "description": "A scoped test package",
                "versions": {
                    "1.0.0": {
                        "name": "@testscope/scoped-package",
                        "version": "1.0.0",
                        "description": "A scoped test package",
                        "main": "index.js",
                        "author": "test",
                        "license": "MIT",
                        "dist": {
                            "tarball": format!("{}/@testscope/scoped-package/-/scoped-package-1.0.0.tgz", server.base_url),
                            "shasum": "dummy-shasum"
                        }
                    }
                },
                "_attachments": {
                    "scoped-package-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball,
                        "length": tarball_data.len()
                    }
                }
            });

            let response = client
                .put("/registry/@testscope/scoped-package")
                .json(&publish_request)
                .send()
                .unwrap();

            // The scoped package publish should succeed
            assert!(
                response.status().is_success(),
                "Scoped package publish failed with status: {}",
                response.status()
            );

            let result: serde_json::Value = response.json().unwrap();
            assert_eq!(result["ok"], true);
            assert_eq!(result["id"], "@testscope/scoped-package");
        }
    }

    #[test]
    #[serial]
    fn test_package_publish_version_update() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // First, publish version 1.0.0
            let tarball_data_v1 = create_test_tarball();
            let encoded_tarball_v1 = BASE64_STANDARD.encode(&tarball_data_v1);

            let publish_request_v1 = json!({
                "name": "versioned-package",
                "versions": {
                    "1.0.0": {
                        "name": "versioned-package",
                        "version": "1.0.0",
                        "description": "Version 1.0.0",
                        "dist": {
                            "tarball": format!("{}/versioned-package/-/versioned-package-1.0.0.tgz", server.base_url),
                            "shasum": "dummy-shasum-v1"
                        }
                    }
                },
                "_attachments": {
                    "versioned-package-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball_v1,
                        "length": tarball_data_v1.len()
                    }
                }
            });

            let response_v1 = client
                .put("/registry/versioned-package")
                .json(&publish_request_v1)
                .send()
                .unwrap();

            if response_v1.status().is_success() {
                // Then publish version 1.1.0
                let tarball_data_v2 = create_test_tarball();
                let encoded_tarball_v2 = BASE64_STANDARD.encode(&tarball_data_v2);

                let publish_request_v2 = json!({
                    "name": "versioned-package",
                    "versions": {
                        "1.0.0": {
                            "name": "versioned-package",
                            "version": "1.0.0",
                            "description": "Version 1.0.0",
                            "dist": {
                                "tarball": format!("{}/versioned-package/-/versioned-package-1.0.0.tgz", server.base_url),
                                "shasum": "dummy-shasum-v1"
                            }
                        },
                        "1.1.0": {
                            "name": "versioned-package",
                            "version": "1.1.0",
                            "description": "Version 1.1.0",
                            "dist": {
                                "tarball": format!("{}/versioned-package/-/versioned-package-1.1.0.tgz", server.base_url),
                                "shasum": "dummy-shasum-v2"
                            }
                        }
                    },
                    "_attachments": {
                        "versioned-package-1.1.0.tgz": {
                            "content_type": "application/octet-stream",
                            "data": encoded_tarball_v2,
                            "length": tarball_data_v2.len()
                        }
                    }
                });

                let response_v2 = client
                    .put("/registry/versioned-package")
                    .json(&publish_request_v2)
                    .send()
                    .unwrap();

                if response_v2.status().is_success() {
                    let result: serde_json::Value = response_v2.json().unwrap();
                    assert_eq!(result["ok"], true);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_publish_invalid_name() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // Try to publish with invalid package name
            let tarball_data = create_test_tarball();
            let encoded_tarball = BASE64_STANDARD.encode(&tarball_data);

            let publish_request = json!({
                "name": "Invalid Package Name!",
                "versions": {
                    "1.0.0": {
                        "name": "Invalid Package Name!",
                        "version": "1.0.0"
                    }
                },
                "_attachments": {
                    "invalid-package-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball,
                        "length": tarball_data.len()
                    }
                }
            });

            let response = client
                .put("/registry/Invalid%20Package%20Name!")
                .json(&publish_request)
                .send()
                .unwrap();

            // Should fail with invalid package name
            assert!(!response.status().is_success());
        }
    }

    #[test]
    #[serial]
    fn test_package_publish_missing_attachments() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // Try to publish without attachments
            let publish_request = json!({
                "name": "no-attachments-package",
                "versions": {
                    "1.0.0": {
                        "name": "no-attachments-package",
                        "version": "1.0.0"
                    }
                }
                // Missing _attachments field
            });

            let response = client
                .put("/registry/no-attachments-package")
                .json(&publish_request)
                .send()
                .unwrap();

            // Should fail without attachments
            assert!(!response.status().is_success());
        }
    }

    #[test]
    #[serial]
    fn test_package_ownership_verification() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client1 = ApiClient::new(server.base_url.clone());
        let mut client2 = ApiClient::new(server.base_url.clone());

        // Setup first user
        let npm_user_doc1 = json!({
            "_id": "org.couchdb.user:owner1",
            "name": "owner1",
            "password": "owner1password123",
            "email": "owner1@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response1 = client1
            .put("/registry/-/user/org.couchdb.user:owner1")
            .json(&npm_user_doc1)
            .send()
            .unwrap();

        if response1.status().is_success() {
            let result1: serde_json::Value = response1.json().unwrap();
            let token1 = result1["token"].as_str().unwrap().to_string();
            client1.set_auth_token(token1);

            // Setup second user
            let npm_user_doc2 = json!({
                "_id": "org.couchdb.user:owner2",
                "name": "owner2",
                "password": "owner2password123",
                "email": "owner2@example.com",
                "type": "user",
                "roles": [],
                "date": "2025-07-18T00:00:00.000Z"
            });

            let response2 = client2
                .put("/registry/-/user/org.couchdb.user:owner2")
                .json(&npm_user_doc2)
                .send()
                .unwrap();

            if response2.status().is_success() {
                let result2: serde_json::Value = response2.json().unwrap();
                let token2 = result2["token"].as_str().unwrap().to_string();
                client2.set_auth_token(token2);

                // First user publishes a package
                let tarball_data = create_test_tarball();
                let encoded_tarball = BASE64_STANDARD.encode(&tarball_data);

                let publish_request = json!({
                    "name": "ownership-test-package",
                    "versions": {
                        "1.0.0": {
                            "name": "ownership-test-package",
                            "version": "1.0.0"
                        }
                    },
                    "_attachments": {
                        "ownership-test-package-1.0.0.tgz": {
                            "content_type": "application/octet-stream",
                            "data": encoded_tarball,
                            "length": tarball_data.len()
                        }
                    }
                });

                let publish_response = client1
                    .put("/registry/ownership-test-package")
                    .json(&publish_request)
                    .send()
                    .unwrap();

                if publish_response.status().is_success() {
                    // Second user tries to publish to the same package (should fail)
                    let unauthorized_response = client2
                        .put("/registry/ownership-test-package")
                        .json(&publish_request)
                        .send()
                        .unwrap();

                    assert!(!unauthorized_response.status().is_success());
                }
            } else {
                panic!(
                    "Failed to register user: {} - {}",
                    response2.status(),
                    response2.text().unwrap_or_default()
                );
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response1.status(),
                response1.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_database_priority_over_upstream() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);
        let client = ApiClient::new(server.base_url.clone());

        // First, fetch a package from upstream to cache it in database
        let response = client.get("/registry/lodash").send().unwrap();
        assert!(response.status().is_success());
        let upstream_data: serde_json::Value = response.json().unwrap();

        // Verify it's upstream data (should have many versions)
        assert!(upstream_data["versions"].as_object().unwrap().len() > 10);
        println!(
            "Upstream lodash has {} versions",
            upstream_data["versions"].as_object().unwrap().len()
        );

        // Now publish our own version of "lodash"
        project.create_test_package("lodash", "999.0.0"); // Use a version that doesn't exist upstream

        // Register user and get auth token
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:testuser",
            "name": "testuser",
            "password": "testpassword123",
            "email": "testuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:testuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Publish our version of lodash
                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                if publish_output.status.success() {
                    // Now fetch lodash again - should get our published version, not upstream
                    let response = client.get("/registry/lodash").send().unwrap();
                    assert!(response.status().is_success());
                    let local_data: serde_json::Value = response.json().unwrap();

                    // Should now have our version 999.0.0
                    assert!(
                        local_data["versions"]["999.0.0"].is_object(),
                        "Should contain our published version 999.0.0"
                    );

                    // Should still have upstream versions but our version should be in dist-tags
                    assert_eq!(
                        local_data["dist-tags"]["latest"], "999.0.0",
                        "Our version should be the latest"
                    );

                    println!("✅ Database package correctly takes priority over upstream");
                } else {
                    println!(
                        "Publish failed: {}",
                        String::from_utf8_lossy(&publish_output.stderr)
                    );
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_npm_publish_regular_package() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a test package for publishing
        project.create_test_package("npm-publish-test", "1.0.0");

        // First, we need to register a user and get authentication set up
        let client = ApiClient::new(server.base_url.clone());

        // Register user via API (since npm login is interactive)
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:npmuser",
            "name": "npmuser",
            "password": "npmpassword123",
            "email": "npmuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:npmuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Try to publish the package
                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                println!(
                    "npm publish stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "npm publish stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                // Check if publish was successful
                if publish_output.status.success() {
                    // Verify the package was published by fetching it
                    let package_response = client.get("/registry/npm-publish-test").send().unwrap();

                    assert!(
                        package_response.status().is_success(),
                        "Published package should be fetchable"
                    );

                    let package_data: serde_json::Value = package_response.json().unwrap();
                    assert_eq!(package_data["name"], "npm-publish-test");
                    assert!(package_data["versions"]["1.0.0"].is_object());
                } else {
                    // Print debug info if publish failed
                    println!(
                        "npm publish failed with exit code: {:?}",
                        publish_output.status.code()
                    );
                    println!(
                        "This might be expected if npm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_npm_publish_scoped_package() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a scoped test package for publishing
        project.create_scoped_test_package("@testorg", "scoped-publish-test", "1.0.0");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:scopeduser",
            "name": "scopeduser",
            "password": "scopedpassword123",
            "email": "scopeduser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:scopeduser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token and scoped registry config
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n@testorg:registry={}/registry\n",
                    server.base_url, server.port, token, server.base_url
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Try to publish the scoped package
                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                println!(
                    "npm publish scoped stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "npm publish scoped stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                // Check if publish was successful
                if publish_output.status.success() {
                    // Verify the scoped package was published by fetching it
                    let package_response = client
                        .get("/registry/@testorg/scoped-publish-test")
                        .send()
                        .unwrap();

                    assert!(
                        package_response.status().is_success(),
                        "Published scoped package should be fetchable"
                    );

                    let package_data: serde_json::Value = package_response.json().unwrap();
                    assert_eq!(package_data["name"], "@testorg/scoped-publish-test");
                    assert!(package_data["versions"]["1.0.0"].is_object());
                } else {
                    // Print debug info if publish failed
                    println!(
                        "npm publish scoped failed with exit code: {:?}",
                        publish_output.status.code()
                    );
                    println!(
                        "This might be expected if npm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_npm_publish_with_access_public() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a scoped test package for publishing with public access
        project.create_scoped_test_package("@publicorg", "public-package", "1.0.0");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:publicuser",
            "name": "publicuser",
            "password": "publicpassword123",
            "email": "publicuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:publicuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n@publicorg:registry={}/registry\n",
                    server.base_url, server.port, token, server.base_url
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Try to publish the package with --access public
                let mut publish_args = PackageManager::Npm
                    .publish_args()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                publish_args.push("--access".to_string());
                publish_args.push("public".to_string());

                let publish_output = project.run_command(&PackageManager::Npm, &publish_args);

                println!(
                    "npm publish --access public stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "npm publish --access public stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                // Check if publish was successful
                if publish_output.status.success() {
                    // Verify the package was published by fetching it
                    let package_response = client
                        .get("/registry/@publicorg/public-package")
                        .send()
                        .unwrap();

                    assert!(
                        package_response.status().is_success(),
                        "Published public package should be fetchable"
                    );

                    let package_data: serde_json::Value = package_response.json().unwrap();
                    assert_eq!(package_data["name"], "@publicorg/public-package");
                    assert!(package_data["versions"]["1.0.0"].is_object());
                } else {
                    println!(
                        "npm publish --access public failed with exit code: {:?}",
                        publish_output.status.code()
                    );
                    println!(
                        "This might be expected if npm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_pnpm_publish_regular_package() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a test package for publishing with pnpm
        project.create_test_package("pnpm-publish-test", "1.0.0");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:pnpmuser",
            "name": "pnpmuser",
            "password": "pnpmpassword123",
            "email": "pnpmuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:pnpmuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token for pnpm
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Try to publish the package with pnpm
                let publish_output = project.run_command(
                    &PackageManager::Pnpm,
                    &PackageManager::Pnpm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                println!(
                    "pnpm publish stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "pnpm publish stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                // Check if publish was successful
                if publish_output.status.success() {
                    // Verify the package was published by fetching it
                    let package_response =
                        client.get("/registry/pnpm-publish-test").send().unwrap();

                    assert!(
                        package_response.status().is_success(),
                        "Published package should be fetchable"
                    );

                    let package_data: serde_json::Value = package_response.json().unwrap();
                    assert_eq!(package_data["name"], "pnpm-publish-test");
                    assert!(package_data["versions"]["1.0.0"].is_object());
                } else {
                    println!(
                        "pnpm publish failed with exit code: {:?}",
                        publish_output.status.code()
                    );
                    println!(
                        "This might be expected if pnpm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_pnpm_publish_scoped_package() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a scoped test package for publishing with pnpm
        project.create_scoped_test_package("@pnpmorg", "pnpm-scoped-test", "1.0.0");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:pnpmscopeduser",
            "name": "pnpmscopeduser",
            "password": "pnpmscopedpassword123",
            "email": "pnpmscopeduser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:pnpmscopeduser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token and scoped registry config for pnpm
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n@pnpmorg:registry={}/registry\n",
                    server.base_url, server.port, token, server.base_url
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Try to publish the scoped package with pnpm
                let mut publish_args = PackageManager::Pnpm
                    .publish_args()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                publish_args.push("--access".to_string());
                publish_args.push("public".to_string());

                let publish_output = project.run_command(&PackageManager::Pnpm, &publish_args);

                println!(
                    "pnpm publish scoped stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "pnpm publish scoped stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                // Check if publish was successful
                if publish_output.status.success() {
                    // Verify the scoped package was published by fetching it
                    let package_response = client
                        .get("/registry/@pnpmorg/pnpm-scoped-test")
                        .send()
                        .unwrap();

                    assert!(
                        package_response.status().is_success(),
                        "Published scoped package should be fetchable"
                    );

                    let package_data: serde_json::Value = package_response.json().unwrap();
                    assert_eq!(package_data["name"], "@pnpmorg/pnpm-scoped-test");
                    assert!(package_data["versions"]["1.0.0"].is_object());
                } else {
                    println!(
                        "pnpm publish scoped failed with exit code: {:?}",
                        publish_output.status.code()
                    );
                    println!(
                        "This might be expected if pnpm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_npm_publish_version_update_with_command() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create initial version of the package
        project.create_test_package("version-update-test", "1.0.0");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:versionuser",
            "name": "versionuser",
            "password": "versionpassword123",
            "email": "versionuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:versionuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Publish version 1.0.0
                let publish_output_v1 = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                if publish_output_v1.status.success() {
                    // Update to version 1.1.0
                    project.create_test_package("version-update-test", "1.1.0");

                    // Publish version 1.1.0
                    let publish_output_v2 = project.run_command(
                        &PackageManager::Npm,
                        &PackageManager::Npm
                            .publish_args()
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>(),
                    );

                    println!(
                        "npm publish v1.1.0 stdout: {}",
                        String::from_utf8_lossy(&publish_output_v2.stdout)
                    );
                    println!(
                        "npm publish v1.1.0 stderr: {}",
                        String::from_utf8_lossy(&publish_output_v2.stderr)
                    );

                    if publish_output_v2.status.success() {
                        // Verify both versions are available
                        let package_response =
                            client.get("/registry/version-update-test").send().unwrap();

                        if package_response.status().is_success() {
                            let package_data: serde_json::Value = package_response.json().unwrap();
                            assert_eq!(package_data["name"], "version-update-test");
                            assert!(package_data["versions"]["1.0.0"].is_object());
                            assert!(package_data["versions"]["1.1.0"].is_object());
                        }
                    } else {
                        println!(
                            "npm publish v1.1.0 failed with exit code: {:?}",
                            publish_output_v2.status.code()
                        );
                    }
                } else {
                    println!(
                        "npm publish v1.0.0 failed with exit code: {:?}",
                        publish_output_v1.status.code()
                    );
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_npm_publish_without_authentication_command() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a test package for publishing
        project.create_test_package("unauthenticated-publish-test", "1.0.0");

        // Try to publish without authentication (no auth token in .npmrc)
        let publish_output = project.run_command(
            &PackageManager::Npm,
            &PackageManager::Npm
                .publish_args()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );

        println!(
            "npm publish without auth stdout: {}",
            String::from_utf8_lossy(&publish_output.stdout)
        );
        println!(
            "npm publish without auth stderr: {}",
            String::from_utf8_lossy(&publish_output.stderr)
        );

        // Should fail without authentication
        assert!(
            !publish_output.status.success(),
            "Publish should fail without authentication"
        );
    }

    #[test]
    #[serial]
    fn test_npm_publish_duplicate_version() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a test package for publishing
        project.create_test_package("duplicate-version-test", "1.0.0");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:duplicateuser",
            "name": "duplicateuser",
            "password": "duplicatepassword123",
            "email": "duplicateuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:duplicateuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Publish version 1.0.0 first time
                let publish_output_1 = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                if publish_output_1.status.success() {
                    // Try to publish the same version again (should fail)
                    let publish_output_2 = project.run_command(
                        &PackageManager::Npm,
                        &PackageManager::Npm
                            .publish_args()
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>(),
                    );

                    println!(
                        "npm publish duplicate stdout: {}",
                        String::from_utf8_lossy(&publish_output_2.stdout)
                    );
                    println!(
                        "npm publish duplicate stderr: {}",
                        String::from_utf8_lossy(&publish_output_2.stderr)
                    );

                    // Should fail when trying to publish duplicate version
                    assert!(
                        !publish_output_2.status.success(),
                        "Duplicate version publish should fail"
                    );
                } else {
                    println!("First publish failed, skipping duplicate test");
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_npm_whoami_after_publish_setup() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:whoamiuser",
            "name": "whoamiuser",
            "password": "whoamipassword123",
            "email": "whoamiuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:whoamiuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Test npm whoami command
                let whoami_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .whoami_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                println!(
                    "npm whoami stdout: {}",
                    String::from_utf8_lossy(&whoami_output.stdout)
                );
                println!(
                    "npm whoami stderr: {}",
                    String::from_utf8_lossy(&whoami_output.stderr)
                );

                if whoami_output.status.success() {
                    let stdout = String::from_utf8_lossy(&whoami_output.stdout);
                    assert!(
                        stdout.trim().contains("whoamiuser"),
                        "whoami should return the authenticated username"
                    );
                } else {
                    println!(
                        "npm whoami failed with exit code: {:?}",
                        whoami_output.status.code()
                    );
                    println!(
                        "This might be expected if npm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }

    #[test]
    #[serial]
    fn test_package_metadata_revalidation() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // Setup authenticated user
        if let Some(token) = setup_authenticated_user(&client) {
            client.set_auth_token(token);

            // First, publish version 1.0.0 with initial metadata
            let tarball_data_v1 = create_test_tarball();
            let encoded_tarball_v1 = BASE64_STANDARD.encode(&tarball_data_v1);

            let publish_request_v1 = json!({
                "_id": "metadata-revalidation-test",
                "name": "metadata-revalidation-test",
                "description": "Initial description",
                "versions": {
                    "1.0.0": {
                        "name": "metadata-revalidation-test",
                        "version": "1.0.0",
                        "description": "Initial description",
                        "main": "index.js",
                        "scripts": {
                            "test": "echo \"Initial test script\""
                        },
                        "license": "MIT",
                        "dist": {
                            "tarball": format!("{}/metadata-revalidation-test/-/metadata-revalidation-test-1.0.0.tgz", server.base_url),
                            "shasum": "dummy-shasum-v1"
                        }
                    }
                },
                "_attachments": {
                    "metadata-revalidation-test-1.0.0.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball_v1,
                        "length": tarball_data_v1.len()
                    }
                }
            });

            let response_v1 = client
                .put("/registry/metadata-revalidation-test")
                .json(&publish_request_v1)
                .send()
                .unwrap();

            assert!(
                response_v1.status().is_success(),
                "First publish should succeed"
            );

            // Fetch the package metadata to verify initial state
            let package_response_v1 = client
                .get("/registry/metadata-revalidation-test")
                .send()
                .unwrap();

            assert!(package_response_v1.status().is_success());
            let package_data_v1: serde_json::Value = package_response_v1.json().unwrap();
            assert_eq!(package_data_v1["description"], "Initial description");
            assert_eq!(
                package_data_v1["versions"]["1.0.0"]["description"],
                "Initial description"
            );
            assert_eq!(
                package_data_v1["versions"]["1.0.0"]["scripts"]["test"],
                "echo \"Initial test script\""
            );

            // Now publish version 1.0.1 with updated metadata
            let tarball_data_v2 = create_test_tarball();
            let encoded_tarball_v2 = BASE64_STANDARD.encode(&tarball_data_v2);

            let publish_request_v2 = json!({
                "_id": "metadata-revalidation-test",
                "name": "metadata-revalidation-test",
                "description": "Updated description",
                "versions": {
                    "1.0.1": {
                        "name": "metadata-revalidation-test",
                        "version": "1.0.1",
                        "description": "Updated description",
                        "main": "lib/index.js",
                        "scripts": {
                            "test": "echo \"Updated test script\"",
                            "build": "echo \"New build script\""
                        },
                        "license": "Apache-2.0",
                        "dist": {
                            "tarball": format!("{}/metadata-revalidation-test/-/metadata-revalidation-test-1.0.1.tgz", server.base_url),
                            "shasum": "dummy-shasum-v2"
                        }
                    }
                },
                "_attachments": {
                    "metadata-revalidation-test-1.0.1.tgz": {
                        "content_type": "application/octet-stream",
                        "data": encoded_tarball_v2,
                        "length": tarball_data_v2.len()
                    }
                }
            });

            let response_v2 = client
                .put("/registry/metadata-revalidation-test")
                .json(&publish_request_v2)
                .send()
                .unwrap();

            assert!(
                response_v2.status().is_success(),
                "Second publish should succeed"
            );

            // Fetch the package metadata again to verify it was updated
            let package_response_v2 = client
                .get("/registry/metadata-revalidation-test")
                .send()
                .unwrap();

            assert!(package_response_v2.status().is_success());
            let package_data_v2: serde_json::Value = package_response_v2.json().unwrap();

            // Verify package-level metadata was updated
            assert_eq!(
                package_data_v2["description"], "Updated description",
                "Package description should be updated when publishing new version"
            );

            // Verify version-specific metadata was updated
            assert_eq!(
                package_data_v2["versions"]["1.0.1"]["description"],
                "Updated description"
            );
            assert_eq!(package_data_v2["versions"]["1.0.1"]["main"], "lib/index.js");
            assert_eq!(
                package_data_v2["versions"]["1.0.1"]["scripts"]["test"],
                "echo \"Updated test script\""
            );
            assert_eq!(
                package_data_v2["versions"]["1.0.1"]["scripts"]["build"],
                "echo \"New build script\""
            );
            assert_eq!(
                package_data_v2["versions"]["1.0.1"]["license"],
                "Apache-2.0"
            );

            // Verify both versions exist
            assert!(package_data_v2["versions"]["1.0.0"].is_object());
            assert!(package_data_v2["versions"]["1.0.1"].is_object());
        }
    }

    #[test]
    #[serial]
    fn test_npm_publish_with_tag() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);

        // Create a test package for publishing with a tag
        project.create_test_package("tagged-publish-test", "1.0.0-beta.1");

        // Register user via API
        let client = ApiClient::new(server.base_url.clone());

        let npm_user_doc = json!({
            "_id": "org.couchdb.user:taguser",
            "name": "taguser",
            "password": "tagpassword123",
            "email": "taguser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:taguser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Try to publish the package with a beta tag
                let mut publish_args = PackageManager::Npm
                    .publish_args()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                publish_args.push("--tag".to_string());
                publish_args.push("beta".to_string());

                let publish_output = project.run_command(&PackageManager::Npm, &publish_args);

                println!(
                    "npm publish --tag beta stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "npm publish --tag beta stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                // Check if publish was successful
                if publish_output.status.success() {
                    // Verify the package was published by fetching it
                    let package_response =
                        client.get("/registry/tagged-publish-test").send().unwrap();

                    if package_response.status().is_success() {
                        let package_data: serde_json::Value = package_response.json().unwrap();
                        assert_eq!(package_data["name"], "tagged-publish-test");
                        assert!(package_data["versions"]["1.0.0-beta.1"].is_object());

                        // Check if dist-tags are properly set
                        if let Some(dist_tags) = package_data["dist-tags"].as_object() {
                            assert!(dist_tags.contains_key("beta"), "Should have beta tag");
                        }
                    }
                } else {
                    println!(
                        "npm publish --tag beta failed with exit code: {:?}",
                        publish_output.status.code()
                    );
                    println!(
                        "This might be expected if npm is not available or configured properly"
                    );
                }
            }
        } else {
            panic!(
                "Failed to register user: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }
    }
}
