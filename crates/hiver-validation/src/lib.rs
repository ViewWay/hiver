//! Hiver Validation - 验证模块 / Validation Module
//!
//! 提供请求参数校验功能，等价于 Spring Validation / Provides request parameter validation, equivalent to Spring Validation

#![allow(clippy::result_large_err)]
//!
//! # Spring Equivalent / Spring等价物
//!
//! - `@Valid` → `#[Valid]` - Cascading validation / 级联验证
//! - `@Validated` → `#[Validated]` - Group validation / 分组验证
//! - `@NotNull` → `#[NotNull]` - Non-null check / 非空检查
//! - `@Size` → `#[Size]` - Size validation / 大小验证
//! - `@Min/@Max` → `#[Min]/#[Max]` - Numeric bounds / 数值边界
//! - `@Email` → `#[Email]` - Email validation / 邮箱验证
//! - `@Pattern` → `#[Pattern]` - Regex validation / 正则验证
//!
//! # Basic Usage / 基本使用
//!
//! ```rust,ignore
//! use hiver_validation::{Valid, Validated, NotNull, Size, Email};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct CreateUserRequest {
//!     #[NotNull]
//!     #[Size(min = 3, max = 20)]
//!     username: String,
//!
//!     #[Email]
//!     email: String,
//!
//!     #[Min(18)]
//!     #[Max(120)]
//!     age: u32,
//! }
//!
//! // Auto-validation with #[Valid] annotation
//! // 使用#[Valid]注解自动验证
//! #[hiver_macros::post("/users")]
//! async fn create_user(
//!     #[Valid] request: CreateUserRequest,
//! ) -> Result<Json<User>, Error> {
//!     // request is already validated
//!     // request 已经被验证
//!     Ok(Json(user))
//! }
//!
//! // Group validation with #[Validated]
//! // 使用#[Validated]进行分组验证
//! #[hiver_macros::post("/users")]
//! async fn create_user_validated(
//!     #[Validated(CreateGroup::default())] request: CreateUserRequest,
//! ) -> Result<Json<User>, Error> {
//!     Ok(Json(user))
//! }
//! ```
//!
//! # Validation Groups / 验证分组
//!
//! ```rust,ignore
//! use hiver_validation::Validated;
//!
//! #[derive(Debug, Default)]
//! struct CreateGroup;
//!
//! #[derive(Debug, Default)]
//! struct UpdateGroup;
//!
//! #[derive(Debug, Deserialize)]
//! struct UserRequest {
//!     #[NotNull(group = "CreateGroup")]
//!     username: String,
//!
//!     #[NotNull(group = "UpdateGroup")]
//!     id: u64,
//! }
//!
//! // Only validates fields marked with CreateGroup
//! // 仅验证标记为CreateGroup的字段
//! async fn create_user(
//!     #[Validated(CreateGroup)] request: UserRequest,
//! ) -> Result<Json<User>, Error> {
//!     Ok(Json(user))
//! }
//! ```

#[cfg(test)]
mod tests;

pub mod annotations;
pub mod custom;
pub mod error;
pub mod extractor;
pub mod groups;
pub mod nested;
pub mod traits;
pub mod validators;

// Re-exports commonly used types / 重新导出常用类型
pub use error::{ValidationError, ValidationErrors};
pub use extractor::Valid;
pub use groups::Validated;
pub use traits::Validate;
pub use validators::*;

// Re-export validation annotations / 重新导出验证注解
pub use annotations::{
    AssertFalse, AssertTrue, CreditCardNumber, DecimalMax, DecimalMin, Digits, Email,
    Future, FutureOrPresent, Length, Max, Min, Negative, NegativeOrZero, NotBlank,
    NotEmpty, NotNull, Past, PastOrPresent, Pattern, Positive, PositiveOrZero, Size,
    Url,
};

// Re-export nested validation / 重新导出嵌套验证
pub use nested::{Nested, ValidateNested};

// Re-export custom validators / 重新导出自定义验证器
pub use custom::{
    CompositeValidator, ConditionalValidator, CustomValidator, FieldMatchValidator,
    FieldProvider, ValidationReport, ValidationErrorExt, ValidatorRegistry,
};
pub use custom::field_match;

use std::fmt;

/// 验证结果 / Validation result
pub type ValidationResult<T> = Result<T, ValidationErrors>;

/// 验证上下文 / Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// 字段名 / Field name
    pub field: String,
    /// 字段值 / Field value
    pub value: String,
    /// 自定义消息 / Custom message
    pub message: Option<String>,
    /// 代码 / Code
    pub code: String,
}

impl ValidationContext {
    /// Create a new validation context
    /// 创建新的验证上下文
    pub fn new(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
            message: None,
            code: "validation_failed".to_string(),
        }
    }

    /// Set custom message
    /// 设置自定义消息
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set error code
    /// 设置错误代码
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }
}

/// 验证规则 / Validation rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRule {
    /// 非空 / Not empty
    NotEmpty,
    /// 长度范围 / Length range
    Length {
        /// Minimum length
        /// 最小长度
        min: Option<usize>,
        /// Maximum length
        /// 最大长度
        max: Option<usize>,
    },
    /// 数值范围 / Range
    Range {
        /// Minimum value
        /// 最小值
        min: Option<i64>,
        /// Maximum value
        /// 最大值
        max: Option<i64>
    },
    /// 邮箱 / Email
    Email,
    /// URL
    Url,
    /// 正则表达式 / Regex
    Regex(&'static str),
    /// 自定义 / Custom
    Custom(&'static str),
}

impl fmt::Display for ValidationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationRule::NotEmpty => write!(f, "not_empty"),
            ValidationRule::Length { min, max } => {
                write!(f, "length(min={:?}, max={:?})", min, max)
            },
            ValidationRule::Range { min, max } => {
                write!(f, "range(min={:?}, max={:?})", min, max)
            },
            ValidationRule::Email => write!(f, "email"),
            ValidationRule::Url => write!(f, "url"),
            ValidationRule::Regex(pattern) => write!(f, "regex({})", pattern),
            ValidationRule::Custom(name) => write!(f, "custom({})", name),
        }
    }
}
