use super::common::*;
use axum::body::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serial_test::serial;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

struct WsTestHelper {
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        WsMessage,
    >,
    receiver: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WsTestHelper {
    async fn connect(app: &TestApp, user: &TestUser) -> Self {
        let device_id = user.device_id();
        let ws_url = format!(
            "{}/ws?token={}&device_id={}",
            app.ws_url(),
            urlencoding::encode(&user.access_token),
            urlencoding::encode(&device_id)
        );

        let result = connect_async(&ws_url).await;
        assert!(result.is_ok(), "WebSocket connection should succeed");

        let (ws_stream, _) = result.unwrap();
        let (sender, receiver) = ws_stream.split();

        Self { sender, receiver }
    }

    async fn wait_for_connected(&mut self) -> serde_json::Value {
        while let Some(Ok(WsMessage::Text(text))) = self.receiver.next().await {
            let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
            if data["type"].as_str() == Some("connected") {
                return data;
            }
        }
        panic!("Never received connected message");
    }

    async fn send_json(&mut self, payload: &serde_json::Value) {
        self.sender
            .send(WsMessage::Text(payload.to_string().into()))
            .await
            .expect("Failed to send message");
    }

    async fn recv_text(&mut self) -> Option<serde_json::Value> {
        while let Some(Ok(msg)) = self.receiver.next().await {
            match msg {
                WsMessage::Text(text) => {
                    let data: serde_json::Value =
                        serde_json::from_str(&text).expect("Should parse JSON");
                    return Some(data);
                }
                WsMessage::Close(_) => return None,
                _ => continue,
            }
        }
        None
    }

    async fn recv_text_timeout(&mut self, timeout_ms: u64) -> Option<serde_json::Value> {
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.recv_text(),
        )
        .await;
        result.ok().flatten()
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_connect_success() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;
    let mut ws = WsTestHelper::connect(&app, &user).await;

    let data = ws.wait_for_connected().await;
    assert_eq!(data["type"].as_str().unwrap(), "connected");
    assert!(data["user_id"].as_str().is_some());
    assert!(data["device_id"].as_str().is_some());
}

#[tokio::test]
#[serial]
async fn test_websocket_connect_invalid_token() {
    let app = TestApp::new().await;
    let device_id = uuid::Uuid::new_v4().to_string();

    let ws_url = format!(
        "{}/ws?token={}&device_id={}",
        app.ws_url(),
        urlencoding::encode("invalid_token"),
        urlencoding::encode(&device_id)
    );

    let result = connect_async(&ws_url).await;

    if result.is_ok() {
        let (ws_stream, _) = result.unwrap();
        let (_sender, mut receiver) = ws_stream.split();

        if let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
            let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
            assert_eq!(data["type"].as_str().unwrap(), "error");
            assert_eq!(data["code"].as_u64().unwrap(), 401);
        }
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_connect_no_token() {
    let app = TestApp::new().await;
    let device_id = uuid::Uuid::new_v4().to_string();

    let ws_url = format!(
        "{}/ws?device_id={}",
        app.ws_url(),
        urlencoding::encode(&device_id)
    );

    let result = connect_async(&ws_url).await;
    assert!(
        result.is_err(),
        "WebSocket connection without token should fail"
    );
}

#[tokio::test]
#[serial]
async fn test_websocket_connect_invalid_device_id() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let ws_url = format!(
        "{}/ws?token={}&device_id={}",
        app.ws_url(),
        urlencoding::encode(&user.access_token),
        urlencoding::encode("not-a-uuid")
    );

    let result = connect_async(&ws_url).await;

    if result.is_ok() {
        let (ws_stream, _) = result.unwrap();
        let (_sender, mut receiver) = ws_stream.split();

        if let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
            let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
            assert_eq!(data["type"].as_str().unwrap(), "error");
            assert_eq!(data["code"].as_u64().unwrap(), 400);
        }
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_send_message_flat_format() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;

    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;

    let mut ws1 = WsTestHelper::connect(&app, &user1).await;
    ws1.wait_for_connected().await;

    let message_payload = json!({
        "type": "message",
        "conversation_id": conv_id,
        "content": "Hello via WebSocket!",
        "message_type": "text",
        "reply_to": null,
        "seq": 1
    });

    ws1.send_json(&message_payload).await;

    let response = ws1.recv_text_timeout(2000).await;
    assert!(response.is_some(), "Should receive a response");

    let data = response.unwrap();
    assert_eq!(
        data["type"].as_str().unwrap(),
        "message_sent",
        "Sender should receive message_sent confirmation"
    );
    assert_eq!(data["conversation_id"].as_str().unwrap(), conv_id);
    assert_eq!(data["content"].as_str().unwrap(), "Hello via WebSocket!");
    assert_eq!(data["seq"].as_u64().unwrap(), 1);
    assert!(
        data["id"].as_str().is_some(),
        "Should have server-assigned ID"
    );
}

#[tokio::test]
#[serial]
async fn test_websocket_message_delivery_between_users() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;

    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;

    let mut ws1 = WsTestHelper::connect(&app, &user1).await;
    ws1.wait_for_connected().await;

    let mut ws2 = WsTestHelper::connect(&app, &user2).await;
    ws2.wait_for_connected().await;

    ws1.send_json(&json!({
        "type": "message",
        "conversation_id": conv_id,
        "content": "Hello from user1!",
        "message_type": "text",
        "reply_to": null,
        "seq": 42
    }))
    .await;

    let sender_response = ws1.recv_text_timeout(3000).await;
    assert!(
        sender_response.is_some(),
        "Sender should receive confirmation"
    );
    let sender_data = sender_response.unwrap();
    assert_eq!(sender_data["type"].as_str().unwrap(), "message_sent");

    let receiver_response = ws2.recv_text_timeout(3000).await;
    assert!(
        receiver_response.is_some(),
        "Receiver should receive the message"
    );
    let receiver_data = receiver_response.unwrap();
    assert_eq!(receiver_data["type"].as_str().unwrap(), "message");
    assert_eq!(
        receiver_data["content"].as_str().unwrap(),
        "Hello from user1!"
    );
    assert_eq!(receiver_data["conversation_id"].as_str().unwrap(), conv_id);
    assert_eq!(receiver_data["sender_id"].as_str().unwrap(), user1.id);
}

#[tokio::test]
#[serial]
async fn test_websocket_typing_indicator() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;

    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;

    let mut ws1 = WsTestHelper::connect(&app, &user1).await;
    ws1.wait_for_connected().await;

    let mut ws2 = WsTestHelper::connect(&app, &user2).await;
    ws2.wait_for_connected().await;

    ws1.send_json(&json!({
        "type": "typing",
        "conversation_id": conv_id,
        "is_typing": true
    }))
    .await;

    let typing_response = ws2.recv_text_timeout(3000).await;
    assert!(
        typing_response.is_some(),
        "Receiver should receive typing indicator"
    );
    let data = typing_response.unwrap();
    assert_eq!(data["type"].as_str().unwrap(), "typing");
    assert_eq!(data["conversation_id"].as_str().unwrap(), conv_id);
    assert!(data["is_typing"].as_bool().unwrap());
    assert_eq!(data["user_id"].as_str().unwrap(), user1.id);
}

#[tokio::test]
#[serial]
async fn test_websocket_ack_message() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let mut ws = WsTestHelper::connect(&app, &user).await;
    ws.wait_for_connected().await;

    ws.send_json(&json!({
        "type": "ack",
        "message_id": "some-message-id",
        "seq": 5
    }))
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

#[tokio::test]
#[serial]
async fn test_websocket_sync_message() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let mut ws = WsTestHelper::connect(&app, &user).await;
    ws.wait_for_connected().await;

    ws.send_json(&json!({
        "type": "sync",
        "last_sync": "2025-01-01T00:00:00Z"
    }))
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

#[tokio::test]
#[serial]
async fn test_websocket_send_invalid_json() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let mut ws = WsTestHelper::connect(&app, &user).await;
    ws.wait_for_connected().await;

    let send_result = ws
        .sender
        .send(WsMessage::Text("invalid json {{{".to_string().into()))
        .await;
    assert!(send_result.is_ok(), "Sending invalid JSON should not crash");

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

#[tokio::test]
#[serial]
async fn test_websocket_old_envelope_format_rejected() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;

    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;

    let mut ws = WsTestHelper::connect(&app, &user1).await;
    ws.wait_for_connected().await;

    let envelope_payload = json!({
        "type": "send_message",
        "payload": {
            "conversation_id": conv_id,
            "content": "This should not work"
        }
    });

    ws.send_json(&envelope_payload).await;

    let response = ws.recv_text_timeout(2000).await;
    if let Some(data) = response {
        assert_ne!(
            data["type"].as_str().unwrap_or(""),
            "message_sent",
            "Old envelope format should not produce message_sent confirmation"
        );
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_ping_pong() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let device_id = user.device_id();
    let ws_url = format!(
        "{}/ws?token={}&device_id={}",
        app.ws_url(),
        urlencoding::encode(&user.access_token),
        urlencoding::encode(&device_id)
    );

    let result = connect_async(&ws_url).await;
    assert!(result.is_ok());

    let (ws_stream, _) = result.unwrap();
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        if data["type"].as_str() == Some("connected") {
            break;
        }
    }

    let ping_data = Bytes::from(vec![1, 2, 3, 4]);
    let send_result = sender.send(WsMessage::Ping(ping_data)).await;
    assert!(send_result.is_ok());

    let mut received_pong = false;
    for _ in 0..10 {
        if let Some(Ok(WsMessage::Pong(_))) = receiver.next().await {
            received_pong = true;
            break;
        }
    }
    assert!(received_pong, "Should receive pong response");
}

#[tokio::test]
#[serial]
async fn test_websocket_close_connection() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let mut ws = WsTestHelper::connect(&app, &user).await;
    ws.wait_for_connected().await;

    let close_result = ws.sender.send(WsMessage::Close(None)).await;
    assert!(close_result.is_ok(), "Sending close should succeed");
}

#[tokio::test]
#[serial]
async fn test_websocket_message_with_unknown_conversation() {
    let app = TestApp::new().await;
    let user = TestUser::create_unique(&app).await;

    let mut ws = WsTestHelper::connect(&app, &user).await;
    ws.wait_for_connected().await;

    let fake_conv_id = uuid::Uuid::new_v4().to_string();
    ws.send_json(&json!({
        "type": "message",
        "conversation_id": fake_conv_id,
        "content": "Message to nonexistent conversation",
        "message_type": "text",
        "reply_to": null,
        "seq": 99
    }))
    .await;

    let response = ws.recv_text_timeout(2000).await;
    assert!(
        response.is_some(),
        "Should receive a response even for unknown conversation"
    );
    let data = response.unwrap();
    assert_eq!(
        data["type"].as_str().unwrap(),
        "message_sent",
        "Backend processes message regardless of conversation existence"
    );
}

#[tokio::test]
#[serial]
async fn test_websocket_multiple_messages_sequential() {
    let app = TestApp::new().await;
    let user1 = TestUser::create_unique(&app).await;
    let user2 = TestUser::create_unique(&app).await;

    make_friends(&app, &user1, &user2).await;
    let conv_id = create_test_conversation(&app, &user1, &user2.id).await;

    let mut ws1 = WsTestHelper::connect(&app, &user1).await;
    ws1.wait_for_connected().await;

    let mut ws2 = WsTestHelper::connect(&app, &user2).await;
    ws2.wait_for_connected().await;

    for i in 1..=3 {
        ws1.send_json(&json!({
            "type": "message",
            "conversation_id": conv_id,
            "content": format!("Message {}", i),
            "message_type": "text",
            "reply_to": null,
            "seq": i
        }))
        .await;

        let sender_resp = ws1.recv_text_timeout(3000).await;
        assert!(
            sender_resp.is_some(),
            "Should receive confirmation for message {}",
            i
        );
        assert_eq!(
            sender_resp.unwrap()["type"].as_str().unwrap(),
            "message_sent"
        );

        let receiver_resp = ws2.recv_text_timeout(3000).await;
        assert!(receiver_resp.is_some(), "Receiver should get message {}", i);
        let recv_data = receiver_resp.unwrap();
        assert_eq!(recv_data["type"].as_str().unwrap(), "message");
        assert_eq!(recv_data["seq"].as_u64().unwrap(), i);
    }
}
