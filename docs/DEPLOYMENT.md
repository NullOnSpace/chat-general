# Chat-General 部署指南

本文档介绍如何在 Linux/Ubuntu 服务器上部署 Chat-General 应用。

## 目录

1. [系统要求](#系统要求)
2. [环境准备](#环境准备)
3. [数据库配置](#数据库配置)
4. [Redis 配置](#redis-配置)
5. [应用部署](#应用部署)
6. [Systemd 服务配置](#systemd-服务配置)
7. [Nginx 反向代理](#nginx-反向代理)
8. [SSL 配置](#ssl-配置)
9. [监控与日志](#监控与日志)
10. [故障排查](#故障排查)

---

## 系统要求

### 硬件要求

| 配置 | 最低 | 推荐 |
|------|------|------|
| CPU | 1核 | 2核+ |
| 内存 | 1GB | 2GB+ |
| 存储 | 10GB | 20GB+ |

### 软件要求

- Ubuntu 20.04/22.04 LTS 或其他 Linux 发行版
- Rust 1.70+ (编译时)
- PostgreSQL 13+
- Redis 6+ (可选)
- Nginx (推荐)

---

## 环境准备

### 1. 更新系统

```bash
sudo apt update && sudo apt upgrade -y
```

### 2. 安装基础依赖

```bash
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    nginx \
    certbot \
    python3-certbot-nginx
```

### 3. 安装 Rust (编译环境)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

验证安装：

```bash
rustc --version
cargo --version
```

---

## 数据库配置

### 1. 安装 PostgreSQL

```bash
sudo apt install -y postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### 2. 创建数据库和用户

```bash
sudo -u postgres psql
```

在 PostgreSQL 命令行中执行：

```sql
-- 创建用户
CREATE USER chat_user WITH PASSWORD 'your_secure_password';

-- 创建数据库
CREATE DATABASE chat_general OWNER chat_user;

-- 授权
GRANT ALL PRIVILEGES ON DATABASE chat_general TO chat_user;

-- 退出
\q
```

### 3. 配置 PostgreSQL 访问

编辑 `/etc/postgresql/*/main/pg_hba.conf`：

```bash
sudo nano /etc/postgresql/14/main/pg_hba.conf
```

添加以下行（如果需要远程连接）：

```
# 允许本地应用连接
host    chat_general    chat_user    127.0.0.1/32    md5
```

重启 PostgreSQL：

```bash
sudo systemctl restart postgresql
```

### 4. 运行数据库迁移

```bash
# 克隆项目（或上传代码）
git clone https://github.com/your-org/chat-general.git
cd chat-general

# 运行迁移
psql -h localhost -U chat_user -d chat_general -f migrations/001_initial_schema.sql
psql -h localhost -U chat_user -d chat_general -f migrations/002_friend_system.sql
```

---

## Redis 配置

Redis 用于 Token 黑名单和缓存（可选但推荐）。

### 1. 安装 Redis

```bash
sudo apt install -y redis-server
```

### 2. 配置 Redis

编辑 `/etc/redis/redis.conf`：

```bash
sudo nano /etc/redis/redis.conf
```

修改以下配置：

```conf
# 绑定本地地址
bind 127.0.0.1

# 设置密码
requirepass your_redis_password

# 持久化
appendonly yes
```

### 3. 启动 Redis

```bash
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

验证连接：

```bash
redis-cli -a your_redis_password ping
```

---

## 应用部署

### 1. 编译应用

```bash
cd chat-general

# 生产编译
cargo build --release
```

编译产物位于 `target/release/chat-server`。

### 2. 创建应用目录

```bash
sudo mkdir -p /opt/chat-general
sudo mkdir -p /opt/chat-general/config
sudo mkdir -p /opt/chat-general/logs
sudo mkdir -p /var/log/chat-general
```

### 3. 复制文件

```bash
# 复制可执行文件
sudo cp target/release/chat-server /opt/chat-general/

# 复制配置文件
sudo cp config/default.toml /opt/chat-general/config/

# 复制静态文件（如果有）
sudo cp -r static /opt/chat-general/
```

### 4. 创建环境配置

创建 `/opt/chat-general/.env`：

```bash
sudo nano /opt/chat-general/.env
```

内容：

```env
# Server Configuration
CHAT__SERVER__HOST=127.0.0.1
CHAT__SERVER__PORT=8080

# Database Configuration
CHAT__DATABASE__HOST=127.0.0.1
CHAT__DATABASE__PORT=5432
CHAT__DATABASE__USERNAME=chat_user
CHAT__DATABASE__PASSWORD=your_secure_password
CHAT__DATABASE__DATABASE=chat_general

# Redis Configuration
CHAT__REDIS__HOST=127.0.0.1
CHAT__REDIS__PORT=6379
CHAT__REDIS__PASSWORD=your_redis_password
CHAT__REDIS__DATABASE=0

# JWT Configuration (使用 openssl rand -base64 32 生成)
CHAT__JWT__SECRET=your_generated_jwt_secret_key
CHAT__JWT__ACCESS_TOKEN_EXPIRY=3600
CHAT__JWT__REFRESH_TOKEN_EXPIRY=604800

# Environment
CHAT_ENV=production

# Logging
CHAT_LOG_LEVEL=info
CHAT_LOG_FORMAT=compact
```

### 5. 设置权限

```bash
sudo useradd -r -s /bin/false chat-general
sudo chown -R chat-general:chat-general /opt/chat-general
sudo chown -R chat-general:chat-general /var/log/chat-general
sudo chmod 600 /opt/chat-general/.env
```

---

## Systemd 服务配置

### 1. 创建服务文件

创建 `/etc/systemd/system/chat-general.service`：

```bash
sudo nano /etc/systemd/system/chat-general.service
```

内容：

```ini
[Unit]
Description=Chat-General Server
After=network.target postgresql.service redis-server.service
Wants=postgresql.service redis-server.service

[Service]
Type=simple
User=chat-general
Group=chat-general
WorkingDirectory=/opt/chat-general
EnvironmentFile=/opt/chat-general/.env
ExecStart=/opt/chat-general/chat-server
Restart=always
RestartSec=5
StandardOutput=append:/var/log/chat-general/stdout.log
StandardError=append:/var/log/chat-general/stderr.log

# 安全配置
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=/opt/chat-general /var/log/chat-general

[Install]
WantedBy=multi-user.target
```

### 2. 启用并启动服务

```bash
sudo systemctl daemon-reload
sudo systemctl enable chat-general
sudo systemctl start chat-general
```

### 3. 检查服务状态

```bash
sudo systemctl status chat-general
```

### 4. 服务管理命令

```bash
# 重启服务
sudo systemctl restart chat-general

# 停止服务
sudo systemctl stop chat-general

# 查看日志
sudo journalctl -u chat-general -f
```

---

## Nginx 反向代理

### 1. 创建 Nginx 配置

创建 `/etc/nginx/sites-available/chat-general`：

```bash
sudo nano /etc/nginx/sites-available/chat-general
```

内容：

```nginx
upstream chat_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name your-domain.com;

    # 安全头
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # 静态文件
    location /static/ {
        alias /opt/chat-general/static/;
        expires 7d;
        add_header Cache-Control "public, immutable";
    }

    # WebSocket
    location /ws {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }

    # API
    location /api/ {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 60s;
        proxy_read_timeout 60s;
        proxy_send_timeout 60s;
    }

    # 根路径
    location / {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 2. 启用配置

```bash
sudo ln -s /etc/nginx/sites-available/chat-general /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## SSL 配置

### 使用 Certbot 配置 SSL

```bash
# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期测试
sudo certbot renew --dry-run
```

Certbot 会自动修改 Nginx 配置添加 SSL。

### 手动 SSL 配置

如果使用自签名证书或其他证书：

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # HSTS
    add_header Strict-Transport-Security "max-age=63072000" always;

    # 其他配置同上...
}

# HTTP 重定向到 HTTPS
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

---

## 监控与日志

### 日志位置

| 日志类型 | 位置 |
|---------|------|
| 应用 stdout | `/var/log/chat-general/stdout.log` |
| 应用 stderr | `/var/log/chat-general/stderr.log` |
| Systemd 日志 | `journalctl -u chat-general` |
| Nginx 日志 | `/var/log/nginx/` |

### 日志轮转

创建 `/etc/logrotate.d/chat-general`：

```bash
/var/log/chat-general/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 chat-general chat-general
    sharedscripts
    postrotate
        systemctl reload chat-general > /dev/null 2>&1 || true
    endscript
}
```

### 健康检查

```bash
# 检查服务状态
curl http://localhost:8080/api/v1/health

# 或通过 Nginx
curl https://your-domain.com/api/v1/health
```

---

## 故障排查

### 常见问题

#### 1. 服务无法启动

```bash
# 查看详细错误
sudo journalctl -u chat-general -n 50 --no-pager

# 检查配置文件
cat /opt/chat-general/.env

# 检查数据库连接
psql -h localhost -U chat_user -d chat_general -c "SELECT 1;"
```

#### 2. 数据库连接失败

```bash
# 检查 PostgreSQL 状态
sudo systemctl status postgresql

# 检查连接权限
sudo -u postgres psql -c "\du"

# 测试连接
psql -h 127.0.0.1 -U chat_user -d chat_general
```

#### 3. WebSocket 连接失败

检查 Nginx WebSocket 配置：

```bash
# 测试 WebSocket
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Key: test" -H "Sec-WebSocket-Version: 13" \
    https://your-domain.com/ws
```

#### 4. 内存不足

```bash
# 查看内存使用
free -h

# 查看进程内存
ps aux --sort=-%mem | head -10
```

---

## 更新部署

### 更新流程

```bash
# 1. 拉取最新代码
cd chat-general
git pull

# 2. 编译新版本
cargo build --release

# 3. 停止服务
sudo systemctl stop chat-general

# 4. 备份旧版本
sudo cp /opt/chat-general/chat-server /opt/chat-general/chat-server.bak

# 5. 更新文件
sudo cp target/release/chat-server /opt/chat-general/

# 6. 运行数据库迁移（如有）
psql -h localhost -U chat_user -d chat_general -f migrations/new_migration.sql

# 7. 启动服务
sudo systemctl start chat-general

# 8. 验证
sudo systemctl status chat-general
curl http://localhost:8080/api/v1/health
```

---

## 安全建议

1. **JWT 密钥**: 使用强随机密钥 (`openssl rand -base64 32`)
2. **数据库密码**: 使用强密码，定期更换
3. **防火墙**: 只开放必要端口 (80, 443)
4. **定期更新**: 保持系统和依赖更新
5. **日志监控**: 定期检查异常日志
6. **备份**: 定期备份数据库

### 防火墙配置

```bash
# 使用 ufw
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

---

## 快速部署脚本

项目提供了自动化部署脚本 `deploy.sh`，使用方法：

```bash
# 下载脚本
chmod +x deploy.sh

# 运行部署
./deploy.sh --domain your-domain.com --db-password your_password
```

详细参数请参考脚本注释。