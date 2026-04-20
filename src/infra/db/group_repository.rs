use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{Group, GroupId, GroupMember, GroupRole, UserId};
use crate::error::{AppError, AppResult};

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create(&self, group: &Group) -> AppResult<Group>;
    async fn find_by_id(&self, id: &GroupId) -> AppResult<Option<Group>>;
    async fn find_by_user(&self, user_id: &UserId) -> AppResult<Vec<Group>>;
    async fn update(&self, group: &Group) -> AppResult<Group>;
    async fn delete(&self, id: &GroupId) -> AppResult<()>;
    async fn add_member(&self, group_id: &GroupId, member: &GroupMember) -> AppResult<()>;
    async fn remove_member(&self, group_id: &GroupId, user_id: &UserId) -> AppResult<()>;
    async fn update_member_role(
        &self,
        group_id: &GroupId,
        user_id: &UserId,
        role: GroupRole,
    ) -> AppResult<()>;
    async fn find_public_groups(&self, limit: i64, offset: i64) -> AppResult<Vec<Group>>;
}

pub struct PostgresGroupRepository {
    pool: PgPool,
}

impl PostgresGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for PostgresGroupRepository {
    async fn create(&self, group: &Group) -> AppResult<Group> {
        let mut tx = self.pool.begin().await?;

        let record = sqlx::query_as::<_, Group>(
            r#"
            INSERT INTO groups (id, name, description, avatar_url, owner_id, max_members, is_public, invite_link, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(group.id.as_uuid())
        .bind(&group.name)
        .bind(&group.description)
        .bind(&group.avatar_url)
        .bind(group.owner_id.as_uuid())
        .bind(group.max_members)
        .bind(group.is_public)
        .bind(&group.invite_link)
        .bind(group.created_at)
        .bind(group.updated_at)
        .fetch_one(&mut *tx)
        .await?;

        for member in &group.members {
            sqlx::query(
                r#"
                INSERT INTO group_members (group_id, user_id, role, nickname, muted_until, joined_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(group.id.as_uuid())
            .bind(member.user_id.as_uuid())
            .bind(member.role.to_string())
            .bind(&member.nickname)
            .bind(member.muted_until)
            .bind(member.joined_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(record)
    }

    async fn find_by_id(&self, id: &GroupId) -> AppResult<Option<Group>> {
        let mut record = sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(ref mut group) = record {
            let members =
                sqlx::query_as::<_, GroupMember>("SELECT * FROM group_members WHERE group_id = $1")
                    .bind(id.as_uuid())
                    .fetch_all(&self.pool)
                    .await?;

            group.members = members;
        }

        Ok(record)
    }

    async fn find_by_user(&self, user_id: &UserId) -> AppResult<Vec<Group>> {
        let records = sqlx::query_as::<_, Group>(
            r#"
            SELECT g.* 
            FROM groups g
            INNER JOIN group_members gm ON g.id = gm.group_id
            WHERE gm.user_id = $1
            ORDER BY g.updated_at DESC
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn update(&self, group: &Group) -> AppResult<Group> {
        let record = sqlx::query_as::<_, Group>(
            r#"
            UPDATE groups
            SET name = $2, description = $3, avatar_url = $4, max_members = $5, is_public = $6, invite_link = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(group.id.as_uuid())
        .bind(&group.name)
        .bind(&group.description)
        .bind(&group.avatar_url)
        .bind(group.max_members)
        .bind(group.is_public)
        .bind(&group.invite_link)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn delete(&self, id: &GroupId) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM groups WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Group not found".to_string()));
        }

        Ok(())
    }

    async fn add_member(&self, group_id: &GroupId, member: &GroupMember) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO group_members (group_id, user_id, role, nickname, muted_until, joined_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(group_id.as_uuid())
        .bind(member.user_id.as_uuid())
        .bind(member.role.to_string())
        .bind(&member.nickname)
        .bind(member.muted_until)
        .bind(member.joined_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_member(&self, group_id: &GroupId, user_id: &UserId) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id.as_uuid())
            .bind(user_id.as_uuid())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Group member not found".to_string()));
        }

        Ok(())
    }

    async fn update_member_role(
        &self,
        group_id: &GroupId,
        user_id: &UserId,
        role: GroupRole,
    ) -> AppResult<()> {
        let result =
            sqlx::query("UPDATE group_members SET role = $3 WHERE group_id = $1 AND user_id = $2")
                .bind(group_id.as_uuid())
                .bind(user_id.as_uuid())
                .bind(role.to_string())
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Group member not found".to_string()));
        }

        Ok(())
    }

    async fn find_public_groups(&self, limit: i64, offset: i64) -> AppResult<Vec<Group>> {
        let records = sqlx::query_as::<_, Group>(
            r#"
            SELECT * FROM groups
            WHERE is_public = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}
