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
│  │  (Axum)      │  │ (WebSocket)  │  │ (Auth/CORS/Trace)        │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Core Library Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Auth Module  │  │ Message Core │  │ Session Manager          │  │
│  │ (Trait-based)│  │ (Handler/Store)│  │ (Device/Connection)      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Group Engine │  │ Event System │  │ Friend System            │  │
│  │              │  │ (Pub/Sub)    │  │ (Request/Permission)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Infrastructure Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ PostgreSQL   │  │ Redis        │  │ In-Memory Store          │  │
│  │ (Persistent) │  │ (Cache/Pub)  │  │ (Development/Testing)    │  │
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
│   ├── config/                   # 配置管理
│   │   ├── mod.rs
│   │   ├── settings.rs           # 配置结构定义
│   │   └── logging.rs            # 日志配置
│   │
│   ├── domain/                   # 领域模型（核心业务）
│   │   ├── mod.rs
│   │   ├── user.rs               # 用户实体
│   │   ├── device.rs             # 设备实体
│   │   ├── message.rs            # 消息实体
│   │   ├── conversation.rs       # 会话实体
│   │   ├── group.rs              # 群组实体
│   │   └── friendship.rs         # 好友关系实体
│   │
│   ├── auth/                     # 认证模块（可扩展）
│   │   ├── mod.rs
│   │   ├── trait.rs              # AuthProvider trait
│   │   ├── jwt.rs                # JWT 实现
│   │   ├── api_key.rs            # API Key 实现（机器人用）
│   │   └── password.rs           # 密码哈希（Argon2）
│   │
│   ├── session/                  # 会话管理
│   │   ├── mod.rs
│   │   ├── manager.rs            # 会话管理器
│   │   └── device_registry.rs    # 设备注册
│   │
│   ├── message/                  # 消息处理
│   │   ├── mod.rs
│   │   ├── router.rs             # 消息路由
│   │   ├── store.rs              # 消息存储
│   │   └── handler.rs            # 消息处理器链
│   │
│   ├── group/                    # 群组功能
│   │   ├── mod.rs
│   │   └── manager.rs            # 群组管理服务
│   │
│   ├── friend/                   # 好友系统
│   │   ├── mod.rs
│   │   ├── manager.rs            # 好友管理服务
│   │   └── permission.rs         # 聊天权限控制
│   │
│   ├── event/                    # 事件系统
│   │   ├── mod.rs
│   │   ├── bus.rs                # 事件总线
│   │   └── types.rs              # 事件类型定义
│   │
│   ├── api/                      # HTTP/WebSocket API
│   │   ├── mod.rs
│   │   ├── routes.rs             # 路由定义（内联）
│   │   ├── auth_extractor.rs     # 认证提取器
│   │   ├── dto.rs                # 数据传输对象
│   │   ├── websocket.rs          # WebSocket 处理
│   │   └── handlers/             # 请求处理器
│   │       ├── mod.rs
│   │       ├── auth.rs
│   │       ├── message.rs
│   │       ├── group.rs
│   │       └── friend.rs
│   │
│   ├── infra/                    # 基础设施实现
│   │   ├── mod.rs
│   │   ├── user_store.rs         # 用户存储
│   │   ├── db/                   # 数据库仓储
│   │   │   ├── mod.rs
│   │   │   ├── user_repository.rs
│   │   │   ├── device_repository.rs
│   │   │   ├── message_repository.rs
│   │   │   ├── conversation_repository.rs
│   │   │   ├── group_repository.rs
│   │   │   └── friend_repository.rs
│   │   └── cache/                # 缓存层
│   │       ├── mod.rs
│   │       └── redis.rs
│   │
│   ├── server/                   # 服务器组装
│   │   └── mod.rs                # ChatServer, ServerBuilder
│   │
│   └── error.rs                  # 错误类型定义
│
├── migrations/                   # 数据库迁移
│   ├── 001_initial_schema.sql
│   └── 002_friend_system.sql
│
├── static/                       # Web 前端
│   ├── index.html
│   ├── chat.html
│   ├── friends.html
│   └── js/
│
└── tests/                        # 测试
    └── e2e/                      # 端到端测试
```

## 4. 核心设计

### 4.1 认证模块（可扩展）

```rust
// src/auth/trait.rs

use async_trait::async_trait;
use crate::domain::UserId;
use crate::error::AppResult;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    type Claims: Send + Sync + Clone + std::fmt::Debug;

    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims>;
    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims>;
    async fn generate_tokens(&self, user_id: &UserId) -> AppResult<TokenPair>;
    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair>;
    async fn revoke_token(&self, token_id: &str) -> AppResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: UserId,
    pub username: String,
    pub roles: Vec<String>,
}
```

**内置实现：**
- `JwtAuthProvider` - JWT 令牌认证
- `ApiKeyAuthProvider` - API Key 认证（适用于机器人/第三方服务）

### 4.2 消息处理器链

```rust
// src/message/handler.rs

use async_trait::async_trait;
use crate::domain::Message;
use crate::session::Session;
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub enum HandlerAction {
    Continue,           // 继续处理
    Modify(Message),    // 修改消息后继续
    Reject(String),     // 拒绝消息
    Respond(Message),   // 直接响应
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn on_message(&self, message: &Message, session: &Session) -> AppResult<HandlerAction>;
}

pub struct HandlerChain {
    handlers: Vec<Box<dyn MessageHandler>>,
}

impl HandlerChain {
    pub fn new() -> Self { ... }
    
    pub fn with_handler(mut self, handler: Box<dyn MessageHandler>) -> Self { ... }
    
    pub async fn process(&self, message: Message, session: &Session) -> AppResult<Message> { ... }
}
```

**内置处理器：**
- `LoggingHandler` - 消息日志记录
- `ContentFilterHandler` - 敏感词过滤

### 4.3 事件系统

```rust
// src/event/types.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    MessageReceived { message: Message },
    MessageDelivered { message_id: String, to_user: UserId, to_device: String },
    MessageRead { message_id: String, by_user: UserId },
    UserOnline { user_id: UserId, device_id: String },
    UserOffline { user_id: UserId, device_id: String },
    GroupCreated { group_id: GroupId, creator: UserId },
    GroupMemberJoined { group_id: GroupId, user_id: UserId },
    GroupMemberLeft { group_id: GroupId, user_id: UserId },
    TypingStart { conversation_id: String, user_id: UserId },
    TypingStop { conversation_id: String, user_id: UserId },
    FriendRequestReceived { request: FriendRequest },
    FriendRequestAccepted { friendship: Friendship },
    FriendRequestRejected { request_id: FriendRequestId },
    FriendRemoved { user_id: UserId, friend_id: UserId },
}
```

### 4.4 设备与会话管理

```rust
// src/session/device_registry.rs

pub struct DeviceInfo {
    pub device_id: DeviceId,
    pub device_type: DeviceType,
    pub last_active: DateTime<Utc>,
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
    devices: RwLock<HashMap<UserId, Vec<DeviceInfo>>>,
}

impl DeviceRegistry {
    pub async fn register(&self, user_id: UserId, device: DeviceInfo) { ... }
    pub async fn get_devices(&self, user_id: &UserId) -> Vec<DeviceInfo> { ... }
    pub async fn get_online_devices(&self, user_id: &UserId) -> Vec<DeviceInfo> { ... }
    pub async fn push_to_user(&self, user_id: &UserId, message: &Message) { ... }
    pub async fn push_to_device(&self, device_id: &DeviceId, message: &Message) { ... }
}
```

### 4.5 好友系统

好友系统管理用户之间的好友关系，并控制单聊权限。

#### 领域模型

```rust
// src/domain/friendship.rs

pub enum FriendshipStatus {
    Pending,    // 待处理
    Accepted,   // 已接受
    Blocked,    // 已拉黑
}

pub struct FriendRequest {
    pub id: FriendRequestId,
    pub from_user: UserId,
    pub to_user: UserId,
    pub message: Option<String>,
    pub status: FriendshipStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Friendship {
    pub id: FriendshipId,
    pub user_id: UserId,
    pub friend_id: UserId,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

#### 好友管理服务

```rust
// src/friend/manager.rs

#[async_trait]
pub trait FriendService: Send + Sync {
    async fn send_request(&self, from: UserId, to: UserId, message: Option<String>) -> AppResult<FriendRequest>;
    async fn accept_request(&self, request_id: &FriendRequestId) -> AppResult<Friendship>;
    async fn reject_request(&self, request_id: &FriendRequestId) -> AppResult<()>;
    async fn get_friends(&self, user_id: &UserId) -> AppResult<Vec<Friendship>>;
    async fn get_pending_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn get_sent_requests(&self, user_id: &UserId) -> AppResult<Vec<FriendRequest>>;
    async fn remove_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<()>;
    async fn is_friend(&self, user_id: &UserId, friend_id: &UserId) -> AppResult<bool>;
}
```

### 4.6 群组管理

```rust
// src/domain/group.rs

pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: UserId,
    pub max_members: u32,
    pub is_public: bool,
    pub invite_link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct GroupMember {
    pub group_id: GroupId,
    pub user_id: UserId,
    pub role: GroupRole,
    pub nickname: Option<String>,
    pub muted_until: Option<DateTime<Utc>>,
    pub joined_at: DateTime<Utc>,
}

pub enum GroupRole {
    Owner,
    Admin,
    Member,
}
```

### 4.7 错误处理

```rust
// src/error.rs

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
}

pub type AppResult<T> = Result<T, AppError>;
```

## 5. 数据库设计

### 5.1 核心表结构

#### 用户表 (users)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| username | VARCHAR(50) | 用户名，唯一 |
| email | VARCHAR(255) | 邮箱，唯一 |
| password_hash | VARCHAR(255) | 密码哈希 |
| display_name | VARCHAR(100) | 显示名称 |
| avatar_url | VARCHAR(500) | 头像 URL |
| status | VARCHAR(20) | 状态 (online/offline/away) |
| created_at | TIMESTAMPTZ | 创建时间 |
| updated_at | TIMESTAMPTZ | 更新时间 |

#### 设备表 (devices)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| user_id | UUID | 用户 ID，外键 |
| device_type | VARCHAR(20) | 设备类型 |
| device_name | VARCHAR(100) | 设备名称 |
| push_token | VARCHAR(500) | 推送令牌 |
| last_active_at | TIMESTAMPTZ | 最后活跃时间 |
| created_at | TIMESTAMPTZ | 创建时间 |

#### 会话表 (conversations)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| conversation_type | VARCHAR(20) | 类型 (private/group) |
| last_message_id | UUID | 最后消息 ID |
| last_message_at | TIMESTAMPTZ | 最后消息时间 |
| created_at | TIMESTAMPTZ | 创建时间 |
| updated_at | TIMESTAMPTZ | 更新时间 |

#### 消息表 (messages)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| conversation_id | UUID | 会话 ID，外键 |
| sender_id | UUID | 发送者 ID，外键 |
| message_type | VARCHAR(20) | 消息类型 |
| content | TEXT | 消息内容 |
| metadata | JSONB | 元数据 |
| status | VARCHAR(20) | 状态 |
| reply_to | UUID | 回复消息 ID |
| created_at | TIMESTAMPTZ | 创建时间 |
| updated_at | TIMESTAMPTZ | 更新时间 |

#### 群组表 (groups)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| name | VARCHAR(100) | 群组名称 |
| description | TEXT | 描述 |
| avatar_url | VARCHAR(500) | 头像 URL |
| owner_id | UUID | 群主 ID，外键 |
| max_members | INTEGER | 最大成员数 |
| is_public | BOOLEAN | 是否公开 |
| invite_link | VARCHAR(100) | 邀请链接 |
| created_at | TIMESTAMPTZ | 创建时间 |
| updated_at | TIMESTAMPTZ | 更新时间 |

#### 好友请求表 (friend_requests)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| from_user_id | UUID | 发送者 ID，外键 |
| to_user_id | UUID | 接收者 ID，外键 |
| message | TEXT | 请求附言 |
| status | VARCHAR(20) | 状态 (pending/accepted/rejected) |
| created_at | TIMESTAMPTZ | 创建时间 |
| updated_at | TIMESTAMPTZ | 更新时间 |

#### 好友关系表 (friendships)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| user_id | UUID | 用户 ID，外键 |
| friend_id | UUID | 好友 ID，外键 |
| remark | VARCHAR(100) | 好友备注 |
| created_at | TIMESTAMPTZ | 创建时间 |

### 5.2 索引设计

```sql
-- 用户索引
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);

-- 设备索引
CREATE INDEX idx_devices_user_id ON devices(user_id);
CREATE INDEX idx_devices_last_active ON devices(last_active_at);

-- 消息索引
CREATE INDEX idx_messages_conversation ON messages(conversation_id, created_at DESC);
CREATE INDEX idx_messages_sender ON messages(sender_id);
CREATE INDEX idx_messages_created ON messages(created_at DESC);

-- 好友索引
CREATE INDEX idx_friend_requests_to_user ON friend_requests(to_user_id, status);
CREATE INDEX idx_friendships_user ON friendships(user_id);
```

## 6. API 设计

### 6.1 RESTful API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/v1/auth/register | 用户注册 |
| POST | /api/v1/auth/login | 用户登录 |
| POST | /api/v1/auth/refresh | 刷新令牌 |
| POST | /api/v1/auth/logout | 登出 |
| GET | /api/v1/auth/me | 获取当前用户 |
| GET | /api/v1/users/me/devices | 获取用户设备 |
| GET | /api/v1/users/search | 搜索用户 |
| GET | /api/v1/conversations | 获取会话列表 |
| POST | /api/v1/conversations | 创建会话 |
| GET | /api/v1/conversations/{id} | 获取会话详情 |
| GET | /api/v1/conversations/{id}/messages | 获取消息历史 |
| POST | /api/v1/messages | 发送消息 |
| GET | /api/v1/groups | 获取用户群组 |
| POST | /api/v1/groups | 创建群组 |
| GET | /api/v1/groups/{id} | 获取群组详情 |
| GET | /api/v1/groups/{id}/members | 获取群组成员 |
| PUT | /api/v1/groups/{id}/members | 添加群组成员 |
| DELETE | /api/v1/groups/{id}/members/{uid} | 移除群组成员 |
| GET | /api/v1/friends | 获取好友列表 |
| DELETE | /api/v1/friends/{uid} | 删除好友 |
| GET | /api/v1/friends/requests | 获取好友请求 |
| POST | /api/v1/friends/requests | 发送好友请求 |
| GET | /api/v1/friends/requests/sent | 获取已发送请求 |
| PUT | /api/v1/friends/requests/{id}/accept | 接受好友请求 |
| DELETE | /api/v1/friends/requests/{id}/reject | 拒绝好友请求 |

### 6.2 WebSocket 协议

连接地址：`ws://host/ws?token=<jwt>&device_id=<uuid>`

#### 客户端消息

```json
// 发送消息
{ "type": "message", "conversation_id": "uuid", "content": "text", "seq": 1 }

// 消息确认
{ "type": "ack", "message_id": "uuid", "seq": 1 }

// 输入状态
{ "type": "typing", "conversation_id": "uuid", "is_typing": true }

// 同步消息
{ "type": "sync", "last_sync": "2024-01-01T00:00:00Z" }
```

#### 服务端消息

```json
// 连接成功
{ "type": "connected", "user_id": "uuid", "device_id": "uuid" }

// 新消息
{ "type": "message", "id": "uuid", "conversation_id": "uuid", "sender_id": "uuid", "content": "text", "created_at": "...", "seq": 1 }

// 消息确认
{ "type": "ack", "message_id": "uuid", "status": "sent", "seq": 1 }

// 用户状态
{ "type": "presence", "user_id": "uuid", "device_id": "uuid", "is_online": true }

// 错误
{ "type": "error", "code": 400, "message": "Invalid message format" }
```

## 7. 扩展点

### 7.1 自定义认证

实现 `AuthProvider` trait：

```rust
pub struct MyAuthProvider { ... }

#[async_trait]
impl AuthProvider for MyAuthProvider {
    type Claims = MyClaims;
    
    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims> { ... }
    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims> { ... }
    // ...
}
```

### 7.2 自定义消息处理器

实现 `MessageHandler` trait：

```rust
pub struct MyHandler { ... }

#[async_trait]
impl MessageHandler for MyHandler {
    async fn on_message(&self, message: &Message, session: &Session) -> AppResult<HandlerAction> {
        // 处理逻辑
        Ok(HandlerAction::Continue)
    }
}

// 注册处理器
let server = ChatServer::builder()
    .add_handler(Box::new(MyHandler::new()))
    .build()?;
```

### 7.3 事件订阅

```rust
use crate::event::{Event, EventBus};

let event_bus = EventBus::new();
event_bus.subscribe(|event: &Event| {
    match event {
        Event::MessageReceived { message } => { ... }
        Event::UserOnline { user_id, device_id } => { ... }
        _ => {}
    }
});
```

## 8. 部署架构

```
                    ┌─────────────┐
                    │   Nginx     │
                    │ (反向代理)   │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐       ┌────▼────┐       ┌────▼────┐
    │ Server  │       │ Server  │       │ Server  │
    │  Node 1 │       │  Node 2 │       │  Node 3 │
    └────┬────┘       └────┬────┘       └────┬────┘
         │                 │                 │
         └─────────────────┼─────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐       ┌────▼────┐       ┌────▼────┐
    │PostgreSQL│      │  Redis  │       │  Redis  │
    │ Primary │       │  Master │       │  Slave  │
    └─────────┘       └─────────┘       └─────────┘
```

## 9. 性能考虑

### 9.1 连接管理

- 使用 Tokio 异步运行时处理大量并发连接
- WebSocket 连接按用户分组，支持多设备同时在线
- 心跳检测自动清理断开的连接

### 9.2 消息存储

- 消息按会话分区存储
- 使用索引优化历史消息查询
- 支持消息分页加载

### 9.3 缓存策略

- Redis 缓存用户在线状态
- 缓存会话元数据减少数据库查询
- 支持消息预加载提升响应速度

## 10. 安全考虑

### 10.1 认证安全

- JWT 令牌签名验证
- 访问令牌短期有效（默认 1 小时）
- 刷新令牌长期有效（默认 7 天）
- 支持 Token 撤销

### 10.2 密码安全

- Argon2 密码哈希算法
- 随机盐值
- 可配置哈希强度

### 10.3 传输安全

- HTTPS 强制加密
- WebSocket Secure (WSS)
- CORS 配置

### 10.4 输入验证

- 请求数据验证（Garde）
- SQL 注入防护（SQLx 参数化查询）
- XSS 防护（内容转义）
