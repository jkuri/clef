use super::*;
use serde_json::json;
use serial_test::serial;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_user_registration_api() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test user registration
        let register_data = json!({
            "name": "testuser",
            "email": "testuser@example.com",
            "password": "testpassword123"
        });

        let response = client
            .post("/register")
            .json(&register_data)
            .send()
            .unwrap();

        // The register endpoint should succeed
        assert!(
            response.status().is_success(),
            "Register endpoint failed with status: {}",
            response.status()
        );

        let result: serde_json::Value = response.json().unwrap();
        assert_eq!(result["ok"], true);
        assert!(result["token"].is_string());
    }

    #[test]
    #[serial]
    fn test_user_login_api() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First register a user
        let register_data = json!({
            "name": "loginuser",
            "email": "loginuser@example.com",
            "password": "loginpassword123"
        });

        let register_response = client
            .post("/register")
            .json(&register_data)
            .send()
            .unwrap();

        // The register endpoint should succeed
        assert!(
            register_response.status().is_success(),
            "Register endpoint failed with status: {}",
            register_response.status()
        );

        // Then test login
        let login_data = json!({
            "name": "loginuser",
            "password": "loginpassword123"
        });

        let login_response = client.post("/login").json(&login_data).send().unwrap();

        // The login endpoint should succeed
        assert!(
            login_response.status().is_success(),
            "Login endpoint failed with status: {}",
            login_response.status()
        );

        let result: serde_json::Value = login_response.json().unwrap();
        assert_eq!(result["ok"], true);
        assert!(result["token"].is_string());
    }

    #[test]
    #[serial]
    fn test_npm_login_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test npm-style login (PUT /-/user/org.couchdb.user:username)
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

        // The npm user registration should succeed
        assert!(
            response.status().is_success(),
            "NPM user registration failed with status: {}",
            response.status()
        );

        let result: serde_json::Value = response.json().unwrap();
        assert_eq!(result["ok"], true);
        assert_eq!(result["id"], "org.couchdb.user:npmuser");
        assert!(result["token"].is_string());
    }

    #[test]
    #[serial]
    fn test_npm_whoami_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());

        // First login to get a token
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:whoamiuser",
            "name": "whoamiuser",
            "password": "whoamipassword123",
            "email": "whoamiuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let login_response = client
            .put("/registry/-/user/org.couchdb.user:whoamiuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        // The login response should succeed
        assert!(
            login_response.status().is_success(),
            "Login response failed with status: {}",
            login_response.status()
        );

        let login_result: serde_json::Value = login_response.json().unwrap();
        let token = login_result["token"].as_str().unwrap();
        client.set_auth_token(token.to_string());

        // Test whoami endpoint
        let whoami_response = client.get("/registry/-/whoami").send().unwrap();

        // The whoami endpoint should succeed
        assert!(
            whoami_response.status().is_success(),
            "Whoami endpoint failed with status: {}",
            whoami_response.status()
        );

        let result: serde_json::Value = whoami_response.json().unwrap();
        assert_eq!(result["username"], "whoamiuser");
    }

    #[test]
    #[serial]
    fn test_invalid_login_credentials() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test login with invalid credentials
        let login_data = json!({
            "name": "nonexistentuser",
            "password": "wrongpassword"
        });

        let response = client.post("/login").json(&login_data).send().unwrap();

        assert!(!response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_npm_login_invalid_user_id_format() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test npm login with invalid user ID format
        let npm_user_doc = json!({
            "name": "testuser",
            "password": "testpassword123",
            "email": "testuser@example.com"
        });

        let response = client
            .put("/registry/-/user/invalid-format")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        assert!(!response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_npm_login_username_mismatch() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test npm login with username mismatch
        let npm_user_doc = json!({
            "name": "differentuser",
            "password": "testpassword123",
            "email": "testuser@example.com"
        });

        let response = client
            .put("/registry/-/user/org.couchdb.user:testuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        assert!(!response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_whoami_without_authentication() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test whoami without authentication token
        let response = client.get("/registry/-/whoami").send().unwrap();
        assert!(!response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_whoami_with_invalid_token() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let mut client = ApiClient::new(server.base_url.clone());
        client.set_auth_token("invalid-token-12345".to_string());

        // Test whoami with invalid token
        let response = client.get("/registry/-/whoami").send().unwrap();
        assert!(!response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_existing_user_login() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First, register a user
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:existinguser",
            "name": "existinguser",
            "password": "existingpassword123",
            "email": "existinguser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let first_response = client
            .put("/registry/-/user/org.couchdb.user:existinguser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        // The first registration should succeed
        assert!(
            first_response.status().is_success(),
            "First user registration failed with status: {}",
            first_response.status()
        );

        // Then try to "login" again (should authenticate existing user)
        let second_response = client
            .put("/registry/-/user/org.couchdb.user:existinguser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        // The second authentication should succeed
        assert!(
            second_response.status().is_success(),
            "Second user authentication failed with status: {}",
            second_response.status()
        );

        let result: serde_json::Value = second_response.json().unwrap();
        assert_eq!(result["ok"], true);
        assert!(result["token"].is_string());
    }

    #[test]
    #[serial]
    fn test_duplicate_user_registration() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Register a user
        let register_data = json!({
            "name": "duplicateuser",
            "email": "duplicateuser@example.com",
            "password": "duplicatepassword123"
        });

        let first_response = client
            .post("/register")
            .json(&register_data)
            .send()
            .unwrap();

        // The first registration should succeed
        assert!(
            first_response.status().is_success(),
            "First user registration failed with status: {}",
            first_response.status()
        );

        // Try to register the same user again
        let second_response = client
            .post("/register")
            .json(&register_data)
            .send()
            .unwrap();

        // Should fail with conflict or bad request
        assert!(!second_response.status().is_success());
    }

    #[test]
    #[serial]
    fn test_npm_logout_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First login to get a token
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:logoutuser",
            "name": "logoutuser",
            "password": "logoutpassword123",
            "email": "logoutuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let login_response = client
            .put("/registry/-/user/org.couchdb.user:logoutuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        // The login response should succeed
        assert!(
            login_response.status().is_success(),
            "Login response failed with status: {}",
            login_response.status()
        );

        let login_result: serde_json::Value = login_response.json().unwrap();
        assert_eq!(login_result["ok"], true);
        let token = login_result["token"].as_str().unwrap();

        // Verify the token works by calling whoami
        let whoami_response = client
            .get("/registry/-/whoami")
            .bearer_auth(token)
            .send()
            .unwrap();

        assert!(
            whoami_response.status().is_success(),
            "Whoami failed with status: {}",
            whoami_response.status()
        );

        // Now logout using the token
        let logout_response = client
            .delete(&format!("/registry/-/user/token/{}", token))
            .send()
            .unwrap();

        // The logout should succeed
        assert!(
            logout_response.status().is_success(),
            "Logout failed with status: {}",
            logout_response.status()
        );

        let logout_result: serde_json::Value = logout_response.json().unwrap();
        assert_eq!(logout_result["ok"], true);

        // Verify the token no longer works by calling whoami again
        let whoami_after_logout = client
            .get("/registry/-/whoami")
            .bearer_auth(token)
            .send()
            .unwrap();

        // Should fail with unauthorized
        assert_eq!(whoami_after_logout.status(), 401);
    }

    #[test]
    #[serial]
    fn test_npm_logout_invalid_token() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Try to logout with an invalid token
        let invalid_token = "invalid-token-12345";
        let logout_response = client
            .delete(&format!("/registry/-/user/token/{}", invalid_token))
            .send()
            .unwrap();

        // The logout should still succeed (idempotent behavior)
        // Even if the token doesn't exist, logout should return ok: true
        assert!(
            logout_response.status().is_success(),
            "Logout with invalid token failed with status: {}",
            logout_response.status()
        );

        let logout_result: serde_json::Value = logout_response.json().unwrap();
        assert_eq!(logout_result["ok"], true);
    }

    #[test]
    #[serial]
    fn test_npm_logout_already_revoked_token() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First login to get a token
        let npm_user_doc = json!({
            "_id": "org.couchdb.user:revokeuser",
            "name": "revokeuser",
            "password": "revokepassword123",
            "email": "revokeuser@example.com",
            "type": "user",
            "roles": [],
            "date": "2025-07-18T00:00:00.000Z"
        });

        let login_response = client
            .put("/registry/-/user/org.couchdb.user:revokeuser")
            .json(&npm_user_doc)
            .send()
            .unwrap();

        assert!(login_response.status().is_success());
        let login_result: serde_json::Value = login_response.json().unwrap();
        let token = login_result["token"].as_str().unwrap();

        // Logout once
        let first_logout = client
            .delete(&format!("/registry/-/user/token/{}", token))
            .send()
            .unwrap();

        assert!(first_logout.status().is_success());

        // Try to logout again with the same token
        let second_logout = client
            .delete(&format!("/registry/-/user/token/{}", token))
            .send()
            .unwrap();

        // Should still succeed (idempotent)
        assert!(
            second_logout.status().is_success(),
            "Second logout failed with status: {}",
            second_logout.status()
        );

        let logout_result: serde_json::Value = second_logout.json().unwrap();
        assert_eq!(logout_result["ok"], true);
    }
}
