use std::net::SocketAddr;

use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::api::{create_routes, AppState};
use crate::config::Settings;
use crate::error::AppResult;
use crate::event::EventBus;
use crate::message::HandlerChain;

pub struct ChatServer {
    settings: Settings,
    app_state: AppState,
}

impl ChatServer {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(self) -> AppResult<()> {
        let addr: SocketAddr = self.settings.server.addr();
        
        tracing::info!("Starting server on {}", addr);
        
        let listener = TcpListener::bind(addr).await
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        
        let router = create_routes()
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(self.app_state);
        
        axum::serve(listener, router)
            .await
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        
        Ok(())
    }
}

#[derive(Default)]
pub struct ServerBuilder {
    settings: Option<Settings>,
    event_bus: Option<EventBus>,
    handlers: HandlerChain,
}

impl ServerBuilder {
    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn add_handler(mut self, handler: Box<dyn crate::message::MessageHandler>) -> Self {
        self.handlers = self.handlers.add(handler);
        self
    }

    pub fn build(self) -> AppResult<ChatServer> {
        let settings = self.settings.unwrap_or_default();
        let _event_bus = self.event_bus.unwrap_or_default();

        let app_state = AppState::new();

        Ok(ChatServer { settings, app_state })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let server = ChatServer::builder()
            .settings(Settings::default())
            .build()
            .unwrap();
        
        assert_eq!(server.settings.server.port, 8080);
    }
}
