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
use crate::domain::{DeviceId, MessageId, UserId};

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

#[derive(Debug, Serialize, Deserialize)]
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
    Ack {
        message_id: String,
        status: String,
        seq: u64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConversation {
    pub conversation_id: String,
    pub messages: Vec<MessageData>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    let (mut sender, mut receiver) = socket.split();

    let connected_msg = WsServerMessage::Connected {
        user_id: user_id.to_string(),
        device_id: device_id.to_string(),
    };
    if let Err(e) = sender
        .send(WsMessage::Text(
            serde_json::to_string(&connected_msg).unwrap().into(),
        ))
        .await
    {
        tracing::error!("Failed to send connected message: {}", e);
        return;
    }

    let session = state.session_manager.create(user_id, device_id).await.ok();

    tracing::info!(
        "WebSocket connected: user={}, device={}",
        user_id,
        device_id
    );

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                if let Ok(payload) = serde_json::from_str::<WsMessagePayload>(&text) {
                    handle_client_message(&mut sender, payload, &user_id).await;
                } else {
                    let _ = sender
                        .send(WsMessage::Text(
                            serde_json::to_string(&WsServerMessage::Error {
                                code: 400,
                                message: "Invalid message format".to_string(),
                            })
                            .unwrap()
                            .into(),
                        ))
                        .await;
                }
            }
            Ok(WsMessage::Ping(data)) => {
                let _ = sender.send(WsMessage::Pong(data)).await;
            }
            Ok(WsMessage::Pong(_)) => {}
            Ok(WsMessage::Binary(_)) => {}
            Ok(WsMessage::Close(frame)) => {
                tracing::info!("WebSocket closed: {:?}", frame);
                let _ = sender.close().await;
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    if let Some(s) = session {
        let _ = state.session_manager.terminate(&s.id).await;
    }

    tracing::info!(
        "WebSocket disconnected: user={}, device={}",
        user_id,
        device_id
    );
}

async fn handle_client_message(
    sender: &mut futures::stream::SplitSink<WebSocket, WsMessage>,
    payload: WsMessagePayload,
    user_id: &UserId,
) {
    match payload {
        WsMessagePayload::Message {
            conversation_id,
            content,
            seq,
            ..
        } => {
            let msg = WsServerMessage::Message {
                id: MessageId::new().to_string(),
                conversation_id,
                sender_id: user_id.to_string(),
                content,
                message_type: "text".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                seq,
            };
            let _ = sender
                .send(WsMessage::Text(serde_json::to_string(&msg).unwrap().into()))
                .await;
        }
        WsMessagePayload::Ack { message_id, seq } => {
            let ack = WsServerMessage::Ack {
                message_id,
                status: "delivered".to_string(),
                seq,
            };
            let _ = sender
                .send(WsMessage::Text(serde_json::to_string(&ack).unwrap().into()))
                .await;
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
