pub mod user_repository;
pub mod device_repository;
pub mod conversation_repository;
pub mod message_repository;
pub mod group_repository;
pub mod friend_repository;

pub use user_repository::{UserRepository, PostgresUserRepository};
pub use device_repository::{DeviceRepository, PostgresDeviceRepository};
pub use conversation_repository::{ConversationRepository, PostgresConversationRepository};
pub use message_repository::{MessageRepository, PostgresMessageRepository};
pub use group_repository::{GroupRepository, PostgresGroupRepository};
pub use friend_repository::{FriendRepository, PostgresFriendRepository, InMemoryFriendRepository};

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::config::DatabaseSettings;
use crate::error::AppResult;

pub async fn create_pool(config: &DatabaseSettings) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .connect(&config.connection_string())
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Migration failed: {}", e)))?;

    Ok(())
}
