use super::common::test_app_db::TestAppWithDb;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_db_connection() {
    let app = TestAppWithDb::new().await;
    app.cleanup().await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "db_test_user",
            "email": "db_test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");

    assert!(response.status().is_success(), "Register should succeed");
}

#[tokio::test]
#[serial]
async fn test_db_friend_flow() {
    let app = TestAppWithDb::new().await;
    app.cleanup().await;

    let client = app.client();

    let user1_response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "friend_user1",
            "email": "friend1@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user1");
    assert!(user1_response.status().is_success());

    let login1 = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": "friend_user1",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login user1");
    let user1_data: serde_json::Value = login1.json().await.expect("Failed to parse");
    let user1_token = user1_data["access_token"].as_str().expect("No token");
    let _user1_id = user1_data["user"]["id"].as_str().expect("No user id");

    let user2_response = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "friend_user2",
            "email": "friend2@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user2");
    assert!(user2_response.status().is_success());

    let login2 = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": "friend_user2",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login user2");
    let user2_data: serde_json::Value = login2.json().await.expect("Failed to parse");
    let user2_token = user2_data["access_token"].as_str().expect("No token");
    let user2_id = user2_data["user"]["id"].as_str().expect("No user id");

    let send_request = client
        .post(format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1_token))
        .json(&json!({
            "to_user_id": user2_id,
            "message": "Let's be friends!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    if !send_request.status().is_success() {
        let error: serde_json::Value = send_request.json().await.expect("Failed to parse error");
        panic!("Send friend request failed: {:?}", error);
    }

    let get_requests = client
        .get(format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .expect("Failed to get requests");
    assert!(get_requests.status().is_success());

    let requests_data: serde_json::Value = get_requests.json().await.expect("Failed to parse");
    let requests = requests_data["requests"]
        .as_array()
        .expect("No requests array");
    assert!(!requests.is_empty(), "Should have friend request");

    let request_id = requests[0]["id"].as_str().expect("No request id");

    let accept_request = client
        .put(format!(
            "{}/api/v1/friends/requests/{}/accept",
            app.base_url(),
            request_id
        ))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .expect("Failed to accept request");
    assert!(
        accept_request.status().is_success(),
        "Accept request should succeed"
    );

    let get_friends = client
        .get(format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1_token))
        .send()
        .await
        .expect("Failed to get friends");
    assert!(get_friends.status().is_success());

    let friends_data: serde_json::Value = get_friends.json().await.expect("Failed to parse");
    let friends = friends_data["friends"]
        .as_array()
        .expect("No friends array");
    assert!(!friends.is_empty(), "Should have friends");

    app.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_db_create_conversation_with_friend() {
    let app = TestAppWithDb::new().await;
    app.cleanup().await;

    let client = app.client();

    let reg1 = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "conv_user1",
            "email": "conv1@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    assert!(reg1.status().is_success());

    let login1 = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": "conv_user1",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    let user1_data: serde_json::Value = login1.json().await.expect("Failed to parse");
    let user1_token = user1_data["access_token"].as_str().expect("No token");
    let _user1_id = user1_data["user"]["id"].as_str().expect("No user id");

    let reg2 = client
        .post(format!("{}/api/v1/auth/register", app.base_url()))
        .json(&json!({
            "username": "conv_user2",
            "email": "conv2@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register");
    assert!(reg2.status().is_success());

    let login2 = client
        .post(format!("{}/api/v1/auth/login", app.base_url()))
        .json(&json!({
            "username": "conv_user2",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login");
    let user2_data: serde_json::Value = login2.json().await.expect("Failed to parse");
    let user2_token = user2_data["access_token"].as_str().expect("No token");
    let user2_id = user2_data["user"]["id"].as_str().expect("No user id");

    let send_req = client
        .post(format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1_token))
        .json(&json!({
            "to_user_id": user2_id
        }))
        .send()
        .await
        .expect("Failed to send request");

    if !send_req.status().is_success() {
        let error: serde_json::Value = send_req.json().await.expect("Failed to parse error");
        panic!("Send friend request failed: {:?}", error);
    }

    let get_reqs = client
        .get(format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .expect("Failed to get requests");
    let reqs_data: serde_json::Value = get_reqs.json().await.expect("Failed to parse");
    let req_id = reqs_data["requests"][0]["id"]
        .as_str()
        .expect("No request id");

    let accept = client
        .put(format!(
            "{}/api/v1/friends/requests/{}/accept",
            app.base_url(),
            req_id
        ))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .expect("Failed to accept");
    assert!(accept.status().is_success());

    let create_conv = client
        .post(format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1_token))
        .json(&json!({
            "participant_ids": [user2_id]
        }))
        .send()
        .await
        .expect("Failed to create conversation");
    assert!(
        create_conv.status().is_success(),
        "Create conversation with friend should succeed"
    );

    app.cleanup().await;
}
