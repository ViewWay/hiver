//! 常用验证器 / Common validators
//!
//! 提供预定义的验证规则和工具函数
//! Provides predefined validation rules and utility functions

#![allow(clippy::expect_used)]

use crate::{ValidationError, ValidationErrors};
use regex::Regex;
use std::collections::HashMap;

/// 邮箱正则 / Email regex
pub static EMAIL_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").expect("unexpected error")
});

/// URL 正则 / URL regex
/// Supports: domain names, localhost, and IP addresses
/// 支持: 域名、localhost 和 IP 地址
pub static URL_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^https?://([a-zA-Z0-9.-]+\.[a-zA-Z]{2,}|localhost|[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})(:[0-9]+)?(/.*)?$").expect("unexpected error")
});

/// 用户名正则 / Username regex
pub static USERNAME_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]{3,20}$").expect("unexpected error"));

/// 手机号正则 / Phone regex
pub static PHONE_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^[+]?[1-9]\d{1,14}$").expect("unexpected error"));

/// 验证规则 / Validation rules
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// 非空字符串 / Non-empty string
    NotEmpty,
    /// 长度范围 / Length range
    Length {
        /// Minimum length (inclusive)
        /// 最小长度（包含）
        min: usize,
        /// Maximum length (inclusive)
        /// 最大长度（包含）
        max: usize,
    },
    /// 数值范围 / Numeric range
    Range {
        /// Minimum value (inclusive)
        /// 最小值（包含）
        min: i64,
        /// Maximum value (inclusive)
        /// 最大值（包含）
        max: i64,
    },
    /// 正则匹配 / Regex match
    Regex(&'static str),
    /// 邮箱格式 / Email format
    Email,
    /// URL 格式 / URL format
    Url,
    /// 用户名格式 / Username format
    Username,
    /// 手机号格式 / Phone format
    Phone,
    /// 自定义验证器 / Custom validator
    Custom(fn(&str) -> Result<(), String>),
}

impl ValidationRule {
    /// 验证字符串值 / Validate string value
    pub fn validate_str(&self, field: &str, value: &str) -> Result<(), ValidationError> {
        match self {
            ValidationRule::NotEmpty => {
                if value.trim().is_empty() {
                    return Err(ValidationError::new(field, "Cannot be empty"));
                }
            },
            ValidationRule::Length { min, max } => {
                let len = value.len();
                if len < *min || len > *max {
                    return Err(ValidationError::invalid_length(field, *min, *max));
                }
            },
            // Range validation for numeric strings
            // Range 验证用于数字字符串
            ValidationRule::Range { min, max } => {
                // Try to parse as i64 for range validation
                // 尝试解析为 i64 进行范围验证
                if let Ok(num) = value.parse::<i64>() {
                    if num < *min || num > *max {
                        return Err(ValidationError::out_of_range(field, *min, *max));
                    }
                } else {
                    // If not a valid number, return an error
                    // 如果不是有效数字，则返回错误
                    return Err(ValidationError::new(field, "Must be a valid number"));
                }
            },
            ValidationRule::Regex(pattern) => {
                let regex = Regex::new(pattern).expect("unexpected error");
                if !regex.is_match(value) {
                    return Err(ValidationError::pattern_mismatch(field, pattern));
                }
            },
            ValidationRule::Email => {
                if !EMAIL_REGEX.is_match(value) {
                    return Err(ValidationError::invalid_email(field));
                }
            },
            ValidationRule::Url => {
                if !URL_REGEX.is_match(value) {
                    return Err(ValidationError::new(field, "Invalid URL format"));
                }
            },
            ValidationRule::Username => {
                if !USERNAME_REGEX.is_match(value) {
                    return Err(ValidationError::new(
                        field,
                        "Username must be 3-20 characters (alphanumeric, underscore, hyphen)",
                    ));
                }
            },
            ValidationRule::Phone => {
                if !PHONE_REGEX.is_match(value) {
                    return Err(ValidationError::new(field, "Invalid phone number format"));
                }
            },
            ValidationRule::Custom(validator) => {
                validator(value).map_err(|msg| ValidationError::new(field, msg))?;
            },
        }
        Ok(())
    }

    /// 获取规则描述 / Get rule description
    pub fn description(&self) -> String {
        match self {
            ValidationRule::NotEmpty => "not empty".to_string(),
            ValidationRule::Length { min, max } => format!("length between {} and {}", min, max),
            ValidationRule::Range { min, max } => format!("range between {} and {}", min, max),
            ValidationRule::Regex(p) => format!("regex: {}", p),
            ValidationRule::Email => "valid email".to_string(),
            ValidationRule::Url => "valid URL".to_string(),
            ValidationRule::Username => "valid username".to_string(),
            ValidationRule::Phone => "valid phone".to_string(),
            ValidationRule::Custom(_) => "custom".to_string(),
        }
    }
}

/// 验证器构建器 / Validator builder
pub struct ValidatorBuilder {
    rules: Vec<(&'static str, ValidationRule)>,
}

impl Default for ValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorBuilder {
    /// 创建新的验证器构建器 / Create new validator builder
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加非空规则 / Add not empty rule
    pub fn not_empty(mut self, field: &'static str) -> Self {
        self.rules.push((field, ValidationRule::NotEmpty));
        self
    }

    /// 添加长度规则 / Add length rule
    pub fn length(mut self, field: &'static str, min: usize, max: usize) -> Self {
        self.rules
            .push((field, ValidationRule::Length { min, max }));
        self
    }

    /// 添加邮箱规则 / Add email rule
    pub fn email(mut self, field: &'static str) -> Self {
        self.rules.push((field, ValidationRule::Email));
        self
    }

    /// 添加 URL 规则 / Add URL rule
    pub fn url(mut self, field: &'static str) -> Self {
        self.rules.push((field, ValidationRule::Url));
        self
    }

    /// 添加用户名规则 / Add username rule
    pub fn username(mut self, field: &'static str) -> Self {
        self.rules.push((field, ValidationRule::Username));
        self
    }

    /// 添加正则规则 / Add regex rule
    pub fn regex(mut self, field: &'static str, pattern: &'static str) -> Self {
        self.rules.push((field, ValidationRule::Regex(pattern)));
        self
    }

    /// 添加自定义规则 / Add custom rule
    pub fn custom(mut self, field: &'static str, f: fn(&str) -> Result<(), String>) -> Self {
        self.rules.push((field, ValidationRule::Custom(f)));
        self
    }

    /// 验证数据 / Validate data
    pub fn validate(&self, data: &HashMap<String, String>) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        for (field, rule) in &self.rules {
            if let Some(value) = data.get(*field) {
                if let Err(e) = rule.validate_str(field, value) {
                    errors.add_error(e);
                }
            } else {
                // Field missing - treat as empty
                if matches!(rule, ValidationRule::NotEmpty) {
                    errors.add(*field, format!("{} is required", field));
                }
            }
        }

        if errors.has_errors() {
            return Err(errors);
        }
        Ok(())
    }
}

/// 快速验证函数 / Quick validation functions
///
/// 验证邮箱 / Validate email
pub fn is_email(value: &str) -> bool {
    EMAIL_REGEX.is_match(value)
}

/// 验证 URL / Validate URL
pub fn is_url(value: &str) -> bool {
    URL_REGEX.is_match(value)
}

/// 验证用户名 / Validate username
pub fn is_username(value: &str) -> bool {
    USERNAME_REGEX.is_match(value)
}

/// 验证手机号 / Validate phone
pub fn is_phone(value: &str) -> bool {
    PHONE_REGEX.is_match(value)
}

/// 验证非空 / Validate not empty
pub fn is_not_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

/// 验证长度 / Validate length
pub fn is_length(value: &str, min: usize, max: usize) -> bool {
    let len = value.len();
    len >= min && len <= max
}

/// 验证数值范围 / Validate numeric range
pub fn is_in_range(value: i64, min: i64, max: i64) -> bool {
    value >= min && value <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_email("test@example.com"));
        assert!(is_email("user.name+tag@domain.co.uk"));
        assert!(!is_email("invalid"));
        assert!(!is_email("@example.com"));
        assert!(!is_email("test@"));
    }

    #[test]
    fn test_url_validation() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://localhost:8080"));
        assert!(!is_url("not-a-url"));
    }

    #[test]
    fn test_username_validation() {
        assert!(is_username("user123"));
        assert!(is_username("test_user"));
        assert!(is_username("my-name"));
        assert!(!is_username("ab")); // Too short
        assert!(!is_username("user@name")); // Invalid char
    }

    #[test]
    fn test_phone_validation() {
        assert!(is_phone("+1234567890"));
        assert!(is_phone("1234567890"));
        assert!(!is_phone("abc"));
    }

    #[test]
    fn test_validator_builder() {
        let builder = ValidatorBuilder::new()
            .not_empty("username")
            .length("username", 3, 20)
            .email("email");

        let mut data = HashMap::new();
        data.insert("username".to_string(), "testuser".to_string());
        data.insert("email".to_string(), "test@example.com".to_string());

        assert!(builder.validate(&data).is_ok());

        // Test missing required field
        let data2 = HashMap::new();
        assert!(builder.validate(&data2).is_err());
    }

    #[test]
    fn test_validation_rule_email() {
        let rule = ValidationRule::Email;
        assert!(rule.validate_str("email", "test@example.com").is_ok());
        assert!(rule.validate_str("email", "invalid").is_err());
    }

    #[test]
    fn test_validation_rule_length() {
        let rule = ValidationRule::Length { min: 3, max: 10 };
        assert!(rule.validate_str("name", "test").is_ok());
        assert!(rule.validate_str("name", "ab").is_err());
        assert!(rule.validate_str("name", "12345678901").is_err());
    }
}
