# Chat-General 聊天后端架构设计

## 1. 概述

Chat-General 是一个基于 Axum 框架的高性能聊天后端服务，设计目标：

- **核心功能**：单对单聊天、群组聊天、多设备支持、历史消息同步
- **可扩展性**：作为库使用，支持自定义认证、消息处理、扩展开发

## 2. 系统架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Client Layer                               │
│         (Web/Mobile/Desktop/Third-party Services)                   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        API Gateway Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ HTTP Router  │  │ WS Handler   │  │ Middleware Pipeline      │  │
│  │  (Axum)      │  │ (WebSocket)  │  │ (Auth/RateLimit/Log)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Core Library Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Auth Module  │  │ Message Core │  │ Session Manager          │  │
│  │ (Trait-based)│  │ (Router/Store)│  │ (Device/Connection)      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Group Engine │  │ Event System │  │ Extension Points         │  │
│  │              │  │ (Pub/Sub)    │  │ (Hooks/Handlers)         │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Infrastructure Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ PostgreSQL   │  │ Redis        │  │ Message Queue            │  │
│  │ (Persistent) │  │ (Cache/Pub)  │  │ (Optional: NATS/Kafka)   │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## 3. 项目结构

```
chat-general/
├── Cargo.toml
├── ARCHITECTURE.md
├── src/
│   ├── lib.rs                    # 库入口，导出公共API
│   ├── main.rs                   # 可执行程序入口
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs           # 配置管理
│   │
│   ├── domain/                   # 领域模型（核心业务）
│   │   ├── mod.rs
│   │   ├── user.rs               # 用户实体
│   │   ├── device.rs             # 设备实体
│   │   ├── message.rs            # 消息实体
│   │   ├── conversation.rs       # 会话实体
│   │   └── group.rs              # 群组实体
│   │
│   ├── auth/                     # 认证模块（可扩展）
│   │   ├── mod.rs
│   │   ├── trait.rs              # AuthProvider trait
│   │   ├── jwt.rs                # JWT 实现
│   │   ├── oauth.rs              # OAuth 实现（示例）
│   │   └── api_key.rs            # API Key 实现（机器人用）
│   │
│   ├── session/                  # 会话管理
│   │   ├── mod.rs
│   │   ├── manager.rs            # 会话管理器
│   │   ├── device_registry.rs    # 设备注册
│   │   └── connection.rs         # 连接状态
│   │
│   ├── message/                  # 消息处理
│   │   ├── mod.rs
│   │   ├── router.rs             # 消息路由
│   │   ├── store.rs              # 消息存储 trait
│   │   ├── handler.rs            # 消息处理器 trait
│   │   └── history.rs            # 历史消息服务
│   │
│   ├── group/                    # 群组功能
│   │   ├── mod.rs
│   │   ├── manager.rs            # 群组管理
│   │   ├── membership.rs         # 成员管理
│   │   └── dispatcher.rs         # 群消息分发
│   │
│   ├── friend/                   # 好友系统
│   │   ├── mod.rs
│   │   ├── manager.rs            # 好友管理
│   │   ├── request.rs            # 好友请求处理
│   │   └── permission.rs         # 聊天权限控制
│   │
│   ├── event/                    # 事件系统
│   │   ├── mod.rs
│   │   ├── bus.rs                # 事件总线
│   │   ├── types.rs              # 事件类型定义
│   │   └── subscriber.rs         # 订阅者 trait
│   │
│   ├── extension/                # 扩展点
│   │   ├── mod.rs
│   │   ├── hook.rs               # 钩子 trait
│   │   ├── middleware.rs         # 消息中间件 trait
│   │   └── bot.rs                # 机器人集成接口
│   │
│   ├── api/                      # HTTP/WebSocket API
│   │   ├── mod.rs
│   │   ├── routes.rs             # 路由定义
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── message.rs
│   │   │   ├── group.rs
│   │   │   └── websocket.rs
│   │   └── dto/                  # 数据传输对象
│   │       ├── mod.rs
│   │       ├── request.rs
│   │       └── response.rs
│   │
│   ├── infra/                    # 基础设施实现
│   │   ├── mod.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── postgres.rs
│   │   │   └── repository.rs
│   │   ├── cache/
│   │   │   ├── mod.rs
│   │   │   └── redis.rs
│   │   └── queue/
│   │       ├── mod.rs
│   │       └── in_memory.rs
│   │
│   └── server/                   # 服务器组装
│       ├── mod.rs
│       ├── builder.rs            # 服务器构建器
│       └── state.rs              # 应用状态
│
└── tests/                        # 集成测试
    ├── integration/
    └── e2e/
```

## 4. 核心设计

### 4.1 认证模块（可扩展）

```rust
// src/auth/trait.rs

use async_trait::async_trait;
use axum::http::Request;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    type Claims: Send + Sync + Clone;
    
    async fn authenticate<B>(
        &self,
        request: &Request<B>,
    ) -> Result<Self::Claims, AuthError>;
    
    async fn validate_token(&self, token: &str) -> Result<Self::Claims, AuthError>;
    
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthError>;
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    Expired,
    #[error("Unauthorized")]
    Unauthorized,
}

// 用户可通过实现此 trait 来自定义认证方式
// 例如: LDAP, OAuth2, 自定义Token等
```

### 4.2 消息存储抽象

```rust
// src/message/store.rs

use async_trait::async_trait;
use crate::domain::{Message, ConversationId, UserId, DeviceId};
use chrono::{DateTime, Utc};

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn store(&self, message: &Message) -> Result<(), StoreError>;
    
    async fn get_by_id(&self, id: &str) -> Result<Option<Message>, StoreError>;
    
    async fn get_history(
        &self,
        conversation_id: &ConversationId,
        device_id: &DeviceId,
        before: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<Message>, StoreError>;
    
    async fn mark_delivered(
        &self,
        message_id: &str,
        device_id: &DeviceId,
    ) -> Result<(), StoreError>;
    
    async fn mark_read(
        &self,
        message_id: &str,
        user_id: &UserId,
    ) -> Result<(), StoreError>;
}

// 开发者可实现自己的存储后端
// 如: PostgreSQL, MongoDB, Cassandra 等
```

### 4.3 消息处理器（扩展点）

```rust
// src/message/handler.rs

use async_trait::async_trait;
use crate::domain::Message;
use crate::session::SessionContext;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn on_message(
        &self,
        message: &Message,
        context: &SessionContext,
    ) -> Result<HandlerAction, HandlerError>;
}

#[derive(Debug)]
pub enum HandlerAction {
    Continue,
    Modify(Message),
    Reject(String),
    Respond(Message),
}

// 用途示例:
// 1. 内容过滤/审核
// 2. 聊天机器人自动回复
// 3. 消息转换/加密
// 4. 第三方服务通知
```

### 4.4 事件系统

```rust
// src/event/types.rs

use serde::{Deserialize, Serialize};
use crate::domain::{UserId, Message, GroupId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    MessageReceived {
        message: Message,
    },
    MessageDelivered {
        message_id: String,
        to_user: UserId,
        to_device: String,
    },
    UserOnline {
        user_id: UserId,
        device_id: String,
    },
    UserOffline {
        user_id: UserId,
        device_id: String,
    },
    GroupCreated {
        group_id: GroupId,
        creator: UserId,
    },
    GroupMemberJoined {
        group_id: GroupId,
        user_id: UserId,
    },
    // 开发者可扩展更多事件类型
}

// src/event/subscriber.rs

#[async_trait]
pub trait EventSubscriber: Send + Sync {
    async fn on_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>>;
}

// 用途:
// 1. 推送通知服务
// 2. 消息同步到其他系统
// 3. 审计日志
// 4. 分析统计
```

### 4.5 设备与会话管理

```rust
// src/session/device_registry.rs

use std::collections::HashMap;
use crate::domain::{UserId, DeviceId};
use tokio::sync::RwLock;

pub struct DeviceInfo {
    pub device_id: DeviceId,
    pub device_type: DeviceType,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub connection: Option<WebSocketConnection>,
}

pub enum DeviceType {
    Web,
    Mobile,
    Desktop,
    Bot,
    ThirdParty,
}

pub struct DeviceRegistry {
    // user_id -> devices
    devices: RwLock<HashMap<UserId, Vec<DeviceInfo>>>,
}

impl DeviceRegistry {
    pub async fn register(&self, user_id: UserId, device: DeviceInfo) {
        // 注册设备
    }
    
    pub async fn get_devices(&self, user_id: &UserId) -> Vec<DeviceInfo> {
        // 获取用户所有设备
    }
    
    pub async fn get_online_devices(&self, user_id: &UserId) -> Vec<DeviceInfo> {
        // 获取在线设备
    }
    
    pub async fn push_to_user(&self, user_id: &UserId, message: &Message) {
        // 推送消息到用户所有在线设备
    }
    
    pub async fn push_to_device(&self, device_id: &DeviceId, message: &Message) {
        // 推送消息到特定设备
    }
}
```

### 4.6 历史消息同步

```rust
// src/message/history.rs

use crate::domain::{UserId, DeviceId, ConversationId, Message};
use crate::message::store::MessageStore;
use chrono::{DateTime, Utc};

pub struct HistoryService<S: MessageStore> {
    store: S,
}

impl<S: MessageStore> HistoryService<S> {
    /// 设备登录后拉取最近消息
    pub async fn sync_for_device(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        last_sync: DateTime<Utc>,
    ) -> Result<SyncResult, HistoryError> {
        // 1. 获取用户所有会话
        // 2. 拉取 last_sync 之后的消息
        // 3. 返回同步结果
    }
    
    /// 拉取特定会话的历史
    pub async fn get_conversation_history(
        &self,
        conversation_id: &ConversationId,
        device_id: &DeviceId,
        before: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<Message>, HistoryError> {
        self.store.get_history(conversation_id, device_id, before, limit).await
    }
}

pub struct SyncResult {
    pub conversations: Vec<ConversationSync>,
    pub has_more: bool,
}

pub struct ConversationSync {
    pub conversation_id: ConversationId,
    pub messages: Vec<Message>,
    pub last_message_id: String,
}
```

### 4.7 好友系统

好友系统管理用户之间的好友关系，并控制单聊权限。

#### 4.7.1 领域模型

```rust
// src/domain/friendship.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 好友关系状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FriendshipStatus {
    Pending,    // 待处理（请求已发送）
    Accepted,   // 已接受
    Blocked,    // 已拉黑
}

/// 好友请求
#[derive(Debug, Clone)]
pub struct FriendRequest {
    pub id: FriendRequestId,
    pub from_user: UserId,
    pub to_user: UserId,
    pub message: Option<String>,      // 请求附言
    pub status: FriendshipStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 好友关系
#[derive(Debug, Clone)]
pub struct Friendship {
    pub id: FriendshipId,
    pub user_id: UserId,
    pub friend_id: UserId,
    pub remark: Option<String>,       // 好友备注名
    pub created_at: DateTime<Utc>,
}

/// 好友请求 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FriendRequestId(pub Uuid);

/// 好友关系 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FriendshipId(pub Uuid);
```

#### 4.7.2 好友管理服务

```rust
// src/friend/manager.rs

use async_trait::async_trait;
use crate::domain::{UserId, FriendRequest, Friendship, FriendshipStatus};
use crate::error::AppResult;

#[async_trait]
pub trait FriendService: Send + Sync {
    /// 发送好友请求
    async fn send_request(
        &self,
        from: UserId,
        to: UserId,
        message: Option<String>,
    ) -> AppResult<FriendRequest>;
    
    /// 接受好友请求
    async fn accept_request(&self, request_id: &FriendRequestId) -> AppResult<Friendship>;
    
    /// 拒绝好友请求
    async fn reject_request(&self, request_id: &FriendRequestId) -> AppResult<()>;
    
    /// 删除好友
    async fn remove_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()>;
    
    /// 获取好友列表
    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>>;
    
    /// 获取收到的好友请求
    async fn get_pending_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    
    /// 获取已发送的好友请求
    async fn get_sent_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    
    /// 检查是否为好友
    async fn is_friend(&self, user_id: &UserId, other_id: &UserId) -> AppResult<bool>;
    
    /// 获取好友关系
    async fn get_friendship(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<Option<Friendship>>;
}

/// 好友管理器实现
pub struct FriendManager<R: FriendRepository> {
    repository: R,
    event_bus: EventBus,
}

impl<R: FriendRepository> FriendManager<R> {
    pub fn new(repository: R, event_bus: EventBus) -> Self {
        Self { repository, event_bus }
    }
}

#[async_trait]
impl<R: FriendRepository> FriendService for FriendManager<R> {
    async fn send_request(
        &self,
        from: UserId,
        to: UserId,
        message: Option<String>,
    ) -> AppResult<FriendRequest> {
        // 1. 检查是否已经是好友
        if self.is_friend(&from, &to).await? {
            return Err(AppError::Validation("Already friends".into()));
        }
        
        // 2. 检查是否已有待处理的请求
        if self.repository.has_pending_request(&from, &to).await? {
            return Err(AppError::Validation("Request already pending".into()));
        }
        
        // 3. 创建好友请求
        let request = FriendRequest::new(from, to, message);
        let saved = self.repository.create_request(&request).await?;
        
        // 4. 发布事件
        self.event_bus.publish(Event::FriendRequestReceived {
            request: saved.clone(),
        }).await;
        
        Ok(saved)
    }
    
    async fn accept_request(&self, request_id: &FriendRequestId) -> AppResult<Friendship> {
        // 1. 获取请求
        let request = self.repository.get_request(request_id).await?
            .ok_or_else(|| AppError::NotFound("Request not found".into()))?;
        
        // 2. 检查状态
        if request.status != FriendshipStatus::Pending {
            return Err(AppError::Validation("Request already processed".into()));
        }
        
        // 3. 更新请求状态
        self.repository.update_request_status(
            request_id,
            FriendshipStatus::Accepted,
        ).await?;
        
        // 4. 创建双向好友关系
        let friendship1 = Friendship::new(request.from_user, request.to_user);
        let friendship2 = Friendship::new(request.to_user, request.from_user);
        
        self.repository.create_friendship(&friendship1).await?;
        self.repository.create_friendship(&friendship2).await?;
        
        // 5. 发布事件
        self.event_bus.publish(Event::FriendRequestAccepted {
            friendship: friendship1.clone(),
        }).await;
        
        Ok(friendship1)
    }
    
    async fn remove_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()> {
        // 删除双向好友关系
        self.repository.delete_friendship(user_id, friend_id).await?;
        self.repository.delete_friendship(friend_id, user_id).await?;
        
        // 发布事件
        self.event_bus.publish(Event::FriendRemoved {
            user_id: *user_id,
            friend_id: *friend_id,
        }).await;
        
        Ok(())
    }
    
    // ... 其他方法实现
}
```

#### 4.7.3 聊天权限控制

```rust
// src/friend/permission.rs

use crate::domain::{UserId, ConversationType};
use crate::friend::FriendService;
use crate::error::AppResult;

/// 聊天权限检查器
pub struct ChatPermissionChecker<F: FriendService> {
    friend_service: F,
}

impl<F: FriendService> ChatPermissionChecker<F> {
    pub fn new(friend_service: F) -> Self {
        Self { friend_service }
    }
    
    /// 检查是否可以发起单聊
    pub async fn can_start_direct_chat(
        &self,
        user_id: &UserId,
        target_id: &UserId,
    ) -> AppResult<bool> {
        // 单聊必须是好友关系
        self.friend_service.is_friend(user_id, target_id).await
    }
    
    /// 检查是否可以发送消息
    pub async fn can_send_message(
        &self,
        sender_id: &UserId,
        conversation_type: ConversationType,
        participants: &[UserId],
    ) -> AppResult<bool> {
        match conversation_type {
            ConversationType::Direct => {
                // 单聊：检查好友关系
                if participants.len() != 2 {
                    return Ok(false);
                }
                let other = participants.iter()
                    .find(|p| *p != sender_id)
                    .ok_or_else(|| AppError::Validation("Invalid participants".into()))?;
                
                self.friend_service.is_friend(sender_id, other).await
            }
            ConversationType::Group => {
                // 群聊：检查是否为群成员（由群组模块处理）
                Ok(true)
            }
        }
    }
}
```

#### 4.7.4 好友仓储接口

```rust
// src/infra/db/friend_repository.rs

use async_trait::async_trait;
use crate::domain::{UserId, FriendRequestId, FriendshipId, FriendRequest, Friendship, FriendshipStatus};
use crate::error::AppResult;

#[async_trait]
pub trait FriendRepository: Send + Sync {
    // 好友请求
    async fn create_request(&self, request: &FriendRequest) -> AppResult<FriendRequest>;
    async fn get_request(&self, id: &FriendRequestId) -> AppResult<Option<FriendRequest>>;
    async fn update_request_status(&self, id: &FriendRequestId, status: FriendshipStatus) -> AppResult<()>;
    async fn get_pending_requests_for_user(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn get_sent_requests_by_user(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn has_pending_request(&self, from: &UserId, to: &UserId) -> AppResult<bool>;
    
    // 好友关系
    async fn create_friendship(&self, friendship: &Friendship) -> AppResult<Friendship>;
    async fn get_friendship(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<Option<Friendship>>;
    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>>;
    async fn delete_friendship(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()>;
    async fn is_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<bool>;
}
```

#### 4.7.5 数据库 Schema

```sql
-- 好友请求表
CREATE TABLE IF NOT EXISTS friend_requests (
    id UUID PRIMARY KEY,
    from_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_user_id, to_user_id)
);

-- 好友关系表
CREATE TABLE IF NOT EXISTS friendships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    remark VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, friend_id)
);

-- 索引
CREATE INDEX idx_friend_requests_to_user ON friend_requests(to_user_id, status);
CREATE INDEX idx_friend_requests_from_user ON friend_requests(from_user_id);
CREATE INDEX idx_friendships_user ON friendships(user_id);
CREATE INDEX idx_friendships_friend ON friendships(friend_id);

-- 触发器
CREATE TRIGGER update_friend_requests_updated_at
    BEFORE UPDATE ON friend_requests
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();
```

#### 4.7.6 事件类型扩展

```rust
// src/event/types.rs 扩展

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    // ... 现有事件
    
    // 好友系统事件
    FriendRequestReceived {
        request: FriendRequest,
    },
    FriendRequestAccepted {
        friendship: Friendship,
    },
    FriendRequestRejected {
        request_id: FriendRequestId,
    },
    FriendRemoved {
        user_id: UserId,
        friend_id: UserId,
    },
}
```

## 5. API 设计

### 5.1 HTTP REST API

```
POST   /api/v1/auth/login          # 登录
POST   /api/v1/auth/refresh        # 刷新Token
POST   /api/v1/auth/logout         # 登出

GET    /api/v1/users/me            # 当前用户信息
GET    /api/v1/users/me/devices    # 当前用户设备列表

GET    /api/v1/conversations       # 会话列表
POST   /api/v1/conversations       # 创建会话
GET    /api/v1/conversations/:id   # 会话详情
GET    /api/v1/conversations/:id/messages  # 历史消息

POST   /api/v1/groups              # 创建群组
GET    /api/v1/groups/:id          # 群组详情
PUT    /api/v1/groups/:id/members  # 添加成员
DELETE /api/v1/groups/:id/members/:uid  # 移除成员

# 好友系统
GET    /api/v1/friends             # 好友列表
DELETE /api/v1/friends/:uid        # 删除好友
GET    /api/v1/friends/requests    # 好友请求列表（收到的）
POST   /api/v1/friends/requests    # 发送好友请求
PUT    /api/v1/friends/requests/:id/accept   # 接受好友请求
DELETE /api/v1/friends/requests/:id/reject   # 拒绝好友请求
GET    /api/v1/friends/requests/sent         # 已发送的好友请求

# 扩展端点（供机器人/第三方服务使用）
POST   /api/v1/bot/send            # 机器人发送消息
POST   /api/v1/bot/webhook         # 注册Webhook
GET    /api/v1/notifications       # 获取通知
```

### 5.2 WebSocket 协议

```
连接: ws://host/ws?token=<access_token>&device_id=<device_id>

消息格式 (JSON):
{
    "type": "message" | "ack" | "typing" | "presence" | "sync",
    "payload": { ... },
    "seq": 123  // 序列号，用于确认
}

客户端 -> 服务器:
- message: 发送消息
- ack: 确认收到消息
- typing: 正在输入状态
- sync: 请求同步

服务器 -> 客户端:
- message: 新消息
- ack: 消息送达确认
- presence: 用户在线状态变化
- sync: 同步数据
```

## 6. 扩展机制

### 6.1 自定义认证

```rust
// 开发者实现自己的认证
pub struct LdapAuthProvider {
    ldap_url: String,
}

#[async_trait]
impl AuthProvider for LdapAuthProvider {
    type Claims = LdapClaims;
    
    async fn authenticate<B>(&self, request: &Request<B>) -> Result<Self::Claims, AuthError> {
        // LDAP 认证逻辑
    }
}

// 注册到服务器
let server = ChatServer::builder()
    .auth_provider(LdapAuthProvider::new("ldap://..."))
    .build();
```

### 6.2 消息处理器链

```rust
// 内容审核处理器
pub struct ContentModerationHandler {
    client: ModerationClient,
}

#[async_trait]
impl MessageHandler for ContentModerationHandler {
    async fn on_message(&self, message: &Message, _: &SessionContext) -> Result<HandlerAction, HandlerError> {
        if self.client.check(message.content()).await?.is_violation() {
            return Ok(HandlerAction::Reject("Content violation".into()));
        }
        Ok(HandlerAction::Continue)
    }
}

// 机器人处理器
pub struct BotHandler {
    bots: Vec<Box<dyn Bot>>,
}

#[async_trait]
impl MessageHandler for BotHandler {
    async fn on_message(&self, message: &Message, _: &SessionContext) -> Result<HandlerAction, HandlerError> {
        for bot in &self.bots {
            if bot.should_respond(message).await {
                let response = bot.process(message).await?;
                return Ok(HandlerAction::Respond(response));
            }
        }
        Ok(HandlerAction::Continue)
    }
}

// 注册处理器
let server = ChatServer::builder()
    .add_handler(ContentModerationHandler::new(client))
    .add_handler(BotHandler::new(bots))
    .build();
```

### 6.3 事件订阅

```rust
// 推送通知订阅者
pub struct PushNotificationSubscriber {
    fcm_client: FcmClient,
}

#[async_trait]
impl EventSubscriber for PushNotificationSubscriber {
    async fn on_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            Event::MessageReceived { message } if !message.is_from_me() => {
                self.fcm_client.send_notification(message).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

// 审计日志订阅者
pub struct AuditLogSubscriber {
    db: Database,
}

#[async_trait]
impl EventSubscriber for AuditLogSubscriber {
    async fn on_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        self.db.insert_audit_log(event).await
    }
}
```

### 6.4 机器人集成

```rust
// src/extension/bot.rs

#[async_trait]
pub trait Bot: Send + Sync {
    fn name(&self) -> &str;
    
    async fn should_respond(&self, message: &Message) -> bool;
    
    async fn process(&self, message: &Message) -> Result<Message, BotError>;
}

// 示例: 命令机器人
pub struct CommandBot {
    commands: HashMap<String, Box<dyn BotCommand>>,
}

impl CommandBot {
    pub fn register_command(&mut self, name: String, command: Box<dyn BotCommand>) {
        self.commands.insert(name, command);
    }
}

#[async_trait]
impl Bot for CommandBot {
    async fn should_respond(&self, message: &Message) -> bool {
        message.content().starts_with('/')
    }
    
    async fn process(&self, message: &Message) -> Result<Message, BotError> {
        let (cmd, args) = parse_command(message.content());
        if let Some(command) = self.commands.get(&cmd) {
            command.execute(args).await
        } else {
            Ok(Message::text("Unknown command"))
        }
    }
}
```

## 7. 服务器构建器

```rust
// src/server/builder.rs

pub struct ChatServerBuilder<A: AuthProvider, S: MessageStore> {
    auth: A,
    store: S,
    handlers: Vec<Box<dyn MessageHandler>>,
    subscribers: Vec<Box<dyn EventSubscriber>>,
    config: ServerConfig,
}

impl<A, S> ChatServerBuilder<A, S>
where
    A: AuthProvider,
    S: MessageStore,
{
    pub fn new(auth: A, store: S) -> Self {
        Self {
            auth,
            store,
            handlers: Vec::new(),
            subscribers: Vec::new(),
            config: ServerConfig::default(),
        }
    }
    
    pub fn add_handler(mut self, handler: impl MessageHandler + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }
    
    pub fn add_subscriber(mut self, subscriber: impl EventSubscriber + 'static) -> Self {
        self.subscribers.push(Box::new(subscriber));
        self
    }
    
    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn build(self) -> ChatServer {
        ChatServer::from_builder(self)
    }
}

// 使用示例
#[tokio::main]
async fn main() {
    let auth = JwtAuthProvider::new(secret);
    let store = PostgresMessageStore::new(db_pool).await;
    
    let server = ChatServer::builder(auth, store)
        .add_handler(ContentModerationHandler::new())
        .add_handler(BotHandler::new())
        .add_subscriber(PushNotificationSubscriber::new())
        .add_subscriber(AuditLogSubscriber::new())
        .config(ServerConfig {
            port: 8080,
            websocket_heartbeat: Duration::from_secs(30),
        })
        .build();
    
    server.run().await
}
```

## 8. 技术栈

| 组件 | 技术选型 | 说明 |
|------|---------|------|
| Web框架 | Axum 0.7 | 高性能、类型安全 |
| 异步运行时 | Tokio | Rust标准异步运行时 |
| 序列化 | serde + serde_json | JSON序列化 |
| 数据库 | SQLx + PostgreSQL | 类型安全的SQL |
| 缓存 | Redis | 会话、在线状态 |
| WebSocket | axum-extra/ws | 原生支持 |
| 认证 | jsonwebtoken | JWT实现 |
| 日志 | tracing + tracing-subscriber | 结构化日志 |
| 配置 | config-rs | 配置管理 |
| 错误处理 | thiserror + anyhow | 错误类型定义 |

## 9. 部署架构

```
                    ┌─────────────┐
                    │   Nginx     │
                    │ (LB/SSL)    │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
    │ Chat    │      │ Chat    │      │ Chat    │
    │ Server 1│      │ Server 2│      │ Server N│
    └────┬────┘      └────┬────┘      └────┬────┘
         │                │                 │
         └────────────────┼─────────────────┘
                          │
         ┌────────────────┼────────────────┐
         │                │                │
    ┌────▼────┐     ┌────▼────┐     ┌────▼────┐
    │PostgreSQL│    │  Redis  │     │  NATS   │
    │(Primary) │    │ Cluster │     │(Optional)│
    └──────────┘    └─────────┘     └─────────┘
```

## 10. 扩展场景示例

### 10.1 聊天机器人服务

```rust
// 作为独立服务接入
let bot_auth = ApiKeyAuthProvider::new(api_key);
let bot_store = InMemoryMessageStore::new();

let bot_server = ChatServer::builder(bot_auth, bot_store)
    .add_handler(MyBotHandler::new())
    .build();

// 或作为库嵌入
let client = ChatClient::new("ws://chat-server/ws")
    .with_api_key(api_key)
    .connect()
    .await;

client.on_message(|msg| async move {
    // 处理消息
}).await;
```

### 10.2 通知Channel

```rust
// 第三方服务发送通知
pub struct NotificationChannel {
    client: ChatClient,
}

impl NotificationChannel {
    pub async fn notify_user(&self, user_id: &UserId, notification: Notification) -> Result<()> {
        let message = Message::builder()
            .to(user_id)
            .content(notification.message)
            .metadata(notification.metadata)
            .build();
        
        self.client.send(message).await
    }
}

// 使用
let channel = NotificationChannel::new(client);
channel.notify_user(&user_id, Notification {
    message: "订单已发货".into(),
    metadata: json!({ "order_id": "12345" }),
}).await?;
```

## 11. 安全考虑

1. **认证授权**：JWT + Refresh Token 机制
2. **传输安全**：强制 TLS/WSS
3. **消息安全**：可选端到端加密支持
4. **速率限制**：基于 IP/用户的限流
5. **输入验证**：所有输入严格校验
6. **审计日志**：关键操作记录

## 12. 性能优化

1. **连接池**：数据库、Redis 连接池
2. **消息批处理**：批量写入优化
3. **缓存策略**：热点数据缓存
4. **水平扩展**：无状态设计，支持多实例
5. **消息压缩**：WebSocket 消息压缩
