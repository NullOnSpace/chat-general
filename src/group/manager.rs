use async_trait::async_trait;

use crate::domain::{Group, GroupId, GroupMember, GroupRole, UserId};
use crate::error::{AppError, AppResult};

#[async_trait]
pub trait GroupService: Send + Sync {
    async fn create_group(&self, name: String, owner_id: UserId) -> AppResult<Group>;
    async fn get_group(&self, id: &GroupId) -> AppResult<Option<Group>>;
    async fn add_member(&self, group_id: &GroupId, user_id: UserId) -> AppResult<GroupMember>;
    async fn remove_member(&self, group_id: &GroupId, user_id: &UserId) -> AppResult<()>;
    async fn update_member_role(
        &self,
        group_id: &GroupId,
        user_id: &UserId,
        role: GroupRole,
    ) -> AppResult<()>;
    async fn get_user_groups(&self, user_id: &UserId) -> AppResult<Vec<Group>>;
    async fn delete_group(&self, id: &GroupId) -> AppResult<()>;
}

pub struct GroupManager {
    groups: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<GroupId, Group>>>,
    user_groups:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<UserId, Vec<GroupId>>>>,
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            user_groups: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    pub async fn is_member(&self, group_id: &GroupId, user_id: &UserId) -> bool {
        let groups = self.groups.read().await;
        if let Some(group) = groups.get(group_id) {
            return group.is_member(user_id);
        }
        false
    }

    pub async fn is_admin(&self, group_id: &GroupId, user_id: &UserId) -> bool {
        let groups = self.groups.read().await;
        if let Some(group) = groups.get(group_id) {
            if let Some(member) = group.get_member(user_id) {
                return member.is_admin();
            }
        }
        false
    }

    pub async fn is_owner(&self, group_id: &GroupId, user_id: &UserId) -> bool {
        let groups = self.groups.read().await;
        if let Some(group) = groups.get(group_id) {
            return &group.owner_id == user_id;
        }
        false
    }
}

#[async_trait]
impl GroupService for GroupManager {
    async fn create_group(&self, name: String, owner_id: UserId) -> AppResult<Group> {
        let group = Group::new(name, owner_id);
        let group_id = group.id;

        {
            let mut groups = self.groups.write().await;
            groups.insert(group_id, group.clone());
        }

        {
            let mut user_groups = self.user_groups.write().await;
            user_groups
                .entry(owner_id)
                .or_insert_with(Vec::new)
                .push(group_id);
        }

        Ok(group)
    }

    async fn get_group(&self, id: &GroupId) -> AppResult<Option<Group>> {
        let groups = self.groups.read().await;
        Ok(groups.get(id).cloned())
    }

    async fn add_member(&self, group_id: &GroupId, user_id: UserId) -> AppResult<GroupMember> {
        let mut groups = self.groups.write().await;
        let group = groups
            .get_mut(group_id)
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        group
            .add_member(user_id)
            .map_err(|e| AppError::Conflict(e.to_string()))?;

        {
            let mut user_groups = self.user_groups.write().await;
            user_groups
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(*group_id);
        }

        Ok(GroupMember::member(user_id))
    }

    async fn remove_member(&self, group_id: &GroupId, user_id: &UserId) -> AppResult<()> {
        let mut groups = self.groups.write().await;
        let group = groups
            .get_mut(group_id)
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        group
            .remove_member(user_id)
            .map_err(|e| AppError::Conflict(e.to_string()))?;

        {
            let mut user_groups = self.user_groups.write().await;
            if let Some(group_list) = user_groups.get_mut(user_id) {
                group_list.retain(|id| id != group_id);
            }
        }

        Ok(())
    }

    async fn update_member_role(
        &self,
        group_id: &GroupId,
        user_id: &UserId,
        role: GroupRole,
    ) -> AppResult<()> {
        let mut groups = self.groups.write().await;
        let group = groups
            .get_mut(group_id)
            .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

        if let Some(member) = group.get_member_mut(user_id) {
            member.role = role;
        } else {
            return Err(AppError::NotFound("Member not found".to_string()));
        }

        Ok(())
    }

    async fn get_user_groups(&self, user_id: &UserId) -> AppResult<Vec<Group>> {
        let user_groups = self.user_groups.read().await;
        let groups = self.groups.read().await;

        let group_ids = user_groups.get(user_id).cloned().unwrap_or_default();
        let result = group_ids
            .iter()
            .filter_map(|id| groups.get(id).cloned())
            .collect();

        Ok(result)
    }

    async fn delete_group(&self, id: &GroupId) -> AppResult<()> {
        let group = {
            let mut groups = self.groups.write().await;
            groups.remove(id)
        };

        if let Some(group) = group {
            let mut user_groups = self.user_groups.write().await;
            for member in group.members {
                if let Some(group_list) = user_groups.get_mut(&member.user_id) {
                    group_list.retain(|gid| gid != id);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_group() {
        let manager = GroupManager::new();
        let owner_id = UserId::new();

        let group = manager
            .create_group("Test Group".to_string(), owner_id)
            .await
            .unwrap();

        assert_eq!(group.name, "Test Group");
        assert_eq!(group.owner_id, owner_id);
    }

    #[tokio::test]
    async fn test_add_member() {
        let manager = GroupManager::new();
        let owner_id = UserId::new();
        let member_id = UserId::new();

        let group = manager
            .create_group("Test".to_string(), owner_id)
            .await
            .unwrap();
        manager.add_member(&group.id, member_id).await.unwrap();

        let retrieved = manager.get_group(&group.id).await.unwrap().unwrap();
        assert_eq!(retrieved.member_count(), 2);
    }

    #[tokio::test]
    async fn test_remove_member() {
        let manager = GroupManager::new();
        let owner_id = UserId::new();
        let member_id = UserId::new();

        let group = manager
            .create_group("Test".to_string(), owner_id)
            .await
            .unwrap();
        manager.add_member(&group.id, member_id).await.unwrap();
        manager.remove_member(&group.id, &member_id).await.unwrap();

        let retrieved = manager.get_group(&group.id).await.unwrap().unwrap();
        assert_eq!(retrieved.member_count(), 1);
    }

    #[tokio::test]
    async fn test_get_user_groups() {
        let manager = GroupManager::new();
        let user_id = UserId::new();

        manager
            .create_group("Group 1".to_string(), user_id)
            .await
            .unwrap();
        manager
            .create_group("Group 2".to_string(), user_id)
            .await
            .unwrap();

        let groups = manager.get_user_groups(&user_id).await.unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[tokio::test]
    async fn test_is_admin() {
        let manager = GroupManager::new();
        let owner_id = UserId::new();
        let member_id = UserId::new();

        let group = manager
            .create_group("Test".to_string(), owner_id)
            .await
            .unwrap();
        manager.add_member(&group.id, member_id).await.unwrap();

        assert!(manager.is_admin(&group.id, &owner_id).await);
        assert!(!manager.is_admin(&group.id, &member_id).await);
    }
}
