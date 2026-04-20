use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{User, UserId, UserStatus};
use crate::error::AppResult;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> AppResult<User>;
    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn update(&self, user: &User) -> AppResult<User>;
    async fn update_status(&self, id: &UserId, status: UserStatus) -> AppResult<()>;
    async fn delete(&self, id: &UserId) -> AppResult<()>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> AppResult<User> {
        let record = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, avatar_url, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
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
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>> {
        let record = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let record = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let record = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    async fn update(&self, user: &User) -> AppResult<User> {
        let record = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET username = $2, email = $3, display_name = $4, avatar_url = $5, status = $6
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user.id.as_uuid())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(user.status.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn update_status(&self, id: &UserId, status: UserStatus) -> AppResult<()> {
        sqlx::query("UPDATE users SET status = $2 WHERE id = $1")
            .bind(id.as_uuid())
            .bind(status.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete(&self, id: &UserId) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
