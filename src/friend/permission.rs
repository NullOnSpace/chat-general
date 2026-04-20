use crate::domain::{ConversationType, UserId};
use crate::error::{AppError, AppResult};
use std::sync::Arc;

use super::FriendService;

pub struct ChatPermissionChecker {
    friend_service: Arc<dyn FriendService>,
}

impl ChatPermissionChecker {
    pub fn new(friend_service: Arc<dyn FriendService>) -> Self {
        Self { friend_service }
    }

    pub async fn can_start_direct_chat(
        &self,
        user_id: &UserId,
        target_id: &UserId,
    ) -> AppResult<bool> {
        if user_id == target_id {
            return Ok(false);
        }
        self.friend_service.is_friend(user_id, target_id).await
    }

    pub async fn can_send_message(
        &self,
        sender_id: &UserId,
        conversation_type: ConversationType,
        participants: &[UserId],
    ) -> AppResult<bool> {
        match conversation_type {
            ConversationType::Direct => {
                if participants.len() != 2 {
                    return Ok(false);
                }
                let other = participants
                    .iter()
                    .find(|p| *p != sender_id)
                    .ok_or_else(|| AppError::Validation("Invalid participants".into()))?;

                self.friend_service.is_friend(sender_id, other).await
            }
            ConversationType::Group => Ok(true),
        }
    }

    pub async fn check_direct_chat_permission(
        &self,
        user_id: &UserId,
        target_id: &UserId,
    ) -> AppResult<()> {
        if !self.can_start_direct_chat(user_id, target_id).await? {
            return Err(AppError::Validation(
                "Cannot start direct chat: users are not friends".into(),
            ));
        }
        Ok(())
    }

    pub async fn check_message_permission(
        &self,
        sender_id: &UserId,
        conversation_type: ConversationType,
        participants: &[UserId],
    ) -> AppResult<()> {
        if !self
            .can_send_message(sender_id, conversation_type, participants)
            .await?
        {
            return Err(AppError::Validation(
                "Cannot send message: permission denied".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FriendRequest, FriendRequestId, Friendship};
    use crate::friend::FriendService;
    use async_trait::async_trait;

    struct MockFriendService {
        friends: std::sync::Arc<tokio::sync::RwLock<Vec<(UserId, UserId)>>>,
    }

    impl MockFriendService {
        fn new() -> Self {
            Self {
                friends: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            }
        }

        async fn add_friend(&self, user: UserId, friend: UserId) {
            let mut friends = self.friends.write().await;
            friends.push((user, friend));
        }
    }

    #[async_trait]
    impl FriendService for MockFriendService {
        async fn send_request(
            &self,
            _from: UserId,
            _to: UserId,
            _message: Option<String>,
        ) -> AppResult<FriendRequest> {
            Ok(FriendRequest::new(UserId::new(), UserId::new(), None))
        }
        async fn accept_request(&self, _request_id: &FriendRequestId) -> AppResult<Friendship> {
            Ok(Friendship::new(UserId::new(), UserId::new()))
        }
        async fn reject_request(&self, _request_id: &FriendRequestId) -> AppResult<()> {
            Ok(())
        }
        async fn remove_friend(&self, _user_id: &UserId, _friend_id: &UserId) -> AppResult<()> {
            Ok(())
        }
        async fn get_friends(&self, _user_id: &UserId) -> AppResult<Vec<Friendship>> {
            Ok(vec![])
        }
        async fn get_pending_requests(&self, _user_id: &UserId) -> AppResult<Vec<FriendRequest>> {
            Ok(vec![])
        }
        async fn get_sent_requests(&self, _user_id: &UserId) -> AppResult<Vec<FriendRequest>> {
            Ok(vec![])
        }
        async fn is_friend(&self, user_id: &UserId, other_id: &UserId) -> AppResult<bool> {
            let friends = self.friends.read().await;
            Ok(friends.iter().any(|(u, f)| u == user_id && f == other_id))
        }
        async fn has_pending_request(&self, _from: &UserId, _to: &UserId) -> AppResult<bool> {
            Ok(false)
        }
        async fn get_friendship(
            &self,
            _user_id: &UserId,
            _friend_id: &UserId,
        ) -> AppResult<Option<Friendship>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_can_start_direct_chat_friends() {
        let mock = MockFriendService::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        mock.add_friend(user1, user2).await;
        mock.add_friend(user2, user1).await;

        let checker = ChatPermissionChecker::new(std::sync::Arc::new(mock));

        assert!(checker.can_start_direct_chat(&user1, &user2).await.unwrap());
        assert!(checker.can_start_direct_chat(&user2, &user1).await.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_start_direct_chat_not_friends() {
        let mock = MockFriendService::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let checker = ChatPermissionChecker::new(std::sync::Arc::new(mock));

        assert!(!checker.can_start_direct_chat(&user1, &user2).await.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_start_direct_chat_with_self() {
        let mock = MockFriendService::new();
        let user = UserId::new();

        let checker = ChatPermissionChecker::new(std::sync::Arc::new(mock));

        assert!(!checker.can_start_direct_chat(&user, &user).await.unwrap());
    }

    #[tokio::test]
    async fn test_can_send_message_group_chat() {
        let mock = MockFriendService::new();
        let user1 = UserId::new();
        let user2 = UserId::new();
        let user3 = UserId::new();

        let checker = ChatPermissionChecker::new(std::sync::Arc::new(mock));

        let participants = vec![user1, user2, user3];
        assert!(checker
            .can_send_message(&user1, ConversationType::Group, &participants)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_check_direct_chat_permission_success() {
        let mock = MockFriendService::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        mock.add_friend(user1, user2).await;
        mock.add_friend(user2, user1).await;

        let checker = ChatPermissionChecker::new(std::sync::Arc::new(mock));

        assert!(checker
            .check_direct_chat_permission(&user1, &user2)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_check_direct_chat_permission_fails() {
        let mock = MockFriendService::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        let checker = ChatPermissionChecker::new(std::sync::Arc::new(mock));

        assert!(checker
            .check_direct_chat_permission(&user1, &user2)
            .await
            .is_err());
    }
}
