use super::common::*;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_send_friend_request_success() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi, let's be friends!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    assert!(response.status().is_success(), "Send friend request should succeed");
}

#[tokio::test]
#[serial]
async fn test_send_friend_request_to_self() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "to_user_id": user.id,
            "message": "Friend myself"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    assert!(!response.status().is_success(), "Send friend request to self should fail");
}

#[tokio::test]
#[serial]
async fn test_send_friend_request_duplicate() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send first friend request");

    let response = client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi again!"
        }))
        .send()
        .await
        .expect("Failed to send duplicate friend request");

    assert!(!response.status().is_success(), "Duplicate friend request should fail");
}

#[tokio::test]
#[serial]
async fn test_send_friend_request_already_friends() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    assert!(!response.status().is_success(), "Friend request to already friend should fail");
}

#[tokio::test]
#[serial]
async fn test_send_friend_request_invalid_user() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "to_user_id": "invalid_user_id",
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    assert!(!response.status().is_success(), "Friend request to invalid user should fail");
}

#[tokio::test]
#[serial]
async fn test_get_pending_requests() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    let response = client
        .get(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to get pending requests");

    assert!(response.status().is_success(), "Get pending requests should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let requests = data["requests"].as_array().expect("Should have requests array");
    assert!(!requests.is_empty(), "Should have at least one request");
}

#[tokio::test]
#[serial]
async fn test_get_pending_requests_empty() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get pending requests");

    assert!(response.status().is_success(), "Get pending requests should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let requests = data["requests"].as_array().expect("Should have requests array");
    assert!(requests.is_empty(), "Should have no requests");
}

#[tokio::test]
#[serial]
async fn test_get_sent_requests() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    let response = client
        .get(&format!("{}/api/v1/friends/requests/sent", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get sent requests");

    assert!(response.status().is_success(), "Get sent requests should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let requests = data["requests"].as_array().expect("Should have requests array");
    assert!(!requests.is_empty(), "Should have at least one sent request");
}

#[tokio::test]
#[serial]
async fn test_accept_friend_request() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    let requests_response = client
        .get(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to get requests");

    let requests_data: serde_json::Value = requests_response.json().await.expect("Failed to parse");
    let request_id = requests_data["requests"][0]["id"].as_str().expect("Should have request ID");

    let response = client
        .put(&format!("{}/api/v1/friends/requests/{}/accept", app.base_url(), request_id))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to accept friend request");

    assert!(response.status().is_success(), "Accept friend request should succeed");
}

#[tokio::test]
#[serial]
async fn test_accept_friend_request_invalid_id() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .put(&format!("{}/api/v1/friends/requests/{}/accept", app.base_url(), "invalid_id"))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to accept friend request");

    assert!(!response.status().is_success(), "Accept invalid request should fail");
}

#[tokio::test]
#[serial]
async fn test_reject_friend_request() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Hi!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    let requests_response = client
        .get(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to get requests");

    let requests_data: serde_json::Value = requests_response.json().await.expect("Failed to parse");
    let request_id = requests_data["requests"][0]["id"].as_str().expect("Should have request ID");

    let response = client
        .delete(&format!("{}/api/v1/friends/requests/{}/reject", app.base_url(), request_id))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to reject friend request");

    assert!(response.status().is_success(), "Reject friend request should succeed");
}

#[tokio::test]
#[serial]
async fn test_get_friends_list() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get friends list");

    assert!(response.status().is_success(), "Get friends list should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let friends = data["friends"].as_array().expect("Should have friends array");
    assert!(!friends.is_empty(), "Should have at least one friend");
}

#[tokio::test]
#[serial]
async fn test_get_friends_list_empty() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get friends list");

    assert!(response.status().is_success(), "Get friends list should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let friends = data["friends"].as_array().expect("Should have friends array");
    assert!(friends.is_empty(), "Should have no friends");
}

#[tokio::test]
#[serial]
async fn test_delete_friend() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    
    let client = app.client();
    let response = client
        .delete(&format!("{}/api/v1/friends/{}", app.base_url(), user2.id))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to delete friend");

    assert!(response.status().is_success(), "Delete friend should succeed");

    let friends_response = client
        .get(&format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get friends list");

    let friends_data: serde_json::Value = friends_response.json().await.expect("Failed to parse");
    let friends = friends_data["friends"].as_array().expect("Should have friends array");
    
    let has_friend = friends.iter().any(|f| f["friend_id"].as_str() == Some(&user2.id));
    assert!(!has_friend, "Friend should be removed from list");
}

#[tokio::test]
#[serial]
async fn test_delete_friend_not_friend() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .delete(&format!("{}/api/v1/friends/{}", app.base_url(), user2.id))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to delete friend");

    assert!(!response.status().is_success(), "Delete non-friend should fail");
}

#[tokio::test]
#[serial]
async fn test_friend_flow_complete() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    let send_response = client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "to_user_id": user2.id,
            "message": "Let's be friends!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");
    assert!(send_response.status().is_success());

    let requests_response = client
        .get(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to get requests");
    let requests_data: serde_json::Value = requests_response.json().await.expect("Failed to parse");
    let request_id = requests_data["requests"][0]["id"].as_str().expect("Should have request ID");

    let accept_response = client
        .put(&format!("{}/api/v1/friends/requests/{}/accept", app.base_url(), request_id))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to accept friend request");
    assert!(accept_response.status().is_success());

    let friends1_response = client
        .get(&format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get friends");
    let friends1_data: serde_json::Value = friends1_response.json().await.expect("Failed to parse");
    let has_friend1 = friends1_data["friends"].as_array().unwrap().iter()
        .any(|f| f["friend_id"].as_str() == Some(&user2.id));
    assert!(has_friend1, "User1 should have user2 as friend");

    let friends2_response = client
        .get(&format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to get friends");
    let friends2_data: serde_json::Value = friends2_response.json().await.expect("Failed to parse");
    let has_friend2 = friends2_data["friends"].as_array().unwrap().iter()
        .any(|f| f["friend_id"].as_str() == Some(&user1.id));
    assert!(has_friend2, "User2 should have user1 as friend");

    let delete_response = client
        .delete(&format!("{}/api/v1/friends/{}", app.base_url(), user2.id))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to delete friend");
    assert!(delete_response.status().is_success());

    let final_friends_response = client
        .get(&format!("{}/api/v1/friends", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get friends");
    let final_friends_data: serde_json::Value = final_friends_response.json().await.expect("Failed to parse");
    let still_has_friend = final_friends_data["friends"].as_array().unwrap().iter()
        .any(|f| f["friend_id"].as_str() == Some(&user2.id));
    assert!(!still_has_friend, "User1 should no longer have user2 as friend");
}