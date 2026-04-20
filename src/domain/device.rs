use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct DeviceId(pub Uuid);

impl DeviceId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for DeviceId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<String> for DeviceId {
    fn from(s: String) -> Self {
        Self(Uuid::parse_str(&s).expect("Invalid DeviceId"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    #[default]
    Web,
    Mobile,
    Desktop,
    Bot,
    ThirdParty,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Web => write!(f, "web"),
            DeviceType::Mobile => write!(f, "mobile"),
            DeviceType::Desktop => write!(f, "desktop"),
            DeviceType::Bot => write!(f, "bot"),
            DeviceType::ThirdParty => write!(f, "third_party"),
        }
    }
}

impl std::str::FromStr for DeviceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(DeviceType::Web),
            "mobile" => Ok(DeviceType::Mobile),
            "desktop" => Ok(DeviceType::Desktop),
            "bot" => Ok(DeviceType::Bot),
            "third_party" => Ok(DeviceType::ThirdParty),
            _ => Err(format!("Invalid device type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: DeviceId,
    pub user_id: UserId,
    pub device_type: DeviceType,
    pub device_name: String,
    pub push_token: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Device {
    pub fn new(user_id: UserId, device_type: DeviceType, device_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: DeviceId::new(),
            user_id,
            device_type,
            device_name,
            push_token: None,
            last_active_at: now,
            created_at: now,
        }
    }

    pub fn with_push_token(mut self, token: String) -> Self {
        self.push_token = Some(token);
        self
    }

    pub fn update_last_active(&mut self) {
        self.last_active_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UserId;

    #[test]
    fn test_device_id_creation() {
        let id1 = DeviceId::new();
        let id2 = DeviceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_device_creation() {
        let user_id = UserId::new();
        let device = Device::new(user_id, DeviceType::Web, "Chrome Browser".to_string());

        assert_eq!(device.device_type, DeviceType::Web);
        assert_eq!(device.device_name, "Chrome Browser");
        assert!(device.push_token.is_none());
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(DeviceType::Web.to_string(), "web");
        assert_eq!(DeviceType::Mobile.to_string(), "mobile");
        assert_eq!(DeviceType::Desktop.to_string(), "desktop");
    }

    #[test]
    fn test_device_update_last_active() {
        let user_id = UserId::new();
        let mut device = Device::new(user_id, DeviceType::Web, "Test".to_string());
        let old_time = device.last_active_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        device.update_last_active();

        assert!(device.last_active_at > old_time);
    }

    #[test]
    fn test_device_type_from_str() {
        assert_eq!("web".parse::<DeviceType>().unwrap(), DeviceType::Web);
        assert_eq!("MOBILE".parse::<DeviceType>().unwrap(), DeviceType::Mobile);
    }
}
