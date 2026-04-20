use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingSettings {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default)]
    pub json: bool,
    #[serde(default = "default_span_events")]
    pub span_events: String,
    #[serde(default)]
    pub with_file: bool,
    #[serde(default = "default_with_line_number")]
    pub with_line_number: bool,
    #[serde(default = "default_with_target")]
    pub with_target: bool,
    #[serde(default = "default_with_thread_ids")]
    pub with_thread_ids: bool,
    #[serde(default = "default_with_thread_names")]
    pub with_thread_names: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_span_events() -> String {
    "none".to_string()
}

fn default_with_file() -> bool {
    false
}

fn default_with_line_number() -> bool {
    true
}

fn default_with_target() -> bool {
    true
}

fn default_with_thread_ids() -> bool {
    false
}

fn default_with_thread_names() -> bool {
    false
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            json: false,
            span_events: default_span_events(),
            with_file: default_with_file(),
            with_line_number: default_with_line_number(),
            with_target: default_with_target(),
            with_thread_ids: default_with_thread_ids(),
            with_thread_names: default_with_thread_names(),
        }
    }
}

impl LoggingSettings {
    pub fn from_env() -> Self {
        Self {
            level: std::env::var("CHAT_LOG_LEVEL")
                .or_else(|_| std::env::var("RUST_LOG"))
                .unwrap_or_else(|_| default_log_level()),
            format: std::env::var("CHAT_LOG_FORMAT").unwrap_or_else(|_| default_log_format()),
            json: std::env::var("CHAT_LOG_JSON")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            span_events: std::env::var("CHAT_LOG_SPAN_EVENTS")
                .unwrap_or_else(|_| default_span_events()),
            with_file: std::env::var("CHAT_LOG_WITH_FILE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(default_with_file()),
            with_line_number: std::env::var("CHAT_LOG_WITH_LINE_NUMBER")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(default_with_line_number()),
            with_target: std::env::var("CHAT_LOG_WITH_TARGET")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(default_with_target()),
            with_thread_ids: std::env::var("CHAT_LOG_WITH_THREAD_IDS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(default_with_thread_ids()),
            with_thread_names: std::env::var("CHAT_LOG_WITH_THREAD_NAMES")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(default_with_thread_names()),
        }
    }

    pub fn init(&self) {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&self.level));

        let span_events = match self.span_events.as_str() {
            "none" => FmtSpan::NONE,
            "active" => FmtSpan::ACTIVE,
            "full" => FmtSpan::FULL,
            _ => FmtSpan::NONE,
        };

        if self.json {
            let json_layer = fmt::layer()
                .json()
                .with_span_events(span_events)
                .with_file(self.with_file)
                .with_line_number(self.with_line_number)
                .with_target(self.with_target)
                .with_thread_ids(self.with_thread_ids)
                .with_thread_names(self.with_thread_names);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(json_layer)
                .init();
        } else {
            let fmt_layer = match self.format.as_str() {
                "compact" => {
                    let layer = fmt::layer()
                        .compact()
                        .with_span_events(span_events)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names);
                    layer.boxed()
                }
                "pretty" => {
                    let layer = fmt::layer()
                        .pretty()
                        .with_span_events(span_events)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names);
                    layer.boxed()
                }
                _ => {
                    let layer = fmt::layer()
                        .with_span_events(span_events)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names);
                    layer.boxed()
                }
            };

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
    }
}

pub fn init_logging() {
    let settings = LoggingSettings::from_env();
    settings.init();
}

pub fn init_logging_with_settings(settings: &LoggingSettings) {
    settings.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_logging_settings() {
        let settings = LoggingSettings::default();
        assert_eq!(settings.level, "info");
        assert_eq!(settings.format, "pretty");
        assert!(!settings.json);
        assert!(settings.with_line_number);
        assert!(settings.with_target);
    }

    #[test]
    fn test_logging_settings_from_env() {
        std::env::set_var("CHAT_LOG_LEVEL", "debug");
        std::env::set_var("CHAT_LOG_FORMAT", "compact");
        std::env::set_var("CHAT_LOG_JSON", "true");

        let settings = LoggingSettings::from_env();
        assert_eq!(settings.level, "debug");
        assert_eq!(settings.format, "compact");
        assert!(settings.json);

        std::env::remove_var("CHAT_LOG_LEVEL");
        std::env::remove_var("CHAT_LOG_FORMAT");
        std::env::remove_var("CHAT_LOG_JSON");
    }
}
