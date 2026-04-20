use super::common::*;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_conversation_with_friend() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "participant_ids": [user2.id]
        }))
        .send()
        .await
        .expect("Failed to create conversation");

    assert!(response.status().is_success(), "Create conversation with friend should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(data["id"].as_str().is_some(), "Should have conversation ID");
}

#[tokio::test]
#[serial]
async fn test_create_conversation_non_friend() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "participant_ids": [user2.id]
        }))
        .send()
        .await
        .expect("Failed to create conversation");

    assert!(!response.status().is_success(), "Create conversation with non-friend should fail");
}

#[tokio::test]
#[serial]
async fn test_create_conversation_empty_participants() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "participant_ids": []
        }))
        .send()
        .await
        .expect("Failed to create conversation");

    assert!(!response.status().is_success(), "Create conversation with empty participants should fail");
}

#[tokio::test]
#[serial]
async fn test_create_conversation_invalid_participant() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "participant_ids": ["invalid_user_id"]
        }))
        .send()
        .await
        .expect("Failed to create conversation");

    assert!(!response.status().is_success(), "Create conversation with invalid participant should fail");
}

#[tokio::test]
#[serial]
async fn test_create_conversation_unauthenticated() {
    let app = TestApp::new().await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .json(&json!({
            "participant_ids": ["some_user_id"]
        }))
        .send()
        .await
        .expect("Failed to create conversation");

    assert!(!response.status().is_success(), "Create conversation without auth should fail");
}

#[tokio::test]
#[serial]
async fn test_get_conversations() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    create_test_conversation(&app, &user1, &user2.id).await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get conversations");

    assert!(response.status().is_success(), "Get conversations should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(data["conversations"].as_array().is_some(), "Should have conversations array");
}

#[tokio::test]
#[serial]
async fn test_send_message() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/messages", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "conversation_id": conv_id,
            "content": "Hello, this is a test message!"
        }))
        .send()
        .await
        .expect("Failed to send message");

    assert!(response.status().is_success(), "Send message should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(data["id"].as_str().is_some(), "Should have message ID");
    assert_eq!(data["content"].as_str().unwrap(), "Hello, this is a test message!");
}

#[tokio::test]
#[serial]
async fn test_send_empty_message() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/messages", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "conversation_id": conv_id,
            "content": ""
        }))
        .send()
        .await
        .expect("Failed to send message");

    assert!(!response.status().is_success(), "Send empty message should fail");
}

#[tokio::test]
#[serial]
async fn test_send_message_unauthenticated() {
    let app = TestApp::new().await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/messages", app.base_url()))
        .json(&json!({
            "conversation_id": "some_conv_id",
            "content": "Hello"
        }))
        .send()
        .await
        .expect("Failed to send message");

    assert!(!response.status().is_success(), "Send message without auth should fail");
}

#[tokio::test]
#[serial]
async fn test_get_messages() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;
    send_test_message(&app, &user1, &conv_id, "Test message 1").await;
    send_test_message(&app, &user1, &conv_id, "Test message 2").await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/conversations/{}/messages", app.base_url(), conv_id))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get messages");

    assert!(response.status().is_success(), "Get messages should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let messages = data["messages"].as_array().expect("Should have messages array");
    assert!(messages.len() >= 2, "Should have at least 2 messages");
}

#[tokio::test]
#[serial]
async fn test_get_messages_pagination() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;
    
    for i in 0..5 {
        send_test_message(&app, &user1, &conv_id, &format!("Message {}", i)).await;
    }
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/conversations/{}/messages?limit=2", app.base_url(), conv_id))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get messages");

    assert!(response.status().is_success(), "Get messages with pagination should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let messages = data["messages"].as_array().expect("Should have messages array");
    assert!(messages.len() <= 2, "Should have at most 2 messages due to limit");
}

#[tokio::test]
#[serial]
async fn test_message_flow_complete() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;
    
    make_friends(&app, &user1, &user2).await;
    
    let client = app.client();
    
    let conv_response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "participant_ids": [user2.id]
        }))
        .send()
        .await
        .expect("Failed to create conversation");
    assert!(conv_response.status().is_success());
    let conv_data: serde_json::Value = conv_response.json().await.expect("Failed to parse");
    let conv_id = conv_data["id"].as_str().expect("Should have conversation ID");

    let msg1_response = client
        .post(&format!("{}/api/v1/messages", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&json!({
            "conversation_id": conv_id,
            "content": "Hello from user1!"
        }))
        .send()
        .await
        .expect("Failed to send message");
    assert!(msg1_response.status().is_success());

    let msg2_response = client
        .post(&format!("{}/api/v1/messages", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .json(&json!({
            "conversation_id": conv_id,
            "content": "Hi there from user2!"
        }))
        .send()
        .await
        .expect("Failed to send message");
    assert!(msg2_response.status().is_success());

    let messages_response = client
        .get(&format!("{}/api/v1/conversations/{}/messages", app.base_url(), conv_id))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .send()
        .await
        .expect("Failed to get messages");
    assert!(messages_response.status().is_success());
    
    let messages_data: serde_json::Value = messages_response.json().await.expect("Failed to parse");
    let messages = messages_data["messages"].as_array().expect("Should have messages");
    assert!(messages.len() >= 2, "Should have at least 2 messages");
}