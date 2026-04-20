use redis::{AsyncCommands, Client as RedisClient};

use crate::domain::UserId;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct RedisCache {
    client: RedisClient,
}

impl RedisCache {
    pub fn new(url: &str) -> AppResult<Self> {
        let client = RedisClient::open(url)
            .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> AppResult<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> AppResult<()> {
        let mut conn = self.get_connection().await?;
        
        match ttl {
            Some(seconds) => {
                conn.set_ex::<_, _, ()>(key, value, seconds).await?;
            }
            None => {
                conn.set::<_, _, ()>(key, value).await?;
            }
        }
        
        Ok(())
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.get_connection().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    pub async fn set_json<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Option<u64>) -> AppResult<()> {
        let json = serde_json::to_string(value)?;
        self.set(key, &json, ttl).await
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> AppResult<Option<T>> {
        match self.get(key).await? {
            Some(json) => {
                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

pub struct OnlineStatusCache {
    cache: RedisCache,
    prefix: String,
}

impl OnlineStatusCache {
    pub fn new(cache: RedisCache) -> Self {
        Self {
            cache,
            prefix: "online:".to_string(),
        }
    }

    pub async fn set_online(&self, user_id: &UserId, device_id: &str, ttl: u64) -> AppResult<()> {
        let key = format!("{}{}", self.prefix, user_id);
        let mut conn = self.cache.get_connection().await?;
        
        conn.sadd::<_, _, ()>(&key, device_id).await?;
        conn.expire::<_, ()>(&key, ttl as i64).await?;
        
        Ok(())
    }

    pub async fn set_offline(&self, user_id: &UserId, device_id: &str) -> AppResult<()> {
        let key = format!("{}{}", self.prefix, user_id);
        let mut conn = self.cache.get_connection().await?;
        
        conn.srem::<_, _, ()>(&key, device_id).await?;
        
        Ok(())
    }

    pub async fn is_online(&self, user_id: &UserId) -> AppResult<bool> {
        let key = format!("{}{}", self.prefix, user_id);
        let mut conn = self.cache.get_connection().await?;
        
        let count: i64 = conn.scard(&key).await?;
        Ok(count > 0)
    }

    pub async fn get_online_devices(&self, user_id: &UserId) -> AppResult<Vec<String>> {
        let key = format!("{}{}", self.prefix, user_id);
        let mut conn = self.cache.get_connection().await?;
        
        let devices: Vec<String> = conn.smembers(&key).await?;
        Ok(devices)
    }

    pub async fn get_online_users(&self, user_ids: &[UserId]) -> AppResult<Vec<bool>> {
        let mut results = Vec::with_capacity(user_ids.len());
        
        for user_id in user_ids {
            results.push(self.is_online(user_id).await?);
        }
        
        Ok(results)
    }
}

pub struct TokenBlacklist {
    cache: RedisCache,
    prefix: String,
}

impl TokenBlacklist {
    pub fn new(cache: RedisCache) -> Self {
        Self {
            cache,
            prefix: "blacklist:".to_string(),
        }
    }

    pub async fn add(&self, token_id: &str, ttl: u64) -> AppResult<()> {
        let key = format!("{}{}", self.prefix, token_id);
        self.cache.set(&key, "1", Some(ttl)).await
    }

    pub async fn is_blacklisted(&self, token_id: &str) -> AppResult<bool> {
        let key = format!("{}{}", self.prefix, token_id);
        self.cache.exists(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_cache_creation() {
        let result = RedisCache::new("redis://localhost:6379/0");
        assert!(result.is_ok());
    }
}
