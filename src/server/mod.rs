use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::api::{create_routes, AppState};
use crate::config::{CorsSettings, Settings};
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

        tracing::info!(
            server.host = %self.settings.server.host,
            server.port = %self.settings.server.port,
            "Starting Chat-General server"
        );

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to bind to address {}", addr);
            crate::error::AppError::Internal(e.to_string())
        })?;

        tracing::info!("Server listening on {}", addr);

        let cors = build_cors_layer(&self.settings.cors);

        let router = create_routes()
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(self.app_state);

        let shutdown_signal = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C handler");
            tracing::info!("Received shutdown signal, draining connections...");
        };

        if let Err(e) = axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal)
            .await
        {
            tracing::error!(error = %e, "Server error");
            return Err(crate::error::AppError::Internal(e.to_string()));
        }

        tracing::info!("Server shutdown complete");
        Ok(())
    }
}

#[derive(Default)]
pub struct ServerBuilder {
    settings: Option<Settings>,
    event_bus: Option<EventBus>,
    handlers: HandlerChain,
    use_in_memory: bool,
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

    pub fn add_handler(mut self, handler: Arc<dyn crate::message::MessageHandler>) -> Self {
        self.handlers = self.handlers.with_handler(handler);
        self
    }

    pub fn in_memory(mut self) -> Self {
        self.use_in_memory = true;
        self
    }

    pub async fn build(self) -> AppResult<ChatServer> {
        let settings = self.settings.unwrap_or_default();
        let _event_bus = self.event_bus.unwrap_or_default();

        if !self.use_in_memory {
            settings.jwt.validate_secret().map_err(|e| {
                tracing::error!(error = %e, "JWT secret validation failed");
                crate::error::AppError::Internal(e)
            })?;
        }

        let app_state = if self.use_in_memory {
            AppState::new()
        } else {
            AppState::from_settings(&settings).await?
        };

        Ok(ChatServer {
            settings,
            app_state,
        })
    }
}

fn build_cors_layer(cors: &CorsSettings) -> CorsLayer {
    let origins: Vec<_> = cors.origins.iter().filter_map(|o| o.parse().ok()).collect();

    let methods: Vec<_> = cors
        .methods
        .iter()
        .filter_map(|m| match m.to_uppercase().as_str() {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "PATCH" => Some(Method::PATCH),
            "HEAD" => Some(Method::HEAD),
            "OPTIONS" => Some(Method::OPTIONS),
            _ => None,
        })
        .collect();

    let headers: Vec<_> = cors
        .headers
        .iter()
        .filter_map(|h| match h.to_lowercase().as_str() {
            "authorization" => Some(AUTHORIZATION),
            "content-type" => Some(CONTENT_TYPE),
            _ => None,
        })
        .collect();

    let mut layer = CorsLayer::new();

    if !origins.is_empty() {
        layer = layer.allow_origin(origins);
    }
    if !methods.is_empty() {
        layer = layer.allow_methods(methods);
    }
    if !headers.is_empty() {
        layer = layer.allow_headers(headers);
    }

    layer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let settings = Settings::default();
        assert_eq!(settings.server.port, 8080);
    }
}
