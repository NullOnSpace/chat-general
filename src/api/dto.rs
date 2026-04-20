use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageRequest {
    pub conversation_id: String,
    #[validate(length(min = 1, max = 10000))]
    pub content: String,
    pub message_type: Option<String>,
    pub reply_to: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub participant_ids: Vec<String>,
    pub conversation_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGroupRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub max_members: Option<u32>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddGroupMemberRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetHistoryRequest {
    pub before: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl From<crate::domain::User> for UserResponse {
    fn from(user: crate::domain::User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status.to_string(),
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

impl From<crate::auth::TokenPair> for TokenResponse {
    fn from(tokens: crate::auth::TokenPair) -> Self {
        Self {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub message_type: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
}

impl From<crate::domain::Message> for MessageResponse {
    fn from(msg: crate::domain::Message) -> Self {
        Self {
            id: msg.id.to_string(),
            conversation_id: msg.conversation_id.to_string(),
            sender_id: msg.sender_id.to_string(),
            message_type: msg.message_type.to_string(),
            content: msg.content,
            status: msg.status.to_string(),
            created_at: msg.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: String,
    pub conversation_type: String,
    pub participants: Vec<String>,
    pub last_message_at: Option<String>,
    pub created_at: String,
}

impl From<crate::domain::Conversation> for ConversationResponse {
    fn from(conv: crate::domain::Conversation) -> Self {
        Self {
            id: conv.id.to_string(),
            conversation_type: conv.conversation_type.to_string(),
            participants: conv.participants.iter().map(|p| p.to_string()).collect(),
            last_message_at: conv.last_message_at.map(|t| t.to_rfc3339()),
            created_at: conv.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub member_count: usize,
    pub is_public: bool,
    pub created_at: String,
}

impl From<crate::domain::Group> for GroupResponse {
    fn from(group: crate::domain::Group) -> Self {
        Self {
            id: group.id.to_string(),
            name: group.name,
            description: group.description,
            owner_id: group.owner_id.to_string(),
            member_count: group.members.len(),
            is_public: group.is_public,
            created_at: group.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchUsersQuery {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendFriendRequest {
    pub to_user_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FriendRequestResponse {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub message: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl From<crate::domain::FriendRequest> for FriendRequestResponse {
    fn from(req: crate::domain::FriendRequest) -> Self {
        Self {
            id: req.id.to_string(),
            from_user_id: req.from_user.to_string(),
            to_user_id: req.to_user.to_string(),
            message: req.message,
            status: req.status.to_string(),
            created_at: req.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FriendshipResponse {
    pub id: String,
    pub friend_id: String,
    pub remark: Option<String>,
    pub created_at: String,
}

impl From<crate::domain::Friendship> for FriendshipResponse {
    fn from(f: crate::domain::Friendship) -> Self {
        Self {
            id: f.id.to_string(),
            friend_id: f.friend_id.to_string(),
            remark: f.remark,
            created_at: f.created_at.to_rfc3339(),
        }
    }
}
