use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct GroupId(pub Uuid);

impl GroupId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s).map(Self)
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for GroupId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl TryFrom<String> for GroupId {
    type Error = uuid::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Uuid::parse_str(&s).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GroupRole {
    Owner,
    Admin,
    #[default]
    Member,
}

impl std::fmt::Display for GroupRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupRole::Owner => write!(f, "owner"),
            GroupRole::Admin => write!(f, "admin"),
            GroupRole::Member => write!(f, "member"),
        }
    }
}

impl std::str::FromStr for GroupRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(GroupRole::Owner),
            "admin" => Ok(GroupRole::Admin),
            "member" => Ok(GroupRole::Member),
            _ => Err(format!("Invalid group role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub user_id: UserId,
    pub role: GroupRole,
    pub nickname: Option<String>,
    pub muted_until: Option<DateTime<Utc>>,
    pub joined_at: DateTime<Utc>,
}

impl GroupMember {
    pub fn new(user_id: UserId, role: GroupRole) -> Self {
        Self {
            user_id,
            role,
            nickname: None,
            muted_until: None,
            joined_at: Utc::now(),
        }
    }

    pub fn owner(user_id: UserId) -> Self {
        Self::new(user_id, GroupRole::Owner)
    }

    pub fn admin(user_id: UserId) -> Self {
        Self::new(user_id, GroupRole::Admin)
    }

    pub fn member(user_id: UserId) -> Self {
        Self::new(user_id, GroupRole::Member)
    }

    pub fn with_nickname(mut self, nickname: String) -> Self {
        self.nickname = Some(nickname);
        self
    }

    pub fn is_owner(&self) -> bool {
        self.role == GroupRole::Owner
    }

    pub fn is_admin(&self) -> bool {
        self.role == GroupRole::Admin || self.role == GroupRole::Owner
    }

    pub fn is_muted(&self) -> bool {
        self.muted_until
            .map(|until| until > Utc::now())
            .unwrap_or(false)
    }

    pub fn mute_until(&mut self, until: DateTime<Utc>) {
        self.muted_until = Some(until);
    }

    pub fn unmute(&mut self) {
        self.muted_until = None;
    }

    pub fn promote_to_admin(&mut self) {
        self.role = GroupRole::Admin;
    }

    pub fn demote_to_member(&mut self) {
        self.role = GroupRole::Member;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: UserId,
    pub max_members: i32,
    pub is_public: bool,
    pub invite_link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub members: Vec<GroupMember>,
}

impl Group {
    pub fn new(name: String, owner_id: UserId) -> Self {
        let now = Utc::now();
        let owner_member = GroupMember::owner(owner_id);
        Self {
            id: GroupId::new(),
            name,
            description: None,
            avatar_url: None,
            owner_id,
            members: vec![owner_member],
            max_members: 500,
            is_public: false,
            invite_link: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_avatar(mut self, url: String) -> Self {
        self.avatar_url = Some(url);
        self
    }

    pub fn public(mut self) -> Self {
        self.is_public = true;
        self
    }

    pub fn with_max_members(mut self, max: u32) -> Self {
        self.max_members = max as i32;
        self
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_full(&self) -> bool {
        self.members.len() >= self.max_members as usize
    }

    pub fn is_member(&self, user_id: &UserId) -> bool {
        self.members.iter().any(|m| &m.user_id == user_id)
    }

    pub fn get_member(&self, user_id: &UserId) -> Option<&GroupMember> {
        self.members.iter().find(|m| &m.user_id == user_id)
    }

    pub fn get_member_mut(&mut self, user_id: &UserId) -> Option<&mut GroupMember> {
        self.members.iter_mut().find(|m| &m.user_id == user_id)
    }

    pub fn add_member(&mut self, user_id: UserId) -> Result<(), GroupError> {
        if self.is_full() {
            return Err(GroupError::GroupFull);
        }
        if self.is_member(&user_id) {
            return Err(GroupError::AlreadyMember);
        }
        self.members.push(GroupMember::member(user_id));
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_member(&mut self, user_id: &UserId) -> Result<GroupMember, GroupError> {
        if &self.owner_id == user_id {
            return Err(GroupError::CannotRemoveOwner);
        }
        let pos = self
            .members
            .iter()
            .position(|m| &m.user_id == user_id)
            .ok_or(GroupError::NotMember)?;
        let member = self.members.remove(pos);
        self.updated_at = Utc::now();
        Ok(member)
    }

    pub fn transfer_ownership(&mut self, new_owner_id: &UserId) -> Result<(), GroupError> {
        if !self.is_member(new_owner_id) {
            return Err(GroupError::NotMember);
        }

        let old_owner_id = self.owner_id;
        self.owner_id = *new_owner_id;

        if let Some(old_owner) = self.get_member_mut(&old_owner_id) {
            old_owner.role = GroupRole::Admin;
        }
        if let Some(new_owner) = self.get_member_mut(new_owner_id) {
            new_owner.role = GroupRole::Owner;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn generate_invite_link(&mut self) -> String {
        let link = format!("invite_{}", Uuid::new_v4());
        self.invite_link = Some(link.clone());
        self.updated_at = Utc::now();
        link
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum GroupError {
    #[error("Group is full")]
    GroupFull,
    #[error("User is already a member")]
    AlreadyMember,
    #[error("User is not a member")]
    NotMember,
    #[error("Cannot remove the owner")]
    CannotRemoveOwner,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("User is muted")]
    UserMuted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_id_creation() {
        let id1 = GroupId::new();
        let id2 = GroupId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_group_creation() {
        let owner = UserId::new();
        let group = Group::new("Test Group".to_string(), owner);

        assert_eq!(group.name, "Test Group");
        assert_eq!(group.owner_id, owner);
        assert_eq!(group.member_count(), 1);
        assert!(!group.is_public);
    }

    #[test]
    fn test_group_member_management() {
        let owner = UserId::new();
        let member1 = UserId::new();
        let member2 = UserId::new();
        let mut group = Group::new("Test".to_string(), owner);

        assert!(group.add_member(member1).is_ok());
        assert_eq!(group.member_count(), 2);

        assert!(group.add_member(member2).is_ok());
        assert_eq!(group.member_count(), 3);

        assert!(group.is_member(&member1));
        assert!(group.is_member(&member2));
    }

    #[test]
    fn test_cannot_add_duplicate_member() {
        let owner = UserId::new();
        let member = UserId::new();
        let mut group = Group::new("Test".to_string(), owner);

        assert!(group.add_member(member).is_ok());
        assert!(matches!(
            group.add_member(member),
            Err(GroupError::AlreadyMember)
        ));
    }

    #[test]
    fn test_remove_member() {
        let owner = UserId::new();
        let member = UserId::new();
        let mut group = Group::new("Test".to_string(), owner);

        group.add_member(member).unwrap();
        assert_eq!(group.member_count(), 2);

        let removed = group.remove_member(&member).unwrap();
        assert_eq!(removed.user_id, member);
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_cannot_remove_owner() {
        let owner = UserId::new();
        let mut group = Group::new("Test".to_string(), owner);

        assert!(matches!(
            group.remove_member(&owner),
            Err(GroupError::CannotRemoveOwner)
        ));
    }

    #[test]
    fn test_transfer_ownership() {
        let owner = UserId::new();
        let new_owner = UserId::new();
        let mut group = Group::new("Test".to_string(), owner);

        group.add_member(new_owner).unwrap();
        group.transfer_ownership(&new_owner).unwrap();

        assert_eq!(group.owner_id, new_owner);
        assert!(group.get_member(&owner).unwrap().role == GroupRole::Admin);
        assert!(group.get_member(&new_owner).unwrap().is_owner());
    }

    #[test]
    fn test_group_member_roles() {
        let user = UserId::new();

        let owner = GroupMember::owner(user);
        assert!(owner.is_owner());
        assert!(owner.is_admin());

        let admin = GroupMember::admin(user);
        assert!(!admin.is_owner());
        assert!(admin.is_admin());

        let member = GroupMember::member(user);
        assert!(!member.is_owner());
        assert!(!member.is_admin());
    }

    #[test]
    fn test_group_full() {
        let owner = UserId::new();
        let mut group = Group::new("Test".to_string(), owner).with_max_members(2);

        let member1 = UserId::new();
        assert!(group.add_member(member1).is_ok());
        assert!(group.is_full());

        let member2 = UserId::new();
        assert!(matches!(
            group.add_member(member2),
            Err(GroupError::GroupFull)
        ));
    }

    #[test]
    fn test_member_mute() {
        let user = UserId::new();
        let mut member = GroupMember::member(user);

        assert!(!member.is_muted());

        member.mute_until(Utc::now() + chrono::Duration::hours(1));
        assert!(member.is_muted());

        member.unmute();
        assert!(!member.is_muted());
    }

    #[test]
    fn test_group_role_from_str() {
        assert_eq!("owner".parse::<GroupRole>().unwrap(), GroupRole::Owner);
        assert_eq!("ADMIN".parse::<GroupRole>().unwrap(), GroupRole::Admin);
        assert_eq!("member".parse::<GroupRole>().unwrap(), GroupRole::Member);
    }
}
