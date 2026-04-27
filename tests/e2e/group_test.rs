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
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "name": "Test Group",
            "description": "A test group for E2E testing"
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(
        response.status().is_success(),
        "Create group should succeed"
    );

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
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&json!({
            "name": ""
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(
        !response.status().is_success(),
        "Create group with empty name should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_create_group_unauthenticated() {
    let app = TestApp::new().await;

    let client = app.client();
    let response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .json(&json!({
            "name": "Test Group"
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(
        !response.status().is_success(),
        "Create group without auth should fail"
    );
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
        .get(format!("{}/api/v1/groups", app.base_url()))
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
        .get(format!("{}/api/v1/groups", app.base_url()))
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
        .get(format!("{}/api/v1/groups/{}", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get group detail");

    assert!(
        response.status().is_success(),
        "Get group detail should succeed"
    );

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
        .get(format!("{}/api/v1/groups/{}", app.base_url(), fake_id))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get group detail");

    assert!(
        !response.status().is_success(),
        "Get non-existent group should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_get_group_invalid_id() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let client = app.client();
    let response = client
        .get(format!("{}/api/v1/groups/invalid_id", app.base_url()))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .send()
        .await
        .expect("Failed to get group detail");

    assert!(
        !response.status().is_success(),
        "Get group with invalid ID should fail"
    );
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
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add group member");

    assert!(
        response.status().is_success(),
        "Add group member should succeed"
    );
}

#[tokio::test]
#[serial]
async fn test_add_group_member_invalid_user() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let group_id = create_test_group(&app, &owner, "Test Group").await;

    let client = app.client();
    let response = client
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": "invalid_user_id"
        }))
        .send()
        .await
        .expect("Failed to add group member");

    assert!(
        !response.status().is_success(),
        "Add invalid member should fail"
    );
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
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add member");

    let response = client
        .get(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get group members");

    assert!(
        response.status().is_success(),
        "Get group members should succeed"
    );

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let members = data["members"]
        .as_array()
        .expect("Should have members array");
    assert!(
        members.len() >= 2,
        "Should have at least 2 members (owner + added member)"
    );
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
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add member");

    let response = client
        .delete(format!(
            "{}/api/v1/groups/{}/members/{}",
            app.base_url(),
            group_id,
            member.id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to remove group member");

    assert!(
        response.status().is_success(),
        "Remove group member should succeed"
    );
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
        .post(format!("{}/api/v1/groups", app.base_url()))
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
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member1.id
        }))
        .send()
        .await
        .expect("Failed to add member1");
    assert!(add1_response.status().is_success());

    let add2_response = client
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member2.id
        }))
        .send()
        .await
        .expect("Failed to add member2");
    assert!(add2_response.status().is_success());

    let members_response = client
        .get(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get members");
    assert!(members_response.status().is_success());
    let members_data: serde_json::Value = members_response.json().await.expect("Failed to parse");
    let members = members_data["members"]
        .as_array()
        .expect("Should have members");
    assert!(members.len() >= 3, "Should have at least 3 members");

    let remove_response = client
        .delete(format!(
            "{}/api/v1/groups/{}/members/{}",
            app.base_url(),
            group_id,
            member1.id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to remove member");
    assert!(remove_response.status().is_success());

    let final_members_response = client
        .get(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get final members");
    let final_members_data: serde_json::Value = final_members_response
        .json()
        .await
        .expect("Failed to parse");
    let final_members = final_members_data["members"]
        .as_array()
        .expect("Should have members");

    let has_member1 = final_members
        .iter()
        .any(|m| m["user_id"].as_str() == Some(&member1.id));
    assert!(!has_member1, "Member1 should be removed");
}

#[tokio::test]
#[serial]
async fn test_create_group_with_member_ids() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member1 = TestUser::create_unique(&app).await;
    let member2 = TestUser::create_unique(&app).await;

    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Group With Members",
            "description": "Created with initial members",
            "member_ids": [member1.id, member2.id]
        }))
        .send()
        .await
        .expect("Failed to create group with member_ids");

    assert!(
        response.status().is_success(),
        "Create group with member_ids should succeed"
    );

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    let group_id = data["id"].as_str().expect("Should have group ID");

    let members_response = client
        .get(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get group members");

    let members_data: serde_json::Value = members_response.json().await.expect("Failed to parse");
    let members = members_data["members"]
        .as_array()
        .expect("Should have members array");

    let has_member1 = members
        .iter()
        .any(|m| m["user_id"].as_str() == Some(&member1.id));
    let has_member2 = members
        .iter()
        .any(|m| m["user_id"].as_str() == Some(&member2.id));
    let has_owner = members
        .iter()
        .any(|m| m["user_id"].as_str() == Some(&owner.id));

    assert!(has_owner, "Owner should be in the group");
    assert!(has_member1, "Member1 should be in the group");
    assert!(has_member2, "Member2 should be in the group");
}

#[tokio::test]
#[serial]
async fn test_create_group_with_invalid_member_ids() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;

    let client = app.client();

    let response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Group With Bad Members",
            "member_ids": ["not-a-valid-uuid"]
        }))
        .send()
        .await
        .expect("Failed to create group");

    assert!(
        response.status().is_success(),
        "Create group should succeed (invalid member_ids are silently skipped)"
    );

    let data: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(data["id"].as_str().is_some(), "Should have group ID");
}

#[tokio::test]
#[serial]
async fn test_non_owner_cannot_add_member() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let non_owner = TestUser::create_unique(&app).await;
    let new_member = TestUser::create_unique(&app).await;

    let client = app.client();

    let create_response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Owner Only Group"
        }))
        .send()
        .await
        .expect("Failed to create group");
    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let group_id = create_data["id"].as_str().expect("Should have group ID");

    let add_response = client
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header(
            "Authorization",
            format!("Bearer {}", non_owner.access_token),
        )
        .json(&json!({
            "user_id": new_member.id
        }))
        .send()
        .await
        .expect("Failed to add member");

    assert!(
        !add_response.status().is_success(),
        "Non-owner should not be able to add members"
    );
}

#[tokio::test]
#[serial]
async fn test_non_owner_cannot_remove_member() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member = TestUser::create_unique(&app).await;
    let non_owner = TestUser::create_unique(&app).await;

    let client = app.client();

    let create_response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Owner Only Group",
            "member_ids": [member.id]
        }))
        .send()
        .await
        .expect("Failed to create group");
    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let group_id = create_data["id"].as_str().expect("Should have group ID");

    let remove_response = client
        .delete(format!(
            "{}/api/v1/groups/{}/members/{}",
            app.base_url(),
            group_id,
            member.id
        ))
        .header(
            "Authorization",
            format!("Bearer {}", non_owner.access_token),
        )
        .send()
        .await
        .expect("Failed to remove member");

    assert!(
        !remove_response.status().is_success(),
        "Non-owner should not be able to remove members"
    );
}

#[tokio::test]
#[serial]
async fn test_add_duplicate_member() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member = TestUser::create_unique(&app).await;

    let group_id = create_test_group(&app, &owner, "Duplicate Member Group").await;

    let client = app.client();

    let add1_response = client
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add member first time");
    assert!(add1_response.status().is_success());

    let add2_response = client
        .put(format!(
            "{}/api/v1/groups/{}/members",
            app.base_url(),
            group_id
        ))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "user_id": member.id
        }))
        .send()
        .await
        .expect("Failed to add member second time");

    assert!(
        !add2_response.status().is_success(),
        "Adding duplicate member should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_remove_self_from_group_requires_admin() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;
    let member = TestUser::create_unique(&app).await;

    let client = app.client();

    let create_response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Leave Group Test",
            "member_ids": [member.id]
        }))
        .send()
        .await
        .expect("Failed to create group");
    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let group_id = create_data["id"].as_str().expect("Should have group ID");

    let leave_response = client
        .delete(format!(
            "{}/api/v1/groups/{}/members/{}",
            app.base_url(),
            group_id,
            member.id
        ))
        .header("Authorization", format!("Bearer {}", member.access_token))
        .send()
        .await
        .expect("Failed to leave group");

    assert!(
        !leave_response.status().is_success(),
        "Regular member should not be able to remove themselves (requires admin)"
    );
}

#[tokio::test]
#[serial]
async fn test_get_group_detail_includes_fields() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;

    let client = app.client();

    let create_response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "Detailed Group",
            "description": "A group with description"
        }))
        .send()
        .await
        .expect("Failed to create group");
    let create_data: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let group_id = create_data["id"].as_str().expect("Should have group ID");

    let detail_response = client
        .get(format!("{}/api/v1/groups/{}", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get group detail");
    assert!(detail_response.status().is_success());

    let detail: serde_json::Value = detail_response.json().await.expect("Failed to parse");
    assert_eq!(detail["name"].as_str().unwrap(), "Detailed Group");
    assert_eq!(
        detail["description"].as_str().unwrap(),
        "A group with description"
    );
    assert_eq!(detail["owner_id"].as_str().unwrap(), owner.id);
    assert!(detail["member_count"].as_u64().is_some());
}

#[tokio::test]
#[serial]
async fn test_get_group_detail_null_description() {
    let app = TestApp::new().await;
    let owner = TestUser::create_unique(&app).await;

    let client = app.client();

    let create_response = client
        .post(format!("{}/api/v1/groups", app.base_url()))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .json(&json!({
            "name": "No Description Group"
        }))
        .send()
        .await
        .expect("Failed to create group");
    let create_data: serde_json::Value = create_response.json().await.expect("Failed to parse");
    let group_id = create_data["id"].as_str().expect("Should have group ID");

    let detail_response = client
        .get(format!("{}/api/v1/groups/{}", app.base_url(), group_id))
        .header("Authorization", format!("Bearer {}", owner.access_token))
        .send()
        .await
        .expect("Failed to get group detail");
    assert!(detail_response.status().is_success());

    let detail: serde_json::Value = detail_response.json().await.expect("Failed to parse");
    assert_eq!(detail["name"].as_str().unwrap(), "No Description Group");
    assert!(
        detail["description"].is_null(),
        "Description should be null when not provided"
    );
}
