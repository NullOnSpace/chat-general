use async_trait::async_trait;
use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;

use chat_general::api::{create_routes, AppState};
use chat_general::config::Settings;
use chat_general::domain::User;
use chat_general::error::{AppError, AppResult};
use chat_general::infra::{run_migrations, UserStorage};

static NEXT_PORT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(20000));

pub struct TestAppWithDb {
    pub address: String,
    pub server_handle: Option<JoinHandle<()>>,
    pub pool: PgPool,
}

impl TestAppWithDb {
    pub async fn new() -> Self {
        let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
        let address = format!("127.0.0.1:{}", port);

        let pool = create_test_pool().await;
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        let state = create_app_state(pool.clone());
        let router = create_routes().with_state(state);

        let addr: std::net::SocketAddr = address.parse().expect("Invalid address");
        let server_handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("Failed to bind");
            axum::serve(listener, router).await.expect("Server error");
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        Self {
            address,
            server_handle: Some(server_handle),
            pool,
        }
    }

    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client")
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub async fn cleanup(&self) {
        let queries = vec![
            "DELETE FROM message_deliveries",
            "DELETE FROM messages",
            "DELETE FROM group_members",
            "DELETE FROM groups",
            "DELETE FROM friendships",
            "DELETE FROM friend_requests",
            "DELETE FROM conversation_participants",
            "DELETE FROM conversations",
            "DELETE FROM devices",
            "DELETE FROM users",
        ];

        for query in queries {
            sqlx::query(query).execute(&self.pool).await.ok();
        }
    }
}

impl Drop for TestAppWithDb {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

async fn create_test_pool() -> PgPool {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set in .env file");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

fn create_app_state(pool: PgPool) -> AppState {
    use chat_general::auth::JwtAuthProvider;
    use chat_general::event::EventBus;
    use chat_general::friend::FriendManager;
    use chat_general::group::GroupManager;
    use chat_general::infra::PostgresFriendRepository;
    use chat_general::message::InMemoryMessageStore;
    use chat_general::session::{DeviceRegistry, SessionManager};
    use std::sync::Arc;

    let event_bus = EventBus::new();
    let friend_repo = PostgresFriendRepository::new(pool.clone());
    let friend_service = Arc::new(FriendManager::new(friend_repo, event_bus));

    let group_service = Arc::new(GroupManager::new());

    let jwt_settings = Settings::default().jwt;
    let jwt_provider = Arc::new(JwtAuthProvider::new(&jwt_settings));

    let user_store = Arc::new(DbUserStore::new(pool));

    AppState {
        device_registry: DeviceRegistry::new(),
        session_manager: SessionManager::new(),
        message_store: InMemoryMessageStore::new(),
        friend_service,
        group_service,
        jwt_provider,
        user_store,
    }
}

pub struct DbUserStore {
    pool: PgPool,
}

impl DbUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStorage for DbUserStore {
    async fn create(&self, user: User) -> AppResult<User> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, avatar_url, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(user.id.as_uuid())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.status.to_string())
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::Conflict("Username or email already exists".to_string())
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

        Ok(user)
    }

    async fn get_by_id(&self, id: &str) -> Option<User> {
        let id = uuid::Uuid::parse_str(id).ok()?;
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .ok()?
    }

    async fn get_by_username(&self, username: &str) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .ok()?
    }

    async fn get_by_email(&self, email: &str) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .ok()?
    }

    async fn search(&self, query: &str) -> Vec<User> {
        let pattern = format!("%{}%", query.to_lowercase());
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE LOWER(username) LIKE $1 OR LOWER(email) LIKE $1 OR LOWER(display_name) LIKE $1"
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }
}
