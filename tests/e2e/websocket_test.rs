use super::common::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serial_test::serial;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

#[tokio::test]
#[serial]
async fn test_websocket_connect_success() {
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
    assert!(result.is_ok(), "WebSocket connection should succeed");

    let (ws_stream, _) = result.unwrap();
    let (mut _sender, mut receiver) = ws_stream.split();

    if let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        assert_eq!(
            data["type"].as_str().unwrap(),
            "connected",
            "Should receive connected message"
        );
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_connect_invalid_token() {
    let app = TestApp::new().await;
    let device_id = format!("device-{}", uuid::Uuid::new_v4());

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
            assert_eq!(
                data["type"].as_str().unwrap(),
                "error",
                "Should receive error message"
            );
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
        urlencoding::encode("invalid_device_id")
    );

    let result = connect_async(&ws_url).await;

    if result.is_ok() {
        let (ws_stream, _) = result.unwrap();
        let (_sender, mut receiver) = ws_stream.split();

        if let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
            let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
            assert_eq!(
                data["type"].as_str().unwrap(),
                "error",
                "Should receive error message for invalid device_id"
            );
        }
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_send_message() {
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
    assert!(result.is_ok(), "WebSocket connection should succeed");

    let (ws_stream, _) = result.unwrap();
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        if data["type"].as_str() == Some("connected") {
            break;
        }
    }

    let message_payload = json!({
        "type": "send_message",
        "payload": {
            "conversation_id": "test-conversation-id",
            "content": "Hello via WebSocket!",
            "temp_id": "temp-123"
        }
    });

    let send_result = sender
        .send(WsMessage::Text(message_payload.to_string()))
        .await;
    assert!(send_result.is_ok(), "Sending message should succeed");
}

#[tokio::test]
#[serial]
async fn test_websocket_send_invalid_json() {
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
    assert!(result.is_ok(), "WebSocket connection should succeed");

    let (ws_stream, _) = result.unwrap();
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        if data["type"].as_str() == Some("connected") {
            break;
        }
    }

    let send_result = sender
        .send(WsMessage::Text("invalid json {{{".to_string()))
        .await;
    assert!(send_result.is_ok(), "Sending invalid JSON should not crash");

    if let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        assert_eq!(
            data["type"].as_str().unwrap(),
            "error",
            "Should receive error for invalid JSON"
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
    assert!(result.is_ok(), "WebSocket connection should succeed");

    let (ws_stream, _) = result.unwrap();
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        if data["type"].as_str() == Some("connected") {
            break;
        }
    }

    let ping_data = vec![1, 2, 3, 4];
    let send_result = sender.send(WsMessage::Ping(ping_data.clone())).await;
    assert!(send_result.is_ok(), "Sending ping should succeed");

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
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(WsMessage::Text(text))) = receiver.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).expect("Should parse JSON");
        if data["type"].as_str() == Some("connected") {
            break;
        }
    }

    let close_result = sender.send(WsMessage::Close(None)).await;
    assert!(close_result.is_ok(), "Sending close should succeed");
}
