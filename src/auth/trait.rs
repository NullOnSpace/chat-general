use async_trait::async_trait;

use crate::domain::UserId;
use crate::error::AppResult;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    type Claims: Send + Sync + Clone + std::fmt::Debug;

    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims>;
    
    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims>;
    
    async fn generate_tokens(&self, user_id: &UserId) -> AppResult<TokenPair>;
    
    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair>;
    
    async fn revoke_token(&self, token_id: &str) -> AppResult<()>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

impl Default for TokenPair {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            refresh_token: String::new(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthUser {
    pub user_id: UserId,
    pub username: String,
    pub roles: Vec<String>,
}

impl AuthUser {
    pub fn new(user_id: UserId, username: String) -> Self {
        Self {
            user_id,
            username,
            roles: Vec::new(),
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

pub fn extract_token_from_header(header_value: &str) -> Option<String> {
    if header_value.starts_with("Bearer ") {
        Some(header_value[7..].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_pair_default() {
        let pair = TokenPair::default();
        assert_eq!(pair.token_type, "Bearer");
        assert_eq!(pair.expires_in, 3600);
    }

    #[test]
    fn test_auth_user_creation() {
        let user_id = UserId::new();
        let auth_user = AuthUser::new(user_id, "testuser".to_string());
        
        assert_eq!(auth_user.username, "testuser");
        assert!(auth_user.roles.is_empty());
    }

    #[test]
    fn test_auth_user_roles() {
        let user_id = UserId::new();
        let auth_user = AuthUser::new(user_id, "testuser".to_string())
            .with_roles(vec!["admin".to_string(), "user".to_string()]);
        
        assert!(auth_user.has_role("admin"));
        assert!(auth_user.has_role("user"));
        assert!(!auth_user.has_role("guest"));
    }

    #[test]
    fn test_extract_token_from_header() {
        let token = extract_token_from_header("Bearer test_token_123");
        assert_eq!(token, Some("test_token_123".to_string()));
        
        let no_token = extract_token_from_header("Basic abc123");
        assert!(no_token.is_none());
    }
}
