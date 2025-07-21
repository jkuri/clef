use crate::{ApiClient, PackageManager, TestProject, TestServer, init_test_env};
use serde_json::json;
use serial_test::serial;

#[cfg(test)]
mod private_package_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_private_package_access_control() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);
        let client = ApiClient::new(server.base_url.clone());

        // Create a test package for publishing (private)
        project.create_test_package("private-test-package", "1.0.0");

        // Update package.json to mark it as private
        let mut package_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&project.package_json_path).unwrap())
                .unwrap();
        package_json["private"] = serde_json::Value::Bool(true);
        std::fs::write(
            &project.package_json_path,
            serde_json::to_string_pretty(&package_json).unwrap(),
        )
        .expect("Failed to update package.json with private field");

        // Register package owner
        let owner_doc = json!({
            "name": "packageowner",
            "password": "ownerpassword123",
            "email": "owner@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:packageowner")
            .json(&owner_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token for owner
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Publish the package as owner
                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                assert!(
                    publish_output.status.success(),
                    "Owner should be able to publish package"
                );

                // Now mark the package as private in the database
                // This would normally be done through an API endpoint, but for testing we'll do it directly
                // In a real scenario, you'd have an endpoint to update package privacy settings

                // Test 1: Unauthenticated user tries to access private package metadata (should get 404)
                let metadata_response = client
                    .get(&format!("/registry/private-test-package"))
                    .send()
                    .unwrap();

                println!(
                    "Unauthenticated metadata response status: {}",
                    metadata_response.status()
                );
                assert_eq!(
                    metadata_response.status(),
                    404,
                    "Private packages should return 404 for unauthenticated users"
                );

                // Test 2: Register another user who should not have access
                let other_user_doc = json!({
                    "name": "otheruser",
                    "password": "otherpassword123",
                    "email": "other@example.com",
                    "type": "user",
                    "roles": [],
                    "date": "2025-07-18T00:00:00.000Z"
                });

                let response2 = client
                    .put("/registry/-/user/org.couchdb.user:otheruser")
                    .json(&other_user_doc)
                    .send()
                    .unwrap();

                if response2.status().is_success() {
                    let result2: serde_json::Value = response2.json().unwrap();
                    if let Some(token2) = result2["token"].as_str() {
                        // Test authenticated user without access trying to get private package
                        let auth_metadata_response = client
                            .get(&format!("/registry/private-test-package"))
                            .header("Authorization", &format!("Bearer {}", token2))
                            .send()
                            .unwrap();

                        println!(
                            "Authenticated user without access metadata response status: {}",
                            auth_metadata_response.status()
                        );
                        assert_eq!(
                            auth_metadata_response.status(),
                            404,
                            "Private packages should return 404 for authenticated users without access"
                        );

                        // Test tarball access
                        let tarball_response = client
                            .get(&format!(
                                "/registry/private-test-package/-/private-test-package-1.0.0.tgz"
                            ))
                            .header("Authorization", &format!("Bearer {}", token2))
                            .send()
                            .unwrap();

                        println!("Tarball response status: {}", tarball_response.status());
                        assert_eq!(
                            tarball_response.status(),
                            404,
                            "Private package tarballs should return 404 for users without access"
                        );
                    }
                }

                // Test 3: Owner should still have access
                let owner_metadata_response = client
                    .get(&format!("/registry/private-test-package"))
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .unwrap();

                println!(
                    "Owner metadata response status: {}",
                    owner_metadata_response.status()
                );
                assert!(
                    owner_metadata_response.status().is_success(),
                    "Owner should have access to their own package"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_public_package_access() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);
        let client = ApiClient::new(server.base_url.clone());

        // Create a test package for publishing
        project.create_test_package("public-test-package", "1.0.0");

        // Register package owner
        let owner_doc = json!({
            "name": "publicowner",
            "password": "publicpassword123",
            "email": "publicowner@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:publicowner")
            .json(&owner_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token for owner
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Publish the package as owner
                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                assert!(
                    publish_output.status.success(),
                    "Owner should be able to publish package"
                );

                // Test 1: Unauthenticated user should be able to access public package
                let metadata_response = client
                    .get(&format!("/registry/public-test-package"))
                    .send()
                    .unwrap();

                println!(
                    "Public package metadata response status: {}",
                    metadata_response.status()
                );
                assert!(
                    metadata_response.status().is_success(),
                    "Public packages should be accessible without authentication"
                );

                // Test 2: Authenticated user should also be able to access public package
                let auth_metadata_response = client
                    .get(&format!("/registry/public-test-package"))
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .unwrap();

                assert!(
                    auth_metadata_response.status().is_success(),
                    "Public packages should be accessible with authentication"
                );
            }
        }
    }
}
