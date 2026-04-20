# ===========================================
# 后端配置步骤
# ===========================================
#
# 1. PostgreSQL 数据库配置
#    - 安装 PostgreSQL: https://www.postgresql.org/download/
#    - 创建数据库: CREATE DATABASE chat_general;
#    - 运行迁移: psql -d chat_general -f migrations/001_initial_schema.sql
#    - 运行好友系统迁移: psql -d chat_general -f migrations/002_friend_system.sql
#
# 2. Redis 配置 (可选，用于 Token 黑名单和缓存)
#    - 安装 Redis: https://redis.io/download
#    - 启动 Redis: redis-server
#
# 3. JWT 密钥配置
#    - 生成安全的密钥: openssl rand -base64 32
#    - 将生成的密钥填入 CHAT__JWT__SECRET
#
# 4. 启动服务
#    - 开发模式: cargo run
#    - 生产模式: cargo run --release

# ===========================================
# 测试数据库配置 (E2E 测试)
# ===========================================
# 设置环境变量 TEST_DATABASE_URL 来使用测试数据库
# 示例: export TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/chat_test

# ===========================================
# 后端配置步骤
# ===========================================
#
# 1. PostgreSQL 数据库配置
#    安装 PostgreSQL: https://www.postgresql.org/download/
#    
#    创建主数据库:
#    CREATE DATABASE chat_general;
#    
#    创建测试数据库:
#    CREATE DATABASE chat_test;
#    
#    运行迁移:
#    psql -d chat_general -f migrations/001_initial_schema.sql
#    psql -d chat_general -f migrations/002_friend_system.sql
#    psql -d chat_test -f migrations/001_initial_schema.sql
#    psql -d chat_test -f migrations/002_friend_system.sql
#
# 2. Redis 配置 (可选)
#    安装 Redis: https://redis.io/download
#    启动: redis-server
#
# 3. JWT 密钥配置
#    生成安全密钥: openssl rand -base64 32
#    填入 CHAT__JWT__SECRET
#
# 4. 启动服务
#    开发模式: cargo run
#    生产模式: cargo run --release
#
# ===========================================
# 测试运行
# ===========================================
#
# E2E 测试 (使用内存存储，无需数据库):
# cargo test --test e2e_tests
#
# E2E 测试 (使用测试数据库):
# export TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/chat_test
# cargo test --test e2e_tests --features database-tests
