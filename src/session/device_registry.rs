use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::domain::{DeviceId, DeviceType, UserId};
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: DeviceId,
    pub user_id: UserId,
    pub device_type: DeviceType,
    pub device_name: String,
    pub last_active: DateTime<Utc>,
    pub is_online: bool,
}

impl DeviceInfo {
    pub fn new(
        device_id: DeviceId,
        user_id: UserId,
        device_type: DeviceType,
        device_name: String,
    ) -> Self {
        Self {
            device_id,
            user_id,
            device_type,
            device_name,
            last_active: Utc::now(),
            is_online: true,
        }
    }

    pub fn update_last_active(&mut self) {
        self.last_active = Utc::now();
    }

    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_active = Utc::now();
        }
    }
}

#[derive(Clone)]
pub struct DeviceRegistry {
    devices: Arc<RwLock<HashMap<DeviceId, DeviceInfo>>>,
    user_devices: Arc<RwLock<HashMap<UserId, Vec<DeviceId>>>>,
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            user_devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, device: DeviceInfo) -> AppResult<()> {
        let device_id = device.device_id;
        let user_id = device.user_id;

        {
            let mut devices = self.devices.write().await;
            devices.insert(device_id, device);
        }

        {
            let mut user_devices = self.user_devices.write().await;
            user_devices
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(device_id);
        }

        Ok(())
    }

    pub async fn unregister(&self, device_id: &DeviceId) -> AppResult<Option<DeviceInfo>> {
        let device = {
            let mut devices = self.devices.write().await;
            devices.remove(device_id)
        };

        if let Some(ref info) = device {
            let mut user_devices = self.user_devices.write().await;
            if let Some(device_list) = user_devices.get_mut(&info.user_id) {
                device_list.retain(|id| id != device_id);
                if device_list.is_empty() {
                    user_devices.remove(&info.user_id);
                }
            }
        }

        Ok(device)
    }

    pub async fn get_device(&self, device_id: &DeviceId) -> Option<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    pub async fn get_user_devices(&self, user_id: &UserId) -> Vec<DeviceInfo> {
        let user_devices = self.user_devices.read().await;
        let devices = self.devices.read().await;

        match user_devices.get(user_id) {
            Some(device_ids) => device_ids
                .iter()
                .filter_map(|id| devices.get(id).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    pub async fn get_online_devices(&self, user_id: &UserId) -> Vec<DeviceInfo> {
        self.get_user_devices(user_id)
            .await
            .into_iter()
            .filter(|d| d.is_online)
            .collect()
    }

    pub async fn set_device_online(&self, device_id: &DeviceId, online: bool) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.set_online(online);
        }
        Ok(())
    }

    pub async fn update_last_active(&self, device_id: &DeviceId) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.update_last_active();
        }
        Ok(())
    }

    pub async fn is_user_online(&self, user_id: &UserId) -> bool {
        self.get_online_devices(user_id).await.iter().any(|d| d.is_online)
    }

    pub async fn get_online_users(&self) -> Vec<UserId> {
        let devices = self.devices.read().await;
        let mut online_users = std::collections::HashSet::new();
        
        for device in devices.values() {
            if device.is_online {
                online_users.insert(device.user_id);
            }
        }
        
        online_users.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_device() {
        let registry = DeviceRegistry::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();
        
        let device = DeviceInfo::new(
            device_id,
            user_id,
            DeviceType::Web,
            "Test Device".to_string(),
        );
        
        registry.register(device).await.unwrap();
        
        let retrieved = registry.get_device(&device_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().device_name, "Test Device");
    }

    #[tokio::test]
    async fn test_get_user_devices() {
        let registry = DeviceRegistry::new();
        let user_id = UserId::new();
        
        let device1 = DeviceInfo::new(
            DeviceId::new(),
            user_id,
            DeviceType::Web,
            "Web Device".to_string(),
        );
        let device2 = DeviceInfo::new(
            DeviceId::new(),
            user_id,
            DeviceType::Mobile,
            "Mobile Device".to_string(),
        );
        
        registry.register(device1).await.unwrap();
        registry.register(device2).await.unwrap();
        
        let devices = registry.get_user_devices(&user_id).await;
        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    async fn test_unregister_device() {
        let registry = DeviceRegistry::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();
        
        let device = DeviceInfo::new(
            device_id,
            user_id,
            DeviceType::Web,
            "Test Device".to_string(),
        );
        
        registry.register(device).await.unwrap();
        registry.unregister(&device_id).await.unwrap();
        
        let retrieved = registry.get_device(&device_id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_set_device_online() {
        let registry = DeviceRegistry::new();
        let user_id = UserId::new();
        let device_id = DeviceId::new();
        
        let mut device = DeviceInfo::new(
            device_id,
            user_id,
            DeviceType::Web,
            "Test Device".to_string(),
        );
        device.set_online(false);
        
        registry.register(device).await.unwrap();
        registry.set_device_online(&device_id, true).await.unwrap();
        
        let retrieved = registry.get_device(&device_id).await.unwrap();
        assert!(retrieved.is_online);
    }

    #[tokio::test]
    async fn test_is_user_online() {
        let registry = DeviceRegistry::new();
        let user_id = UserId::new();
        
        assert!(!registry.is_user_online(&user_id).await);
        
        let device = DeviceInfo::new(
            DeviceId::new(),
            user_id,
            DeviceType::Web,
            "Test Device".to_string(),
        );
        
        registry.register(device).await.unwrap();
        assert!(registry.is_user_online(&user_id).await);
    }
}
