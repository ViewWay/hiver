//! Loggers endpoint - view and modify logger levels
//! Loggers 端点 - 查看和修改日志级别
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! `/actuator/loggers` - Lists and modifies logger configurations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Log level.
/// 日志级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[derive(Default)]
pub enum LogLevel
{
    /// Trace level / Trace 级别
    Trace,
    /// Debug level / Debug 级别
    Debug,
    /// Info level / Info 级别
    #[default]
    Info,
    /// Warn level / Warn 级别
    Warn,
    /// Error level / Error 级别
    Error,
    /// Off (no logging) / 关闭日志
    Off,
}

impl std::fmt::Display for LogLevel
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Off => write!(f, "OFF"),
        }
    }
}

/// Single logger descriptor.
/// 单个日志器描述符。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerDescriptor
{
    /// Effective log level.
    /// 有效日志级别。
    pub effective_level: LogLevel,
    /// Configured (explicit) level, if set.
    /// 配置的（显式）级别。
    pub configured_level: Option<LogLevel>,
}

/// Response for /actuator/loggers.
/// /actuator/loggers 的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggersResponse
{
    /// Available log levels.
    /// 可用日志级别。
    pub levels: Vec<LogLevel>,
    /// Map of logger name → descriptor.
    /// 日志器名称到描述符的映射。
    pub loggers: HashMap<String, LoggerDescriptor>,
}

/// Logger level manager.
/// 日志级别管理器。
#[derive(Debug, Clone)]
pub struct LoggerManager
{
    loggers: HashMap<String, LoggerDescriptor>,
}

impl LoggerManager
{
    /// Create a new manager with root logger at INFO.
    /// 创建带根日志器（INFO 级别）的新管理器。
    pub fn new() -> Self
    {
        let mut loggers = HashMap::new();
        loggers.insert("ROOT".to_string(), LoggerDescriptor {
            effective_level: LogLevel::Info,
            configured_level: Some(LogLevel::Info),
        });
        Self { loggers }
    }

    /// Register a logger.
    /// 注册日志器。
    pub fn register(&mut self, name: impl Into<String>, level: LogLevel)
    {
        self.loggers.insert(name.into(), LoggerDescriptor {
            effective_level: level,
            configured_level: Some(level),
        });
    }

    /// Set the level for a logger.
    /// 设置日志器的级别。
    pub fn set_level(&mut self, name: &str, level: LogLevel)
    {
        if let Some(desc) = self.loggers.get_mut(name)
        {
            desc.configured_level = Some(level);
            desc.effective_level = level;
        }
    }

    /// Get a logger descriptor.
    /// 获取日志器描述符。
    pub fn get(&self, name: &str) -> Option<&LoggerDescriptor>
    {
        self.loggers.get(name)
    }

    /// Build the response.
    /// 构建响应。
    pub fn to_response(&self) -> LoggersResponse
    {
        LoggersResponse {
            levels: vec![
                LogLevel::Trace,
                LogLevel::Debug,
                LogLevel::Info,
                LogLevel::Warn,
                LogLevel::Error,
                LogLevel::Off,
            ],
            loggers: self.loggers.clone(),
        }
    }
}

impl Default for LoggerManager
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_logger_manager_new()
    {
        let mgr = LoggerManager::new();
        let root = mgr.get("ROOT").unwrap();
        assert_eq!(root.effective_level, LogLevel::Info);
    }

    #[test]
    fn test_register_and_set()
    {
        let mut mgr = LoggerManager::new();
        mgr.register("hiver::http", LogLevel::Debug);
        assert_eq!(mgr.get("hiver::http").unwrap().effective_level, LogLevel::Debug);
        mgr.set_level("hiver::http", LogLevel::Warn);
        assert_eq!(mgr.get("hiver::http").unwrap().effective_level, LogLevel::Warn);
    }

    #[test]
    fn test_to_response()
    {
        let mut mgr = LoggerManager::new();
        mgr.register("app", LogLevel::Trace);
        let resp = mgr.to_response();
        assert_eq!(resp.levels.len(), 6);
        assert_eq!(resp.loggers.len(), 2);
    }

    #[test]
    fn test_log_level_display()
    {
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
    }
}
