use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthProvider;
use crate::domain::{DeviceId, UserId};
use crate::session::Session;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessagePayload {
    Message {
        conversation_id: String,
        content: String,
        message_type: Option<String>,
        reply_to: Option<String>,
        seq: u64,
    },
    Ack {
        message_id: String,
        seq: u64,
    },
    Typing {
        conversation_id: String,
        is_typing: bool,
    },
    Sync {
        last_sync: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    Message {
        id: String,
        conversation_id: String,
        sender_id: String,
        content: String,
        message_type: String,
        created_at: String,
        seq: u64,
    },
    MessageSent {
        id: String,
        temp_id: String,
        conversation_id: String,
        sender_id: String,
        content: String,
        message_type: String,
        created_at: String,
        seq: u64,
    },
    Ack {
        message_id: String,
        status: String,
        seq: u64,
    },
    Typing {
        user_id: String,
        conversation_id: String,
        is_typing: bool,
    },
    Presence {
        user_id: String,
        device_id: String,
        is_online: bool,
    },
    Sync {
        conversations: Vec<SyncConversation>,
    },
    Error {
        code: u16,
        message: String,
    },
    Connected {
        user_id: String,
        device_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncConversation {
    pub conversation_id: String,
    pub messages: Vec<MessageData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageData {
    pub id: String,
    pub content: String,
    pub sender_id: String,
    pub created_at: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    let device_id = match Uuid::parse_str(&query.device_id) {
        Ok(id) => DeviceId::from(id),
        Err(_) => {
            return ws.on_upgrade(|socket| async move {
                let (mut sender, _) = socket.split();
                let _ = sender
                    .send(WsMessage::Text(
                        serde_json::to_string(&WsServerMessage::Error {
                            code: 400,
                            message: "Invalid device_id".to_string(),
                        })
                        .unwrap()
                        .into(),
                    ))
                    .await;
            });
        }
    };

    let user_id = match validate_token(&state, &query.token).await {
        Ok(id) => id,
        Err(e) => {
            return ws.on_upgrade(|socket| async move {
                let (mut sender, _) = socket.split();
                let _ = sender
                    .send(WsMessage::Text(
                        serde_json::to_string(&WsServerMessage::Error {
                            code: 401,
                            message: format!("Authentication failed: {}", e),
                        })
                        .unwrap()
                        .into(),
                    ))
                    .await;
            });
        }
    };

    ws.on_upgrade(move |socket| handle_websocket(socket, user_id, device_id, state))
}

async fn validate_token(state: &AppState, token: &str) -> Result<UserId, String> {
    let auth_user = state
        .jwt_provider
        .validate_token(token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(auth_user.user_id)
}

async fn handle_websocket(
    socket: WebSocket,
    user_id: UserId,
    device_id: DeviceId,
    state: AppState,
) {
    let (mut ws_sender, mut receiver) = socket.split();

    let connected_msg = WsServerMessage::Connected {
        user_id: user_id.to_string(),
        device_id: device_id.to_string(),
    };
    if let Err(e) = ws_sender
        .send(WsMessage::Text(
            serde_json::to_string(&connected_msg).unwrap().into(),
        ))
        .await
    {
        tracing::error!("Failed to send connected message: {}", e);
        return;
    }

    let session = state.session_manager.create(user_id, device_id).await.ok();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    state.session_manager.register_sender(user_id, tx).await;

    tracing::info!(
        "WebSocket connected: user={}, device={}",
        user_id,
        device_id
    );

    let sender_user_id = user_id;
    let state_clone = state.clone();
    let (pong_tx, mut pong_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(payload) = serde_json::from_str::<WsMessagePayload>(&text) {
                        handle_client_message(&state_clone, &sender_user_id, payload).await;
                    }
                }
                Ok(WsMessage::Ping(data)) => {
                    let _ = pong_tx.send(data.to_vec());
                }
                Ok(WsMessage::Close(_)) => break,
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(text) = rx.recv() => {
                    if ws_sender.send(WsMessage::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Some(data) = pong_rx.recv() => {
                    if ws_sender.send(WsMessage::Pong(data.into())).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }
    });

    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }

    state.session_manager.unregister_sender(&user_id).await;

    if let Some(s) = session {
        let _ = state.session_manager.terminate(&s.id).await;
    }

    tracing::info!(
        "WebSocket disconnected: user={}, device={}",
        user_id,
        device_id
    );
}

async fn handle_client_message(state: &AppState, user_id: &UserId, payload: WsMessagePayload) {
    match payload {
        WsMessagePayload::Message {
            conversation_id,
            content,
            seq,
            ..
        } => {
            let conv_id = match crate::domain::ConversationId::try_from(conversation_id.clone()) {
                Ok(id) => id,
                Err(_) => return,
            };

            let device_id = DeviceId::new();
            let session = Session::new(*user_id, device_id);
            let message = crate::domain::Message::text(conv_id, *user_id, content.clone());

            let message = match state.handler_chain.process(message, &session).await {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::warn!(error = %e, "Message rejected by handler chain");
                    return;
                }
            };

            let msg = WsServerMessage::Message {
                id: message.id.to_string(),
                conversation_id: message.conversation_id.to_string(),
                sender_id: message.sender_id.to_string(),
                content: message.content.clone(),
                message_type: "text".to_string(),
                created_at: message.created_at.to_rfc3339(),
                seq,
            };

            let msg_sent = WsServerMessage::MessageSent {
                id: message.id.to_string(),
                temp_id: "".to_string(),
                conversation_id: message.conversation_id.to_string(),
                sender_id: message.sender_id.to_string(),
                content: message.content.clone(),
                message_type: "text".to_string(),
                created_at: message.created_at.to_rfc3339(),
                seq,
            };

            let msg_json = serde_json::to_string(&msg).unwrap();
            let msg_sent_json = serde_json::to_string(&msg_sent).unwrap();

            let participants = state
                .message_store
                .get_conversation_participants(&message.conversation_id)
                .await
                .unwrap_or_default();

            state
                .session_manager
                .send_to_user(user_id, &msg_sent_json)
                .await;

            for participant_id in participants {
                if participant_id != *user_id {
                    state
                        .session_manager
                        .send_to_user(&participant_id, &msg_json)
                        .await;
                }
            }
        }
        WsMessagePayload::Ack { .. } => {}
        WsMessagePayload::Typing {
            conversation_id,
            is_typing,
        } => {
            let typing_msg = WsServerMessage::Typing {
                user_id: user_id.to_string(),
                conversation_id: conversation_id.clone(),
                is_typing,
            };
            let msg_json = serde_json::to_string(&typing_msg).unwrap();

            let conv_id = match crate::domain::ConversationId::try_from(conversation_id) {
                Ok(id) => id,
                Err(_) => return,
            };
            let participants = state
                .message_store
                .get_conversation_participants(&conv_id)
                .await
                .unwrap_or_default();

            for participant_id in participants {
                if participant_id != *user_id {
                    state
                        .session_manager
                        .send_to_user(&participant_id, &msg_json)
                        .await;
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_payload_serialization() {
        let payload = WsMessagePayload::Message {
            conversation_id: "test-conv".to_string(),
            content: "Hello".to_string(),
            message_type: Some("text".to_string()),
            reply_to: None,
            seq: 1,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("message"));
    }

    #[test]
    fn test_ws_server_message_serialization() {
        let msg = WsServerMessage::Ack {
            message_id: "msg-123".to_string(),
            status: "delivered".to_string(),
            seq: 1,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("ack"));
    }
}
