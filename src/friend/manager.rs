use async_trait::async_trait;

use crate::domain::{
    FriendError, FriendRequest, FriendRequestId, Friendship, FriendshipStatus, UserId,
};
use crate::error::{AppError, AppResult};
use crate::event::EventBus;
use crate::infra::FriendRepository;

#[async_trait]
pub trait FriendService: Send + Sync {
    async fn send_request(
        &self,
        from: UserId,
        to: UserId,
        message: Option<String>,
    ) -> AppResult<FriendRequest>;
    async fn accept_request(&self, request_id: &FriendRequestId) -> AppResult<Friendship>;
    async fn reject_request(&self, request_id: &FriendRequestId) -> AppResult<()>;
    async fn remove_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()>;
    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>>;
    async fn get_pending_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn get_sent_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn is_friend(&self, user_id: &UserId, other_id: &UserId) -> AppResult<bool>;
    async fn has_pending_request(&self, from: &UserId, to: &UserId) -> AppResult<bool>;
    async fn get_friendship(
        &self,
        user_id: &UserId,
        friend_id: &UserId,
    ) -> AppResult<Option<Friendship>>;
}

pub struct FriendManager<R: FriendRepository> {
    repository: R,
    event_bus: EventBus,
}

impl<R: FriendRepository> FriendManager<R> {
    pub fn new(repository: R, event_bus: EventBus) -> Self {
        Self {
            repository,
            event_bus,
        }
    }
}

#[async_trait]
impl<R: FriendRepository + 'static> FriendService for FriendManager<R> {
    async fn send_request(
        &self,
        from: UserId,
        to: UserId,
        message: Option<String>,
    ) -> AppResult<FriendRequest> {
        if from == to {
            return Err(AppError::Validation(FriendError::SelfFriend.to_string()));
        }

        if self.repository.is_friend(&from, &to).await? {
            return Err(AppError::Validation(
                FriendError::AlreadyFriends.to_string(),
            ));
        }

        if self.repository.has_pending_request(&from, &to).await? {
            return Err(AppError::Validation(
                FriendError::RequestPending.to_string(),
            ));
        }

        let request = FriendRequest::new(from, to, message);
        let saved = self.repository.create_request(&request).await?;

        let _ = self
            .event_bus
            .publish(crate::event::Event::FriendRequestReceived {
                request: saved.clone(),
            })
            .await;

        Ok(saved)
    }

    async fn accept_request(&self, request_id: &FriendRequestId) -> AppResult<Friendship> {
        let request = self
            .repository
            .get_request(request_id)
            .await?
            .ok_or_else(|| AppError::NotFound(FriendError::RequestNotFound.to_string()))?;

        if !request.is_pending() {
            return Err(AppError::Validation(
                FriendError::RequestProcessed.to_string(),
            ));
        }

        self.repository
            .update_request_status(request_id, FriendshipStatus::Accepted)
            .await?;

        let friendship1 = Friendship::new(request.from_user, request.to_user);
        let friendship2 = Friendship::new(request.to_user, request.from_user);

        self.repository.create_friendship(&friendship1).await?;
        self.repository.create_friendship(&friendship2).await?;

        let _ = self
            .event_bus
            .publish(crate::event::Event::FriendRequestAccepted {
                friendship: friendship1.clone(),
            })
            .await;

        Ok(friendship1)
    }

    async fn reject_request(&self, request_id: &FriendRequestId) -> AppResult<()> {
        let request = self
            .repository
            .get_request(request_id)
            .await?
            .ok_or_else(|| AppError::NotFound(FriendError::RequestNotFound.to_string()))?;

        if !request.is_pending() {
            return Err(AppError::Validation(
                FriendError::RequestProcessed.to_string(),
            ));
        }

        self.repository
            .update_request_status(request_id, FriendshipStatus::Rejected)
            .await?;

        let _ = self
            .event_bus
            .publish(crate::event::Event::FriendRequestRejected {
                request_id: *request_id,
            })
            .await;

        Ok(())
    }

    async fn remove_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()> {
        if !self.repository.is_friend(user_id, friend_id).await? {
            return Err(AppError::NotFound(FriendError::NotFriends.to_string()));
        }

        self.repository
            .delete_friendship(user_id, friend_id)
            .await?;
        self.repository
            .delete_friendship(friend_id, user_id)
            .await?;

        let _ = self
            .event_bus
            .publish(crate::event::Event::FriendRemoved {
                user_id: *user_id,
                friend_id: *friend_id,
            })
            .await;

        Ok(())
    }

    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>> {
        self.repository.get_friends(user_id).await
    }

    async fn get_pending_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>> {
        self.repository.get_pending_requests_for_user(user_id).await
    }

    async fn get_sent_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>> {
        self.repository.get_sent_requests_by_user(user_id).await
    }

    async fn is_friend(&self, user_id: &UserId, other_id: &UserId) -> AppResult<bool> {
        self.repository.is_friend(user_id, other_id).await
    }

    async fn has_pending_request(&self, from: &UserId, to: &UserId) -> AppResult<bool> {
        self.repository.has_pending_request(from, to).await
    }

    async fn get_friendship(
        &self,
        user_id: &UserId,
        friend_id: &UserId,
    ) -> AppResult<Option<Friendship>> {
        self.repository.get_friendship(user_id, friend_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventBus;
    use crate::infra::InMemoryFriendRepository;

    #[tokio::test]
    async fn test_send_friend_request() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let from = UserId::new();
        let to = UserId::new();

        let request = manager
            .send_request(from, to, Some("Hello".to_string()))
            .await
            .unwrap();

        assert_eq!(request.from_user, from);
        assert_eq!(request.to_user, to);
        assert!(request.is_pending());
    }

    #[tokio::test]
    async fn test_send_request_to_self_fails() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let user = UserId::new();
        let result = manager.send_request(user, user, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_accept_friend_request() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let from = UserId::new();
        let to = UserId::new();

        let request = manager.send_request(from, to, None).await.unwrap();
        let friendship = manager.accept_request(&request.id).await.unwrap();

        assert_eq!(friendship.user_id, from);
        assert_eq!(friendship.friend_id, to);
        assert!(manager.is_friend(&from, &to).await.unwrap());
        assert!(manager.is_friend(&to, &from).await.unwrap());
    }

    #[tokio::test]
    async fn test_reject_friend_request() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let from = UserId::new();
        let to = UserId::new();

        let request = manager.send_request(from, to, None).await.unwrap();
        manager.reject_request(&request.id).await.unwrap();

        assert!(!manager.is_friend(&from, &to).await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_friend() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let from = UserId::new();
        let to = UserId::new();

        let request = manager.send_request(from, to, None).await.unwrap();
        manager.accept_request(&request.id).await.unwrap();

        assert!(manager.is_friend(&from, &to).await.unwrap());

        manager.remove_friend(&from, &to).await.unwrap();

        assert!(!manager.is_friend(&from, &to).await.unwrap());
        assert!(!manager.is_friend(&to, &from).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_pending_requests() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let user1 = UserId::new();
        let user2 = UserId::new();
        let user3 = UserId::new();

        manager.send_request(user1, user3, None).await.unwrap();
        manager.send_request(user2, user3, None).await.unwrap();

        let pending = manager.get_pending_requests(&user3).await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_duplicate_request_fails() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let from = UserId::new();
        let to = UserId::new();

        manager.send_request(from, to, None).await.unwrap();
        let result = manager.send_request(from, to, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_already_friends_fails() {
        let repo = InMemoryFriendRepository::new();
        let event_bus = EventBus::new();
        let manager = FriendManager::new(repo, event_bus);

        let from = UserId::new();
        let to = UserId::new();

        let request = manager.send_request(from, to, None).await.unwrap();
        manager.accept_request(&request.id).await.unwrap();

        let result = manager.send_request(from, to, None).await;
        assert!(result.is_err());
    }
}
