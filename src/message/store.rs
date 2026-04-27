use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{Conversation, ConversationId, DeviceId, Message, MessageId, UserId};
use crate::error::AppResult;

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn store(&self, message: &Message) -> AppResult<Message>;
    async fn get_by_id(&self, id: &MessageId) -> AppResult<Option<Message>>;
    async fn get_history(
        &self,
        conversation_id: &ConversationId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<Message>>;
    async fn mark_delivered(
        &self,
        message_id: &MessageId,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> AppResult<()>;
    async fn mark_read(
        &self,
        message_id: &MessageId,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> AppResult<()>;
    async fn save_conversation(&self, conversation: Conversation) -> AppResult<()>;
    async fn get_user_conversations(&self, user_id: &UserId) -> AppResult<Vec<Conversation>>;
}

#[derive(Clone)]
pub struct InMemoryMessageStore {
    messages: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<MessageId, Message>>>,
    conversations: std::sync::Arc<
        tokio::sync::RwLock<std::collections::HashMap<ConversationId, Conversation>>,
    >,
    conversation_participants:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<ConversationId, Vec<UserId>>>>,
}

impl Default for InMemoryMessageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMessageStore {
    pub fn new() -> Self {
        Self {
            messages: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            conversations: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            conversation_participants: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    pub async fn get_conversation_participants(
        &self,
        conversation_id: &ConversationId,
    ) -> Option<Vec<UserId>> {
        let participants = self.conversation_participants.read().await;
        participants.get(conversation_id).cloned()
    }

    pub async fn add_conversation_participants(
        &self,
        conversation_id: ConversationId,
        user_ids: Vec<UserId>,
    ) {
        let mut participants = self.conversation_participants.write().await;
        participants
            .entry(conversation_id)
            .or_insert_with(|| user_ids);
    }
}

#[async_trait]
impl MessageStore for InMemoryMessageStore {
    async fn store(&self, message: &Message) -> AppResult<Message> {
        let mut messages = self.messages.write().await;
        let id = message.id;
        messages.insert(id, message.clone());
        Ok(message.clone())
    }

    async fn get_by_id(&self, id: &MessageId) -> AppResult<Option<Message>> {
        let messages = self.messages.read().await;
        Ok(messages.get(id).cloned())
    }

    async fn get_history(
        &self,
        conversation_id: &ConversationId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        let messages = self.messages.read().await;
        let mut result: Vec<Message> = messages
            .values()
            .filter(|m| &m.conversation_id == conversation_id)
            .filter(|m| match before {
                Some(before_time) => m.created_at < before_time,
                None => true,
            })
            .cloned()
            .collect();

        result.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        result.truncate(limit as usize);

        Ok(result)
    }

    async fn mark_delivered(
        &self,
        _message_id: &MessageId,
        _user_id: &UserId,
        _device_id: &DeviceId,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn mark_read(
        &self,
        _message_id: &MessageId,
        _user_id: &UserId,
        _device_id: &DeviceId,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn save_conversation(&self, conversation: Conversation) -> AppResult<()> {
        let mut convs = self.conversations.write().await;
        convs.insert(conversation.id, conversation);
        Ok(())
    }

    async fn get_user_conversations(&self, user_id: &UserId) -> AppResult<Vec<Conversation>> {
        let convs = self.conversations.read().await;
        let participants = self.conversation_participants.read().await;
        let user_convs: Vec<Conversation> = convs
            .values()
            .filter(|c| {
                participants
                    .get(&c.id)
                    .map(|p| p.contains(user_id))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        Ok(user_convs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = InMemoryMessageStore::new();
        let conv_id = ConversationId::new();
        let user_id = UserId::new();

        let message = Message::text(conv_id, user_id, "Hello".to_string());
        let stored = store.store(&message).await.unwrap();

        let retrieved = store.get_by_id(&stored.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Hello");
    }

    #[tokio::test]
    async fn test_get_history() {
        let store = InMemoryMessageStore::new();
        let conv_id = ConversationId::new();
        let user_id = UserId::new();

        for i in 0..5 {
            let msg = Message::text(conv_id, user_id, format!("Message {}", i));
            store.store(&msg).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let history = store.get_history(&conv_id, None, 3).await.unwrap();
        assert_eq!(history.len(), 3);
    }
}
