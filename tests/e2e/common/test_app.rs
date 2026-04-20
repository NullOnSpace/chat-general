use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::task::JoinHandle;

use chat_general::api::{AppState, create_routes};

static NEXT_PORT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(19000));

pub struct TestApp {
    pub address: String,
    pub server_handle: Option<JoinHandle<()>>,
}

impl TestApp {
    pub async fn new() -> Self {
        let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
        let address = format!("127.0.0.1:{}", port);
        let addr: SocketAddr = address.parse().expect("Invalid address");
        
        let state = AppState::new();
        let router = create_routes().with_state(state);
        
        let server_handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
            axum::serve(listener, router).await.expect("Server error");
        });
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Self {
            address,
            server_handle: Some(server_handle),
        }
    }

    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client")
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn ws_url(&self) -> String {
        format!("ws://{}", self.address)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}