use super::common::*;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_group() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "name": "Test Group",
            "description": "A test group for E2E testing"
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(response.status().is_success(), "Create group should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(data["id"].as_str().is_some(), "Should have group ID");
    assert_eq!(data["name"].as_str().unwrap(), "Test Group");
}

#[tokio::test]
#[serial]
async fn test_create_group_empty_name() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "name": ""
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(!response.status().is_success(), "Create group with empty name should fail");
}

#[tokio::test]
#[serial]
async fn test_create_group_unauthenticated() {
    let app = TestApp::new().await;
    
    let client = app.client();
    let response = client
        .post(&format!("{}/api/v1/groups", app.base_url()))
        .json(&json!({
            "name": "Test Group"
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(!response.status().is_success(), "Create group without auth should fail");
}

#[tokio::test]
#[serial]
async fn test_get_groups() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    create_test_group(&app, &user, "Test Group 1").await;
    create_test_group(&app, &user, "Test Group 2").await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get groups");

    assert!(response.status().is_success(), "Get groups should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let groups = data["groups"].as_array().expect("Should have groups array");
    assert!(groups.len() >= 2, "Should have at least 2 groups");
}

#[tokio::test]
#[serial]
async fn test_get_groups_empty() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get groups");

    assert!(response.status().is_success(), "Get groups should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let groups = data["groups"].as_array().expect("Should have groups array");
    assert!(groups.is_empty(), "Should have no groups");
}

#[tokio::test]
#[serial]
async fn test_get_group_detail() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    let group_id = create_test_group(&app, &user, "Detail Test Group").await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/groups/{}", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get group detail");

    assert!(response.status().is_success(), "Get group detail should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(data["id"].as_str().unwrap(), group_id);
    assert_eq!(data["name"].as_str().unwrap(), "Detail Test Group");
}

#[tokio::test]
#[serial]
async fn test_get_group_not_found() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let fake_id = uuid::Uuid::new_v4().to_string();
    let response = client
        .get(&format!("{}/api/v1/groups/{}", app.base_url(), fake_id))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get group detail");

    assert!(!response.status().is_success(), "Get non-existent group should fail");
}

#[tokio::test]
#[serial]
async fn test_get_group_invalid_id() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    
    let client = app.client();
    let response = client
        .get(&format!("{}/api/v1/groups/invalid_id", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get group detail");

    assert!(!response.status().is_success(), "Get group with invalid ID should fail");
}

#[tokio::test]
#[serial]
async fn test_add_group_member() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member = TestUser::create_unique(&app).await;
    let group_id = create_test_group(&app, &owner, "Member Test Group").await;
    
    let client = app.client();
    let response = client
        .put(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add group member");

    assert!(response.status().is_success(), "Add group member should succeed");
}

#[tokio::test]
#[serial]
async fn test_add_group_member_invalid_user() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let group_id = create_test_group(&app, &owner, "Test Group").await;
    
    let client = app.client();
    let response = client
        .put(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": "invalid_user_id"
        }))
        .send()
        .await
        .expect("Failed to add group member");

    assert!(!response.status().is_success(), "Add invalid member should fail");
}

#[tokio::test]
#[serial]
async fn test_get_group_members() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member = TestUser::create_unique(&app).await;
    let group_id = create_test_group(&app, &owner, "Members Test Group").await;
    
    let client = app.client();
    
    client
        .put(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add member");

    let response = client
        .get(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get group members");

    assert!(response.status().is_success(), "Get group members should succeed");

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let members = data["members"].as_array().expect("Should have members array");
    assert!(members.len() >= 2, "Should have at least 2 members (owner + added member)");
}

#[tokio::test]
#[serial]
async fn test_remove_group_member() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member = TestUser::create_unique(&app).await;
    let group_id = create_test_group(&app, &owner, "Remove Member Test").await;
    
    let client = app.client();
    
    client
        .put(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add member");

    let response = client
        .delete(&format!("{}/api/v1/groups/{}/members/{}", app.base_url(), group_id, member.id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to remove group member");

    assert!(response.status().is_success(), "Remove group member should succeed");
}

#[tokio::test]
#[serial]
async fn test_group_flow_complete() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member1 = TestUser::create_unique(&app).await;
    let member2 = TestUser::create_unique(&app).await;
    
    let client = app.client();
    
    let create_response = client
        .post(&format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Complete Flow Test Group",
            "description": "Testing complete group flow"
        }))
        .send()
        .await
        .expect("Failed to create group");
    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let group_id = create_data["id"].as_str().expect("Should have group ID");

    let add1_response = client
        .put(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member1.id
        }))
        .send()
        .await
        .expect("Failed to add member1");
    assert!(add1_response.status().is_success());

    let add2_response = client
        .put(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member2.id
        }))
        .send()
        .await
        .expect("Failed to add member2");
    assert!(add2_response.status().is_success());

    let members_response = client
        .get(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get members");
    assert!(members_response.status().is_success());
    let members_data: serde_json::Value = members_response.json().await.expect("Failed to parse");
    let members = members_data["members"].as_array().expect("Should have members");
    assert!(members.len() >= 3, "Should have at least 3 members");

    let remove_response = client
        .delete(&format!("{}/api/v1/groups/{}/members/{}", app.base_url(), group_id, member1.id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to remove member");
    assert!(remove_response.status().is_success());

    let final_members_response = client
        .get(&format!("{}/api/v1/groups/{}/members", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get final members");
    let final_members_data: serde_json::Value = final_members_response.json().await.expect("Failed to parse");
    let final_members = final_members_data["members"].as_array().expect("Should have members");
    
    let has_member1 = final_members.iter().any(|m| m["user_id"].as_str() == Some(&member1.id));
    assert!(!has_member1, "Member1 should be removed");
}