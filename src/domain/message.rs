use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use super::{ConversationId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct MessageId(pub Uuid);

impl MessageId {
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

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl TryFrom<String> for MessageId {
    type Error = uuid::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Uuid::parse_str(&s).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    #[default]
    Text,
    Image,
    Video,
    Audio,
    File,
    System,
    Custom,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::Image => write!(f, "image"),
            MessageType::Video => write!(f, "video"),
            MessageType::Audio => write!(f, "audio"),
            MessageType::File => write!(f, "file"),
            MessageType::System => write!(f, "system"),
            MessageType::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(MessageType::Text),
            "image" => Ok(MessageType::Image),
            "video" => Ok(MessageType::Video),
            "audio" => Ok(MessageType::Audio),
            "file" => Ok(MessageType::File),
            "system" => Ok(MessageType::System),
            "custom" => Ok(MessageType::Custom),
            _ => Err(format!("Invalid message type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    #[default]
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Sending => write!(f, "sending"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Read => write!(f, "read"),
            MessageStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for MessageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sending" => Ok(MessageStatus::Sending),
            "sent" => Ok(MessageStatus::Sent),
            "delivered" => Ok(MessageStatus::Delivered),
            "read" => Ok(MessageStatus::Read),
            "failed" => Ok(MessageStatus::Failed),
            _ => Err(format!("Invalid message status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: MessageId,
    pub conversation_id: ConversationId,
    pub sender_id: UserId,
    pub message_type: MessageType,
    pub content: String,
    pub metadata: sqlx::types::Json<HashMap<String, serde_json::Value>>,
    pub status: MessageStatus,
    pub reply_to: Option<MessageId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Message {
    pub fn new(
        conversation_id: ConversationId,
        sender_id: UserId,
        message_type: MessageType,
        content: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: MessageId::new(),
            conversation_id,
            sender_id,
            message_type,
            content,
            metadata: sqlx::types::Json(HashMap::new()),
            status: MessageStatus::Sending,
            reply_to: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn text(conversation_id: ConversationId, sender_id: UserId, content: String) -> Self {
        Self::new(conversation_id, sender_id, MessageType::Text, content)
    }

    pub fn system(conversation_id: ConversationId, content: String) -> Self {
        Self::new(
            conversation_id,
            UserId::from(Uuid::nil()),
            MessageType::System,
            content,
        )
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.0.insert(key, value);
        self
    }

    pub fn with_reply_to(mut self, message_id: MessageId) -> Self {
        self.reply_to = Some(message_id);
        self
    }

    pub fn mark_sent(&mut self) {
        self.status = MessageStatus::Sent;
        self.updated_at = Utc::now();
    }

    pub fn mark_delivered(&mut self) {
        self.status = MessageStatus::Delivered;
        self.updated_at = Utc::now();
    }

    pub fn mark_read(&mut self) {
        self.status = MessageStatus::Read;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self) {
        self.status = MessageStatus::Failed;
        self.updated_at = Utc::now();
    }

    pub fn is_system(&self) -> bool {
        self.message_type == MessageType::System
    }

    pub fn is_from(&self, user_id: &UserId) -> bool {
        &self.sender_id == user_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDelivery {
    pub id: Option<uuid::Uuid>,
    pub message_id: MessageId,
    pub user_id: UserId,
    pub device_id: super::DeviceId,
    pub delivered_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

impl MessageDelivery {
    pub fn new(message_id: MessageId, user_id: UserId, device_id: super::DeviceId) -> Self {
        Self {
            id: None,
            message_id,
            user_id,
            device_id,
            delivered_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn mark_read(&mut self) {
        self.read_at = Some(Utc::now());
    }

    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_conversation_id() -> ConversationId {
        ConversationId::new()
    }

    fn create_test_user_id() -> UserId {
        UserId::new()
    }

    #[test]
    fn test_message_id_creation() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_text_message_creation() {
        let conv_id = create_test_conversation_id();
        let user_id = create_test_user_id();
        let message = Message::text(conv_id, user_id, "Hello, World!".to_string());

        assert_eq!(message.message_type, MessageType::Text);
        assert_eq!(message.content, "Hello, World!");
        assert_eq!(message.status, MessageStatus::Sending);
        assert!(message.reply_to.is_none());
    }

    #[test]
    fn test_message_status_transitions() {
        let conv_id = create_test_conversation_id();
        let user_id = create_test_user_id();
        let mut message = Message::text(conv_id, user_id, "Test".to_string());

        assert_eq!(message.status, MessageStatus::Sending);

        message.mark_sent();
        assert_eq!(message.status, MessageStatus::Sent);

        message.mark_delivered();
        assert_eq!(message.status, MessageStatus::Delivered);

        message.mark_read();
        assert_eq!(message.status, MessageStatus::Read);
    }

    #[test]
    fn test_message_with_metadata() {
        let conv_id = create_test_conversation_id();
        let user_id = create_test_user_id();
        let message = Message::text(conv_id, user_id, "Test".to_string())
            .with_metadata("file_size".to_string(), serde_json::json!(1024))
            .with_metadata("file_name".to_string(), serde_json::json!("test.pdf"));

        assert_eq!(
            message.metadata.0.get("file_size").unwrap(),
            &serde_json::json!(1024)
        );
    }

    #[test]
    fn test_system_message() {
        let conv_id = create_test_conversation_id();
        let message = Message::system(conv_id, "User joined the group".to_string());

        assert!(message.is_system());
        assert_eq!(message.message_type, MessageType::System);
    }

    #[test]
    fn test_message_delivery() {
        let message_id = MessageId::new();
        let user_id = create_test_user_id();
        let device_id = super::super::DeviceId::new();

        let mut delivery = MessageDelivery::new(message_id, user_id, device_id);

        assert!(delivery.delivered_at <= Utc::now());
        assert!(!delivery.is_read());

        delivery.mark_read();
        assert!(delivery.is_read());
        assert!(delivery.read_at.is_some());
    }

    #[test]
    fn test_message_type_from_str() {
        assert_eq!("text".parse::<MessageType>().unwrap(), MessageType::Text);
        assert_eq!("IMAGE".parse::<MessageType>().unwrap(), MessageType::Image);
    }

    #[test]
    fn test_message_status_from_str() {
        assert_eq!(
            "sent".parse::<MessageStatus>().unwrap(),
            MessageStatus::Sent
        );
        assert_eq!(
            "READ".parse::<MessageStatus>().unwrap(),
            MessageStatus::Read
        );
    }
}
