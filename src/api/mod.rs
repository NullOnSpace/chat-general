pub mod auth_extractor;
pub mod dto;
pub mod handlers;
pub mod websocket;

pub use auth_extractor::AuthorizationHeader;
pub use dto::*;
pub use handlers::*;
pub use websocket::*;

use crate::auth::JwtAuthProvider;
use crate::event::EventBus;
use crate::friend::FriendService;
use crate::group::GroupService;
use crate::infra::{create_user_store, InMemoryFriendRepository, UserStorage};
use crate::message::InMemoryMessageStore;
use crate::session::{DeviceRegistry, SessionManager};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub device_registry: DeviceRegistry,
    pub session_manager: SessionManager,
    pub message_store: InMemoryMessageStore,
    pub friend_service: Arc<dyn FriendService>,
    pub group_service: Arc<dyn GroupService>,
    pub jwt_provider: Arc<JwtAuthProvider>,
    pub user_store: Arc<dyn UserStorage>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let event_bus = EventBus::new();
        let friend_repo = InMemoryFriendRepository::new();
        let friend_service = Arc::new(crate::friend::FriendManager::new(friend_repo, event_bus));

        let group_service = Arc::new(crate::group::GroupManager::new());

        let jwt_settings = crate::config::Settings::default().jwt;
        let jwt_provider = Arc::new(JwtAuthProvider::new(&jwt_settings));

        let user_store = create_user_store();

        Self {
            device_registry: DeviceRegistry::new(),
            session_manager: SessionManager::new(),
            message_store: InMemoryMessageStore::new(),
            friend_service,
            group_service,
            jwt_provider,
            user_store,
        }
    }
}

pub fn create_routes() -> Router<AppState> {
    let api_routes = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/me", get(handlers::auth::get_current_user))
        .route("/users/me", get(handlers::auth::get_current_user))
        .route("/users/me/devices", get(handlers::auth::get_user_devices))
        .route("/users/search", get(handlers::auth::search_users))
        .route("/conversations", get(handlers::message::get_conversations))
        .route(
            "/conversations",
            post(handlers::message::create_conversation),
        )
        .route(
            "/conversations/{id}",
            get(handlers::message::get_conversation),
        )
        .route(
            "/conversations/{id}/messages",
            get(handlers::message::get_messages),
        )
        .route("/messages", post(handlers::message::send_message))
        .route("/groups", get(handlers::group::get_user_groups))
        .route("/groups", post(handlers::group::create_group))
        .route("/groups/{id}", get(handlers::group::get_group))
        .route(
            "/groups/{id}/members",
            get(handlers::group::get_group_members),
        )
        .route("/groups/{id}/members", put(handlers::group::add_member))
        .route(
            "/groups/{id}/members/{uid}",
            delete(handlers::group::remove_member),
        )
        .route("/friends", get(handlers::friend::get_friends))
        .route("/friends/{uid}", delete(handlers::friend::delete_friend))
        .route(
            "/friends/requests",
            get(handlers::friend::get_pending_requests),
        )
        .route(
            "/friends/requests",
            post(handlers::friend::send_friend_request),
        )
        .route(
            "/friends/requests/sent",
            get(handlers::friend::get_sent_requests),
        )
        .route(
            "/friends/requests/{id}/accept",
            put(handlers::friend::accept_friend_request),
        )
        .route(
            "/friends/requests/{id}/reject",
            delete(handlers::friend::reject_friend_request),
        );

    Router::new()
        .route("/ws", get(websocket::ws_handler))
        .nest("/api/v1", api_routes)
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_creation() {
        let _router = create_routes();
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.device_registry.get_online_users().await.is_empty());
    }
}