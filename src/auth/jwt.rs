use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::JwtSettings;
use crate::domain::UserId;
use crate::error::{AppResult, AuthError};
use crate::infra::TokenBlacklistStore;

use super::{AuthProvider, AuthUser, TokenPair};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub roles: Vec<String>,
    pub token_type: TokenType,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub iss: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

pub struct JwtAuthProvider {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
    issuer: String,
    blacklist: Option<Arc<dyn TokenBlacklistStore>>,
}

impl JwtAuthProvider {
    pub fn new(config: &JwtSettings) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            access_token_expiry: Duration::seconds(config.access_token_expiry as i64),
            refresh_token_expiry: Duration::seconds(config.refresh_token_expiry as i64),
            issuer: config.issuer.clone(),
            blacklist: None,
        }
    }

    pub fn with_blacklist(mut self, blacklist: Arc<dyn TokenBlacklistStore>) -> Self {
        self.blacklist = Some(blacklist);
        self
    }

    pub fn refresh_token_expiry_seconds(&self) -> u64 {
        self.refresh_token_expiry.num_seconds() as u64
    }

    fn generate_token(
        &self,
        user_id: &UserId,
        username: &str,
        roles: &[String],
        token_type: TokenType,
        expiry: Duration,
    ) -> AppResult<String> {
        let now = Utc::now();
        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles: roles.to_vec(),
            token_type,
            exp: (now + expiry).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            iss: self.issuer.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::from(e).into())
    }

    fn decode_token(&self, token: &str) -> AppResult<JwtClaims> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);

        let token_data =
            decode::<JwtClaims>(token, &self.decoding_key, &validation).map_err(AuthError::from)?;

        Ok(token_data.claims)
    }

    pub fn generate_tokens_for_user(
        &self,
        user_id: &UserId,
        username: &str,
        roles: &[String],
    ) -> AppResult<TokenPair> {
        let access_token = self.generate_token(
            user_id,
            username,
            roles,
            TokenType::Access,
            self.access_token_expiry,
        )?;

        let refresh_token = self.generate_token(
            user_id,
            username,
            roles,
            TokenType::Refresh,
            self.refresh_token_expiry,
        )?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry.num_seconds() as u64,
        })
    }
}

#[async_trait]
impl AuthProvider for JwtAuthProvider {
    type Claims = AuthUser;

    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims> {
        self.validate_token(token).await
    }

    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims> {
        let claims = self.decode_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AuthError::InvalidToken.into());
        }

        if let Some(ref blacklist) = self.blacklist {
            if blacklist.is_blacklisted(&claims.jti).await? {
                return Err(AuthError::InvalidToken.into());
            }
        }

        Ok(AuthUser {
            user_id: UserId::from(
                claims
                    .sub
                    .parse::<Uuid>()
                    .map_err(|_| AuthError::InvalidToken)?,
            ),
            username: claims.username,
            roles: claims.roles,
            jti: Some(claims.jti),
        })
    }

    async fn generate_tokens(&self, user_id: &UserId) -> AppResult<TokenPair> {
        self.generate_tokens_for_user(user_id, &user_id.to_string(), &[])
    }

    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair> {
        let claims = self.decode_token(refresh_token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidToken.into());
        }

        if let Some(ref blacklist) = self.blacklist {
            if blacklist.is_blacklisted(&claims.jti).await? {
                return Err(AuthError::InvalidToken.into());
            }
        }

        let user_id = UserId::from(
            claims
                .sub
                .parse::<Uuid>()
                .map_err(|_| AuthError::InvalidToken)?,
        );

        self.generate_tokens_for_user(&user_id, &claims.username, &claims.roles)
    }

    async fn revoke_token(&self, token_id: &str) -> AppResult<()> {
        if let Some(ref blacklist) = self.blacklist {
            let ttl = self.refresh_token_expiry.num_seconds() as u64;
            blacklist.add(token_id, ttl).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_provider() -> JwtAuthProvider {
        JwtAuthProvider::new(&JwtSettings {
            secret: "test_secret_key_for_testing_purposes_only".to_string(),
            access_token_expiry: 3600,
            refresh_token_expiry: 604800,
            issuer: "test_issuer".to_string(),
        })
    }

    #[test]
    fn test_generate_and_validate_token() {
        let provider = create_test_provider();
        let user_id = UserId::new();

        let tokens = provider
            .generate_tokens_for_user(&user_id, "testuser", &["user".to_string()])
            .unwrap();

        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert_eq!(tokens.token_type, "Bearer");
    }

    #[tokio::test]
    async fn test_validate_access_token() {
        let provider = create_test_provider();
        let user_id = UserId::new();

        let tokens = provider
            .generate_tokens_for_user(&user_id, "testuser", &["user".to_string()])
            .unwrap();

        let auth_user = provider.validate_token(&tokens.access_token).await.unwrap();

        assert_eq!(auth_user.username, "testuser");
        assert!(auth_user.has_role("user"));
    }

    #[tokio::test]
    async fn test_refresh_token_fails_for_access_token() {
        let provider = create_test_provider();
        let user_id = UserId::new();

        let tokens = provider
            .generate_tokens_for_user(&user_id, "testuser", &[])
            .unwrap();

        let result = provider.refresh_token(&tokens.access_token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_token_succeeds() {
        let provider = create_test_provider();
        let user_id = UserId::new();

        let tokens = provider
            .generate_tokens_for_user(&user_id, "testuser", &["admin".to_string()])
            .unwrap();

        let new_tokens = provider.refresh_token(&tokens.refresh_token).await.unwrap();

        assert!(!new_tokens.access_token.is_empty());
        assert!(!new_tokens.refresh_token.is_empty());
    }
}
