use async_trait::async_trait;
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
