use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{
    FriendRequest, FriendRequestId, Friendship, FriendshipId, FriendshipStatus, UserId,
};
use crate::error::AppResult;

#[async_trait]
pub trait FriendRepository: Send + Sync {
    async fn create_request(&self, request: &FriendRequest) -> AppResult<FriendRequest>;
    async fn get_request(&self, id: &FriendRequestId) -> AppResult<Option<FriendRequest>>;
    async fn update_request_status(
        &self,
        id: &FriendRequestId,
        status: FriendshipStatus,
    ) -> AppResult<()>;
    async fn get_pending_requests_for_user(
        &self,
        user_id: &UserId,
    ) -> AppResult<Vec<FriendRequest>>;
    async fn get_sent_requests_by_user(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn has_pending_request(&self, from: &UserId, to: &UserId) -> AppResult<bool>;

    async fn create_friendship(&self, friendship: &Friendship) -> AppResult<Friendship>;
    async fn get_friendship(
        &self,
        user_id: &UserId,
        friend_id: &UserId,
    ) -> AppResult<Option<Friendship>>;
    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>>;
    async fn delete_friendship(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()>;
    async fn is_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<bool>;
}

pub struct PostgresFriendRepository {
    pool: PgPool,
}

impl PostgresFriendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FriendRepository for PostgresFriendRepository {
    async fn create_request(&self, request: &FriendRequest) -> AppResult<FriendRequest> {
        sqlx::query(
            r#"
            INSERT INTO friend_requests (id, from_user_id, to_user_id, message, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(request.id.0)
        .bind(request.from_user.0)
        .bind(request.to_user.0)
        .bind(&request.message)
        .bind(request.status.to_string())
        .bind(request.created_at)
        .bind(request.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(request.clone())
    }

    async fn get_request(&self, id: &FriendRequestId) -> AppResult<Option<FriendRequest>> {
        let row = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                uuid::Uuid,
                Option<String>,
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, from_user_id, to_user_id, message, status, created_at, updated_at
            FROM friend_requests
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(id, from_user_id, to_user_id, message, status, created_at, updated_at)| {
                FriendRequest {
                    id: FriendRequestId(id),
                    from_user: UserId::from(from_user_id),
                    to_user: UserId::from(to_user_id),
                    message,
                    status: status.parse().unwrap_or(FriendshipStatus::Pending),
                    created_at,
                    updated_at,
                }
            },
        ))
    }

    async fn update_request_status(
        &self,
        id: &FriendRequestId,
        status: FriendshipStatus,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE friend_requests
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .bind(status.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_pending_requests_for_user(
        &self,
        user_id: &UserId,
    ) -> AppResult<Vec<FriendRequest>> {
        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                uuid::Uuid,
                Option<String>,
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, from_user_id, to_user_id, message, status, created_at, updated_at
            FROM friend_requests
            WHERE to_user_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, from_user_id, to_user_id, message, status, created_at, updated_at)| {
                    FriendRequest {
                        id: FriendRequestId(id),
                        from_user: UserId::from(from_user_id),
                        to_user: UserId::from(to_user_id),
                        message,
                        status: status.parse().unwrap_or(FriendshipStatus::Pending),
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    async fn get_sent_requests_by_user(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>> {
        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                uuid::Uuid,
                Option<String>,
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, from_user_id, to_user_id, message, status, created_at, updated_at
            FROM friend_requests
            WHERE from_user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, from_user_id, to_user_id, message, status, created_at, updated_at)| {
                    FriendRequest {
                        id: FriendRequestId(id),
                        from_user: UserId::from(from_user_id),
                        to_user: UserId::from(to_user_id),
                        message,
                        status: status.parse().unwrap_or(FriendshipStatus::Pending),
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    async fn has_pending_request(&self, from: &UserId, to: &UserId) -> AppResult<bool> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM friend_requests
            WHERE from_user_id = $1 AND to_user_id = $2 AND status = 'pending'
            "#,
        )
        .bind(from.0)
        .bind(to.0)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0 > 0)
    }

    async fn create_friendship(&self, friendship: &Friendship) -> AppResult<Friendship> {
        sqlx::query(
            r#"
            INSERT INTO friendships (id, user_id, friend_id, remark, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(friendship.id.0)
        .bind(friendship.user_id.0)
        .bind(friendship.friend_id.0)
        .bind(&friendship.remark)
        .bind(friendship.created_at)
        .execute(&self.pool)
        .await?;

        Ok(friendship.clone())
    }

    async fn get_friendship(
        &self,
        user_id: &UserId,
        friend_id: &UserId,
    ) -> AppResult<Option<Friendship>> {
        let row = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                uuid::Uuid,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, user_id, friend_id, remark, created_at
            FROM friendships
            WHERE user_id = $1 AND friend_id = $2
            "#,
        )
        .bind(user_id.0)
        .bind(friend_id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            row.map(|(id, user_id, friend_id, remark, created_at)| Friendship {
                id: FriendshipId(id),
                user_id: UserId::from(user_id),
                friend_id: UserId::from(friend_id),
                remark,
                created_at,
            }),
        )
    }

    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>> {
        let rows = sqlx::query_as::<
            _,
            (
                uuid::Uuid,
                uuid::Uuid,
                uuid::Uuid,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, user_id, friend_id, remark, created_at
            FROM friendships
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, user_id, friend_id, remark, created_at)| Friendship {
                id: FriendshipId(id),
                user_id: UserId::from(user_id),
                friend_id: UserId::from(friend_id),
                remark,
                created_at,
            })
            .collect())
    }

    async fn delete_friendship(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM friendships
            WHERE user_id = $1 AND friend_id = $2
            "#,
        )
        .bind(user_id.0)
        .bind(friend_id.0)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn is_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<bool> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM friendships
            WHERE user_id = $1 AND friend_id = $2
            "#,
        )
        .bind(user_id.0)
        .bind(friend_id.0)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0 > 0)
    }
}

pub struct InMemoryFriendRepository {
    requests: std::sync::Arc<tokio::sync::RwLock<Vec<FriendRequest>>>,
    friendships: std::sync::Arc<tokio::sync::RwLock<Vec<Friendship>>>,
}

impl Default for InMemoryFriendRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryFriendRepository {
    pub fn new() -> Self {
        Self {
            requests: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            friendships: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl FriendRepository for InMemoryFriendRepository {
    async fn create_request(&self, request: &FriendRequest) -> AppResult<FriendRequest> {
        let mut requests = self.requests.write().await;
        requests.push(request.clone());
        Ok(request.clone())
    }

    async fn get_request(&self, id: &FriendRequestId) -> AppResult<Option<FriendRequest>> {
        let requests = self.requests.read().await;
        Ok(requests.iter().find(|r| r.id == *id).cloned())
    }

    async fn update_request_status(
        &self,
        id: &FriendRequestId,
        status: FriendshipStatus,
    ) -> AppResult<()> {
        let mut requests = self.requests.write().await;
        if let Some(request) = requests.iter_mut().find(|r| r.id == *id) {
            request.status = status;
            request.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    async fn get_pending_requests_for_user(
        &self,
        user_id: &UserId,
    ) -> AppResult<Vec<FriendRequest>> {
        let requests = self.requests.read().await;
        Ok(requests
            .iter()
            .filter(|r| r.to_user == *user_id && r.status == FriendshipStatus::Pending)
            .cloned()
            .collect())
    }

    async fn get_sent_requests_by_user(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>> {
        let requests = self.requests.read().await;
        Ok(requests
            .iter()
            .filter(|r| r.from_user == *user_id)
            .cloned()
            .collect())
    }

    async fn has_pending_request(&self, from: &UserId, to: &UserId) -> AppResult<bool> {
        let requests = self.requests.read().await;
        Ok(requests.iter().any(|r| {
            r.from_user == *from && r.to_user == *to && r.status == FriendshipStatus::Pending
        }))
    }

    async fn create_friendship(&self, friendship: &Friendship) -> AppResult<Friendship> {
        let mut friendships = self.friendships.write().await;
        friendships.push(friendship.clone());
        Ok(friendship.clone())
    }

    async fn get_friendship(
        &self,
        user_id: &UserId,
        friend_id: &UserId,
    ) -> AppResult<Option<Friendship>> {
        let friendships = self.friendships.read().await;
        Ok(friendships
            .iter()
            .find(|f| f.user_id == *user_id && f.friend_id == *friend_id)
            .cloned())
    }

    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>> {
        let friendships = self.friendships.read().await;
        Ok(friendships
            .iter()
            .filter(|f| f.user_id == *user_id)
            .cloned()
            .collect())
    }

    async fn delete_friendship(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()> {
        let mut friendships = self.friendships.write().await;
        friendships.retain(|f| !(f.user_id == *user_id && f.friend_id == *friend_id));
        Ok(())
    }

    async fn is_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<bool> {
        let friendships = self.friendships.read().await;
        Ok(friendships
            .iter()
            .any(|f| f.user_id == *user_id && f.friend_id == *friend_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_create_and_get_request() {
        let repo = InMemoryFriendRepository::new();
        let from = UserId::new();
        let to = UserId::new();
        let request = FriendRequest::new(from, to, Some("Hello".to_string()));

        repo.create_request(&request).await.unwrap();
        let retrieved = repo.get_request(&request.id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().from_user, from);
    }

    #[tokio::test]
    async fn test_in_memory_create_and_get_friendship() {
        let repo = InMemoryFriendRepository::new();
        let user_id = UserId::new();
        let friend_id = UserId::new();
        let friendship = Friendship::new(user_id, friend_id);

        repo.create_friendship(&friendship).await.unwrap();

        assert!(repo.is_friend(&user_id, &friend_id).await.unwrap());
        assert!(!repo.is_friend(&friend_id, &user_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_delete_friendship() {
        let repo = InMemoryFriendRepository::new();
        let user_id = UserId::new();
        let friend_id = UserId::new();
        let friendship = Friendship::new(user_id, friend_id);

        repo.create_friendship(&friendship).await.unwrap();
        assert!(repo.is_friend(&user_id, &friend_id).await.unwrap());

        repo.delete_friendship(&user_id, &friend_id).await.unwrap();
        assert!(!repo.is_friend(&user_id, &friend_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_pending_requests() {
        let repo = InMemoryFriendRepository::new();
        let from = UserId::new();
        let to = UserId::new();
        let request = FriendRequest::new(from, to, None);

        repo.create_request(&request).await.unwrap();

        assert!(repo.has_pending_request(&from, &to).await.unwrap());
        assert!(!repo.has_pending_request(&to, &from).await.unwrap());

        let pending = repo.get_pending_requests_for_user(&to).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].from_user, from);
    }
}
