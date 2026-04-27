use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
    #[serde(default)]
    pub cors: CorsSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub workers: Option<usize>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

impl ServerSettings {
    pub fn addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid server address")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseSettings {
    #[serde(default = "default_db_host")]
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    #[serde(default = "default_db_username")]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_db_name")]
    pub database: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
}

fn default_db_host() -> String {
    "localhost".to_string()
}

fn default_db_port() -> u16 {
    5432
}

fn default_db_username() -> String {
    "postgres".to_string()
}

fn default_db_name() -> String {
    "chat_general".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisSettings {
    #[serde(default = "default_redis_host")]
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    pub password: Option<String>,
    #[serde(default)]
    pub database: u8,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

fn default_pool_size() -> u32 {
    10
}

impl RedisSettings {
    pub fn connection_string(&self) -> String {
        match &self.password {
            Some(pwd) if !pwd.is_empty() => format!(
                "redis://:{}@{}:{}/{}",
                pwd, self.host, self.port, self.database
            ),
            _ => format!("redis://{}:{}/{}", self.host, self.port, self.database),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtSettings {
    #[serde(default = "default_jwt_secret")]
    pub secret: String,
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry: u64,
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry: u64,
    #[serde(default = "default_issuer")]
    pub issuer: String,
}

fn default_jwt_secret() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_access_token_expiry() -> u64 {
    3600
}

fn default_refresh_token_expiry() -> u64 {
    604800
}

fn default_issuer() -> String {
    "chat-general".to_string()
}

impl JwtSettings {
    pub fn validate_secret(&self) -> Result<(), String> {
        if self.secret.len() < 32 {
            return Err(format!(
                "JWT secret must be at least 32 characters, got {}. Set CHAT__JWT__SECRET environment variable.",
                self.secret.len()
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsSettings {
    #[serde(default = "default_cors_origins")]
    pub origins: Vec<String>,
    #[serde(default = "default_cors_methods")]
    pub methods: Vec<String>,
    #[serde(default = "default_cors_headers")]
    pub headers: Vec<String>,
}

impl Default for CorsSettings {
    fn default() -> Self {
        Self {
            origins: default_cors_origins(),
            methods: default_cors_methods(),
            headers: default_cors_headers(),
        }
    }
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "PATCH".to_string(),
    ]
}

fn default_cors_headers() -> Vec<String> {
    vec!["Authorization".to_string(), "Content-Type".to_string()]
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(
                config::File::with_name(&format!("config/{}", Self::get_env())).required(false),
            )
            .add_source(config::Environment::with_prefix("CHAT").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    fn get_env() -> String {
        std::env::var("CHAT_ENV").unwrap_or_else(|_| "development".into())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                host: default_host(),
                port: default_port(),
                workers: None,
            },
            database: DatabaseSettings {
                host: default_db_host(),
                port: default_db_port(),
                username: default_db_username(),
                password: String::new(),
                database: default_db_name(),
                max_connections: default_max_connections(),
                min_connections: default_min_connections(),
                connect_timeout: default_connect_timeout(),
                idle_timeout: default_idle_timeout(),
            },
            redis: RedisSettings {
                host: default_redis_host(),
                port: default_redis_port(),
                password: None,
                database: 0,
                pool_size: default_pool_size(),
            },
            jwt: JwtSettings {
                secret: default_jwt_secret(),
                access_token_expiry: default_access_token_expiry(),
                refresh_token_expiry: default_refresh_token_expiry(),
                issuer: default_issuer(),
            },
            cors: CorsSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.redis.host, "localhost");
    }

    #[test]
    fn test_database_connection_string() {
        let db = DatabaseSettings {
            host: "localhost".to_string(),
            port: 5432,
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            database: "testdb".to_string(),
            ..Settings::default().database
        };
        assert_eq!(
            db.connection_string(),
            "postgres://testuser:testpass@localhost:5432/testdb"
        );
    }

    #[test]
    fn test_redis_connection_string() {
        let redis = RedisSettings {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            database: 0,
            ..Settings::default().redis
        };
        assert_eq!(redis.connection_string(), "redis://localhost:6379/0");
    }

    #[test]
    fn test_server_addr() {
        let server = ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ..Settings::default().server
        };
        assert_eq!(server.addr().to_string(), "127.0.0.1:3000");
    }
}
