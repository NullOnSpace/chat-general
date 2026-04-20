#!/bin/bash

set -e

REPO_URL="https://github.com/your-org/chat-general.git"
APP_NAME="chat-general"
APP_DIR="/opt/chat-general"
LOG_DIR="/var/log/chat-general"
SERVICE_USER="chat-general"

DEFAULT_DOMAIN=""
DEFAULT_DB_PASSWORD=""
DEFAULT_DB_USER="chat_user"
DEFAULT_DB_NAME="chat_general"
DEFAULT_PORT=8080

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "此脚本需要 root 权限运行"
        log_info "请使用: sudo $0 $@"
        exit 1
    fi
}

print_banner() {
    echo ""
    echo "========================================"
    echo "   Chat-General 部署脚本"
    echo "========================================"
    echo ""
}

usage() {
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  --domain <domain>       服务器域名 (必需)"
    echo "  --db-password <pass>    数据库密码 (必需)"
    echo "  --db-user <user>        数据库用户名 (默认: chat_user)"
    echo "  --db-name <name>        数据库名称 (默认: chat_general)"
    echo "  --port <port>           应用端口 (默认: 8080)"
    echo "  --skip-deps             跳过依赖安装"
    echo "  --skip-nginx            跳过 Nginx 配置"
    echo "  --skip-ssl              跳过 SSL 配置"
    echo "  --help                  显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 --domain example.com --db-password mypassword"
    echo ""
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --domain)
                DOMAIN="$2"
                shift 2
                ;;
            --db-password)
                DB_PASSWORD="$2"
                shift 2
                ;;
            --db-user)
                DB_USER="$2"
                shift 2
                ;;
            --db-name)
                DB_NAME="$2"
                shift 2
                ;;
            --port)
                APP_PORT="$2"
                shift 2
                ;;
            --skip-deps)
                SKIP_DEPS=true
                shift
                ;;
            --skip-nginx)
                SKIP_NGINX=true
                shift
                ;;
            --skip-ssl)
                SKIP_SSL=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                usage
                exit 1
                ;;
        esac
    done

    if [[ -z "$DOMAIN" ]]; then
        log_error "必须指定 --domain 参数"
        usage
        exit 1
    fi

    if [[ -z "$DB_PASSWORD" ]]; then
        log_error "必须指定 --db-password 参数"
        usage
        exit 1
    fi
}

install_dependencies() {
    log_info "安装系统依赖..."

    apt update

    apt install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        curl \
        git \
        nginx \
        postgresql \
        postgresql-contrib \
        redis-server \
        certbot \
        python3-certbot-nginx \
        ufw

    log_success "系统依赖安装完成"
}

install_rust() {
    if command -v rustc &> /dev/null; then
        log_info "Rust 已安装: $(rustc --version)"
        return
    fi

    log_info "安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"

    log_success "Rust 安装完成: $(rustc --version)"
}

configure_postgresql() {
    log_info "配置 PostgreSQL..."

    sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" 2>/dev/null || true
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;" 2>/dev/null || true
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;" 2>/dev/null || true

    systemctl start postgresql
    systemctl enable postgresql

    log_success "PostgreSQL 配置完成"
}

configure_redis() {
    log_info "配置 Redis..."

    REDIS_PASSWORD=$(openssl rand -base64 24 | tr -d '/+=' | head -c 32)

    sed -i "s/^# requirepass .*/requirepass $REDIS_PASSWORD/" /etc/redis/redis.conf
    sed -i "s/^requirepass .*/requirepass $REDIS_PASSWORD/" /etc/redis/redis.conf

    if ! grep -q "^requirepass" /etc/redis/redis.conf; then
        echo "requirepass $REDIS_PASSWORD" >> /etc/redis/redis.conf
    fi

    sed -i 's/^bind .*/bind 127.0.0.1/' /etc/redis/redis.conf

    systemctl restart redis-server
    systemctl enable redis-server

    REDIS_PASSWORD_FINAL="$REDIS_PASSWORD"
    log_success "Redis 配置完成"
}

build_application() {
    log_info "编译应用程序..."

    BUILD_DIR=$(mktemp -d)
    cd "$BUILD_DIR"

    if [[ -d "/vagrant" && -f "/vagrant/Cargo.toml" ]]; then
        log_info "从本地目录复制代码..."
        cp -r /vagrant/* .
    else
        log_info "克隆代码仓库..."
        git clone "$REPO_URL" . 2>/dev/null || {
            log_warn "无法克隆仓库，请确保代码已上传到服务器"
            log_info "请将代码放置在 $APP_DIR 目录"
            return 1
        }
    fi

    cargo build --release

    log_success "应用程序编译完成"
}

setup_application() {
    log_info "设置应用程序..."

    mkdir -p "$APP_DIR"
    mkdir -p "$APP_DIR/config"
    mkdir -p "$LOG_DIR"

    if [[ -f "$BUILD_DIR/target/release/chat-server" ]]; then
        cp "$BUILD_DIR/target/release/chat-server" "$APP_DIR/"
    fi

    if [[ -d "$BUILD_DIR/config" ]]; then
        cp -r "$BUILD_DIR/config/"* "$APP_DIR/config/"
    fi

    if [[ -d "$BUILD_DIR/static" ]]; then
        cp -r "$BUILD_DIR/static" "$APP_DIR/"
    fi

    if [[ -d "$BUILD_DIR/migrations" ]]; then
        cp -r "$BUILD_DIR/migrations" "$APP_DIR/"
    fi

    JWT_SECRET=$(openssl rand -base64 32)

    cat > "$APP_DIR/.env" << EOF
# Server Configuration
CHAT__SERVER__HOST=127.0.0.1
CHAT__SERVER__PORT=$APP_PORT

# Database Configuration
CHAT__DATABASE__HOST=127.0.0.1
CHAT__DATABASE__PORT=5432
CHAT__DATABASE__USERNAME=$DB_USER
CHAT__DATABASE__PASSWORD=$DB_PASSWORD
CHAT__DATABASE__DATABASE=$DB_NAME

# Redis Configuration
CHAT__REDIS__HOST=127.0.0.1
CHAT__REDIS__PORT=6379
CHAT__REDIS__PASSWORD=$REDIS_PASSWORD_FINAL
CHAT__REDIS__DATABASE=0

# JWT Configuration
CHAT__JWT__SECRET=$JWT_SECRET
CHAT__JWT__ACCESS_TOKEN_EXPIRY=3600
CHAT__JWT__REFRESH_TOKEN_EXPIRY=604800

# Environment
CHAT_ENV=production

# Logging
CHAT_LOG_LEVEL=info
CHAT_LOG_FORMAT=compact
EOF

    chmod 600 "$APP_DIR/.env"

    log_success "应用程序设置完成"
}

run_migrations() {
    log_info "运行数据库迁移..."

    if [[ -d "$APP_DIR/migrations" ]]; then
        for migration in "$APP_DIR/migrations/"*.sql; do
            if [[ -f "$migration" ]]; then
                log_info "运行迁移: $(basename $migration)"
                sudo -u postgres psql -d "$DB_NAME" -f "$migration" || true
            fi
        done
    fi

    log_success "数据库迁移完成"
}

create_user() {
    log_info "创建服务用户..."

    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd -r -s /bin/false "$SERVICE_USER"
    fi

    chown -R "$SERVICE_USER:$SERVICE_USER" "$APP_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$LOG_DIR"

    log_success "服务用户创建完成"
}

create_systemd_service() {
    log_info "创建 Systemd 服务..."

    cat > /etc/systemd/system/chat-general.service << EOF
[Unit]
Description=Chat-General Server
After=network.target postgresql.service redis-server.service
Wants=postgresql.service redis-server.service

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$APP_DIR
EnvironmentFile=$APP_DIR/.env
ExecStart=$APP_DIR/chat-server
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/stdout.log
StandardError=append:$LOG_DIR/stderr.log

NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=$APP_DIR $LOG_DIR

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable chat-general

    log_success "Systemd 服务创建完成"
}

configure_nginx() {
    log_info "配置 Nginx..."

    cat > /etc/nginx/sites-available/chat-general << EOF
upstream chat_backend {
    server 127.0.0.1:$APP_PORT;
    keepalive 32;
}

server {
    listen 80;
    server_name $DOMAIN;

    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    location /static/ {
        alias $APP_DIR/static/;
        expires 7d;
        add_header Cache-Control "public, immutable";
    }

    location /ws {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }

    location /api/ {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_connect_timeout 60s;
        proxy_read_timeout 60s;
        proxy_send_timeout 60s;
    }

    location / {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

    ln -sf /etc/nginx/sites-available/chat-general /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default

    nginx -t && systemctl reload nginx

    log_success "Nginx 配置完成"
}

configure_ssl() {
    log_info "配置 SSL 证书..."

    if command -v certbot &> /dev/null; then
        certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "admin@$DOMAIN" || {
            log_warn "SSL 证书配置失败，请手动配置"
            return 1
        }
        log_success "SSL 证书配置完成"
    else
        log_warn "Certbot 未安装，跳过 SSL 配置"
    fi
}

configure_firewall() {
    log_info "配置防火墙..."

    ufw allow 22/tcp
    ufw allow 80/tcp
    ufw allow 443/tcp
    ufw --force enable

    log_success "防火墙配置完成"
}

setup_logrotate() {
    log_info "配置日志轮转..."

    cat > /etc/logrotate.d/chat-general << EOF
$LOG_DIR/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 $SERVICE_USER $SERVICE_USER
    sharedscripts
    postrotate
        systemctl reload chat-general > /dev/null 2>&1 || true
    endscript
}
EOF

    log_success "日志轮转配置完成"
}

start_service() {
    log_info "启动服务..."

    systemctl start chat-general
    sleep 3

    if systemctl is-active --quiet chat-general; then
        log_success "服务启动成功"
    else
        log_error "服务启动失败"
        journalctl -u chat-general -n 20 --no-pager
        return 1
    fi
}

print_summary() {
    echo ""
    echo "========================================"
    echo "   部署完成!"
    echo "========================================"
    echo ""
    echo "应用信息:"
    echo "  域名:     https://$DOMAIN"
    echo "  端口:     $APP_PORT"
    echo "  目录:     $APP_DIR"
    echo ""
    echo "数据库信息:"
    echo "  主机:     localhost"
    echo "  数据库:   $DB_NAME"
    echo "  用户:     $DB_USER"
    echo ""
    echo "常用命令:"
    echo "  查看状态:   sudo systemctl status chat-general"
    echo "  重启服务:   sudo systemctl restart chat-general"
    echo "  查看日志:   sudo journalctl -u chat-general -f"
    echo "  应用日志:   tail -f $LOG_DIR/stdout.log"
    echo ""
    echo "配置文件: $APP_DIR/.env"
    echo ""
}

main() {
    print_banner
    check_root "$@"
    parse_args "$@"

    : ${DB_USER:=$DEFAULT_DB_USER}
    : ${DB_NAME:=$DEFAULT_DB_NAME}
    : ${APP_PORT:=$DEFAULT_PORT}
    : ${SKIP_DEPS:=false}
    : ${SKIP_NGINX:=false}
    : ${SKIP_SSL:=false}
    : ${REDIS_PASSWORD_FINAL:=""}

    log_info "开始部署 Chat-General..."
    log_info "域名: $DOMAIN"
    log_info "数据库: $DB_NAME"

    if [[ "$SKIP_DEPS" != true ]]; then
        install_dependencies
        install_rust
    fi

    configure_postgresql
    configure_redis
    build_application
    setup_application
    run_migrations
    create_user
    create_systemd_service

    if [[ "$SKIP_NGINX" != true ]]; then
        configure_nginx
    fi

    if [[ "$SKIP_NGINX" != true && "$SKIP_SSL" != true ]]; then
        configure_ssl
    fi

    configure_firewall
    setup_logrotate
    start_service
    print_summary
}

main "$@"