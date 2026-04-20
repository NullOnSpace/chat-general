# Chat-General

一个基于 Axum 框架的可扩展聊天后端服务框架，支持单聊、群聊、多设备登录和设备历史消息同步。

## 特性

- **单聊与群聊**：支持一对一私聊和多人群组聊天
- **好友系统**：完整的好友关系管理，单聊仅限好友之间
- **多设备支持**：用户可在多个设备同时登录，按设备拉取历史消息
- **实时通信**：基于 WebSocket 的实时消息推送
- **可扩展认证**：支持 JWT、API Key 等多种认证方式，可自定义扩展
- **事件驱动**：内置事件总线，支持消息、用户、群组等事件订阅
- **消息处理器链**：可插拔的消息处理中间件（过滤、审核、日志等）

## 技术栈

- **Web 框架**：Axum 0.7
- **数据库**：PostgreSQL + SQLx
- **缓存**：Redis
- **认证**：JWT (jsonwebtoken)
- **密码哈希**：Argon2
- **序列化**：Serde
- **异步运行时**：Tokio

## 快速开始

### 环境要求

- Rust 1.75+
- PostgreSQL 14+
- Redis 6+

### 安装与运行

1. **克隆项目**

```bash
git clone https://github.com/your-org/chat-general.git
cd chat-general
```

2. **配置环境变量**

```bash
cp .env.example .env
# 编辑 .env 文件，填入你的配置
```

3. **创建数据库**

```bash
# 创建 PostgreSQL 数据库
createdb chat_general

# 运行迁移
psql -d chat_general -f migrations/001_initial_schema.sql
```

4. **启动服务**

```bash
cargo run --release
```

服务将在 `http://0.0.0.0:8080` 启动。

### 配置说明

配置可通过环境变量或配置文件进行，环境变量优先级更高。

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `CHAT__SERVER__HOST` | 服务监听地址 | `0.0.0.0` |
| `CHAT__SERVER__PORT` | 服务监听端口 | `8080` |
| `CHAT__DATABASE__HOST` | 数据库地址 | `localhost` |
| `CHAT__DATABASE__PORT` | 数据库端口 | `5432` |
| `CHAT__DATABASE__USERNAME` | 数据库用户名 | `postgres` |
| `CHAT__DATABASE__PASSWORD` | 数据库密码 | - |
| `CHAT__DATABASE__DATABASE` | 数据库名称 | `chat_general` |
| `CHAT__REDIS__HOST` | Redis 地址 | `localhost` |
| `CHAT__REDIS__PORT` | Redis 端口 | `6379` |
| `CHAT__JWT__SECRET` | JWT 密钥 | - |
| `CHAT__JWT__ACCESS_TOKEN_EXPIRY` | 访问令牌有效期（秒） | `3600` |
| `CHAT__JWT__REFRESH_TOKEN_EXPIRY` | 刷新令牌有效期（秒） | `604800` |

## API 文档

### 认证接口

#### 用户注册

```http
POST /api/v1/auth/register
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```

#### 用户登录

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "testuser",
  "password": "password123"
}
```

响应：

```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "username": "testuser",
    "email": "test@example.com"
  }
}
```

#### 刷新令牌

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refresh_token": "your_refresh_token"
}
```

### 会话接口

#### 获取会话列表

```http
GET /api/v1/conversations
Authorization: Bearer <access_token>
```

#### 创建会话

```http
POST /api/v1/conversations
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "participant_ids": ["user_uuid_1", "user_uuid_2"]
}
```

#### 获取消息历史

```http
GET /api/v1/conversations/{conversation_id}/messages?limit=50
Authorization: Bearer <access_token>
```

#### 发送消息

```http
POST /api/v1/messages
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "conversation_id": "conversation_uuid",
  "content": "Hello, World!"
}
```

### 群组接口

#### 创建群组

```http
POST /api/v1/groups
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "My Group",
  "description": "A test group"
}
```

#### 获取用户群组

```http
GET /api/v1/groups
Authorization: Bearer <access_token>
```

#### 添加群组成员

```http
PUT /api/v1/groups/{group_id}/members
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "user_id": "user_uuid"
}
```

### 好友系统接口

好友系统管理用户之间的好友关系，**单聊功能仅限于好友之间使用**。

#### 获取好友列表

```http
GET /api/v1/friends
Authorization: Bearer <access_token>
```

响应：

```json
{
  "friends": [
    {
      "id": "friendship_uuid",
      "friend_id": "user_uuid",
      "friend_name": "username",
      "remark": "备注名",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### 发送好友请求

```http
POST /api/v1/friends/requests
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "to_user_id": "target_user_uuid",
  "message": "你好，我想加你为好友"
}
```

#### 获取收到的好友请求

```http
GET /api/v1/friends/requests
Authorization: Bearer <access_token>
```

响应：

```json
{
  "requests": [
    {
      "id": "request_uuid",
      "from_user": {
        "id": "user_uuid",
        "username": "sender_name"
      },
      "message": "你好，我想加你为好友",
      "status": "pending",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### 获取已发送的好友请求

```http
GET /api/v1/friends/requests/sent
Authorization: Bearer <access_token>
```

#### 接受好友请求

```http
PUT /api/v1/friends/requests/{request_id}/accept
Authorization: Bearer <access_token>
```

#### 拒绝好友请求

```http
DELETE /api/v1/friends/requests/{request_id}/reject
Authorization: Bearer <access_token>
```

#### 删除好友

```http
DELETE /api/v1/friends/{friend_id}
Authorization: Bearer <access_token>
```

#### 单聊权限说明

- 创建单聊会话时，系统会验证双方是否为好友关系
- 非好友用户无法发起单聊
- 群聊不受好友关系限制

### WebSocket 接口

连接地址：

```
ws://localhost:8080/ws?token=<access_token>&device_id=<device_id>
```

消息格式：

```json
{
  "type": "send_message",
  "payload": {
    "conversation_id": "uuid",
    "content": "Hello!",
    "temp_id": "temp-123"
  }
}
```

消息类型：

| 类型 | 说明 |
|------|------|
| `send_message` | 发送消息 |
| `typing` | 正在输入 |
| `mark_read` | 标记已读 |
| `message` | 接收新消息 |
| `message_sent` | 消息发送确认 |
| `user_online` | 用户上线通知 |
| `user_offline` | 用户下线通知 |

## Web 前端

项目内置了一个响应式 Web 前端，访问根路径 `/` 即可使用。

功能包括：
- 用户登录/注册
- 会话列表
- 群组列表
- 实时消息收发
- 在线状态显示
- 正在输入提示

## 项目结构

```
chat-general/
├── config/                 # 配置文件
│   └── default.toml
├── migrations/             # 数据库迁移
│   └── 001_initial_schema.sql
├── static/                 # 静态前端文件
│   ├── index.html
│   ├── chat.html
│   └── js/
│       ├── api.js
│       ├── auth.js
│       ├── websocket.js
│       └── chat.js
├── src/
│   ├── api/               # HTTP API 和 WebSocket
│   ├── auth/              # 认证模块
│   ├── config/            # 配置解析
│   ├── domain/            # 领域模型
│   ├── error/             # 错误处理
│   ├── event/             # 事件系统
│   ├── friend/            # 好友系统
│   ├── group/             # 群组管理
│   ├── infra/             # 基础设施（数据库、缓存）
│   ├── message/           # 消息处理
│   ├── server/            # 服务器组装
│   ├── session/           # 会话管理
│   ├── lib.rs
│   └── main.rs
├── Cargo.toml
└── README.md
```

## 测试

运行测试：

```bash
cargo test
```

运行特定测试：

```bash
cargo test test_message_creation
```

## 许可证

MIT License
