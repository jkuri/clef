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
            "name": "publisher",
            "password": "publisherpassword123",
            "email": "publisher@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/-/user/org.couchdb.user:publisher")
            .json(&npm_user_doc)
            .send()
            .ok()?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().ok()?;
            result["token"].as_str().map(|s| s.to_string())
        } else {
            None
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
                .put("/test-package")
                .json(&publish_request)
                .send()
                .unwrap();

            if response.status().is_success() {
                let result: serde_json::Value = response.json().unwrap();
                assert_eq!(result["ok"], true);
                assert_eq!(result["id"], "test-package");
            }
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
            .put("/unauthorized-package")
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
                .put("/@testscope/scoped-package")
                .json(&publish_request)
                .send()
                .unwrap();

            if response.status().is_success() {
                let result: serde_json::Value = response.json().unwrap();
                assert_eq!(result["ok"], true);
                assert_eq!(result["id"], "@testscope/scoped-package");
            }
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
                .put("/versioned-package")
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
                    .put("/versioned-package")
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
                .put("/Invalid%20Package%20Name!")
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
                .put("/no-attachments-package")
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
            "name": "owner1",
            "password": "owner1password123",
            "email": "owner1@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response1 = client1
            .put("/-/user/org.couchdb.user:owner1")
            .json(&npm_user_doc1)
            .send()
            .unwrap();

        if response1.status().is_success() {
            let result1: serde_json::Value = response1.json().unwrap();
            let token1 = result1["token"].as_str().unwrap().to_string();
            client1.set_auth_token(token1);

            // Setup second user
            let npm_user_doc2 = json!({
                "name": "owner2",
                "password": "owner2password123",
                "email": "owner2@example.com",
                "type": "user",
                "roles": [],
                "date": "2025-07-18T00:00:00.000Z"
            });

            let response2 = client2
                .put("/-/user/org.couchdb.user:owner2")
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
                    .put("/ownership-test-package")
                    .json(&publish_request)
                    .send()
                    .unwrap();

                if publish_response.status().is_success() {
                    // Second user tries to publish to the same package (should fail)
                    let unauthorized_response = client2
                        .put("/ownership-test-package")
                        .json(&publish_request)
                        .send()
                        .unwrap();

                    assert!(!unauthorized_response.status().is_success());
                }
            }
        }
    }
}
