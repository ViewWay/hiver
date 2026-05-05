//! 验证错误类型 / Validation error types
//!
//! # 验证错误 / Validation Errors
//!
//! ```rust,ignore
//! use nexus_validation::{ValidationError, ValidationErrors};
//!
//! // 单个字段错误
//! let error = ValidationError::new("username", "Username is required");
//!
//! // 多个字段错误
//! let mut errors = ValidationErrors::new();
//! errors.add("username", "Username is required");
//! errors.add("email", "Email is invalid");
//! ```

use std::collections::HashMap;
use std::fmt;

/// 单个验证错误 / Single validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 字段名 / Field name
    pub field: String,
    /// 错误消息 / Error message
    pub message: String,
    /// 错误代码 / Error code
    pub code: String,
    /// 无效值 / Invalid value
    pub value: Option<String>,
}

impl ValidationError {
    /// 创建新的验证错误 / Create new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: "validation_failed".to_string(),
            value: None,
        }
    }

    /// 设置错误代码 / Set error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    /// 设置无效值 / Set invalid value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// 必填错误 / Required error
    pub fn required(field: impl Into<String>) -> Self {
        let field_name = field.into();
        Self {
            field: field_name.clone(),
            message: format!("{} is required", field_name),
            code: "required".to_string(),
            value: None,
        }
    }

    /// 邮箱格式错误 / Email format error
    pub fn invalid_email(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: "Invalid email format".to_string(),
            code: "invalid_email".to_string(),
            value: None,
        }
    }

    /// 长度错误 / Length error
    pub fn invalid_length(field: impl Into<String>, min: usize, max: usize) -> Self {
        let field_name = field.into();
        Self {
            field: field_name.clone(),
            message: format!("{} length must be between {} and {}", field_name, min, max),
            code: "invalid_length".to_string(),
            value: None,
        }
    }

    /// 范围错误 / Range error
    pub fn out_of_range(field: impl Into<String>, min: i64, max: i64) -> Self {
        let field_name = field.into();
        Self {
            field: field_name.clone(),
            message: format!("{} must be between {} and {}", field_name, min, max),
            code: "out_of_range".to_string(),
            value: None,
        }
    }

    /// 正则表达式错误 / Regex error
    pub fn pattern_mismatch(field: impl Into<String>, pattern: &str) -> Self {
        let field_name = field.into();
        Self {
            field: field_name.clone(),
            message: format!("{} does not match required pattern", field_name),
            code: "pattern_mismatch".to_string(),
            value: Some(pattern.to_string()),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Convert `ValidationErrors` to `ValidationError`
/// 将 `ValidationErrors` 转换为 `ValidationError`
///
/// This conversion is used when propagating validation errors
/// through error handling mechanisms that expect a single error.
/// 当通过期望单个错误的错误处理机制传播验证错误时使用此转换。
impl From<ValidationErrors> for ValidationError {
    fn from(errors: ValidationErrors) -> Self {
        // Create a single validation error that summarizes all field errors
        // 创建一个汇总所有字段错误的单个验证错误
        let field_count = errors.len();
        let error_messages: Vec<String> = errors
            .errors
            .into_iter()
            .flat_map(|(field, errs)| {
                errs.into_iter()
                    .map(move |e| format!("{}: {}", field, e.message))
            })
            .collect();

        Self {
            field: "multiple".to_string(),
            message: format!(
                "Validation failed for {} field(s): {}",
                field_count,
                error_messages.join(", ")
            ),
            code: "validation_failed".to_string(),
            value: None,
        }
    }
}

/// 多个验证错误集合 / Collection of validation errors
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    /// 字段错误映射 / Field error mapping
    pub errors: HashMap<String, Vec<ValidationError>>,
}

impl ValidationErrors {
    /// 创建新的错误集合 / Create new error collection
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加字段错误 / Add field error
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        let field = field.into();
        let error = ValidationError::new(&field, message);
        self.errors.entry(field).or_default().push(error);
    }

    /// 添加验证错误对象 / Add validation error object
    pub fn add_error(&mut self, error: ValidationError) {
        let field = error.field.clone();
        self.errors.entry(field).or_default().push(error);
    }

    /// 合并其他错误集合 / Merge another error collection
    pub fn merge(&mut self, other: ValidationErrors) {
        for (field, errors) in other.errors {
            self.errors.entry(field).or_default().extend(errors);
        }
    }

    /// 检查是否有错误 / Check if has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 获取字段数量 / Get field count
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// 检查是否为空 / Check if empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// 获取所有字段名 / Get all field names
    pub fn fields(&self) -> Vec<String> {
        self.errors.keys().cloned().collect()
    }

    /// 获取指定字段的错误 / Get errors for specific field
    pub fn get(&self, field: &str) -> Option<&[ValidationError]> {
        self.errors.get(field).map(Vec::as_slice)
    }

    /// 转换为 JSON 值 / Convert to JSON value
    pub fn to_map(&self) -> HashMap<String, Vec<String>> {
        self.errors
            .iter()
            .map(|(field, errors)| {
                let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
                (field.clone(), messages)
            })
            .collect()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation errors: ")?;
        for (field, errors) in &self.errors {
            for error in errors {
                write!(f, "\n  - {}: {}", field, error.message)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// From 实现，方便从单个错误转换 / From impl for easy conversion
impl From<ValidationError> for ValidationErrors {
    fn from(error: ValidationError) -> Self {
        let mut errors = Self::new();
        errors.add_error(error);
        errors
    }
}

/// From 实现，方便从向量转换 / From impl for Vec
impl From<Vec<ValidationError>> for ValidationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        let mut result = Self::new();
        for error in errors {
            result.add_error(error);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let error = ValidationError::new("username", "Username is required");
        assert_eq!(error.field, "username");
        assert_eq!(error.message, "Username is required");
    }

    #[test]
    fn test_validation_errors() {
        let mut errors = ValidationErrors::new();
        errors.add("username", "Username is required");
        errors.add("email", "Email is invalid");

        assert_eq!(errors.len(), 2);
        assert!(errors.has_errors());
        assert_eq!(errors.get("username").unwrap().len(), 1);
    }

    #[test]
    fn test_required_error() {
        let error = ValidationError::required("password");
        assert_eq!(error.code, "required");
    }

    #[test]
    fn test_invalid_email() {
        let error = ValidationError::invalid_email("email");
        assert_eq!(error.code, "invalid_email");
    }

    #[test]
    fn test_to_map() {
        let mut errors = ValidationErrors::new();
        errors.add("username", "Username is required");
        let map = errors.to_map();
        assert_eq!(map.get("username").unwrap().len(), 1);
    }
}
