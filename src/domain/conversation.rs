use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use super::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct ConversationId(pub Uuid);

impl ConversationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ConversationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ConversationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<String> for ConversationId {
    fn from(s: String) -> Self {
        Self(Uuid::parse_str(&s).expect("Invalid ConversationId"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ConversationType {
    #[default]
    Direct,
    Group,
}

impl std::fmt::Display for ConversationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationType::Direct => write!(f, "direct"),
            ConversationType::Group => write!(f, "group"),
        }
    }
}

impl std::str::FromStr for ConversationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "direct" => Ok(ConversationType::Direct),
            "group" => Ok(ConversationType::Group),
            _ => Err(format!("Invalid conversation type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: ConversationId,
    pub conversation_type: ConversationType,
    pub last_message_id: Option<super::MessageId>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub participants: Vec<UserId>,
    #[sqlx(skip)]
    pub unread_count: HashMap<String, u32>,
}

impl Conversation {
    pub fn new_direct(user1: UserId, user2: UserId) -> Self {
        let now = Utc::now();
        Self {
            id: ConversationId::new(),
            conversation_type: ConversationType::Direct,
            participants: vec![user1, user2],
            last_message_id: None,
            last_message_at: None,
            unread_count: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_group(participants: Vec<UserId>) -> Self {
        let now = Utc::now();
        Self {
            id: ConversationId::new(),
            conversation_type: ConversationType::Group,
            participants,
            last_message_id: None,
            last_message_at: None,
            unread_count: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_direct(&self) -> bool {
        self.conversation_type == ConversationType::Direct
    }

    pub fn is_group(&self) -> bool {
        self.conversation_type == ConversationType::Group
    }

    pub fn is_participant(&self, user_id: &UserId) -> bool {
        self.participants.contains(user_id)
    }

    pub fn other_participant(&self, user_id: &UserId) -> Option<&UserId> {
        if self.is_direct() {
            self.participants.iter().find(|id| *id != user_id)
        } else {
            None
        }
    }

    pub fn update_last_message(&mut self, message_id: super::MessageId, sent_at: DateTime<Utc>) {
        self.last_message_id = Some(message_id);
        self.last_message_at = Some(sent_at);
        self.updated_at = Utc::now();
    }

    pub fn increment_unread(&mut self, user_id: &UserId) {
        let key = user_id.to_string();
        *self.unread_count.entry(key).or_insert(0) += 1;
    }

    pub fn clear_unread(&mut self, user_id: &UserId) {
        let key = user_id.to_string();
        self.unread_count.insert(key, 0);
    }

    pub fn get_unread_count(&self, user_id: &UserId) -> u32 {
        self.unread_count
            .get(&user_id.to_string())
            .copied()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_id_creation() {
        let id1 = ConversationId::new();
        let id2 = ConversationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_direct_conversation_creation() {
        let user1 = UserId::new();
        let user2 = UserId::new();
        let conv = Conversation::new_direct(user1, user2);

        assert!(conv.is_direct());
        assert!(!conv.is_group());
        assert_eq!(conv.participants.len(), 2);
        assert!(conv.is_participant(&user1));
        assert!(conv.is_participant(&user2));
    }

    #[test]
    fn test_group_conversation_creation() {
        let users: Vec<UserId> = (0..5).map(|_| UserId::new()).collect();
        let conv = Conversation::new_group(users.clone());

        assert!(conv.is_group());
        assert!(!conv.is_direct());
        assert_eq!(conv.participants.len(), 5);
    }

    #[test]
    fn test_other_participant() {
        let user1 = UserId::new();
        let user2 = UserId::new();
        let conv = Conversation::new_direct(user1, user2);

        assert_eq!(conv.other_participant(&user1), Some(&user2));
        assert_eq!(conv.other_participant(&user2), Some(&user1));
    }

    #[test]
    fn test_unread_count() {
        let user1 = UserId::new();
        let user2 = UserId::new();
        let mut conv = Conversation::new_direct(user1, user2);

        assert_eq!(conv.get_unread_count(&user1), 0);

        conv.increment_unread(&user1);
        conv.increment_unread(&user1);
        assert_eq!(conv.get_unread_count(&user1), 2);

        conv.clear_unread(&user1);
        assert_eq!(conv.get_unread_count(&user1), 0);
    }

    #[test]
    fn test_update_last_message() {
        let user1 = UserId::new();
        let user2 = UserId::new();
        let mut conv = Conversation::new_direct(user1, user2);

        assert!(conv.last_message_id.is_none());

        let msg_id = super::super::MessageId::new();
        conv.update_last_message(msg_id, Utc::now());

        assert!(conv.last_message_id.is_some());
        assert!(conv.last_message_at.is_some());
    }

    #[test]
    fn test_conversation_type_from_str() {
        assert_eq!(
            "direct".parse::<ConversationType>().unwrap(),
            ConversationType::Direct
        );
        assert_eq!(
            "GROUP".parse::<ConversationType>().unwrap(),
            ConversationType::Group
        );
    }
}
