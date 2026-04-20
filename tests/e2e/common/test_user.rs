use serde_json::json;
use uuid::Uuid;

use super::TestApp;

pub struct TestUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl TestUser {
    pub async fn create(app: &TestApp, username: &str) -> Self {
        let client = app.client();
        let email = format!("{}@test.example.com", username);
        
        let _register_response = client
            .post(&format!("{}/api/v1/auth/register", app.base_url()))
            .json(&json!({
                "username": username,
                "email": email,
                "password": "password123"
            }))
            .send()
            .await
            .expect("Failed to register user");

        let login_response = client
            .post(&format!("{}/api/v1/auth/login", app.base_url()))
            .json(&json!({
                "username": username,
                "password": "password123"
            }))
            .send()
            .await
            .expect("Failed to login user");

        let login_data: serde_json::Value = login_response
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse login response");

        Self {
            id: login_data["user"]["id"].as_str().unwrap_or_default().to_string(),
            username: username.to_string(),
            email,
            access_token: login_data["access_token"].as_str().unwrap_or_default().to_string(),
            refresh_token: login_data["refresh_token"].as_str().unwrap_or_default().to_string(),
        }
    }

    pub async fn create_unique(app: &TestApp) -> Self {
        let unique_id = Uuid::new_v4().to_string()[..8].to_string();
        let username = format!("test_user_{}", unique_id);
        Self::create(app, &username).await
    }

    pub fn device_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}
