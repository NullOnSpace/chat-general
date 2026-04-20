use super::{TestApp, TestUser};

pub async fn make_friends(app: &TestApp, user1: &TestUser, user2: &TestUser) {
    let client = app.client();
    
    client
        .post(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .json(&serde_json::json!({
            "to_user_id": user2.id,
            "message": "Let's be friends!"
        }))
        .send()
        .await
        .expect("Failed to send friend request");

    let requests_response = client
        .get(&format!("{}/api/v1/friends/requests", app.base_url()))
        .header("Authorization", format!("Bearer {}", user2.access_token))
        .send()
        .await
        .expect("Failed to get friend requests");

    let requests: serde_json::Value = requests_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse requests");

    if let Some(requests_arr) = requests["requests"].as_array() {
        if let Some(request) = requests_arr.first() {
            let request_id = request["id"].as_str().unwrap_or_default();
            
            client
                .put(&format!("{}/api/v1/friends/requests/{}/accept", app.base_url(), request_id))
                .header("Authorization", format!("Bearer {}", user2.access_token))
                .send()
                .await
                .expect("Failed to accept friend request");
        }
    }
}

pub async fn create_test_conversation(app: &TestApp, user: &TestUser, friend_id: &str) -> String {
    let client = app.client();
    
    let response = client
        .post(&format!("{}/api/v1/conversations", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&serde_json::json!({
            "participant_ids": [friend_id]
        }))
        .send()
        .await
        .expect("Failed to create conversation");

    let data: serde_json::Value = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response");

    data["id"].as_str().unwrap_or_default().to_string()
}

pub async fn send_test_message(app: &TestApp, user: &TestUser, conv_id: &str, content: &str) -> String {
    let client = app.client();
    
    let response = client
        .post(&format!("{}/api/v1/messages", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&serde_json::json!({
            "conversation_id": conv_id,
            "content": content
        }))
        .send()
        .await
        .expect("Failed to send message");

    let data: serde_json::Value = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response");

    data["id"].as_str().unwrap_or_default().to_string()
}

pub async fn create_test_group(app: &TestApp, user: &TestUser, name: &str) -> String {
    let client = app.client();
    
    let response = client
        .post(&format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&serde_json::json!({
            "name": name
        }))
        .send()
        .await
        .expect("Failed to create group");

    let data: serde_json::Value = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response");

    data["id"].as_str().unwrap_or_default().to_string()
}
