use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::User;
use crate::error::{AppError, AppResult};

pub type UserStore = Arc<RwLock<InMemoryUserStore>>;

#[async_trait]
pub trait UserStorage: Send + Sync {
    async fn create(&self, user: User) -> AppResult<User>;
    async fn get_by_id(&self, id: &str) -> Option<User>;
    async fn get_by_username(&self, username: &str) -> Option<User>;
    async fn get_by_email(&self, email: &str) -> Option<User>;
    async fn search(&self, query: &str) -> Vec<User>;
}

#[derive(Debug, Default)]
pub struct InMemoryUserStore {
    users_by_id: HashMap<String, User>,
    users_by_username: HashMap<String, User>,
    users_by_email: HashMap<String, User>,
}

impl InMemoryUserStore {
    pub fn new() -> Self {
        Self {
            users_by_id: HashMap::new(),
            users_by_username: HashMap::new(),
            users_by_email: HashMap::new(),
        }
    }

    pub fn create(&mut self, user: User) -> AppResult<User> {
        if self.users_by_username.contains_key(&user.username) {
            return Err(AppError::Conflict("Username already exists".to_string()));
        }
        if self.users_by_email.contains_key(&user.email) {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        let user_clone = user.clone();
        self.users_by_id
            .insert(user.id.to_string(), user_clone.clone());
        self.users_by_username
            .insert(user.username.clone(), user_clone.clone());
        self.users_by_email.insert(user.email.clone(), user_clone);

        Ok(user)
    }

    pub fn get_by_id(&self, id: &str) -> Option<User> {
        self.users_by_id.get(id).cloned()
    }

    pub fn get_by_username(&self, username: &str) -> Option<User> {
        self.users_by_username.get(username).cloned()
    }

    pub fn get_by_email(&self, email: &str) -> Option<User> {
        self.users_by_email.get(email).cloned()
    }

    pub fn search(&self, query: &str) -> Vec<User> {
        let query_lower = query.to_lowercase();
        self.users_by_id
            .values()
            .filter(|u| {
                u.username.to_lowercase().contains(&query_lower)
                    || u.email.to_lowercase().contains(&query_lower)
                    || u.display_name
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

#[async_trait]
impl UserStorage for RwLock<InMemoryUserStore> {
    async fn create(&self, user: User) -> AppResult<User> {
        let mut store = self.write().await;
        store.create(user)
    }

    async fn get_by_id(&self, id: &str) -> Option<User> {
        let store = self.read().await;
        store.get_by_id(id)
    }

    async fn get_by_username(&self, username: &str) -> Option<User> {
        let store = self.read().await;
        store.get_by_username(username)
    }

    async fn get_by_email(&self, email: &str) -> Option<User> {
        let store = self.read().await;
        store.get_by_email(email)
    }

    async fn search(&self, query: &str) -> Vec<User> {
        let store = self.read().await;
        store.search(query)
    }
}

pub fn create_user_store() -> UserStore {
    Arc::new(RwLock::new(InMemoryUserStore::new()))
}

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStorage for PostgresUserStore {
    async fn create(&self, user: User) -> AppResult<User> {
        let existing =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1 OR email = $2")
                .bind(&user.username)
                .bind(&user.email)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(existing) = existing {
            if existing.username == user.username {
                return Err(AppError::Conflict("Username already exists".to_string()));
            }
            if existing.email == user.email {
                return Err(AppError::Conflict("Email already exists".to_string()));
            }
        }

        let record = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, avatar_url, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(user.id.as_uuid())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.status.to_string())
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_by_id(&self, id: &str) -> Option<User> {
        let uuid = match uuid::Uuid::parse_str(id) {
            Ok(u) => u,
            Err(_) => return None,
        };
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }

    async fn get_by_username(&self, username: &str) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }

    async fn get_by_email(&self, email: &str) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }

    async fn search(&self, query: &str) -> Vec<User> {
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username ILIKE $1 OR email ILIKE $1 OR display_name ILIKE $1 LIMIT 20",
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }
}
