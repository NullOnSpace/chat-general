# Chat-General 二次开发指南

本文档面向希望基于 Chat-General 进行二次开发的开发者，介绍如何扩展认证方式、添加消息处理器、自定义事件订阅等。

## 目录

1. [架构概览](#架构概览)
2. [扩展认证方式](#扩展认证方式)
3. [消息处理器链](#消息处理器链)
4. [事件系统](#事件系统)
5. [自定义仓储实现](#自定义仓储实现)
6. [添加新的 API 端点](#添加新的-api-端点)
7. [WebSocket 消息类型扩展](#websocket-消息类型扩展)
8. [最佳实践](#最佳实践)

---

## 架构概览

Chat-General 采用分层架构设计：

```
┌─────────────────────────────────────────────────────────┐
│                      API Layer                          │
│  (HTTP Handlers, WebSocket, DTOs)                       │
├─────────────────────────────────────────────────────────┤
│                    Service Layer                        │
│  (SessionManager, GroupService, MessageRouter)          │
├─────────────────────────────────────────────────────────┤
│                    Domain Layer                         │
│  (User, Message, Conversation, Group, Device)           │
├─────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                    │
│  (Repositories, Cache, EventBus)                        │
└─────────────────────────────────────────────────────────┘
```

### 核心模块

| 模块 | 职责 |
|------|------|
| `domain` | 领域模型定义，包含业务逻辑 |
| `auth` | 认证抽象与实现（JWT、API Key） |
| `session` | 设备注册与会话管理 |
| `message` | 消息存储、处理链、路由 |
| `group` | 群组管理服务 |
| `event` | 事件总线与订阅机制 |
| `api` | HTTP/WebSocket 接口 |
| `infra` | 数据库仓储、Redis 缓存 |

---

## 扩展认证方式

Chat-General 通过 `AuthProvider` trait 支持多种认证方式。你可以实现自己的认证提供者。

### AuthProvider Trait

```rust
// src/auth/trait.rs
#[async_trait]
pub trait AuthProvider: Send + Sync {
    type Claims: Send + Sync + Clone + std::fmt::Debug;

    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims>;
    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims>;
    async fn generate_tokens(&self, user_id: &UserId) -> AppResult<TokenPair>;
    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair>;
    async fn revoke_token(&self, token_id: &str) -> AppResult<()>;
}
```

### 示例：实现 OAuth2 认证

```rust
// src/auth/oauth2.rs
use async_trait::async_trait;
use crate::auth::{AuthProvider, TokenPair};
use crate::domain::UserId;
use crate::error::AppResult;

pub struct OAuth2Provider {
    client_id: String,
    client_secret: String,
    issuer_url: String,
}

impl OAuth2Provider {
    pub fn new(client_id: String, client_secret: String, issuer_url: String) -> Self {
        Self { client_id, client_secret, issuer_url }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClaims {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
}

#[async_trait]
impl AuthProvider for OAuth2Provider {
    type Claims = OAuthClaims;

    async fn authenticate(&self, token: &str) -> AppResult<Self::Claims> {
        // 调用 OAuth2 提供商验证 token
        let response = reqwest::Client::new()
            .get(&format!("{}/userinfo", self.issuer_url))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| AppError::Auth(e.to_string()))?;

        let claims: OAuthClaims = response.json().await
            .map_err(|e| AppError::Auth(e.to_string()))?;

        Ok(claims)
    }

    async fn validate_token(&self, token: &str) -> AppResult<Self::Claims> {
        self.authenticate(token).await
    }

    async fn generate_tokens(&self, user_id: &UserId) -> AppResult<TokenPair> {
        // 生成内部 JWT token
        // ...
    }

    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair> {
        // 刷新 token
        // ...
    }

    async fn revoke_token(&self, token_id: &str) -> AppResult<()> {
        // 撤销 token
        Ok(())
    }
}
```

### 注册自定义认证提供者

```rust
// 在你的应用中使用
use chat_general::auth::OAuth2Provider;

let oauth_provider = OAuth2Provider::new(
    "your_client_id".to_string(),
    "your_client_secret".to_string(),
    "https://oauth.example.com".to_string(),
);

// 在 API 中使用
```

---

## 消息处理器链

消息处理器链允许你在消息处理过程中插入自定义逻辑，如内容审核、敏感词过滤、消息转换等。

### MessageHandler Trait

```rust
// src/message/handler.rs
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn on_message(
        &self,
        message: &Message,
        session: &Session,
    ) -> AppResult<HandlerAction>;
}

pub enum HandlerAction {
    Continue,           // 继续处理
    Modify(Message),    // 修改消息后继续
    Reject(String),     // 拒绝消息
    Respond(Message),   // 直接响应
}
```

### 示例：实现翻译处理器

```rust
// src/message/handlers/translation.rs
use async_trait::async_trait;
use crate::message::{MessageHandler, HandlerAction};
use crate::domain::Message;
use crate::session::Session;
use crate::error::AppResult;

pub struct TranslationHandler {
    api_key: String,
    target_lang: String,
}

impl TranslationHandler {
    pub fn new(api_key: String, target_lang: String) -> Self {
        Self { api_key, target_lang }
    }
}

#[async_trait]
impl MessageHandler for TranslationHandler {
    async fn on_message(
        &self,
        message: &Message,
        _session: &Session,
    ) -> AppResult<HandlerAction> {
        // 调用翻译 API
        let translated = self.translate(&message.content).await?;
        
        // 创建带有翻译的消息
        let mut metadata = message.metadata.clone();
        metadata["translation"] = json!(translated);
        metadata["translated_to"] = json!(self.target_lang);
        
        let mut modified = message.clone();
        modified.metadata = metadata;
        
        Ok(HandlerAction::Modify(modified))
    }
}

impl TranslationHandler {
    async fn translate(&self, text: &str) -> AppResult<String> {
        // 实现翻译逻辑
        Ok(text.to_string())
    }
}
```

### 示例：实现速率限制处理器

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

pub struct RateLimitHandler {
    limits: Arc<RwLock<HashMap<UserId, u32>>>,
    max_per_minute: u32,
}

impl RateLimitHandler {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_per_minute,
        }
    }
}

#[async_trait]
impl MessageHandler for RateLimitHandler {
    async fn on_message(
        &self,
        message: &Message,
        _session: &Session,
    ) -> AppResult<HandlerAction> {
        let mut limits = self.limits.write().await;
        let count = limits.entry(message.sender_id).or_insert(0);
        
        if *count >= self.max_per_minute {
            return Ok(HandlerAction::Reject(
                "Rate limit exceeded. Please wait.".to_string()
            ));
        }
        
        *count += 1;
        Ok(HandlerAction::Continue)
    }
}
```

### 注册处理器

```rust
use chat_general::{ChatServer, message::{LoggingHandler, ContentFilterHandler}};

let server = ChatServer::builder()
    .settings(settings)
    .add_handler(Box::new(LoggingHandler))
    .add_handler(Box::new(ContentFilterHandler::new(vec![
        "spam".to_string(),
        "advertisement".to_string(),
    ])))
    .add_handler(Box::new(RateLimitHandler::new(60)))
    .build()?;
```

---

## 事件系统

事件系统允许你订阅和响应系统中的各种事件。

### 内置事件类型

```rust
// src/event/types.rs
pub enum Event {
    UserOnline(UserId),
    UserOffline(UserId),
    MessageSent(Message),
    MessageDelivered { message_id: MessageId, user_id: UserId },
    MessageRead { message_id: MessageId, user_id: UserId },
    GroupCreated(Group),
    GroupMemberJoined { group_id: GroupId, user_id: UserId },
    GroupMemberLeft { group_id: GroupId, user_id: UserId },
}
```

### 实现事件订阅者

```rust
use async_trait::async_trait;
use crate::event::{Event, EventSubscriber};

pub struct NotificationSubscriber;

#[async_trait]
impl EventSubscriber for NotificationSubscriber {
    async fn on_event(&self, event: &Event) {
        match event {
            Event::MessageSent(msg) => {
                // 发送推送通知
                self.send_push_notification(msg).await;
            }
            Event::UserOnline(user_id) => {
                tracing::info!("User {} is now online", user_id);
            }
            _ => {}
        }
    }
}

impl NotificationSubscriber {
    async fn send_push_notification(&self, message: &Message) {
        // 实现推送通知逻辑
    }
}
```

### 注册订阅者

```rust
use chat_general::event::EventBus;

let event_bus = EventBus::new()
    .subscribe(Box::new(NotificationSubscriber))
    .subscribe(Box::new(AnalyticsSubscriber));

let server = ChatServer::builder()
    .event_bus(event_bus)
    .build()?;
```

---

## 自定义仓储实现

你可以为不同的存储后端实现自定义仓储。

### 仓储 Trait

```rust
// src/infra/db/user_repository.rs
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> AppResult<User>;
    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn update(&self, user: &User) -> AppResult<User>;
    async fn delete(&self, id: &UserId) -> AppResult<()>;
}
```

### 示例：MySQL 仓储实现

```rust
use async_trait::async_trait;
use sqlx::MySqlPool;

pub struct MySqlUserRepository {
    pool: MySqlPool,
}

impl MySqlUserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for MySqlUserRepository {
    async fn create(&self, user: &User) -> AppResult<User> {
        sqlx::query!(
            r#"INSERT INTO users (id, username, email, password_hash, display_name, status)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            user.id.to_string(),
            user.username,
            user.email,
            user.password_hash,
            user.display_name,
            user.status.to_string(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(user.clone())
    }

    // 实现其他方法...
}
```

---

## 添加新的 API 端点

### 1. 定义 DTO

```rust
// src/api/dto.rs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBotRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub webhook_url: String,
}

#[derive(Debug, Serialize)]
pub struct BotResponse {
    pub id: String,
    pub name: String,
    pub webhook_url: String,
    pub created_at: String,
}
```

### 2. 实现处理器

```rust
// src/api/handlers/bot.rs
use axum::{extract::State, Json};
use crate::api::{AppState, dto::*};
use crate::error::AppResult;

pub async fn create_bot(
    State(state): State<AppState>,
    Json(req): Json<CreateBotRequest>,
) -> AppResult<Json<BotResponse>> {
    // 实现创建逻辑
    Ok(Json(BotResponse {
        id: "bot_id".to_string(),
        name: req.name,
        webhook_url: req.webhook_url,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn list_bots(
    State(_state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "bots": []
    })))
}
```

### 3. 注册路由

```rust
// src/api/mod.rs
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // ... 现有路由
        .route("/api/v1/bots", post(handlers::bot::create_bot))
        .route("/api/v1/bots", get(handlers::bot::list_bots))
}
```

---

## WebSocket 消息类型扩展

### 添加新的消息类型

```rust
// src/api/websocket.rs

// 1. 定义新的消息类型
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    SendMessage { payload: SendMessagePayload },
    Typing { payload: TypingPayload },
    MarkRead { payload: MarkReadPayload },
    // 添加新类型
    CreatePoll { payload: CreatePollPayload },
    VotePoll { payload: VotePollPayload },
}

// 2. 在处理函数中处理新类型
async fn handle_message(msg: WsMessage, state: &AppState, sender: &Sender) {
    match msg {
        WsMessage::CreatePoll { payload } => {
            // 处理创建投票
        }
        WsMessage::VotePoll { payload } => {
            // 处理投票
        }
        // ...
    }
}
```

---

## 最佳实践

### 1. 错误处理

使用 `AppResult<T>` 作为函数返回类型：

```rust
pub async fn my_function() -> AppResult<String> {
    let result = some_operation()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(result)
}
```

### 2. 测试

为每个模块编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_feature() {
        // 测试代码
    }
}
```

### 3. 日志

使用 `tracing` 进行日志记录：

```rust
tracing::info!("User {} logged in", user_id);
tracing::debug!("Processing message: {:?}", message);
tracing::error!("Failed to connect: {}", error);
```

### 4. 配置

使用环境变量进行配置，支持 `CHAT__` 前缀：

```bash
CHAT__SERVER__PORT=3000
CHAT__DATABASE__HOST=db.example.com
```

### 5. 性能优化

- 使用连接池管理数据库连接
- 使用 Redis 缓存热点数据
- 异步处理耗时操作
- 批量处理消息发送

---

## 调试与监控

### 启用调试日志

```bash
RUST_LOG=chat_general=debug,tower_http=debug cargo run
```

### 健康检查

```bash
curl http://localhost:8080/api/v1/health
```

### 性能分析

```bash
# 使用 cargo flamegraph 生成火焰图
cargo flamegraph --root
```

---

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 为新功能编写测试
- 更新相关文档
