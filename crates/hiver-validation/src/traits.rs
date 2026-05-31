//! 验证 trait / Validation traits
//!
//! # Validate Trait
//!
//! ```rust,ignore
//! use hiver_validation::Validate;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     username: String,
//!     email: String,
//! }
//!
//! impl Validate for CreateUserRequest {
//!     fn validate(&self) -> Result<(), hiver_validation::ValidationErrors> {
//!         let mut errors = hiver_validation::ValidationErrors::new();
//!         
//!         if self.username.is_empty() {
//!             errors.add("username", "Username is required");
//!         }
//!         
//!         if !errors.is_empty() {
//!             return Err(errors);
//!         }
//!         
//!         Ok(())
//!     }
//! }
//! ```

use crate::{ValidationContext, ValidationErrors, ValidationResult};
use std::fmt;

/// 验证 trait / Validation trait
///
/// 实现此 trait 以支持自定义验证逻辑
/// Implement this trait for custom validation logic
pub trait Validate: fmt::Debug {
    /// 执行验证 / Perform validation
    ///
    /// # 返回 / Returns
    ///
    /// - `Ok(())` - 验证通过 / Validation passed
    /// - `Err(ValidationErrors)` - 验证失败 / Validation failed
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// 字段验证 trait / Field validation trait
///
/// 用于验证单个字段的值
/// Used for validating individual field values
pub trait ValidateField {
    /// 验证字符串字段 / Validate string field
    fn validate_string(
        &self,
        field: &str,
        value: &str,
        context: &ValidationContext,
    ) -> Result<(), ValidationError>;

    /// 验证数值字段 / Validate numeric field
    fn validate_number(
        &self,
        field: &str,
        value: i64,
        context: &ValidationContext,
    ) -> Result<(), ValidationError>;
}

/// 字段级验证错误 / Field-level validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field name that failed validation
    /// 验证失败的字段名
    pub field: String,
    /// Error message describing the validation failure
    /// 描述验证失败原因的错误消息
    pub message: String,
    /// Error code for programmatic handling
    /// 用于程序化处理的错误代码
    pub code: String,
}

impl ValidationError {
    /// Create a new validation error
    /// 创建新的验证错误
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: "validation_failed".to_string(),
        }
    }

    /// Set a custom error code
    /// 设置自定义错误代码
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// 可选值验证 / Optional value validation
pub trait ValidateOptional {
    /// The wrapped item type
    /// 包装的项类型
    type Item;

    /// 验证可选值 / Validate optional value
    fn validate_optional<F>(self, validator: F) -> ValidationResult<Self::Item>
    where
        F: FnOnce(&Self::Item) -> Result<(), ValidationErrors>;
}

impl<T> ValidateOptional for Option<T> {
    type Item = T;

    fn validate_optional<F>(self, validator: F) -> ValidationResult<T>
    where
        F: FnOnce(&T) -> Result<(), ValidationErrors>,
    {
        match self {
            Some(value) => {
                validator(&value)?;
                Ok(value)
            },
            None => Err(ValidationErrors::new()),
        }
    }
}

/// 集合验证 / Collection validation
pub trait ValidateCollection {
    /// The item type in the collection
    /// 集合中的项类型
    type Item;

    /// 验证集合中的每个元素 / Validate each item in collection
    fn validate_each<F>(self, validator: F) -> ValidationResult<Vec<Self::Item>>
    where
        F: FnMut(&Self::Item) -> Result<(), ValidationErrors>;
}

impl<T, I> ValidateCollection for I
where
    I: IntoIterator<Item = T>,
{
    type Item = T;

    fn validate_each<F>(self, mut validator: F) -> ValidationResult<Vec<T>>
    where
        F: FnMut(&T) -> Result<(), ValidationErrors>,
    {
        let mut errors = ValidationErrors::new();
        let mut results = Vec::new();

        for item in self {
            if let Err(e) = validator(&item) {
                errors.merge(e);
            } else {
                results.push(item);
            }
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(results)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestRequest {
        username: String,
        email: String,
    }

    impl Validate for TestRequest {
        fn validate(&self) -> Result<(), ValidationErrors> {
            let mut errors = ValidationErrors::new();

            if self.username.is_empty() {
                errors.add("username", "Username is required");
            }

            if self.email.is_empty() {
                errors.add("email", "Email is required");
            }

            if errors.has_errors() {
                return Err(errors);
            }

            Ok(())
        }
    }

    #[test]
    fn test_validate_trait() {
        let request = TestRequest {
            username: "".to_string(),
            email: "".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_validate_optional() {
        let value: Option<String> = Some("test".to_string());
        let result = value.validate_optional(|v| {
            if v.is_empty() {
                let mut errors = ValidationErrors::new();
                errors.add("value", "Cannot be empty");
                Err(errors)
            } else {
                Ok(())
            }
        });

        assert!(result.is_ok());
    }
}
