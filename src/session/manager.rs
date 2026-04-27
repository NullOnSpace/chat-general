use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DeviceId, UserId};
use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Session {
    pub fn new(user_id: UserId, device_id: DeviceId) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            user_id,
            device_id,
            created_at: now,
            last_active_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn update_last_active(&mut self) {
        self.last_active_at = Utc::now();
    }
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    user_sessions: Arc<RwLock<HashMap<UserId, Vec<SessionId>>>>,
    device_sessions: Arc<RwLock<HashMap<DeviceId, SessionId>>>,
    user_senders: Arc<RwLock<HashMap<UserId, Vec<mpsc::UnboundedSender<String>>>>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            device_sessions: Arc::new(RwLock::new(HashMap::new())),
            user_senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_sender(&self, user_id: UserId, sender: mpsc::UnboundedSender<String>) {
        let mut user_senders = self.user_senders.write().await;
        user_senders.entry(user_id).or_default().push(sender);
    }

    pub async fn unregister_sender(&self, user_id: &UserId) {
        let mut user_senders = self.user_senders.write().await;
        if let Some(senders) = user_senders.get_mut(user_id) {
            senders.retain(|s| !s.is_closed());
            if senders.is_empty() {
                user_senders.remove(user_id);
            }
        }
    }

    pub async fn send_to_user(&self, user_id: &UserId, message: &str) -> usize {
        let user_senders = self.user_senders.read().await;
        if let Some(senders) = user_senders.get(user_id) {
            let mut sent = 0;
            for sender in senders {
                if sender.send(message.to_string()).is_ok() {
                    sent += 1;
                }
            }
            sent
        } else {
            0
        }
    }

    pub async fn is_user_online(&self, user_id: &UserId) -> bool {
        let user_senders = self.user_senders.read().await;
        user_senders.get(user_id).is_some_and(|s| !s.is_empty())
    }

    pub async fn create(&self, user_id: UserId, device_id: DeviceId) -> AppResult<Session> {
        let session = Session::new(user_id, device_id);
        let session_id = session.id;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(session_id);
        }

        {
            let mut device_sessions = self.device_sessions.write().await;
            device_sessions.insert(device_id, session_id);
        }

        Ok(session)
    }

    pub async fn get(&self, session_id: &SessionId) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn get_by_device(&self, device_id: &DeviceId) -> Option<Session> {
        let device_sessions = self.device_sessions.read().await;
        let session_id = device_sessions.get(device_id)?;

        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn get_user_sessions(&self, user_id: &UserId) -> Vec<Session> {
        let user_sessions = self.user_sessions.read().await;
        let sessions = self.sessions.read().await;

        match user_sessions.get(user_id) {
            Some(session_ids) => session_ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    pub async fn update_last_active(&self, session_id: &SessionId) -> AppResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.update_last_active();
        }
        Ok(())
    }

    pub async fn terminate(&self, session_id: &SessionId) -> AppResult<Option<Session>> {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(ref s) = session {
            {
                let mut user_sessions = self.user_sessions.write().await;
                if let Some(session_list) = user_sessions.get_mut(&s.user_id) {
                    session_list.retain(|id| id != session_id);
                    if session_list.is_empty() {
                        user_sessions.remove(&s.user_id);
                    }
                }
            }

            {
                let mut device_sessions = self.device_sessions.write().await;
                device_sessions.remove(&s.device_id);
            }
        }

        Ok(session)
    }

    pub async fn terminate_user_sessions(&self, user_id: &UserId) -> AppResult<Vec<Session>> {
        let session_ids: Vec<SessionId> = {
            let user_sessions = self.user_sessions.read().await;
            user_sessions.get(user_id).cloned().unwrap_or_default()
        };

        let mut terminated = Vec::new();
        for session_id in session_ids {
            if let Some(session) = self.terminate(&session_id).await? {
                terminated.push(session);
            }
        }

        Ok(terminated)
    }

    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();

        let session = manager.create(user_id, device_id).await.unwrap();

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.device_id, device_id);
    }

    #[tokio::test]
    async fn test_get_session() {
        let manager = SessionManager::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();

        let session = manager.create(user_id, device_id).await.unwrap();
        let session_id = session.id;

        let retrieved = manager.get(&session_id).await.unwrap();
        assert_eq!(retrieved.user_id, user_id);
    }

    #[tokio::test]
    async fn test_get_by_device() {
        let manager = SessionManager::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();

        manager.create(user_id, device_id).await.unwrap();

        let session = manager.get_by_device(&device_id).await.unwrap();
        assert_eq!(session.user_id, user_id);
    }

    #[tokio::test]
    async fn test_terminate_session() {
        let manager = SessionManager::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();

        let session = manager.create(user_id, device_id).await.unwrap();
        let session_id = session.id;

        manager.terminate(&session_id).await.unwrap();

        let retrieved = manager.get(&session_id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_get_user_sessions() {
        let manager = SessionManager::new();
        let user_id = UserId::new();

        manager.create(user_id, DeviceId::new()).await.unwrap();
        manager.create(user_id, DeviceId::new()).await.unwrap();

        let sessions = manager.get_user_sessions(&user_id).await;
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_terminate_user_sessions() {
        let manager = SessionManager::new();
        let user_id = UserId::new();

        manager.create(user_id, DeviceId::new()).await.unwrap();
        manager.create(user_id, DeviceId::new()).await.unwrap();

        let terminated = manager.terminate_user_sessions(&user_id).await.unwrap();
        assert_eq!(terminated.len(), 2);

        let sessions = manager.get_user_sessions(&user_id).await;
        assert!(sessions.is_empty());
    }
}
