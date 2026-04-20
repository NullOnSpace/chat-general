use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{Message, MessageId, MessageStatus, ConversationId, UserId, DeviceId, MessageDelivery};
use crate::error::AppResult;

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &Message) -> AppResult<Message>;
    async fn find_by_id(&self, id: &MessageId) -> AppResult<Option<Message>>;
    async fn find_by_conversation(
        &self,
        conversation_id: &ConversationId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<Message>>;
    async fn update_status(&self, id: &MessageId, status: MessageStatus) -> AppResult<()>;
    async fn create_delivery(&self, delivery: &MessageDelivery) -> AppResult<()>;
    async fn mark_delivered(&self, message_id: &MessageId, user_id: &UserId, device_id: &DeviceId) -> AppResult<()>;
    async fn mark_read(&self, message_id: &MessageId, user_id: &UserId, device_id: &DeviceId) -> AppResult<()>;
    async fn find_deliveries(&self, message_id: &MessageId) -> AppResult<Vec<MessageDelivery>>;
}

pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn create(&self, message: &Message) -> AppResult<Message> {
        let record = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, conversation_id, sender_id, message_type, content, metadata, status, reply_to, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(message.id.as_uuid())
        .bind(message.conversation_id.as_uuid())
        .bind(message.sender_id.as_uuid())
        .bind(message.message_type.to_string())
        .bind(&message.content)
        .bind(&message.metadata)
        .bind(message.status.to_string())
        .bind(message.reply_to.map(|r| *r.as_uuid()))
        .bind(message.created_at)
        .bind(message.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn find_by_id(&self, id: &MessageId) -> AppResult<Option<Message>> {
        let record = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn find_by_conversation(
        &self,
        conversation_id: &ConversationId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        let records = match before {
            Some(before_time) => {
                sqlx::query_as::<_, Message>(
                    r#"
                    SELECT * FROM messages 
                    WHERE conversation_id = $1 AND created_at < $2
                    ORDER BY created_at DESC
                    LIMIT $3
                    "#,
                )
                .bind(conversation_id.as_uuid())
                .bind(before_time)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Message>(
                    r#"
                    SELECT * FROM messages 
                    WHERE conversation_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(conversation_id.as_uuid())
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(records)
    }

    async fn update_status(&self, id: &MessageId, status: MessageStatus) -> AppResult<()> {
        sqlx::query(
            "UPDATE messages SET status = $2 WHERE id = $1",
        )
        .bind(id.as_uuid())
        .bind(status.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_delivery(&self, delivery: &MessageDelivery) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO message_deliveries (message_id, user_id, device_id, delivered_at, read_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (message_id, device_id) DO NOTHING
            "#,
        )
        .bind(delivery.message_id.as_uuid())
        .bind(delivery.user_id.as_uuid())
        .bind(delivery.device_id.as_uuid())
        .bind(delivery.delivered_at)
        .bind(delivery.read_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_delivered(&self, message_id: &MessageId, user_id: &UserId, device_id: &DeviceId) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO message_deliveries (message_id, user_id, device_id, delivered_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (message_id, device_id) DO UPDATE SET delivered_at = NOW()
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(device_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_read(&self, message_id: &MessageId, user_id: &UserId, device_id: &DeviceId) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO message_deliveries (message_id, user_id, device_id, delivered_at, read_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (message_id, device_id) DO UPDATE SET read_at = NOW()
            "#,
        )
        .bind(message_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(device_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_deliveries(&self, message_id: &MessageId) -> AppResult<Vec<MessageDelivery>> {
        let records = sqlx::query_as::<_, MessageDelivery>(
            "SELECT * FROM message_deliveries WHERE message_id = $1",
        )
        .bind(message_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}
