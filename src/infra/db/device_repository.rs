use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{Device, DeviceId, UserId};
use crate::error::AppResult;

#[async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn create(&self, device: &Device) -> AppResult<Device>;
    async fn find_by_id(&self, id: &DeviceId) -> AppResult<Option<Device>>;
    async fn find_by_user(&self, user_id: &UserId) -> AppResult<Vec<Device>>;
    async fn update_last_active(&self, id: &DeviceId) -> AppResult<()>;
    async fn update_push_token(&self, id: &DeviceId, token: &str) -> AppResult<()>;
    async fn delete(&self, id: &DeviceId) -> AppResult<()>;
}

pub struct PostgresDeviceRepository {
    pool: PgPool,
}

impl PostgresDeviceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceRepository for PostgresDeviceRepository {
    async fn create(&self, device: &Device) -> AppResult<Device> {
        let record = sqlx::query_as::<_, Device>(
            r#"
            INSERT INTO devices (id, user_id, device_type, device_name, push_token, last_active_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(device.id.as_uuid())
        .bind(device.user_id.as_uuid())
        .bind(device.device_type.to_string())
        .bind(&device.device_name)
        .bind(&device.push_token)
        .bind(device.last_active_at)
        .bind(device.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn find_by_id(&self, id: &DeviceId) -> AppResult<Option<Device>> {
        let record = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn find_by_user(&self, user_id: &UserId) -> AppResult<Vec<Device>> {
        let records = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices WHERE user_id = $1 ORDER BY last_active_at DESC",
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn update_last_active(&self, id: &DeviceId) -> AppResult<()> {
        sqlx::query(
            "UPDATE devices SET last_active_at = NOW() WHERE id = $1",
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_push_token(&self, id: &DeviceId, token: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE devices SET push_token = $2 WHERE id = $1",
        )
        .bind(id.as_uuid())
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &DeviceId) -> AppResult<()> {
        sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
