pub mod redis;

pub use redis::{OnlineStatusCache, RedisCache, TokenBlacklist};

use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppResult;

#[async_trait]
pub trait TokenBlacklistStore: Send + Sync {
    async fn add(&self, token_id: &str, ttl: u64) -> AppResult<()>;
    async fn is_blacklisted(&self, token_id: &str) -> AppResult<bool>;
}

#[async_trait]
impl TokenBlacklistStore for TokenBlacklist {
    async fn add(&self, token_id: &str, ttl: u64) -> AppResult<()> {
        TokenBlacklist::add(self, token_id, ttl).await
    }

    async fn is_blacklisted(&self, token_id: &str) -> AppResult<bool> {
        TokenBlacklist::is_blacklisted(self, token_id).await
    }
}

pub struct InMemoryTokenBlacklist {
    tokens: Arc<RwLock<HashSet<String>>>,
}

impl InMemoryTokenBlacklist {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

impl Default for InMemoryTokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenBlacklistStore for InMemoryTokenBlacklist {
    async fn add(&self, token_id: &str, _ttl: u64) -> AppResult<()> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token_id.to_string());
        Ok(())
    }

    async fn is_blacklisted(&self, token_id: &str) -> AppResult<bool> {
        let tokens = self.tokens.read().await;
        Ok(tokens.contains(token_id))
    }
}
