use async_trait::async_trait;

use crate::domain::{Group, GroupId, GroupMember, GroupRole, UserId};
use crate::error::AppResult;
use crate::infra::GroupRepository;

#[async_trait]
pub trait GroupService: Send + Sync {
    async fn create_group(
        &self,
        name: String,
        owner_id: UserId,
        description: Option<String>,
    ) -> AppResult<Group>;
    async fn get_group(&self, id: &GroupId) -> AppResult<Option<Group>>;
    async fn add_member(
        &self,
        operator_id: &UserId,
        group_id: &GroupId,
        user_id: UserId,
    ) -> AppResult<GroupMember>;
    async fn remove_member(
        &self,
        operator_id: &UserId,
        group_id: &GroupId,
        user_id: &UserId,
    ) -> AppResult<()>;
    async fn update_member_role(
        &self,
        operator_id: &UserId,
        group_id: &GroupId,
        user_id: &UserId,
        role: GroupRole,
    ) -> AppResult<()>;
    async fn get_user_groups(&self, user_id: &UserId) -> AppResult<Vec<Group>>;
    async fn delete_group(&self, id: &GroupId) -> AppResult<()>;
}

pub struct GroupManager<R: GroupRepository> {
    repository: R,
}

impl<R: GroupRepository> GroupManager<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: GroupRepository + 'static> GroupService for GroupManager<R> {
    async fn create_group(
        &self,
        name: String,
        owner_id: UserId,
        description: Option<String>,
    ) -> AppResult<Group> {
        let mut group = Group::new(name, owner_id);
        if let Some(desc) = description {
            group = group.with_description(desc);
        }
        self.repository.create(&group).await
    }

    async fn get_group(&self, id: &GroupId) -> AppResult<Option<Group>> {
        self.repository.find_by_id(id).await
    }

    async fn add_member(
        &self,
        operator_id: &UserId,
        group_id: &GroupId,
        user_id: UserId,
    ) -> AppResult<GroupMember> {
        let mut group = self
            .repository
            .find_by_id(group_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Group not found".to_string()))?;

        let operator = group.get_member(operator_id).ok_or_else(|| {
            crate::error::AppError::Auth(crate::error::AuthError::PermissionDenied)
        })?;

        if !operator.is_admin() {
            return Err(crate::error::AppError::Auth(
                crate::error::AuthError::PermissionDenied,
            ));
        }

        group
            .add_member(user_id)
            .map_err(|e| crate::error::AppError::Conflict(e.to_string()))?;

        let member = GroupMember::member(user_id);
        self.repository.add_member(group_id, &member).await?;
        self.repository.update(&group).await?;

        Ok(member)
    }

    async fn remove_member(
        &self,
        operator_id: &UserId,
        group_id: &GroupId,
        user_id: &UserId,
    ) -> AppResult<()> {
        let mut group = self
            .repository
            .find_by_id(group_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Group not found".to_string()))?;

        let operator = group.get_member(operator_id).ok_or_else(|| {
            crate::error::AppError::Auth(crate::error::AuthError::PermissionDenied)
        })?;

        if !operator.is_admin() {
            return Err(crate::error::AppError::Auth(
                crate::error::AuthError::PermissionDenied,
            ));
        }

        group
            .remove_member(user_id)
            .map_err(|e| crate::error::AppError::Conflict(e.to_string()))?;

        self.repository.remove_member(group_id, user_id).await?;
        self.repository.update(&group).await?;

        Ok(())
    }

    async fn update_member_role(
        &self,
        operator_id: &UserId,
        group_id: &GroupId,
        user_id: &UserId,
        role: GroupRole,
    ) -> AppResult<()> {
        let mut group = self
            .repository
            .find_by_id(group_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Group not found".to_string()))?;

        let operator = group.get_member(operator_id).ok_or_else(|| {
            crate::error::AppError::Auth(crate::error::AuthError::PermissionDenied)
        })?;

        if !operator.is_owner() {
            return Err(crate::error::AppError::Auth(
                crate::error::AuthError::PermissionDenied,
            ));
        }

        if let Some(member) = group.get_member_mut(user_id) {
            member.role = role;
        } else {
            return Err(crate::error::AppError::NotFound(
                "Member not found".to_string(),
            ));
        }

        self.repository
            .update_member_role(group_id, user_id, role)
            .await?;
        self.repository.update(&group).await?;

        Ok(())
    }

    async fn get_user_groups(&self, user_id: &UserId) -> AppResult<Vec<Group>> {
        self.repository.find_by_user(user_id).await
    }

    async fn delete_group(&self, id: &GroupId) -> AppResult<()> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::InMemoryGroupRepository;

    #[tokio::test]
    async fn test_create_group() {
        let repo = InMemoryGroupRepository::new();
        let manager = GroupManager::new(repo);
        let owner_id = UserId::new();

        let group = manager
            .create_group("Test Group".to_string(), owner_id, None)
            .await
            .unwrap();

        assert_eq!(group.name, "Test Group");
        assert_eq!(group.owner_id, owner_id);
    }

    #[tokio::test]
    async fn test_add_member() {
        let repo = InMemoryGroupRepository::new();
        let manager = GroupManager::new(repo);
        let owner_id = UserId::new();
        let member_id = UserId::new();

        let group = manager
            .create_group("Test".to_string(), owner_id, None)
            .await
            .unwrap();
        manager
            .add_member(&owner_id, &group.id, member_id)
            .await
            .unwrap();

        let retrieved = manager.get_group(&group.id).await.unwrap().unwrap();
        assert_eq!(retrieved.member_count(), 2);
    }

    #[tokio::test]
    async fn test_remove_member() {
        let repo = InMemoryGroupRepository::new();
        let manager = GroupManager::new(repo);
        let owner_id = UserId::new();
        let member_id = UserId::new();

        let group = manager
            .create_group("Test".to_string(), owner_id, None)
            .await
            .unwrap();
        manager
            .add_member(&owner_id, &group.id, member_id)
            .await
            .unwrap();
        manager
            .remove_member(&owner_id, &group.id, &member_id)
            .await
            .unwrap();

        let retrieved = manager.get_group(&group.id).await.unwrap().unwrap();
        assert_eq!(retrieved.member_count(), 1);
    }

    #[tokio::test]
    async fn test_get_user_groups() {
        let repo = InMemoryGroupRepository::new();
        let manager = GroupManager::new(repo);
        let user_id = UserId::new();

        manager
            .create_group("Group 1".to_string(), user_id, None)
            .await
            .unwrap();
        manager
            .create_group("Group 2".to_string(), user_id, None)
            .await
            .unwrap();

        let groups = manager.get_user_groups(&user_id).await.unwrap();
        assert_eq!(groups.len(), 2);
    }
}
