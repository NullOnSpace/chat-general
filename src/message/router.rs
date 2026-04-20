use chrono::{DateTime, Utc};

use crate::domain::{ConversationId, DeviceId, Message, MessageId, UserId};
use crate::error::AppResult;
use crate::message::store::MessageStore;
use crate::session::DeviceRegistry;

pub struct MessageRouter<S: MessageStore> {
    store: S,
    device_registry: DeviceRegistry,
}

impl<S: MessageStore> MessageRouter<S> {
    pub fn new(store: S, device_registry: DeviceRegistry) -> Self {
        Self { store, device_registry }
    }

    pub async fn route(&self, message: &Message) -> AppResult<Vec<DeviceId>> {
        let _stored = self.store.store(message).await?;
        
        let recipient_devices = match message.is_system() {
            true => {
                let all_online = self.device_registry.get_online_users().await;
                let mut devices = Vec::new();
                for user_id in all_online {
                    devices.extend(
                        self.device_registry.get_online_devices(&user_id)
                            .await
                            .into_iter()
                            .map(|d| d.device_id)
                    );
                }
                devices
            }
            false => {
                self.device_registry.get_online_devices(&message.sender_id)
                    .await
                    .into_iter()
                    .map(|d| d.device_id)
                    .collect()
            }
        };

        Ok(recipient_devices)
    }

    pub async fn get_history(
        &self,
        conversation_id: &ConversationId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        self.store.get_history(conversation_id, before, limit).await
    }

    pub async fn mark_delivered(
        &self,
        message_id: &MessageId,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> AppResult<()> {
        self.store.mark_delivered(message_id, user_id, device_id).await
    }

    pub async fn mark_read(
        &self,
        message_id: &MessageId,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> AppResult<()> {
        self.store.mark_read(message_id, user_id, device_id).await
    }
}

pub struct HistoryService<S: MessageStore> {
    store: S,
}

impl<S: MessageStore> HistoryService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn sync_for_device(
        &self,
        conversation_ids: &[ConversationId],
        last_sync: DateTime<Utc>,
        limit_per_conversation: i64,
    ) -> AppResult<Vec<(ConversationId, Vec<Message>)>> {
        let mut result = Vec::new();

        for conv_id in conversation_ids {
            let messages = self.store
                .get_history(conv_id, Some(last_sync), limit_per_conversation)
                .await?;
            
            if !messages.is_empty() {
                result.push((*conv_id, messages));
            }
        }

        Ok(result)
    }

    pub async fn get_conversation_history(
        &self,
        conversation_id: &ConversationId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        self.store.get_history(conversation_id, before, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::store::InMemoryMessageStore;

    #[tokio::test]
    async fn test_history_service() {
        let store = InMemoryMessageStore::new();
        let service = HistoryService::new(store);
        
        let conv_id = ConversationId::new();
        let user_id = UserId::new();
        
        for i in 0..3 {
            let msg = Message::text(conv_id, user_id, format!("Message {}", i));
            service.store.store(&msg).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        let history = service.get_conversation_history(&conv_id, None, 10).await.unwrap();
        assert_eq!(history.len(), 3);
    }
}
