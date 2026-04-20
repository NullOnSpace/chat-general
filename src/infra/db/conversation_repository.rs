use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{Conversation, ConversationId, UserId};
use crate::error::AppResult;

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn create(&self, conversation: &Conversation) -> AppResult<Conversation>;
    async fn find_by_id(&self, id: &ConversationId) -> AppResult<Option<Conversation>>;
    async fn find_by_participants(
        &self,
        user1: &UserId,
        user2: &UserId,
    ) -> AppResult<Option<Conversation>>;
    async fn find_by_user(
        &self,
        user_id: &UserId,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Conversation>>;
    async fn add_participant(
        &self,
        conversation_id: &ConversationId,
        user_id: &UserId,
    ) -> AppResult<()>;
    async fn remove_participant(
        &self,
        conversation_id: &ConversationId,
        user_id: &UserId,
    ) -> AppResult<()>;
    async fn update_last_message(
        &self,
        conversation_id: &ConversationId,
        message_id: &uuid::Uuid,
        sent_at: DateTime<Utc>,
    ) -> AppResult<()>;
    async fn delete(&self, id: &ConversationId) -> AppResult<()>;
}

pub struct PostgresConversationRepository {
    pool: PgPool,
}

impl PostgresConversationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationRepository for PostgresConversationRepository {
    async fn create(&self, conversation: &Conversation) -> AppResult<Conversation> {
        let mut tx = self.pool.begin().await?;

        let record = sqlx::query_as::<_, Conversation>(
            r#"
            INSERT INTO conversations (id, conversation_type, last_message_id, last_message_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(conversation.id.as_uuid())
        .bind(conversation.conversation_type.to_string())
        .bind(conversation.last_message_id.map(|m| *m.as_uuid()))
        .bind(conversation.last_message_at)
        .bind(conversation.created_at)
        .bind(conversation.updated_at)
        .fetch_one(&mut *tx)
        .await?;

        for participant_id in &conversation.participants {
            sqlx::query(
                "INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)",
            )
            .bind(conversation.id.as_uuid())
            .bind(participant_id.as_uuid())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(record)
    }

    async fn find_by_id(&self, id: &ConversationId) -> AppResult<Option<Conversation>> {
        let mut record =
            sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = $1")
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await?;

        if let Some(ref mut conv) = record {
            let participants: Vec<UserId> = sqlx::query_scalar(
                "SELECT user_id FROM conversation_participants WHERE conversation_id = $1",
            )
            .bind(id.as_uuid())
            .fetch_all(&self.pool)
            .await?;

            conv.participants = participants;
        }

        Ok(record)
    }

    async fn find_by_participants(
        &self,
        user1: &UserId,
        user2: &UserId,
    ) -> AppResult<Option<Conversation>> {
        let conversation_id: Option<uuid::Uuid> = sqlx::query_scalar(
            r#"
            SELECT cp1.conversation_id
            FROM conversation_participants cp1
            INNER JOIN conversation_participants cp2 ON cp1.conversation_id = cp2.conversation_id
            INNER JOIN conversations c ON c.id = cp1.conversation_id
            WHERE cp1.user_id = $1 
              AND cp2.user_id = $2 
              AND c.conversation_type = 'direct'
            "#,
        )
        .bind(user1.as_uuid())
        .bind(user2.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match conversation_id {
            Some(id) => self.find_by_id(&ConversationId::from(id)).await,
            None => Ok(None),
        }
    }

    async fn find_by_user(
        &self,
        user_id: &UserId,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Conversation>> {
        let records = sqlx::query_as::<_, Conversation>(
            r#"
            SELECT c.* 
            FROM conversations c
            INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
            WHERE cp.user_id = $1
            ORDER BY c.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn add_participant(
        &self,
        conversation_id: &ConversationId,
        user_id: &UserId,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)",
        )
        .bind(conversation_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_participant(
        &self,
        conversation_id: &ConversationId,
        user_id: &UserId,
    ) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM conversation_participants WHERE conversation_id = $1 AND user_id = $2",
        )
        .bind(conversation_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_last_message(
        &self,
        conversation_id: &ConversationId,
        message_id: &uuid::Uuid,
        sent_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE conversations 
            SET last_message_id = $2, last_message_at = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(conversation_id.as_uuid())
        .bind(message_id)
        .bind(sent_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &ConversationId) -> AppResult<()> {
        sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
