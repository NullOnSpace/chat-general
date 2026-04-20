use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FriendRequestId(pub Uuid);

impl FriendRequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FriendRequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FriendRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for FriendRequestId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<FriendRequestId> for Uuid {
    fn from(id: FriendRequestId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FriendshipId(pub Uuid);

impl FriendshipId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FriendshipId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FriendshipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for FriendshipId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<FriendshipId> for Uuid {
    fn from(id: FriendshipId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FriendshipStatus {
    Pending,
    Accepted,
    Rejected,
    Blocked,
}

impl Default for FriendshipStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for FriendshipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FriendshipStatus::Pending => write!(f, "pending"),
            FriendshipStatus::Accepted => write!(f, "accepted"),
            FriendshipStatus::Rejected => write!(f, "rejected"),
            FriendshipStatus::Blocked => write!(f, "blocked"),
        }
    }
}

impl std::str::FromStr for FriendshipStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(FriendshipStatus::Pending),
            "accepted" => Ok(FriendshipStatus::Accepted),
            "rejected" => Ok(FriendshipStatus::Rejected),
            "blocked" => Ok(FriendshipStatus::Blocked),
            _ => Err(format!("Invalid friendship status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: FriendRequestId,
    pub from_user: UserId,
    pub to_user: UserId,
    pub message: Option<String>,
    pub status: FriendshipStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FriendRequest {
    pub fn new(from_user: UserId, to_user: UserId, message: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: FriendRequestId::new(),
            from_user,
            to_user,
            message,
            status: FriendshipStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn accept(mut self) -> Self {
        self.status = FriendshipStatus::Accepted;
        self.updated_at = Utc::now();
        self
    }

    pub fn reject(mut self) -> Self {
        self.status = FriendshipStatus::Rejected;
        self.updated_at = Utc::now();
        self
    }

    pub fn is_pending(&self) -> bool {
        self.status == FriendshipStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friendship {
    pub id: FriendshipId,
    pub user_id: UserId,
    pub friend_id: UserId,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Friendship {
    pub fn new(user_id: UserId, friend_id: UserId) -> Self {
        Self {
            id: FriendshipId::new(),
            user_id,
            friend_id,
            remark: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_remark(mut self, remark: String) -> Self {
        self.remark = Some(remark);
        self
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum FriendError {
    #[error("Already friends")]
    AlreadyFriends,
    #[error("Friend request already pending")]
    RequestPending,
    #[error("Friend request not found")]
    RequestNotFound,
    #[error("Friend request already processed")]
    RequestProcessed,
    #[error("Cannot add yourself as friend")]
    SelfFriend,
    #[error("Not friends")]
    NotFriends,
    #[error("User not found")]
    UserNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_friend_request_id_creation() {
        let id1 = FriendRequestId::new();
        let id2 = FriendRequestId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_friendship_id_creation() {
        let id1 = FriendshipId::new();
        let id2 = FriendshipId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_friendship_status_from_str() {
        assert_eq!(FriendshipStatus::from_str("pending").unwrap(), FriendshipStatus::Pending);
        assert_eq!(FriendshipStatus::from_str("ACCEPTED").unwrap(), FriendshipStatus::Accepted);
        assert_eq!(FriendshipStatus::from_str("rejected").unwrap(), FriendshipStatus::Rejected);
        assert!(FriendshipStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_friend_request_creation() {
        let from = UserId::new();
        let to = UserId::new();
        let request = FriendRequest::new(from, to, Some("Hello".to_string()));
        
        assert!(request.is_pending());
        assert_eq!(request.from_user, from);
        assert_eq!(request.to_user, to);
        assert_eq!(request.message, Some("Hello".to_string()));
    }

    #[test]
    fn test_friend_request_accept() {
        let from = UserId::new();
        let to = UserId::new();
        let request = FriendRequest::new(from, to, None);
        let accepted = request.accept();
        
        assert_eq!(accepted.status, FriendshipStatus::Accepted);
        assert!(!accepted.is_pending());
    }

    #[test]
    fn test_friendship_creation() {
        let user_id = UserId::new();
        let friend_id = UserId::new();
        let friendship = Friendship::new(user_id, friend_id);
        
        assert_eq!(friendship.user_id, user_id);
        assert_eq!(friendship.friend_id, friend_id);
        assert!(friendship.remark.is_none());
    }

    #[test]
    fn test_friendship_with_remark() {
        let user_id = UserId::new();
        let friend_id = UserId::new();
        let friendship = Friendship::new(user_id, friend_id)
            .with_remark("Best friend".to_string());
        
        assert_eq!(friendship.remark, Some("Best friend".to_string()));
    }
}
