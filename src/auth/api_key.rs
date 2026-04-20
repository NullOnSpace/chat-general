use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::UserId;
use crate::error::{AppError, AppResult, AuthError};

use super::{AuthProvider, AuthUser, TokenPair};

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub key: String,
    pub user_id: UserId,
    pub name: String,
    pub scopes: Vec<String>,
}

impl ApiKey {
    pub fn new(user_id: UserId, name: String, scopes: Vec<String>) -> Self {
        Self {
            key: format!("sk_{}", uuid::Uuid::new_v4().to_string().replace('-', "")),
            user_id,
            name,
            scopes,
        }
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope || s == "*")
    }
}

pub struct ApiKeyAuthProvider {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl ApiKeyAuthProvider {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_key(&self, api_key: ApiKey) {
        let mut keys = self.keys.write().await;
        keys.insert(api_key.key.clone(), api_key);
    }

    pub async fn revoke_key(&self, key: &str) -> AppResult<()> {
        let mut keys = self.keys.write().await;
        keys.remove(key)
            .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;
        Ok(())
    }

    pub async fn get_key(&self, key: &str) -> Option<ApiKey> {
        let keys = self.keys.read().await;
        keys.get(key).cloned()
    }
}

impl Default for ApiKeyAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for ApiKeyAuthProvider {
    type Claims = AuthUser;

    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims> {
        self.validate_token(token).await
    }

    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims> {
        let api_key = self.get_key(token).await.ok_or(AuthError::InvalidToken)?;

        Ok(AuthUser {
            user_id: api_key.user_id,
            username: api_key.name,
            roles: api_key.scopes,
        })
    }

    async fn generate_tokens(&self, user_id: &UserId) -> AppResult<TokenPair> {
        let api_key = ApiKey::new(*user_id, "generated".to_string(), vec!["*".to_string()]);
        let key = api_key.key.clone();

        self.register_key(api_key).await;

        Ok(TokenPair {
            access_token: key,
            refresh_token: String::new(),
            token_type: "ApiKey".to_string(),
            expires_in: 0,
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> AppResult<TokenPair> {
        Err(AuthError::InvalidToken.into())
    }

    async fn revoke_token(&self, token_id: &str) -> AppResult<()> {
        self.revoke_key(token_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_creation() {
        let user_id = UserId::new();
        let api_key = ApiKey::new(
            user_id,
            "test-bot".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(api_key.key.starts_with("sk_"));
        assert_eq!(api_key.name, "test-bot");
        assert!(api_key.has_scope("read"));
        assert!(api_key.has_scope("write"));
        assert!(!api_key.has_scope("admin"));
    }

    #[test]
    fn test_api_key_wildcard_scope() {
        let user_id = UserId::new();
        let api_key = ApiKey::new(user_id, "admin-bot".to_string(), vec!["*".to_string()]);

        assert!(api_key.has_scope("read"));
        assert!(api_key.has_scope("write"));
        assert!(api_key.has_scope("admin"));
    }

    #[tokio::test]
    async fn test_api_key_auth_provider() {
        let provider = ApiKeyAuthProvider::new();
        let user_id = UserId::new();
        let api_key = ApiKey::new(user_id, "test-bot".to_string(), vec!["read".to_string()]);
        let key = api_key.key.clone();

        provider.register_key(api_key).await;

        let auth_user = provider.validate_token(&key).await.unwrap();
        assert_eq!(auth_user.username, "test-bot");
    }

    #[tokio::test]
    async fn test_api_key_revoke() {
        let provider = ApiKeyAuthProvider::new();
        let user_id = UserId::new();
        let api_key = ApiKey::new(user_id, "test-bot".to_string(), vec![]);
        let key = api_key.key.clone();

        provider.register_key(api_key).await;
        provider.revoke_key(&key).await.unwrap();

        let result = provider.validate_token(&key).await;
        assert!(result.is_err());
    }
}
