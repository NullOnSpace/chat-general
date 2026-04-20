use super::common::*;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_register_success() {
    let app = TestApp::new().await;
    let client = app.client();

    let uuid_str = uuid::Uuid::new_v4().to_string();
    let username = format!("register_test_{}", &uuid_str[..8]);

    let response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": username,
            "email": format!("{}@test.com", username),
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert!(response.status().is_success(), "Register should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(data["username"].as_str().unwrap(), username);
    assert!(data["id"].as_str().is_some());
}

#[tokio::test]
#[serial]
async fn test_register_short_password() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "short_pass_user",
            "email": "short@test.com",
            "password": "123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert!(
        !response.status().is_success(),
        "Register with short password should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_register_invalid_email() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "invalid_email_user",
            "email": "invalid-email",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert!(
        !response.status().is_success(),
        "Register with invalid email should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_register_duplicate_username() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": user.username,
            "email": format!("different_{}@test.com", uuid::Uuid::new_v4()),
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert!(
        !response.status().is_success(),
        "Register with duplicate username should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_register_duplicate_email() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": format!("different_{}", uuid::Uuid::new_v4()),
            "email": user.email,
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert!(
        !response.status().is_success(),
        "Register with duplicate email should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_login_success() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": user.username,
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert!(response.status().is_success(), "Login should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(
        data["access_token"].as_str().is_some(),
        "Should have access token"
    );
    assert!(
        data["refresh_token"].as_str().is_some(),
        "Should have refresh token"
    );
    assert_eq!(data["user"]["username"].as_str().unwrap(), user.username);
}

#[tokio::test]
#[serial]
async fn test_login_wrong_password() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": user.username,
            "password": "wrong_password"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert!(
        !response.status().is_success(),
        "Login with wrong password should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_login_user_not_found() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": "nonexistent_user",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert!(
        !response.status().is_success(),
        "Login with nonexistent user should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_login_empty_credentials() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": "",
            "password": ""
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert!(
        !response.status().is_success(),
        "Login with empty credentials should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_refresh_token_success() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/refresh", app.base_url()))
        .json(&json!({
            "refresh_token": user.refresh_token
        }))
        .send()
        .await
        .expect("Failed to send refresh request");

    if !response.status().is_success() {
        let error: serde_json::Value = response.json().await.expect("Failed to parse error");
        panic!("Token refresh failed: {:?}", error);
    }

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(
        data["access_token"].as_str().is_some(),
        "Should have new access token"
    );
    assert!(
        data["refresh_token"].as_str().is_some(),
        "Should have new refresh token"
    );
}

#[tokio::test]
#[serial]
async fn test_refresh_token_invalid() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/auth/refresh", app.base_url()))
        .json(&json!({
            "refresh_token": "invalid_token"
        }))
        .send()
        .await
        .expect("Failed to send refresh request");

    assert!(
        !response.status().is_success(),
        "Invalid refresh token should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_refresh_token_with_access_token() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/refresh", app.base_url()))
        .json(&json!({
            "refresh_token": user.access_token
        }))
        .send()
        .await
        .expect("Failed to send refresh request");

    assert!(
        !response.status().is_success(),
        "Using access token as refresh token should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_get_current_user_authenticated() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .get(format!("{}/api/v1/auth/me", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get current user");

    assert!(
        response.status().is_success(),
        "Get current user should succeed"
    );

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(data["username"].as_str().unwrap(), user.username);
}

#[tokio::test]
#[serial]
async fn test_get_current_user_unauthenticated() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .get(format!("{}/api/v1/auth/me", app.base_url()))
        .send()
        .await
        .expect("Failed to get current user");

    assert!(
        !response.status().is_success(),
        "Get current user without auth should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_get_current_user_invalid_token() {
    let app = TestApp::new().await;
    let client = app.client();

    let response = client
        .get(format!("{}/api/v1/auth/me", app.base_url()))
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await
        .expect("Failed to get current user");

    assert!(
        !response.status().is_success(),
        "Get current user with invalid token should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_logout() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/logout", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to logout");

    assert!(response.status().is_success(), "Logout should succeed");
}

#[tokio::test]
#[serial]
async fn test_search_users() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .get(format!(
            "{}/api/v1/users/search?q={}",
            app.base_url(),
            user.username
        ))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to search users");

    assert!(
        response.status().is_success(),
        "Search users should succeed"
    );

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(
        data["users"].as_array().is_some(),
        "Should have users array"
    );
}

#[tokio::test]
#[serial]
async fn test_search_users_empty_query() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .get(format!("{}/api/v1/users/search?q=", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to search users");

    assert!(
        response.status().is_success(),
        "Search users with empty query should succeed"
    );
}
