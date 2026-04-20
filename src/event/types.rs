use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::domain::{FriendRequestId, FriendRequest, Friendship, GroupId, Message, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    MessageReceived {
        message: Message,
    },
    MessageDelivered {
        message_id: String,
        to_user: UserId,
        to_device: String,
    },
    MessageRead {
        message_id: String,
        by_user: UserId,
    },
    UserOnline {
        user_id: UserId,
        device_id: String,
    },
    UserOffline {
        user_id: UserId,
        device_id: String,
    },
    GroupCreated {
        group_id: GroupId,
        creator: UserId,
    },
    GroupMemberJoined {
        group_id: GroupId,
        user_id: UserId,
    },
    GroupMemberLeft {
        group_id: GroupId,
        user_id: UserId,
    },
    TypingStart {
        conversation_id: String,
        user_id: UserId,
    },
    TypingStop {
        conversation_id: String,
        user_id: UserId,
    },
    FriendRequestReceived {
        request: FriendRequest,
    },
    FriendRequestAccepted {
        friendship: Friendship,
    },
    FriendRequestRejected {
        request_id: FriendRequestId,
    },
    FriendRemoved {
        user_id: UserId,
        friend_id: UserId,
    },
}

impl Event {
    pub fn timestamp(&self) -> DateTime<Utc> {
        Utc::now()
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Event::MessageReceived { .. } => "message_received",
            Event::MessageDelivered { .. } => "message_delivered",
            Event::MessageRead { .. } => "message_read",
            Event::UserOnline { .. } => "user_online",
            Event::UserOffline { .. } => "user_offline",
            Event::GroupCreated { .. } => "group_created",
            Event::GroupMemberJoined { .. } => "group_member_joined",
            Event::GroupMemberLeft { .. } => "group_member_left",
            Event::TypingStart { .. } => "typing_start",
            Event::TypingStop { .. } => "typing_stop",
            Event::FriendRequestReceived { .. } => "friend_request_received",
            Event::FriendRequestAccepted { .. } => "friend_request_accepted",
            Event::FriendRequestRejected { .. } => "friend_request_rejected",
            Event::FriendRemoved { .. } => "friend_removed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type() {
        let event = Event::UserOnline {
            user_id: UserId::new(),
            device_id: "test".to_string(),
        };
        assert_eq!(event.event_type(), "user_online");
    }
}
