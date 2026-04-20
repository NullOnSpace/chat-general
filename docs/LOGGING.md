# 日志系统配置说明

Chat-General 使用 `tracing` 和 `tracing-subscriber` 作为日志系统，支持通过环境变量进行灵活配置。

## 环境变量配置

### 基本配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `CHAT_LOG_LEVEL` | 日志级别 | `info` | `debug`, `trace`, `warn`, `error` |
| `RUST_LOG` | 日志级别（备用） | - | `chat_general=debug,tower_http=trace` |
| `CHAT_LOG_FORMAT` | 输出格式 | `pretty` | `pretty`, `compact`, `full` |
| `CHAT_LOG_JSON` | JSON 格式输出 | `false` | `true`, `1` |

### 详细配置

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `CHAT_LOG_SPAN_EVENTS` | Span 事件记录 | `none` |
| `CHAT_LOG_WITH_FILE` | 显示源文件路径 | `false` |
| `CHAT_LOG_WITH_LINE_NUMBER` | 显示行号 | `true` |
| `CHAT_LOG_WITH_TARGET` | 显示目标模块 | `true` |
| `CHAT_LOG_WITH_THREAD_IDS` | 显示线程 ID | `false` |
| `CHAT_LOG_WITH_THREAD_NAMES` | 显示线程名称 | `false` |

## 日志级别

支持的日志级别（从低到高）：

- `trace` - 最详细的调试信息
- `debug` - 调试信息
- `info` - 一般信息（默认）
- `warn` - 警告信息
- `error` - 错误信息

### 模块级别控制

可以通过 `RUST_LOG` 精细控制不同模块的日志级别：

```bash
# 设置全局级别
RUST_LOG=debug

# 设置特定模块级别
RUST_LOG=chat_general=debug,tower_http=trace

# 多模块配置
RUST_LOG=chat_general::auth=trace,chat_general::api=debug,tower_http=info
```

## 输出格式

### Pretty 格式（默认）

适合开发环境，输出美观易读：

```
2024-01-15T10:30:45.123456Z  INFO chat_general::server: Starting server
    at src/server/mod.rs:42
    with config.host="0.0.0.0" config.port=8080
```

### Compact 格式

适合生产环境，输出紧凑：

```
2024-01-15T10:30:45.123456Z INFO chat_general::server Starting server host="0.0.0.0" port=8080
```

### Full 格式

完整格式，包含所有信息：

```
2024-01-15T10:30:45.123456Z  INFO chat_general::server:src/server/mod.rs:42 Starting server host="0.0.0.0" port=8080
```

### JSON 格式

适合日志收集系统（如 ELK、Grafana Loki）：

```json
{"timestamp":"2024-01-15T10:30:45.123456Z","level":"INFO","target":"chat_general::server","message":"Starting server","host":"0.0.0.0","port":8080}
```

## Span 事件

Span 事件用于追踪异步操作的完整生命周期：

| 值 | 说明 |
|----|------|
| `none` | 不记录 span 事件（默认） |
| `active` | 记录活跃 span 的关闭事件 |
| `full` | 记录所有 span 事件（创建、进入、退出、关闭） |

## 使用示例

### 开发环境

```bash
# .env 文件
CHAT_LOG_LEVEL=debug
CHAT_LOG_FORMAT=pretty
CHAT_LOG_WITH_FILE=true
```

### 生产环境

```bash
# .env 文件
CHAT_LOG_LEVEL=info
CHAT_LOG_FORMAT=compact
CHAT_LOG_JSON=false
```

### 日志收集系统

```bash
# .env 文件
CHAT_LOG_LEVEL=info
CHAT_LOG_JSON=true
CHAT_LOG_WITH_FILE=true
CHAT_LOG_WITH_LINE_NUMBER=true
```

### 性能调试

```bash
# .env 文件
RUST_LOG=chat_general=trace,tower_http=debug
CHAT_LOG_SPAN_EVENTS=full
CHAT_LOG_FORMAT=pretty
```

## 在代码中使用日志

### 基本用法

```rust
use tracing::{info, debug, warn, error};

info!("Server started on port {}", port);
debug!("Processing request from user {}", user_id);
warn!("Connection pool running low: {} connections", count);
error!("Failed to connect to database: {}", err);
```

### 结构化日志

```rust
use tracing::info;

info!(
    user.id = %user_id,
    request.method = %method,
    request.path = %path,
    "Processing request"
);
```

### Span 追踪

```rust
use tracing::{info, instrument};

#[instrument(skip(db))]
async fn process_message(db: &Database, msg: Message) -> Result<()> {
    info!("Received message");
    // ...
    info!("Message processed successfully");
}
```

## 初始化日志系统

### 自动初始化（推荐）

```rust
use chat_general::init_logging;

fn main() {
    init_logging();  // 自动读取环境变量配置
}
```

### 自定义配置

```rust
use chat_general::{LoggingSettings, init_logging_with_settings};

fn main() {
    let settings = LoggingSettings {
        level: "debug".to_string(),
        format: "pretty".to_string(),
        json: false,
        ..Default::default()
    };
    init_logging_with_settings(&settings);
}
```

## Tower HTTP 集成

项目已集成 `tower-http` 的追踪中间件，自动记录 HTTP 请求：

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/api", handler)
    .layer(TraceLayer::new_for_http());
```

这会自动记录：
- 请求方法和路径
- 请求 ID
- 响应状态码
- 请求耗时

## 最佳实践

1. **开发环境**：使用 `pretty` 格式，`debug` 级别，开启文件和行号
2. **生产环境**：使用 `compact` 或 `json` 格式，`info` 级别
3. **性能分析**：使用 `trace` 级别和 `full` span 事件
4. **错误排查**：临时提升相关模块到 `debug` 或 `trace` 级别
5. **日志收集**：使用 JSON 格式，便于结构化查询