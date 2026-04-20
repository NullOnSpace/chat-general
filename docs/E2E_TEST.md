# E2E 测试方案

## 1. 概述

本文档描述 chat-general 项目的端到端（E2E）测试方案，覆盖所有核心功能模块的集成测试。

## 2. 测试范围

### 2.1 功能模块

| 模块 | 功能点 | 优先级 |
|------|--------|--------|
| 认证 (Auth) | 注册、登录、Token刷新、登出、获取当前用户 | P0 |
| 消息 (Message) | 创建会话、获取会话列表、发送消息、获取消息历史 | P0 |
| 好友 (Friend) | 发送请求、接受/拒绝请求、好友列表、删除好友 | P0 |
| 群组 (Group) | 创建群组、成员管理、群组信息 | P1 |
| WebSocket | 连接、消息推送、在线状态、打字状态 | P1 |

### 2.2 测试场景

#### 认证模块
- 用户注册（正常/异常：用户名已存在、邮箱已存在、密码过短）
- 用户登录（正常/异常：用户不存在、密码错误）
- Token 刷新（正常/异常：无效 refresh_token）
- 获取当前用户信息（已登录/未登录）
- 用户搜索

#### 消息模块
- 创建单聊会话（好友/非好友）
- 创建群聊会话
- 获取会话列表
- 发送文本消息
- 获取消息历史（分页）
- 消息已读状态

#### 好友模块
- 发送好友请求（正常/异常：已是好友、请求已存在）
- 接受好友请求
- 拒绝好友请求
- 获取好友列表
- 获取收到的好友请求
- 获取已发送的好友请求
- 删除好友

#### 群组模块
- 创建群组
- 获取群组列表
- 获取群组详情
- 添加群成员
- 移除群成员
- 获取群成员列表

#### WebSocket 模块
- WebSocket 连接（正常/异常：无效 token）
- 消息实时推送
- 用户在线/离线状态
- 打字状态通知

## 3. 技术方案

### 3.1 测试框架

使用 Rust 生态的测试工具：

```
测试运行器: tokio-test
HTTP 客户端: reqwest
断言库: assert_matches
WebSocket 客户端: tokio-tungstenite
```

### 3.2 测试架构

```
tests/
├── e2e/
│   ├── mod.rs              # 测试模块入口
│   ├── common/
│   │   ├── mod.rs          # 公共模块
│   │   ├── fixture.rs      # 测试夹具
│   │   ├── test_app.rs     # 测试服务器
│   │   └── test_user.rs    # 测试用户辅助
│   ├── auth_test.rs        # 认证测试
│   ├── message_test.rs     # 消息测试
│   ├── friend_test.rs      # 好友测试
│   ├── group_test.rs       # 群组测试
│   └── websocket_test.rs   # WebSocket 测试
└── integration/
    └── ...                  # 集成测试（可选）
```

### 3.3 测试隔离策略

1. **数据库隔离**: 每个测试套件使用独立的测试数据库
2. **状态隔离**: 每个测试用例独立创建测试数据
3. **并发控制**: 使用 `serial_test` crate 控制需要串行执行的测试

## 4. 测试环境配置

### 4.1 环境变量

```bash
# 测试数据库配置
TEST_DB_HOST=localhost
TEST_DB_PORT=5432
TEST_DB_NAME=chat_test
TEST_DB_USER=postgres
TEST_DB_PASSWORD=postgres

# 测试 Redis 配置
TEST_REDIS_HOST=localhost
TEST_REDIS_PORT=6379
TEST_REDIS_DB=1

# JWT 测试配置
TEST_JWT_SECRET=test_secret_key_for_e2e_tests
TEST_JWT_EXPIRY=3600

# 测试服务器配置
TEST_SERVER_HOST=127.0.0.1
TEST_SERVER_PORT=18080
```

### 4.2 测试数据库

需要预先创建测试数据库：

```sql
CREATE DATABASE chat_test;
```

测试前自动执行迁移，测试后可选择清理数据。

### 4.3 测试账户

测试框架将自动创建测试账户，无需手动准备：

| 账户类型 | 用户名 | 邮箱 | 密码 | 用途 |
|----------|--------|------|------|------|
| 测试用户1 | test_user_1 | test1@example.com | password123 | 通用测试 |
| 测试用户2 | test_user_2 | test2@example.com | password123 | 好友/消息测试 |
| 测试用户3 | test_user_3 | test3@example.com | password123 | 群组测试 |

## 5. 测试用例设计

### 5.1 认证模块测试用例

```rust
// tests/e2e/auth_test.rs

#[tokio::test]
async fn test_register_success() {
    // 正常注册流程
}

#[tokio::test]
async fn test_register_duplicate_username() {
    // 用户名已存在
}

#[tokio::test]
async fn test_register_invalid_email() {
    // 无效邮箱格式
}

#[tokio::test]
async fn test_register_short_password() {
    // 密码过短
}

#[tokio::test]
async fn test_login_success() {
    // 正常登录
}

#[tokio::test]
async fn test_login_wrong_password() {
    // 密码错误
}

#[tokio::test]
async fn test_login_user_not_found() {
    // 用户不存在
}

#[tokio::test]
async fn test_refresh_token_success() {
    // Token 刷新成功
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    // 无效 refresh_token
}

#[tokio::test]
async fn test_get_current_user_authenticated() {
    // 已登录获取用户信息
}

#[tokio::test]
async fn test_get_current_user_unauthenticated() {
    // 未登录获取用户信息（应失败）
}

#[tokio::test]
async fn test_search_users() {
    // 用户搜索
}
```

### 5.2 消息模块测试用例

```rust
// tests/e2e/message_test.rs

#[tokio::test]
async fn test_create_direct_conversation_with_friend() {
    // 与好友创建单聊会话
}

#[tokio::test]
async fn test_create_direct_conversation_non_friend() {
    // 与非好友创建单聊会话（应失败）
}

#[tokio::test]
async fn test_get_conversations() {
    // 获取会话列表
}

#[tokio::test]
async fn test_send_message() {
    // 发送消息
}

#[tokio::test]
async fn test_get_messages_pagination() {
    // 消息历史分页
}

#[tokio::test]
async fn test_message_delivery_status() {
    // 消息送达状态
}
```

### 5.3 好友模块测试用例

```rust
// tests/e2e/friend_test.rs

#[tokio::test]
async fn test_send_friend_request_success() {
    // 发送好友请求成功
}

#[tokio::test]
async fn test_send_friend_request_already_friends() {
    // 已是好友，发送请求失败
}

#[tokio::test]
async fn test_send_friend_request_pending() {
    // 已有待处理请求
}

#[tokio::test]
async fn test_accept_friend_request() {
    // 接受好友请求
}

#[tokio::test]
async fn test_reject_friend_request() {
    // 拒绝好友请求
}

#[tokio::test]
async fn test_get_friends_list() {
    // 获取好友列表
}

#[tokio::test]
async fn test_get_pending_requests() {
    // 获取待处理请求
}

#[tokio::test]
async fn test_get_sent_requests() {
    // 获取已发送请求
}

#[tokio::test]
async fn test_delete_friend() {
    // 删除好友
}

#[tokio::test]
async fn test_friend_flow_complete() {
    // 完整好友流程：发送 -> 接受 -> 验证好友关系 -> 删除
}
```

### 5.4 群组模块测试用例

```rust
// tests/e2e/group_test.rs

#[tokio::test]
async fn test_create_group() {
    // 创建群组
}

#[tokio::test]
async fn test_get_groups() {
    // 获取群组列表
}

#[tokio::test]
async fn test_get_group_detail() {
    // 获取群组详情
}

#[tokio::test]
async fn test_add_group_member() {
    // 添加群成员
}

#[tokio::test]
async fn test_remove_group_member() {
    // 移除群成员
}

#[tokio::test]
async fn test_get_group_members() {
    // 获取群成员列表
}
```

### 5.5 WebSocket 模块测试用例

```rust
// tests/e2e/websocket_test.rs

#[tokio::test]
async fn test_websocket_connect_success() {
    // WebSocket 连接成功
}

#[tokio::test]
async fn test_websocket_connect_invalid_token() {
    // 无效 token 连接失败
}

#[tokio::test]
async fn test_websocket_message_send_receive() {
    // 消息发送和接收
}

#[tokio::test]
async fn test_websocket_typing_indicator() {
    // 打字状态通知
}

#[tokio::test]
async fn test_websocket_online_status() {
    // 在线状态变化
}
```

## 6. 测试夹具设计

### 6.1 TestApp

```rust
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self { ... }
    pub async fn cleanup(&self) { ... }
}
```

### 6.2 TestUser

```rust
pub struct TestUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl TestUser {
    pub async fn create(app: &TestApp, username: &str) -> Self { ... }
    pub fn auth_header(&self) -> (&'static str, String) { ... }
}
```

### 6.3 辅助函数

```rust
pub async fn make_friends(app: &TestApp, user1: &TestUser, user2: &TestUser) { ... }
pub async fn create_test_conversation(app: &TestApp, user: &TestUser, friend_id: &str) -> String { ... }
pub async fn send_test_message(app: &TestApp, user: &TestUser, conv_id: &str, content: &str) { ... }
```

## 7. 运行测试

### 7.1 运行所有 E2E 测试

```bash
cargo test --test e2e
```

### 7.2 运行特定模块测试

```bash
cargo test --test e2e auth
cargo test --test e2e friend
cargo test --test e2e message
```

### 7.3 运行单个测试

```bash
cargo test --test e2e test_friend_flow_complete
```

### 7.4 显示测试输出

```bash
cargo test --test e2e -- --nocapture
```

## 8. CI/CD 集成

### 8.1 GitHub Actions 配置

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: chat_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run E2E tests
        env:
          TEST_DB_HOST: localhost
          TEST_DB_PORT: 5432
          TEST_DB_NAME: chat_test
          TEST_DB_USER: postgres
          TEST_DB_PASSWORD: postgres
          TEST_REDIS_HOST: localhost
          TEST_REDIS_PORT: 6379
          TEST_JWT_SECRET: test_secret_key_for_ci
        run: cargo test --test e2e -- --nocapture
```

## 9. 测试覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| 认证 | 90% |
| 消息 | 85% |
| 好友 | 90% |
| 群组 | 80% |
| WebSocket | 75% |
| **总体** | **85%** |

## 10. 依赖项

需要在 `Cargo.toml` 中添加以下测试依赖：

```toml
[dev-dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
serial_test = "3.0"
assert_matches = "1.5"
once_cell = "1.19"
```

## 11. 实施计划

| 阶段 | 内容 | 预计工作量 |
|------|------|-----------|
| 第一阶段 | 测试基础设施搭建 | 1 天 |
| 第二阶段 | 认证模块测试 | 0.5 天 |
| 第三阶段 | 好友模块测试 | 0.5 天 |
| 第四阶段 | 消息模块测试 | 0.5 天 |
| 第五阶段 | 群组模块测试 | 0.5 天 |
| 第六阶段 | WebSocket 测试 | 1 天 |
| 第七阶段 | CI/CD 集成 | 0.5 天 |

---

**请审核此测试方案，确认后我将开始实施测试代码。**
