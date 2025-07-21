use crate::{ApiClient, PackageManager, TestProject, TestServer, init_test_env};
use serde_json::json;
use serial_test::serial;

#[cfg(test)]
mod package_ownership_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_package_ownership_prevents_unauthorized_publishing() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);
        let client = ApiClient::new(server.base_url.clone());

        // Create a test package for publishing
        project.create_test_package("ownership-test-package", "1.0.0");

        // Register first user via API (since npm login is interactive)
        let jkuri_user_doc = json!({
            "name": "jkuri",
            "password": "jkuripassword123",
            "email": "jkuri@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:jkuri")
            .json(&jkuri_user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
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
                    "jkuri publish stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "jkuri publish stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                assert!(
                    publish_output.status.success(),
                    "jkuri should be able to publish new package"
                );

                // Now register second user and try to publish same package
                let jan_user_doc = json!({
                    "name": "jan",
                    "password": "janpassword123",
                    "email": "jan@example.com",
                    "type": "user",
                    "roles": [],
                    "date": "2025-07-18T00:00:00.000Z"
                });

                let response2 = client
                    .put("/registry/-/user/org.couchdb.user:jan")
                    .json(&jan_user_doc)
                    .send()
                    .unwrap();

                if response2.status().is_success() {
                    let result2: serde_json::Value = response2.json().unwrap();
                    if let Some(token2) = result2["token"].as_str() {
                        // Create .npmrc with auth token for jan
                        let npmrc_content2 = format!(
                            "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
                            server.base_url, server.port, token2
                        );
                        std::fs::write(&project.npmrc_path, npmrc_content2)
                            .expect("Failed to write .npmrc with auth");

                        // Try to publish the same package as jan (should fail)

                        let publish_output = project.run_command(
                            &PackageManager::Npm,
                            &PackageManager::Npm
                                .publish_args()
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>(),
                        );

                        println!(
                            "jan publish stdout: {}",
                            String::from_utf8_lossy(&publish_output.stdout)
                        );
                        println!(
                            "jan publish stderr: {}",
                            String::from_utf8_lossy(&publish_output.stderr)
                        );

                        assert!(
                            !publish_output.status.success(),
                            "jan should not be able to publish to jkuri's package"
                        );

                        // Check that the error message indicates permission denied
                        let stderr = String::from_utf8_lossy(&publish_output.stderr);
                        assert!(
                            stderr.contains("403")
                                || stderr.contains("Forbidden")
                                || stderr.contains("permission"),
                            "Error should indicate permission denied, got: {}",
                            stderr
                        );
                    }
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_package_ownership_allows_authorized_publishing() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project = TestProject::new(&server.base_url);
        let client = ApiClient::new(server.base_url.clone());

        // Create a test package for publishing
        project.create_test_package("authorized-test-package", "1.0.0");

        // Register user via API
        let user_doc = json!({
            "name": "authorizeduser",
            "password": "authorizedpassword123",
            "email": "authorizeduser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:authorizeduser")
            .json(&user_doc)
            .send()
            .unwrap();

        if response.status().is_success() {
            let result: serde_json::Value = response.json().unwrap();
            if let Some(token) = result["token"].as_str() {
                // Create .npmrc with auth token
                let npmrc_content = format!(
                    "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
                    server.base_url, server.port, token
                );
                std::fs::write(&project.npmrc_path, npmrc_content)
                    .expect("Failed to write .npmrc with auth");

                // Publish as authorized user (should succeed - new package)

                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                println!(
                    "authorized publish stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "authorized publish stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                assert!(
                    publish_output.status.success(),
                    "authorized user should be able to publish new package"
                );

                // Update package version and publish again (should succeed - same user)
                project.update_package_version("2.0.0");

                let publish_output = project.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                println!(
                    "authorized republish stdout: {}",
                    String::from_utf8_lossy(&publish_output.stdout)
                );
                println!(
                    "authorized republish stderr: {}",
                    String::from_utf8_lossy(&publish_output.stderr)
                );

                assert!(
                    publish_output.status.success(),
                    "authorized user should be able to publish new version of their own package"
                );
            }
        }
    }

    #[test]
    #[serial]
    fn test_multiple_users_cannot_claim_same_package() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let project1 = TestProject::new(&server.base_url);
        let project2 = TestProject::new(&server.base_url);
        let client = ApiClient::new(server.base_url.clone());

        // Create same package name in both projects
        project1.create_test_package("contested-package", "1.0.0");
        project2.create_test_package("contested-package", "1.0.0");

        // Register first user
        let user1_doc = json!({
            "name": "user1",
            "password": "user1password123",
            "email": "user1@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let response1 = client
            .put("/registry/-/user/org.couchdb.user:user1")
            .json(&user1_doc)
            .send()
            .unwrap();

        if response1.status().is_success() {
            let result1: serde_json::Value = response1.json().unwrap();
            if let Some(token1) = result1["token"].as_str() {
                // Create .npmrc with auth token for user1
                let npmrc_content1 = format!(
                    "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
                    server.base_url, server.port, token1
                );
                std::fs::write(&project1.npmrc_path, npmrc_content1)
                    .expect("Failed to write .npmrc with auth");

                // First user publishes (should succeed)

                let publish_output1 = project1.run_command(
                    &PackageManager::Npm,
                    &PackageManager::Npm
                        .publish_args()
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                );

                assert!(
                    publish_output1.status.success(),
                    "user1 should be able to publish new package"
                );

                // Register second user
                let user2_doc = json!({
                    "name": "user2",
                    "password": "user2password123",
                    "email": "user2@example.com",
                    "type": "user",
                    "roles": [],
                    "date": "2025-07-18T00:00:00.000Z"
                });

                let response2 = client
                    .put("/registry/-/user/org.couchdb.user:user2")
                    .json(&user2_doc)
                    .send()
                    .unwrap();

                if response2.status().is_success() {
                    let result2: serde_json::Value = response2.json().unwrap();
                    if let Some(token2) = result2["token"].as_str() {
                        // Create .npmrc with auth token for user2
                        let npmrc_content2 = format!(
                            "registry={}/registry\n//127.0.0.1:{}/registry/:_authToken={}\n",
                            server.base_url, server.port, token2
                        );
                        std::fs::write(&project2.npmrc_path, npmrc_content2)
                            .expect("Failed to write .npmrc with auth");

                        // Second user tries to publish same package (should fail)
                        let publish_output2 = project2.run_command(
                            &PackageManager::Npm,
                            &PackageManager::Npm
                                .publish_args()
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>(),
                        );

                        assert!(
                            !publish_output2.status.success(),
                            "user2 should not be able to publish package already owned by user1"
                        );

                        // Check that the error message indicates permission denied
                        let stderr = String::from_utf8_lossy(&publish_output2.stderr);
                        assert!(
                            stderr.contains("403")
                                || stderr.contains("Forbidden")
                                || stderr.contains("permission"),
                            "Error should indicate permission denied, got: {}",
                            stderr
                        );
                    }
                }
            }
        }
    }
}
