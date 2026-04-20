# Chat-General 项目实现计划

## 概述

本文档基于 [ARCHITECTURE.md](./ARCHITECTURE.md) 中的架构设计，规划项目的分步实现路线图。

---

## 阶段一：项目基础搭建

**目标**：建立项目骨架，配置开发环境，实现核心领域模型。

### 1.1 项目初始化

- [ ] 配置 `Cargo.toml` 依赖
  - axum, tokio, serde, sqlx, redis, jsonwebtoken 等
  - 配置 workspace（如需多 crate）
- [ ] 创建目录结构
- [ ] 配置开发工具
  - rustfmt.toml
  - clippy.toml
  - .gitignore 完善

### 1.2 配置模块

- [ ] 实现 `config/settings.rs`
  - 支持 TOML/YAML 配置文件
  - 环境变量覆盖
  - 配置验证

### 1.3 领域模型

- [ ] 实现 `domain/user.rs`
  - UserId, User 实体
- [ ] 实现 `domain/device.rs`
  - DeviceId, DeviceInfo, DeviceType
- [ ] 实现 `domain/message.rs`
  - Message, MessageId, MessageType, MessageStatus
- [ ] 实现 `domain/conversation.rs`
  - Conversation, ConversationId, ConversationType
- [ ] 实现 `domain/group.rs`
  - Group, GroupId, GroupMember, GroupRole

### 1.4 错误处理

- [ ] 定义统一错误类型 `error/mod.rs`
- [ ] 实现 API 错误响应转换

---

## 阶段二：基础设施层

**目标**：实现数据库、缓存等基础设施。

### 2.1 数据库设计

- [ ] 设计数据库 Schema
  - users 表
  - devices 表
  - messages 表
  - conversations 表
  - groups 表
  - group_members 表
- [ ] 编写迁移脚本

### 2.2 数据库访问层

- [ ] 实现 `infra/db/postgres.rs`
  - 连接池管理
- [ ] 实现 Repository traits
  - `UserRepository`
  - `MessageRepository`
  - `ConversationRepository`
  - `GroupRepository`

### 2.3 缓存层

- [ ] 实现 `infra/cache/redis.rs`
  - 连接管理
  - 基础缓存操作
  - 在线状态存储

### 2.4 内存队列（可选）

- [ ] 实现 `infra/queue/in_memory.rs`
  - 用于单机开发测试

---

## 阶段三：认证模块

**目标**：实现可扩展的认证系统。

### 3.1 认证 Trait 定义

- [ ] 实现 `auth/trait.rs`
  - AuthProvider trait
  - AuthError 类型
  - TokenPair 类型

### 3.2 JWT 实现

- [ ] 实现 `auth/jwt.rs`
  - JWT 生成与验证
  - Refresh Token 机制
  - Token 黑名单（Redis）

### 3.3 API Key 实现

- [ ] 实现 `auth/api_key.rs`
  - 用于机器人/第三方服务

### 3.4 认证中间件

- [ ] 实现 Axum 认证中间件
  - 从请求提取 Token
  - 验证并注入用户信息

---

## 阶段四：会话与设备管理

**目标**：实现多设备登录和会话管理。

### 4.1 设备注册

- [ ] 实现 `session/device_registry.rs`
  - 设备注册/注销
  - 设备列表查询
  - 在线状态管理

### 4.2 会话管理

- [ ] 实现 `session/manager.rs`
  - 会话创建与销毁
  - 会话状态追踪

### 4.3 连接管理

- [ ] 实现 `session/connection.rs`
  - WebSocket 连接封装
  - 连接状态管理

---

## 阶段五：消息核心

**目标**：实现消息存储、路由和处理。

### 5.1 消息存储

- [ ] 实现 `message/store.rs` trait
- [ ] 实现 PostgreSQL 存储后端
  - 消息持久化
  - 消息索引

### 5.2 消息路由

- [ ] 实现 `message/router.rs`
  - 单聊消息路由
  - 消息投递逻辑

### 5.3 消息处理器

- [ ] 实现 `message/handler.rs` trait
- [ ] 实现处理器链执行器

### 5.4 历史消息服务

- [ ] 实现 `message/history.rs`
  - 设备级消息同步
  - 分页查询
  - 已读/已送达状态

---

## 阶段六：群组功能

**目标**：实现群组聊天功能。

### 6.1 群组管理

- [ ] 实现 `group/manager.rs`
  - 群组创建/解散
  - 群组信息修改

### 6.2 成员管理

- [ ] 实现 `group/membership.rs`
  - 成员加入/退出
  - 角色权限管理
  - 成员列表查询

### 6.3 群消息分发

- [ ] 实现 `group/dispatcher.rs`
  - 群消息广播
  - @提及处理

---

## 阶段七：好友系统

**目标**：实现好友关系管理和单聊权限控制。

### 7.1 领域模型

- [ ] 实现 `domain/friendship.rs`
  - FriendshipStatus 枚举
  - FriendRequest 实体
  - Friendship 实体
  - FriendRequestId, FriendshipId 类型

### 7.2 好友仓储

- [ ] 实现 `infra/db/friend_repository.rs`
  - FriendRepository trait
  - PostgreSQL 实现
  - 好友请求 CRUD
  - 好友关系 CRUD

### 7.3 好友管理服务

- [ ] 实现 `friend/manager.rs`
  - FriendService trait
  - FriendManager 实现
  - 发送好友请求
  - 接受/拒绝请求
  - 删除好友
  - 好友列表查询

### 7.4 聊天权限控制

- [ ] 实现 `friend/permission.rs`
  - ChatPermissionChecker
  - 单聊权限验证
  - 消息发送权限检查

### 7.5 好友 API

- [ ] 实现 `api/handlers/friend.rs`
  - GET /api/v1/friends
  - DELETE /api/v1/friends/:uid
  - GET /api/v1/friends/requests
  - POST /api/v1/friends/requests
  - PUT /api/v1/friends/requests/:id/accept
  - DELETE /api/v1/friends/requests/:id/reject
  - GET /api/v1/friends/requests/sent

### 7.6 数据库迁移

- [ ] 创建 `migrations/002_friend_system.sql`
  - friend_requests 表
  - friendships 表
  - 相关索引

### 7.7 事件集成

- [ ] 扩展 `event/types.rs`
  - FriendRequestReceived 事件
  - FriendRequestAccepted 事件
  - FriendRequestRejected 事件
  - FriendRemoved 事件

### 7.8 单聊权限集成

- [ ] 修改消息发送逻辑
  - 创建会话时检查好友关系
  - 发送消息时验证权限
  - 返回适当的错误信息

---

## 阶段八：事件系统

**目标**：实现事件发布订阅机制。

### 8.1 事件类型

- [ ] 实现 `event/types.rs`
  - 定义所有事件类型
  - 事件序列化

### 8.2 事件总线

- [ ] 实现 `event/bus.rs`
  - 内存事件总线
  - Redis Pub/Sub（可选）

### 8.3 订阅者机制

- [ ] 实现 `event/subscriber.rs` trait
- [ ] 实现订阅者注册与分发

---

## 阶段九：HTTP API

**目标**：实现 RESTful API。

### 9.1 路由定义

- [ ] 实现 `api/routes.rs`
  - 路由组织
  - 版本控制

### 9.2 认证接口

- [ ] 实现 `api/handlers/auth.rs`
  - POST /api/v1/auth/login
  - POST /api/v1/auth/refresh
  - POST /api/v1/auth/logout

### 9.3 用户接口

- [ ] 实现 `api/handlers/user.rs`
  - GET /api/v1/users/me
  - GET /api/v1/users/me/devices

### 9.4 会话接口

- [ ] 实现 `api/handlers/conversation.rs`
  - GET /api/v1/conversations
  - POST /api/v1/conversations
  - GET /api/v1/conversations/:id
  - GET /api/v1/conversations/:id/messages

### 9.5 群组接口

- [ ] 实现 `api/handlers/group.rs`
  - POST /api/v1/groups
  - GET /api/v1/groups/:id
  - PUT /api/v1/groups/:id/members
  - DELETE /api/v1/groups/:id/members/:uid

### 9.6 DTO 定义

- [ ] 实现 `api/dto/request.rs`
- [ ] 实现 `api/dto/response.rs`

---

## 阶段十：WebSocket 支持

**目标**：实现实时通信。

### 10.1 WebSocket 处理器

- [ ] 实现 `api/handlers/websocket.rs`
  - 连接升级
  - 认证验证
  - 设备绑定

### 10.2 消息协议

- [ ] 定义 WebSocket 消息格式
  - 客户端消息类型
  - 服务端消息类型
  - 序列号确认机制

### 10.3 心跳与重连

- [ ] 实现心跳检测
- [ ] 实现断线重连逻辑

---

## 阶段十一：扩展机制

**目标**：实现扩展点，支持二次开发。

### 11.1 钩子系统

- [ ] 实现 `extension/hook.rs`
  - 消息发送前钩子
  - 消息接收后钩子
  - 用户状态变更钩子

### 11.2 消息中间件

- [ ] 实现 `extension/middleware.rs`
  - 中间件 trait
  - 中间件链

### 11.3 机器人接口

- [ ] 实现 `extension/bot.rs`
  - Bot trait
  - 命令解析
  - 自动回复

### 11.4 扩展端点

- [ ] 实现 Bot API
  - POST /api/v1/bot/send
  - POST /api/v1/bot/webhook

---

## 阶段十二：服务器组装

**目标**：整合所有模块，提供统一启动入口。

### 12.1 应用状态

- [ ] 实现 `server/state.rs`
  - AppState 定义
  - 依赖注入

### 12.2 服务器构建器

- [ ] 实现 `server/builder.rs`
  - Builder 模式
  - 灵活配置

### 12.3 启动入口

- [ ] 完善 `main.rs`
  - 配置加载
  - 依赖初始化
  - 优雅关闭

### 12.4 库导出

- [ ] 完善 `lib.rs`
  - 公共 API 导出
  - 文档注释

---

## 阶段十三：测试与文档

**目标**：确保代码质量，完善文档。

### 13.1 单元测试

- [ ] 领域模型测试
- [ ] 认证模块测试
- [ ] 消息处理测试
- [ ] 好友系统测试

### 13.2 集成测试

- [ ] API 集成测试
- [ ] WebSocket 测试
- [ ] 数据库测试
- [ ] 好友权限测试

### 13.3 文档

- [ ] API 文档（OpenAPI/Swagger）
- [ ] 代码文档注释
- [ ] 使用示例

---

## 阶段十四：优化与部署

**目标**：性能优化，生产部署准备。

### 14.1 性能优化

- [ ] 数据库查询优化
- [ ] 缓存策略优化
- [ ] 连接池调优

### 13.2 安全加固

- [ ] 输入验证
- [ ] 速率限制
- [ ] 安全审计

### 13.3 部署配置

- [ ] Docker 镜像
- [ ] docker-compose 配置
- [ ] Kubernetes 配置（可选）

---

## 里程碑时间线

```
阶段一  ████████░░░░░░░░░░░░  项目基础
阶段二  ░░░░░░░░████████░░░░  基础设施
阶段三  ░░░░░░░░░░░░░░████░░  认证模块
阶段四  ░░░░░░░░░░░░░░░░████  会话管理
阶段五  ████████░░░░░░░░░░░░  消息核心（MVP）
阶段六  ░░░░████░░░░░░░░░░░░  群组功能
阶段七  ░░░░░░░░████░░░░░░░░  事件系统
阶段八  ░░░░░░░░░░░░████████  HTTP API
阶段九  ░░░░░░░░░░░░░░░░████  WebSocket
阶段十  ░░░░░░░░░░░░░░░░░░██  扩展机制
阶段十一 ░░░░░░░░░░░░░░░░░░██  服务器组装
阶段十二 ░░░░░░░░░░░░░░░░░░██  测试文档
阶段十三 ░░░░░░░░░░░░░░░░░░██  优化部署
```

---

## MVP 定义

**最小可行产品（MVP）** 包含：

- ✅ 用户认证（JWT）
- ✅ 单对单聊天
- ✅ WebSocket 实时通信
- ✅ 历史消息查询
- ✅ 多设备支持

MVP 完成后即可进行初步测试和反馈收集。

---

## 开发优先级

| 优先级 | 模块 | 说明 |
|--------|------|------|
| P0 | 领域模型 | 核心基础 |
| P0 | 认证模块 | 安全必需 |
| P0 | 消息核心 | 核心功能 |
| P0 | WebSocket | 实时通信 |
| P1 | 会话管理 | 多设备支持 |
| P1 | HTTP API | 完整接口 |
| P2 | 群组功能 | 扩展功能 |
| P2 | 事件系统 | 可观测性 |
| P3 | 扩展机制 | 二次开发 |
| P3 | 优化部署 | 生产就绪 |

---

## 技术债务追踪

开发过程中需要关注的技术债务：

- [ ] 错误处理统一化
- [ ] 日志规范化
- [ ] 配置热更新
- [ ] 监控指标集成
- [ ] 灰度发布支持
