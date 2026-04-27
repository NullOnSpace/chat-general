# E2E 集成测试文档

## 1. 概述

本文档描述 chat-general 项目的端到端（E2E）集成测试的实际实现，涵盖测试架构、测试基础设施、各模块测试用例及覆盖的 API 接口。

项目共有 **101 个集成测试**，分布在 6 个测试模块中，覆盖 5 大功能模块的全部 28 个 API 接口。

## 2. 测试架构

### 2.1 目录结构

```
tests/
├── e2e/
│   ├── mod.rs                    # 测试模块入口，注册所有子模块
│   ├── common/
│   │   ├── mod.rs                # 公共模块导出
│   │   ├── test_app.rs           # TestApp — 内存版测试服务器（无数据库依赖）
│   │   ├── test_app_db.rs        # TestAppWithDb — PostgreSQL 版测试服务器
│   │   ├── test_user.rs          # TestUser — 测试用户创建辅助
│   │   └── fixture.rs            # 辅助函数：make_friends、create_test_conversation 等
│   ├── auth_test.rs              # 认证模块测试（24 个）
│   ├── friend_test.rs            # 好友模块测试（20 个）
│   ├── message_test.rs           # 消息模块测试（18 个）
│   ├── group_test.rs             # 群组模块测试（21 个）
│   ├── websocket_test.rs         # WebSocket 模块测试（15 个）
│   └── db_test.rs                # 数据库集成测试（3 个）
└── e2e_tests.rs                  # 测试入口（cargo test --test e2e_tests）
```

### 2.2 两种测试服务器

项目提供两种测试服务器实现，分别用于不同场景：

#### TestApp（内存版）

| 属性 | 说明 |
|------|------|
| 定义文件 | [test_app.rs](../tests/e2e/common/test_app.rs) |
| 数据存储 | `InMemoryMessageStore`、`InMemoryGroupRepository`、`InMemoryTokenBlacklist` |
| 用户存储 | `InMemoryUserStore`（应用内置） |
| 依赖服务 | 无需 PostgreSQL、无需 Redis |
| 端口范围 | 19000+（`AtomicU16` 自增） |
| 适用场景 | auth、friend、message、group、websocket 大部分测试 |

```rust
pub struct TestApp {
    pub address: String,
    pub server_handle: Option<JoinHandle<()>>,
}
```

- 通过 `AppState::new()` 创建全内存状态
- 使用 `create_routes().with_state(state)` 构建路由
- 每个测试独立启动服务器，端口递增避免冲突
- `Drop` trait 自动 abort 服务器任务

#### TestAppWithDb（数据库版）

| 属性 | 说明 |
|------|------|
| 定义文件 | [test_app_db.rs](../tests/e2e/common/test_app_db.rs) |
| 数据存储 | PostgreSQL（好友关系持久化）、其余仍为内存实现 |
| 用户存储 | `DbUserStore`（自定义实现，直写 PostgreSQL） |
| 依赖服务 | PostgreSQL（需 `TEST_DATABASE_URL` 环境变量） |
| 端口范围 | 20000+ |
| 适用场景 | db_test — 验证数据库完整流程 |

```rust
pub struct TestAppWithDb {
    pub address: String,
    pub server_handle: Option<JoinHandle<()>>,
    pub pool: PgPool,
}
```

- 启动时自动执行 `run_migrations(&pool)`，确保数据库 schema 就绪
- 先删除 `_sqlx_migrations` 表以强制重建，保证测试隔离
- 提供 `cleanup()` 方法按依赖顺序清空所有表数据
- 自定义 `DbUserStore` 实现 `UserStorage` trait，直接操作 PostgreSQL
- 好友模块使用 `PostgresFriendRepository`，消息/群组仍用内存实现

### 2.3 TestUser

```rust
pub struct TestUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}
```

| 方法 | 说明 |
|------|------|
| `create(app, username)` | 注册 + 登录，返回完整用户信息（含 token） |
| `create_unique(app)` | 使用 UUID 生成唯一用户名，避免冲突 |
| `device_id()` | 生成随机 device_id，用于 WebSocket 连接 |

注册流程：
1. `POST /api/v1/auth/register` → 创建用户
2. `POST /api/v1/auth/login` → 获取 access_token 和 refresh_token

### 2.4 辅助函数（fixture）

| 函数 | 说明 |
|------|------|
| `make_friends(app, user1, user2)` | 完成好友建立流程：发送请求 → 获取请求 → 接受请求 |
| `create_test_conversation(app, user, friend_id)` | 创建单聊会话，返回 conversation_id |
| `send_test_message(app, user, conv_id, content)` | 发送消息，返回 message_id |
| `create_test_group(app, user, name)` | 创建群组，返回 group_id |

## 3. 测试隔离策略

### 3.1 串行执行

所有测试使用 `#[serial_test::serial]` 标注，确保同一时刻只有一个测试运行。这是因为：

- 内存版测试共享同一进程内的全局状态（`InMemoryUserStore` 等）
- 端口分配通过 `AtomicU16` 串行递增
- 避免并发注册导致用户名冲突

### 3.2 数据库隔离

- `TestAppWithDb` 每个测试先执行 `cleanup()` 清空所有表
- 清空顺序按外键依赖排列：`message_deliveries → messages → group_members → groups → friendships → friend_requests → conversation_participants → conversations → devices → users`
- 迁移每次重建（先删除 `_sqlx_migrations` 表）

### 3.3 服务器隔离

- 每个测试独立启动新服务器实例（新端口）
- `Drop` trait 确保服务器任务在测试结束后被 abort

## 4. 覆盖的 API 接口

### 4.1 完整接口清单

项目共有 **28 个 API 接口** + 1 个 WebSocket 端点 + 1 个健康检查端点：

#### 认证模块（Auth） — 8 个接口

| 接口 | 方法 | 路径 | Handler |
|------|------|------|---------|
| 用户注册 | POST | `/api/v1/auth/register` | `auth::register` |
| 用户登录 | POST | `/api/v1/auth/login` | `auth::login` |
| Token 刷新 | POST | `/api/v1/auth/refresh` | `auth::refresh` |
| 用户登出 | POST | `/api/v1/auth/logout` | `auth::logout` |
| 获取当前用户（/auth/me） | GET | `/api/v1/auth/me` | `auth::get_current_user` |
| 获取当前用户（/users/me） | GET | `/api/v1/users/me` | `auth::get_current_user` |
| 获取用户设备 | GET | `/api/v1/users/me/devices` | `auth::get_user_devices` |
| 搜索用户 | GET | `/api/v1/users/search` | `auth::search_users` |

#### 消息模块（Message） — 5 个接口

| 接口 | 方法 | 路径 | Handler |
|------|------|------|---------|
| 获取会话列表 | GET | `/api/v1/conversations` | `message::get_conversations` |
| 创建会话 | POST | `/api/v1/conversations` | `message::create_conversation` |
| 获取会话详情 | GET | `/api/v1/conversations/{id}` | `message::get_conversation` |
| 获取消息历史 | GET | `/api/v1/conversations/{id}/messages` | `message::get_messages` |
| 发送消息 | POST | `/api/v1/messages` | `message::send_message` |

#### 好友模块（Friend） — 8 个接口

| 接口 | 方法 | 路径 | Handler |
|------|------|------|---------|
| 获取好友列表 | GET | `/api/v1/friends` | `friend::get_friends` |
| 删除好友 | DELETE | `/api/v1/friends/{uid}` | `friend::delete_friend` |
| 获取收到的好友请求 | GET | `/api/v1/friends/requests` | `friend::get_pending_requests` |
| 发送好友请求 | POST | `/api/v1/friends/requests` | `friend::send_friend_request` |
| 获取已发送的好友请求 | GET | `/api/v1/friends/requests/sent` | `friend::get_sent_requests` |
| 接受好友请求 | PUT | `/api/v1/friends/requests/{id}/accept` | `friend::accept_friend_request` |
| 拒绝好友请求 | DELETE | `/api/v1/friends/requests/{id}/reject` | `friend::reject_friend_request` |

#### 群组模块（Group） — 7 个接口

| 接口 | 方法 | 路径 | Handler |
|------|------|------|---------|
| 获取群组列表 | GET | `/api/v1/groups` | `group::get_user_groups` |
| 创建群组 | POST | `/api/v1/groups` | `group::create_group` |
| 获取群组详情 | GET | `/api/v1/groups/{id}` | `group::get_group` |
| 获取群成员列表 | GET | `/api/v1/groups/{id}/members` | `group::get_group_members` |
| 添加群成员 | PUT | `/api/v1/groups/{id}/members` | `group::add_member` |
| 移除群成员 | DELETE | `/api/v1/groups/{id}/members/{uid}` | `group::remove_member` |

#### 其他端点

| 端点 | 方法 | 路径 | 说明 |
|------|------|------|------|
| 健康检查 | GET | `/health` | 返回 `{ "status": "ok" }` |
| WebSocket | GET | `/ws` | WebSocket 连接端点 |

### 4.2 接口覆盖矩阵

| 接口 | auth_test | friend_test | message_test | group_test | websocket_test | db_test |
|------|-----------|-------------|--------------|------------|----------------|---------|
| POST /auth/register | ✅ | ✅（间接） | ✅（间接） | ✅（间接） | ✅（间接） | ✅ |
| POST /auth/login | ✅ | ✅（间接） | ✅（间接） | ✅（间接） | ✅（间接） | ✅ |
| POST /auth/refresh | ✅ | - | - | - | - | - |
| POST /auth/logout | ✅ | - | - | - | - | - |
| GET /auth/me | ✅ | - | - | - | - | - |
| GET /users/me | ✅ | - | - | - | - | - |
| GET /users/me/devices | ✅ | - | - | - | - | - |
| GET /users/search | ✅ | - | - | - | - | - |
| GET /conversations | - | - | ✅ | - | - | - |
| POST /conversations | - | - | ✅ | - | ✅（间接） | ✅ |
| GET /conversations/{id} | - | - | ✅ | - | - | - |
| GET /conversations/{id}/messages | - | - | ✅ | - | - | - |
| POST /messages | - | - | ✅ | - | ✅ | - |
| GET /friends | - | ✅ | - | - | - | ✅ |
| DELETE /friends/{uid} | - | ✅ | - | - | - | - |
| GET /friends/requests | - | ✅ | - | - | - | ✅ |
| POST /friends/requests | - | ✅ | - | - | ✅（间接） | ✅ |
| GET /friends/requests/sent | - | ✅ | - | - | - | - |
| PUT /friends/requests/{id}/accept | - | ✅ | - | - | - | ✅ |
| DELETE /friends/requests/{id}/reject | - | ✅ | - | - | - | - |
| GET /groups | - | - | - | ✅ | - | - |
| POST /groups | - | - | - | ✅ | - | - |
| GET /groups/{id} | - | - | - | ✅ | - | - |
| GET /groups/{id}/members | - | - | - | ✅ | - | - |
| PUT /groups/{id}/members | - | - | - | ✅ | - | - |
| DELETE /groups/{id}/members/{uid} | - | - | - | ✅ | - | - |
| GET /health | ✅ | - | - | - | - | - |
| WebSocket /ws | - | - | - | - | ✅ | - |

**所有 28 个 API 接口 + WebSocket 端点均已被测试覆盖。**

## 5. 各模块测试详解

### 5.1 认证模块（auth_test.rs） — 24 个测试

#### 正向流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_register_success` | POST /auth/register | 正常注册，验证返回 username 和 id |
| `test_login_success` | POST /auth/login | 正常登录，验证返回 access_token、refresh_token、user 信息 |
| `test_refresh_token_success` | POST /auth/refresh | 使用有效 refresh_token 刷新，验证返回新 token |
| `test_get_current_user_authenticated` | GET /auth/me | 已登录状态获取当前用户 |
| `test_get_current_user_via_users_me` | GET /users/me | 通过 /users/me 請径获取当前用户 |
| `test_get_user_devices` | GET /users/me/devices | 获取用户设备列表 |
| `test_search_users` | GET /users/search | 搜索用户，验证返回 users 数组 |
| `test_logout` | POST /auth/logout | 正常登出 |
| `test_health_check` | GET /health | 健康检查返回 `{ "status": "ok" }` |

#### 异常流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_register_short_password` | POST /auth/register | 密码过短（3字符）应失败 |
| `test_register_invalid_email` | POST /auth/register | 无效邮箱格式应失败 |
| `test_register_duplicate_username` | POST /auth/register | 重复用户名应失败 |
| `test_register_duplicate_email` | POST /auth/register | 重复邮箱应失败 |
| `test_register_short_username` | POST /auth/register | 用户名过短（2字符）应失败 |
| `test_register_long_username` | POST /auth/register | 用户名过长（51字符）应失败 |
| `test_login_wrong_password` | POST /auth/login | 密码错误应失败 |
| `test_login_user_not_found` | POST /auth/login | 不存在用户应失败 |
| `test_login_empty_credentials` | POST /auth/login | 空用户名和密码应失败 |
| `test_refresh_token_invalid` | POST /auth/refresh | 无效 refresh_token 应失败 |
| `test_refresh_token_with_access_token` | POST /auth/refresh | 用 access_token 作 refresh_token 应失败 |
| `test_get_current_user_unauthenticated` | GET /auth/me | 未登录应失败 |
| `test_get_current_user_invalid_token` | GET /auth/me | 无效 token 应失败 |

#### 端到端流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_logout_invalidates_token` | POST /auth/logout → GET /auth/me | 登出后 access_token 应失效 |
| `test_search_users_empty_query` | GET /users/search | 空查询字符串应成功返回 |

### 5.2 好友模块（friend_test.rs） — 20 个测试

#### 正向流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_send_friend_request_success` | POST /friends/requests | 发送好友请求成功 |
| `test_get_pending_requests` | GET /friends/requests | 获取收到的好友请求 |
| `test_get_sent_requests` | GET /friends/requests/sent | 获取已发送的好友请求 |
| `test_accept_friend_request` | PUT /friends/requests/{id}/accept | 接受好友请求 |
| `test_reject_friend_request` | DELETE /friends/requests/{id}/reject | 拒绝好友请求 |
| `test_get_friends_list` | GET /friends | 获取好友列表（有好友） |
| `test_delete_friend` | DELETE /friends/{uid} | 删除好友成功 |

#### 异常流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_send_friend_request_to_self` | POST /friends/requests | 向自己发送请求应失败 |
| `test_send_friend_request_duplicate` | POST /friends/requests | 重复发送请求应失败 |
| `test_send_friend_request_already_friends` | POST /friends/requests | 已是好友再发请求应失败 |
| `test_send_friend_request_invalid_user` | POST /friends/requests | 向无效用户发送应失败 |
| `test_accept_friend_request_invalid_id` | PUT /friends/requests/{id}/accept | 无效请求 ID 应失败 |
| `test_get_pending_requests_empty` | GET /friends/requests | 无请求时返回空数组 |
| `test_get_friends_list_empty` | GET /friends | 无好友时返回空数组 |
| `test_delete_friend_not_friend` | DELETE /friends/{uid} | 删除非好友应失败 |
| `test_accept_friend_request_wrong_recipient` | PUT /friends/requests/{id}/accept | 第三人接受请求应失败 |

#### 端到端流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_friend_flow_complete` | POST → GET → PUT → GET → DELETE → GET | 完整流程：发送→接受→验证→删除→验证 |
| `test_reject_then_resend_friend_request` | POST → DELETE → POST | 拒绝后可重新发送请求 |
| `test_get_friends_list_empty` | GET /friends | 初始好友列表为空 |

### 5.3 消息模块（message_test.rs） — 18 个测试

#### 正向流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_create_conversation_with_friend` | POST /conversations | 与好友创建单聊成功 |
| `test_get_conversations` | GET /conversations | 获取会话列表 |
| `test_send_message` | POST /messages | 发送文本消息 |
| `test_get_messages` | GET /conversations/{id}/messages | 获取消息历史 |
| `test_get_conversation_by_id` | GET /conversations/{id} | 获取会话详情 |
| `test_get_messages_pagination` | GET /conversations/{id}/messages | limit 参数分页 |
| `test_get_messages_with_before_parameter` | GET /conversations/{id}/messages | before 参数时间过滤 |
| `test_get_conversations_empty` | GET /conversations | 无会话时返回空数组 |

#### 异常流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_create_conversation_non_friend` | POST /conversations | 与非好友创建会话应失败 |
| `test_create_conversation_empty_participants` | POST /conversations | 空参与者列表应失败 |
| `test_create_conversation_invalid_participant` | POST /conversations | 无效用户 ID 应失败 |
| `test_create_conversation_unauthenticated` | POST /conversations | 未登录应失败 |
| `test_send_empty_message` | POST /messages | 空消息内容应失败 |
| `test_send_message_unauthenticated` | POST /messages | 未登录应失败 |
| `test_get_conversation_invalid_id` | GET /conversations/{id} | 无效 ID 的响应行为 |

#### 端到端流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_message_flow_complete` | POST /conversations → POST /messages × 2 → GET /messages | 双用户双向消息流程 |
| `test_create_conversation_duplicate` | POST /conversations × 2 | 重复创建返回不同 ID |
| `test_send_message_to_unknown_conversation` | POST /messages | 发送到不存在的会话（当前实现允许） |

### 5.4 群组模块（group_test.rs） — 21 个测试

#### 正向流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_create_group` | POST /groups | 创建群组 |
| `test_get_groups` | GET /groups | 获取群组列表 |
| `test_get_group_detail` | GET /groups/{id} | 获取群组详情 |
| `test_add_group_member` | PUT /groups/{id}/members | 添加群成员 |
| `test_get_group_members` | GET /groups/{id}/members | 获取成员列表 |
| `test_remove_group_member` | DELETE /groups/{id}/members/{uid} | 移除群成员 |
| `test_get_groups_empty` | GET /groups | 无群组时返回空数组 |
| `test_create_group_with_member_ids` | POST /groups | 创建时指定初始成员 |
| `test_create_group_with_invalid_member_ids` | POST /groups | 无效 member_id 被跳过 |

#### 异常流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_create_group_empty_name` | POST /groups | 空群名应失败 |
| `test_create_group_unauthenticated` | POST /groups | 未登录应失败 |
| `test_get_group_not_found` | GET /groups/{id} | 不存在的群组应失败 |
| `test_get_group_invalid_id` | GET /groups/{id} | 无效 ID 应失败 |
| `test_add_group_member_invalid_user` | PUT /groups/{id}/members | 无效用户 ID 应失败 |
| `test_non_owner_cannot_add_member` | PUT /groups/{id}/members | 非群主不能添加成员 |
| `test_non_owner_cannot_remove_member` | DELETE /groups/{id}/members/{uid} | 非群主不能移除成员 |
| `test_add_duplicate_member` | PUT /groups/{id}/members | 重复添加成员应失败 |
| `test_remove_self_from_group_requires_admin` | DELETE /groups/{id}/members/{uid} | 非群主退群行为 |

#### 端到端流程测试

| 测试名 | 覆盖接口 | 说明 |
|--------|---------|------|
| `test_group_flow_complete` | POST → PUT × 2 → GET → DELETE → GET | 完整流程：创建→加人→查看→踢人→验证 |

### 5.5 WebSocket 模块（websocket_test.rs） — 15 个测试

#### 连接测试

| 测试名 | 覆盖端点 | 说明 |
|--------|---------|------|
| `test_websocket_connect_success` | GET /ws | 正常连接，验证收到 `{ "type": "connected" }` |
| `test_websocket_connect_invalid_token` | GET /ws | 无效 token 连接 |
| `test_websocket_connect_no_token` | GET /ws | 无 token 连接应失败 |
| `test_websocket_connect_invalid_device_id` | GET /ws | 无效 device_id 连接 |
| `test_websocket_close_connection` | GET /ws | 正常关闭连接 |

#### 消息收发测试

| 测试名 | 覆盖端点 | 说明 |
|--------|---------|------|
| `test_websocket_send_message_flat_format` | GET /ws | 发送 flat 格式消息，验证 `message_sent` 确认 |
| `test_websocket_message_delivery_between_users` | GET /ws | 双用户消息投递：发送者收到 `message_sent`，接收者收到 `message` |
| `test_websocket_multiple_messages_sequential` | GET /ws | 连续发送 3 条消息，验证 seq 序号正确 |
| `test_websocket_message_with_unknown_conversation` | GET /ws | 向不存在会话发消息（当前实现不校验） |

#### 协议测试

| 测试名 | 覆盖端点 | 说明 |
|--------|---------|------|
| `test_websocket_typing_indicator` | GET /ws | 打字状态通知：发送者发 `{ "type": "typing" }`，接收者收到 |
| `test_websocket_ack_message` | GET /ws | 发送 ack 确认消息 |
| `test_websocket_sync_message` | GET /ws | 发送 sync 同步请求 |
| `test_websocket_old_envelope_format_rejected` | GET /ws | 旧格式 `{ "type": "send_message", "payload": {...} }` 不被接受 |
| `test_websocket_send_invalid_json` | GET /ws | 无效 JSON 不导致连接崩溃 |
| `test_websocket_ping_pong` | GET /ws | Ping/Pong 心跳机制 |

#### WsTestHelper 实现

```rust
struct WsTestHelper {
    sender: SplitSink<...>,   // WebSocket 发送端
    receiver: SplitStream<...>, // WebSocket 接收端
}
```

| 方法 | 说明 |
|------|------|
| `connect(app, user)` | 建立 WebSocket 连接，URL 格式 `ws://addr/ws?token=...&device_id=...` |
| `wait_for_connected()` | 等待服务端发送 `{ "type": "connected" }` 消息 |
| `send_json(payload)` | 发送 JSON 消息 |
| `recv_text()` | 接收一条文本消息 |
| `recv_text_timeout(ms)` | 带超时的消息接收 |

### 5.6 数据库集成测试（db_test.rs） — 3 个测试

这些测试使用 `TestAppWithDb`（PostgreSQL），验证数据库层面的完整流程：

| 测试名 | 说明 |
|--------|------|
| `test_db_connection` | 验证数据库连接和基础注册功能 |
| `test_db_friend_flow` | 完整好友流程（注册→请求→接受→验证），数据持久化到 PostgreSQL |
| `test_db_create_conversation_with_friend` | 数据库版会话创建流程 |

## 6. WebSocket 协议格式

### 6.1 客户端发送消息

**Flat 格式**（当前支持）：

```json
{
  "type": "message",
  "conversation_id": "conv-id",
  "content": "Hello!",
  "message_type": "text",
  "reply_to": null,
  "seq": 1
}
```

**Envelope 格式**（已废弃，测试验证不被接受）：

```json
{
  "type": "send_message",
  "payload": {
    "conversation_id": "conv-id",
    "content": "Hello!"
  }
}
```

### 6.2 服务端响应

**消息发送确认**：

```json
{
  "type": "message_sent",
  "id": "server-assigned-id",
  "conversation_id": "conv-id",
  "content": "Hello!",
  "seq": 1
}
```

**消息投递**：

```json
{
  "type": "message",
  "sender_id": "user-id",
  "conversation_id": "conv-id",
  "content": "Hello!",
  "seq": 1
}
```

**连接确认**：

```json
{
  "type": "connected",
  "user_id": "user-id",
  "device_id": "device-id"
}
```

**打字状态**：

```json
{
  "type": "typing",
  "user_id": "user-id",
  "conversation_id": "conv-id",
  "is_typing": true
}
```

**错误响应**：

```json
{
  "type": "error",
  "code": 401,
  "message": "..."
}
```

## 7. 测试依赖

### 7.1 Cargo.toml dev-dependencies

```toml
[dev-dependencies]
tokio-test = "0.4"
reqwest = { version = "0.13", features = ["json"] }
criterion = { version = "0.8", features = ["async_tokio"] }
tokio-tungstenite = "0.29"
futures-util = "0.3"
serial_test = "3.4"
assert_matches = "1.5"
urlencoding = "2.1"
```

| 依赖 | 用途 |
|------|------|
| `reqwest` | HTTP 客户端，发送 REST API 请求 |
| `tokio-tungstenite` | WebSocket 客户端，测试实时通信 |
| `futures-util` | WebSocket stream 的 `SinkExt`/`StreamExt` |
| `serial_test` | `#[serial]` 标注，保证测试串行执行 |
| `urlencoding` | URL 编码（WebSocket 连接参数） |
| `tokio-test` | 异步测试运行器 |
| `assert_matches` | 模式匹配断言 |
| `criterion` | 性能基准测试 |

### 7.2 运行时依赖（仅 db_test）

| 依赖 | 用途 |
|------|------|
| PostgreSQL | `TestAppWithDb` 需要数据库连接 |
| `dotenvy` | 加载 `.env` 文件中的 `TEST_DATABASE_URL` |
| `sqlx` | 数据库连接池和迁移执行 |

## 8. 运行测试

### 8.1 运行所有 E2E 测试（内存版）

```bash
cargo test --test e2e_tests
```

### 8.2 运行特定模块测试

```bash
cargo test --test e2e_tests auth
cargo test --test e2e_tests friend
cargo test --test e2e_tests message
cargo test --test e2e_tests group
cargo test --test e2e_tests websocket
```

### 8.3 运行数据库集成测试

需要先配置 PostgreSQL：

```bash
# 创建测试数据库
createdb chat_test

# 设置环境变量（或写入 .env 文件）
export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/chat_test"

# 运行测试
cargo test --test e2e_tests db
```

### 8.4 运行单个测试

```bash
cargo test --test e2e_tests test_friend_flow_complete
```

### 8.5 显示测试输出

```bash
cargo test --test e2e_tests -- --nocapture
```

## 9. CI/CD 集成

### 9.1 GitHub Actions 配置

CI 在 `.github/workflows/ci.yml` 中配置 `e2e-tests` job：

```yaml
e2e-tests:
  name: E2E Tests
  runs-on: ubuntu-latest

  services:
    postgres:
      image: postgres:16
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

  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v5
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-e2e-${{ hashFiles('**/Cargo.lock') }}
    - name: Run E2E tests
      env:
        TEST_DATABASE_URL: postgres://postgres:postgres@localhost:5432/chat_test
      run: cargo test --test e2e_tests
```

关键设计决策：

- **PostgreSQL 16**：与应用实际使用的版本一致
- **无需手动迁移**：测试代码自动执行 `run_migrations()`，手动 `psql` 步骤已被移除
- **无需 Redis 配置**：内存版测试不依赖 Redis，E2E 测试使用 `InMemoryMessageStore`
- **services 健康检查**：`pg_isready` 确保 PostgreSQL 就绪后才开始测试

## 10. 测试统计

| 模块 | 测试数量 | 覆盖接口数 | 正向测试 | 异常测试 | 流程测试 |
|------|---------|-----------|---------|---------|---------|
| 认证（auth） | 24 | 8 | 9 | 13 | 2 |
| 好友（friend） | 20 | 7 | 7 | 9 | 4 |
| 消息（message） | 18 | 5 | 8 | 7 | 3 |
| 群组（group） | 21 | 6 | 9 | 9 | 3 |
| WebSocket | 15 | 1 | 5 | 6 | 4 |
| 数据库（db） | 3 | 3 | 3 | 0 | 0 |
| **总计** | **101** | **28+1** | **41** | **44** | **16** |