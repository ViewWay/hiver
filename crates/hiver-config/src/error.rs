//! Configuration error types
//! 配置错误类型

use std::path::PathBuf;
use thiserror::Error;

/// Configuration error type
/// 配置错误类型
///
/// Equivalent to Spring's `ConfigurationPropertiesException`.
/// 等价于Spring的`ConfigurationPropertiesException`。
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O error
    /// I/O错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error
    /// 解析错误
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation error
    /// 验证错误
    #[error("Validation error: {0}")]
    Validation(String),

    /// Missing required property
    /// 缺少必需属性
    #[error("Missing required property: {0}")]
    MissingProperty(String),

    /// Type conversion error
    /// 类型转换错误
    #[error("Type conversion error for '{key}': expected {expected}, got {value}")]
    TypeConversion {
        /// Property key
        /// 属性键
        key: String,
        /// Expected type
        /// 期望类型
        expected: String,
        /// Actual value
        /// 实际值
        value: String,
    },

    /// File not found
    /// 文件未找到
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid format
    /// 无效格式
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    /// Cycle detected in configuration
    /// 配置中检测到循环
    #[error("Cycle detected in configuration: {0}")]
    CycleDetected(String),

    /// Override not allowed
    /// 不允许覆盖
    #[error("Override not allowed for property: {0}")]
    OverrideNotAllowed(String),

    /// Unknown profile
    /// 未知配置文件
    #[error("Unknown profile: {0}")]
    UnknownProfile(String),

    /// Deserialize error
    /// 反序列化错误
    #[error("Deserialize error: {0}")]
    Deserialize(String),
}

/// Configuration result type
/// 配置结果类型
pub type ConfigResult<T> = Result<T, ConfigError>;

impl From<config::ConfigError> for ConfigError {
    fn from(err: config::ConfigError) -> Self {
        ConfigError::Parse(err.to_string())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Deserialize(err.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Parse(err.to_string())
    }
}

// Note: yaml_rust2 error type is not directly exported
// If yaml parsing fails, it will be caught as serde_yaml error
// 注：yaml_rust2 错误类型未直接导出
// 如果 yaml 解析失败，将被捕获为 serde_yaml 错误

#[cfg(test)]
mod tests {
    use super::*;

    /// Test ConfigError display for Io variant
    /// 测试Io变体的ConfigError显示
    #[test]
    fn test_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file gone");
        let err = ConfigError::Io(io_err);
        assert!(err.to_string().contains("I/O error"));
    }

    /// Test ConfigError display for Parse variant
    /// 测试Parse变体的ConfigError显示
    #[test]
    fn test_error_parse() {
        let err = ConfigError::Parse("bad syntax".to_string());
        assert!(err.to_string().contains("Parse error"));
        assert!(err.to_string().contains("bad syntax"));
    }

    /// Test ConfigError display for Validation variant
    /// 测试Validation变体的ConfigError显示
    #[test]
    fn test_error_validation() {
        let err = ConfigError::Validation("invalid port".to_string());
        assert!(err.to_string().contains("Validation error"));
    }

    /// Test ConfigError display for MissingProperty variant
    /// 测试MissingProperty变体的ConfigError显示
    #[test]
    fn test_error_missing_property() {
        let err = ConfigError::MissingProperty("server.port".to_string());
        assert!(err.to_string().contains("server.port"));
        assert!(err.to_string().contains("Missing"));
    }

    /// Test ConfigError display for TypeConversion variant
    /// 测试TypeConversion变体的ConfigError显示
    #[test]
    fn test_error_type_conversion() {
        let err = ConfigError::TypeConversion {
            key: "port".to_string(),
            expected: "i32".to_string(),
            value: "abc".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("port"));
        assert!(msg.contains("i32"));
        assert!(msg.contains("abc"));
    }

    /// Test ConfigError display for FileNotFound variant
    /// 测试FileNotFound变体的ConfigError显示
    #[test]
    fn test_error_file_not_found() {
        let err = ConfigError::FileNotFound(PathBuf::from("/etc/hiver/app.yaml"));
        assert!(err.to_string().contains("not found"));
    }

    /// Test ConfigError display for InvalidFormat variant
    /// 测试InvalidFormat变体的ConfigError显示
    #[test]
    fn test_error_invalid_format() {
        let err = ConfigError::InvalidFormat("not yaml".to_string());
        assert!(err.to_string().contains("Invalid"));
    }

    /// Test ConfigError display for CycleDetected variant
    /// 测试CycleDetected变体的ConfigError显示
    #[test]
    fn test_error_cycle_detected() {
        let err = ConfigError::CycleDetected("a -> b -> a".to_string());
        assert!(err.to_string().contains("Cycle"));
    }

    /// Test ConfigError display for OverrideNotAllowed variant
    /// 测试OverrideNotAllowed变体的ConfigError显示
    #[test]
    fn test_error_override_not_allowed() {
        let err = ConfigError::OverrideNotAllowed("locked.key".to_string());
        assert!(err.to_string().contains("Override not allowed"));
    }

    /// Test ConfigError display for UnknownProfile variant
    /// 测试UnknownProfile变体的ConfigError显示
    #[test]
    fn test_error_unknown_profile() {
        let err = ConfigError::UnknownProfile("nonexistent".to_string());
        assert!(err.to_string().contains("Unknown profile"));
    }

    /// Test ConfigError display for Deserialize variant
    /// 测试Deserialize变体的ConfigError显示
    #[test]
    fn test_error_deserialize() {
        let err = ConfigError::Deserialize("expected string".to_string());
        assert!(err.to_string().contains("Deserialize"));
    }

    /// Test From<std::io::Error> for ConfigError
    /// 测试从std::io::Error转换到ConfigError
    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let config_err: ConfigError = io_err.into();
        match config_err {
            ConfigError::Io(_) => {},
            _ => panic!("Expected Io variant"),
        }
    }

    /// Test From<serde_json::Error> for ConfigError
    /// 测试从serde_json::Error转换到ConfigError
    #[test]
    fn test_from_serde_json_error() {
        let json_err: serde_json::Error = serde_json::from_str::<i32>("not a number").unwrap_err();
        let config_err: ConfigError = json_err.into();
        match config_err {
            ConfigError::Deserialize(_) => {},
            _ => panic!("Expected Deserialize variant"),
        }
    }

    /// Test From<toml::de::Error> for ConfigError
    /// 测试从toml::de::Error转换到ConfigError
    #[test]
    fn test_from_toml_error() {
        let toml_err = toml::from_str::<toml::Value>("{invalid").unwrap_err();
        let config_err: ConfigError = toml_err.into();
        match config_err {
            ConfigError::Parse(_) => {},
            _ => panic!("Expected Parse variant"),
        }
    }

    /// Test ConfigResult is an alias for Result<T, ConfigError>
    /// 测试ConfigResult是Result<T, ConfigError>的类型别名
    #[test]
    fn test_config_result_ok() {
        let result: ConfigResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    /// Test ConfigResult Err case
    /// 测试ConfigResult Err情况
    #[test]
    fn test_config_result_err() {
        let result: ConfigResult<()> = Err(ConfigError::Parse("err".to_string()));
        assert!(result.is_err());
    }
}
