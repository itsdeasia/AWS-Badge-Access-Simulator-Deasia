//! Logging and tracing configuration
//!
//! This module provides centralized logging configuration for the simulation.

use std::io;
use tracing::{info, Level};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level for the application
    pub level: Level,
    /// Whether to enable JSON formatting
    pub json_format: bool,
    /// Whether to log to file
    pub log_to_file: bool,
    /// Log file directory (if logging to file)
    pub log_directory: Option<String>,
    /// Log file prefix (if logging to file)
    pub log_file_prefix: String,
    /// Whether to enable span events
    pub enable_span_events: bool,
    /// Whether to enable ansi colors in console output
    pub enable_ansi: bool,
    /// Custom environment filter
    pub env_filter: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json_format: false,
            log_to_file: false,
            log_directory: None,
            log_file_prefix: "badge-access-simulator".to_string(),
            enable_span_events: false,
            enable_ansi: true,
            env_filter: None,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Enable JSON formatting
    pub fn with_json_format(mut self) -> Self {
        self.json_format = true;
        self
    }

    /// Enable file logging
    pub fn with_file_logging(mut self, directory: impl Into<String>) -> Self {
        self.log_to_file = true;
        self.log_directory = Some(directory.into());
        self
    }

    /// Set log file prefix
    pub fn with_file_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.log_file_prefix = prefix.into();
        self
    }

    /// Enable span events
    pub fn with_span_events(mut self) -> Self {
        self.enable_span_events = true;
        self
    }

    /// Disable ANSI colors
    pub fn without_ansi(mut self) -> Self {
        self.enable_ansi = false;
        self
    }

    /// Set custom environment filter
    pub fn with_env_filter(mut self, filter: impl Into<String>) -> Self {
        self.env_filter = Some(filter.into());
        self
    }

    /// Initialize the global tracing subscriber
    pub fn init(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing logging with configuration: {:?}", self);

        // Create environment filter
        let env_filter = if let Some(filter) = &self.env_filter {
            EnvFilter::try_new(filter)?
        } else {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(format!(
                    "{}={}",
                    env!("CARGO_PKG_NAME").replace('-', "_"),
                    self.level
                ))
            })
        };

        let registry = Registry::default().with(env_filter);

        if self.log_to_file {
            // Set up file logging
            let log_dir = self.log_directory.as_deref().unwrap_or("logs");
            let file_appender = rolling::daily(log_dir, &self.log_file_prefix);
            let (file_writer, _guard) = non_blocking(file_appender);

            // Set up console logging
            let (console_writer, _console_guard) = non_blocking(io::stderr());

            if self.json_format {
                // JSON format for both file and console
                let file_layer = fmt::layer().json().with_writer(file_writer).with_span_events(
                    if self.enable_span_events { FmtSpan::FULL } else { FmtSpan::NONE },
                );

                let console_layer =
                    fmt::layer().json().with_writer(console_writer).with_span_events(
                        if self.enable_span_events { FmtSpan::FULL } else { FmtSpan::NONE },
                    );

                registry.with(file_layer).with(console_layer).init();
            } else {
                // Pretty format for console, JSON for file
                let file_layer = fmt::layer().json().with_writer(file_writer).with_span_events(
                    if self.enable_span_events { FmtSpan::FULL } else { FmtSpan::NONE },
                );

                let console_layer = fmt::layer()
                    .pretty()
                    .with_writer(console_writer)
                    .with_ansi(self.enable_ansi)
                    .with_span_events(if self.enable_span_events {
                        FmtSpan::FULL
                    } else {
                        FmtSpan::NONE
                    });

                registry.with(file_layer).with(console_layer).init();
            }

            // Keep guards alive (in a real application, you'd want to store these)
            std::mem::forget(_guard);
            std::mem::forget(_console_guard);
        } else {
            // Console logging only
            if self.json_format {
                let layer = fmt::layer().json().with_writer(io::stderr).with_span_events(
                    if self.enable_span_events { FmtSpan::FULL } else { FmtSpan::NONE },
                );

                registry.with(layer).init();
            } else {
                let layer = fmt::layer()
                    .pretty()
                    .with_writer(io::stderr)
                    .with_ansi(self.enable_ansi)
                    .with_span_events(if self.enable_span_events {
                        FmtSpan::FULL
                    } else {
                        FmtSpan::NONE
                    });

                registry.with(layer).init();
            }
        }

        info!("Logging initialized successfully");
        Ok(())
    }

    /// Initialize logging for development (pretty console output)
    pub fn init_dev() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::new().with_level(Level::DEBUG).with_span_events().init()
    }

    /// Initialize logging for production (JSON format with file logging)
    pub fn init_prod(
        log_dir: impl Into<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::new()
            .with_level(Level::INFO)
            .with_json_format()
            .with_file_logging(log_dir)
            .without_ansi()
            .init()
    }

    /// Initialize logging for testing (minimal output)
    pub fn init_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::new().with_level(Level::WARN).without_ansi().init()
    }

    /// Initialize verbose logging (INFO level with span events)
    pub fn init_verbose() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::new().with_level(Level::INFO).with_span_events().init()
    }

    /// Initialize debug logging (DEBUG level with span events)
    pub fn init_debug() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::new().with_level(Level::DEBUG).with_span_events().init()
    }
}

/// Macro for creating structured log events with simulation context
#[macro_export]
macro_rules! sim_event {
    ($level:ident, $message:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            message = $message,
            component = "simulation",
            $($key = $value,)*
        );
    };
    ($level:ident, $message:expr) => {
        tracing::$level!(
            message = $message,
            component = "simulation",
        );
    };
}

/// Macro for creating performance measurement spans
#[macro_export]
macro_rules! perf_span {
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::info_span!(
            $name,
            component = "performance",
            $($key = $value,)*
        )
    };
    ($name:expr) => {
        tracing::info_span!(
            $name,
            component = "performance",
        )
    };
}

/// Macro for creating error context spans
#[macro_export]
macro_rules! error_span {
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::error_span!(
            $name,
            component = "error_handling",
            $($key = $value,)*
        )
    };
    ($name:expr) => {
        tracing::error_span!(
            $name,
            component = "error_handling",
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn test_logging_config_creation() {
        let config = LoggingConfig::new();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.json_format);
        assert!(!config.log_to_file);
        assert!(config.log_directory.is_none());
        assert_eq!(config.log_file_prefix, "badge-access-simulator");
        assert!(!config.enable_span_events);
        assert!(config.enable_ansi);
        assert!(config.env_filter.is_none());
    }

    #[test]
    fn test_logging_config_builder_pattern() {
        let config = LoggingConfig::new()
            .with_level(Level::DEBUG)
            .with_json_format()
            .with_file_logging("test_logs")
            .with_file_prefix("test_prefix")
            .with_span_events()
            .without_ansi()
            .with_env_filter("debug");

        assert_eq!(config.level, Level::DEBUG);
        assert!(config.json_format);
        assert!(config.log_to_file);
        assert_eq!(config.log_directory, Some("test_logs".to_string()));
        assert_eq!(config.log_file_prefix, "test_prefix");
        assert!(config.enable_span_events);
        assert!(!config.enable_ansi);
        assert_eq!(config.env_filter, Some("debug".to_string()));
    }

    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.json_format);
        assert!(!config.log_to_file);
    }
}
