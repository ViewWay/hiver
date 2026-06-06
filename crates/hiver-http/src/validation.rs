//! Validation extractor for HTTP requests
//! HTTP 请求的验证提取器
//!
//! # Overview / 概述
//!
//! This module provides automatic validation for request parameters using Bean Validation style
//! annotations. 本模块提供使用 Bean Validation 风格注解的请求参数自动验证。
//!
//! # Features / 功能
//!
//! - `#[Valid]` attribute for automatic validation
//! - `#[Valid]` 属性用于自动验证
//! - Integration with hiver-validation-annotations 与 hiver-validation-annotations 集成
//! - Type-safe validation errors 类型安全的验证错误

use std::{future::Future, pin::Pin};

use regex;
use serde::Deserialize;

use crate::{body::HttpBody, error::Error, request::Request};

/// Validation error details
/// 验证错误详情
#[derive(Debug, Clone)]
pub struct ValidationError
{
    /// Field name that failed validation
    /// 验证失败的字段名
    pub field: String,

    /// Error message
    /// 错误消息
    pub message: String,

    /// Invalid value (if available)
    /// 无效值（如果可用）
    pub value: Option<String>,
}

impl ValidationError
{
    /// Create a new validation error
    /// 创建新的验证错误
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self
    {
        Self {
            field: field.into(),
            message: message.into(),
            value: None,
        }
    }

    /// Create a validation error with value
    /// 创建带值的验证错误
    pub fn with_value(
        field: impl Into<String>,
        message: impl Into<String>,
        value: impl Into<String>,
    ) -> Self
    {
        Self {
            field: field.into(),
            message: message.into(),
            value: Some(value.into()),
        }
    }
}

impl std::fmt::Display for ValidationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Validation failed on field '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Collection of validation errors
/// 验证错误集合
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors
{
    /// Individual validation errors
    /// 单个验证错误
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors
{
    /// Create new validation errors
    /// 创建新的验证错误
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a validation error
    /// 添加验证错误
    pub fn add(&mut self, error: ValidationError)
    {
        self.errors.push(error);
    }

    /// Check if there are any errors
    /// 检查是否有任何错误
    pub fn has_errors(&self) -> bool
    {
        !self.errors.is_empty()
    }

    /// Get the number of errors
    /// 获取错误数量
    pub fn error_count(&self) -> usize
    {
        self.errors.len()
    }

    /// Convert to HTTP error
    /// 转换为 HTTP 错误
    pub fn to_http_error(&self) -> Error
    {
        let error_messages: Vec<String> = self.errors.iter().map(ToString::to_string).collect();

        Error::bad_request(format!("Validation failed: {}", error_messages.join(", ")))
    }
}

impl std::fmt::Display for ValidationErrors
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if self.errors.len() == 1
        {
            let first = self.errors.first().ok_or(std::fmt::Error)?;
            write!(f, "{first}")
        }
        else
        {
            let messages: Vec<String> = self.errors.iter().map(ToString::to_string).collect();
            write!(f, "Multiple validation errors: {}", messages.join(", "))
        }
    }
}

impl std::error::Error for ValidationErrors {}

/// Validated wrapper type
/// 验证包装类型
///
/// This type wraps a value that has been validated.
/// 这个类型包装一个已验证的值。
#[derive(Debug, Clone)]
pub struct Validated<T>
{
    /// The validated value
    /// 已验证的值
    pub inner: T,
}

impl<T> Validated<T>
{
    /// Create a new validated value
    /// 创建新的已验证值
    pub fn new(inner: T) -> Self
    {
        Self { inner }
    }

    /// Get the inner value
    /// 获取内部值
    pub fn into_inner(self) -> T
    {
        self.inner
    }

    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.inner
    }

    /// Get a mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T
    {
        &mut self.inner
    }
}

impl<T> std::ops::Deref for Validated<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Validated<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

/// Validatable trait for types that can be validated
/// 可验证类型的 trait
///
/// Types implement this trait to provide custom validation logic.
/// 类型实现此 trait 以提供自定义验证逻辑。
pub trait Validatable: Sized
{
    /// Validate this value
    /// 验证此值
    ///
    /// Returns Ok if validation passes, Err with validation errors if it fails.
    /// 如果验证通过返回 Ok，如果失败则返回 Err 和验证错误。
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Simple validation extractor for request parameters
/// 请求参数的简单验证提取器
///
/// This extractor can be used as a layer in HTTP handlers to automatically
/// validate request parameters.
///
/// 这个提取器可以用作 HTTP 处理器中的一层来自动验证请求参数。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::{Valid, Validated};
/// use hiver_http::validation::ValidatableExtractor;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// struct CreateUserRequest {
///     username: String,
///     email: String,
///     age: i32,
/// }
///
/// impl Validatable for CreateUserRequest {
///     fn validate(&self) -> Result<(), ValidationErrors> {
///         let mut errors = ValidationErrors::new();
///
///         if self.username.is_empty() {
///             errors.add(ValidationError::new("username", "Username is required"));
///         }
///
///         if self.username.len() < 3 {
///             errors.add(ValidationError::with_value(
///                 "username",
///                 "Username must be at least 3 characters",
///                 format!("{}", self.username.len())
///             ));
///         }
///
///         if !self.email.contains('@') {
///             errors.add(ValidationError::new("email", "Invalid email format"));
///         }
///
///         if self.age < 18 {
///             errors.add(ValidationError::with_value(
///                 "age",
///                 "Must be at least 18 years old",
///                 format!("{}", self.age)
///             ));
///         }
///
///         if errors.has_errors() {
///             return Err(errors);
///         }
///
///         Ok(())
///     }
/// }
///
/// // Use in HTTP handler
/// #[post("/users")]
/// async fn create_user(
///     #[Valid] req: Validated<CreateUserRequest>,
/// ) -> Result<Json<User>, Error> {
///     // req is automatically validated before this function executes
///     // req 在此函数执行前自动验证
///     let user = service.create(req.into_inner()).await?;
///     Ok(Json(user))
/// }
/// ```
pub trait ValidatableExtractor<T>
{
    /// Extract and validate a value from the request
    /// 从请求中提取并验证值
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::validation::ValidatableExtractor;
    ///
    /// async fn extract_validated<T>(
    ///     req: &Request
    /// ) -> Result<Validated<T>, Error>
    /// where
    ///     T: Validatable + for<'de> Deserialize<'de> + Send + Sync
    /// {
    ///     // Extract from request body
    ///     let value: T = extract_json_body(req).await?;
    ///
    ///     // Validate
    ///     value.validate()?;
    ///
    ///     Ok(Validated::new(value))
    /// }
    /// ```
    fn extract_validated(
        req: &Request,
    ) -> Pin<Box<dyn Future<Output = Result<Validated<T>, Error>> + Send + '_>>;
}

/// Generic extractor for validating request body
/// 验证请求体的通用提取器
///
/// This implementation provides validation for JSON request bodies.
/// 此实现为 JSON 请求体提供验证。
pub struct JsonValidator<T>
{
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonValidator<T>
where
    T: Validatable + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Extract and validate JSON from request body
    /// 从请求体中提取并验证 JSON
    pub fn from_request(req: &Request) -> Result<Validated<T>, Error>
    {
        // Extract JSON body
        let json_bytes = req
            .body()
            .as_bytes()
            .ok_or_else(|| Error::bad_request("Request body is not available".to_string()))?;

        // Deserialize
        let value: T = serde_json::from_slice(json_bytes)
            .map_err(|e| Error::bad_request(format!("Failed to parse JSON: {}", e)))?;

        // Validate
        value.validate().map_err(|errors| errors.to_http_error())?;

        Ok(Validated::new(value))
    }
}

/// Validation middleware
/// 验证中间件
///
/// This middleware can be applied to routes to automatically validate request parameters.
/// 此中间件可以应用于路由以自动验证请求参数。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::validation::{ValidatableExtractor, ValidationMiddleware};
///
/// let app = Router::new()
///     .route("/users", create_user)
///     .layer(ValidationMiddleware::new())
///     .build();
///
/// async fn create_user(
///     #[Valid] req: Validated<CreateUserRequest>,
/// ) -> Result<Json<User>, Error> {
///     // req is automatically validated
///     let user = service.create(req.into_inner()).await?;
///     Ok(Json(user))
/// }
/// ```
pub struct ValidationMiddleware;

impl ValidationMiddleware
{
    /// Create a new validation middleware
    /// 创建新的验证中间件
    pub fn new() -> Self
    {
        Self
    }

    /// Process a request through validation
    /// 通过验证处理请求
    pub fn validate_request<T>(&self, req: &Request) -> Result<Validated<T>, Error>
    where
        T: Validatable + for<'de> Deserialize<'de> + Send + Sync,
    {
        JsonValidator::from_request(req)
    }
}

impl Default for ValidationMiddleware
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Validation helper functions
/// 验证辅助函数
///
/// These functions provide common validation logic that can be used in
/// Validatable implementations.
///
/// 这些函数提供可在 Validatable 实现中使用的通用验证逻辑。
pub struct ValidationHelpers;

impl ValidationHelpers
{
    /// Validate that a string is not empty
    /// 验证字符串不为空
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::validation::ValidationHelpers;
    /// use hiver_http::ValidationError;
    ///
    /// let username = "";
    /// if let Some(err) = ValidationHelpers::require_non_empty("username", username) {
    ///     println!("Validation error: {:?}", err);
    /// }
    /// ```
    pub fn require_non_empty(field: &str, value: &str) -> Option<ValidationError>
    {
        if value.trim().is_empty()
        {
            Some(ValidationError::new(field, "Field is required"))
        }
        else
        {
            None
        }
    }

    /// Validate minimum length
    /// 验证最小长度
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::validation::ValidationHelpers;
    /// use hiver_http::ValidationError;
    ///
    /// let username = "ab"; // Too short
    /// if let Some(err) = ValidationHelpers::require_min_length("username", username, 3) {
    ///     println!("Validation error: {:?}", err);
    /// }
    /// ```
    pub fn require_min_length(field: &str, value: &str, min: usize) -> Option<ValidationError>
    {
        if value.len() < min
        {
            Some(ValidationError::with_value(
                field,
                format!("Must be at least {} characters", min),
                value.len().to_string(),
            ))
        }
        else
        {
            None
        }
    }

    /// Validate maximum length
    /// 验证最大长度
    pub fn require_max_length(field: &str, value: &str, max: usize) -> Option<ValidationError>
    {
        if value.len() > max
        {
            Some(ValidationError::with_value(
                field,
                format!("Must be at most {} characters", max),
                value.len().to_string(),
            ))
        }
        else
        {
            None
        }
    }

    /// Validate email format (simple check)
    /// 验证邮箱格式（简单检查）
    pub fn require_email_format(field: &str, value: &str) -> Option<ValidationError>
    {
        if !value.contains('@') || !value.contains('.')
        {
            Some(ValidationError::new(field, "Invalid email format"))
        }
        else
        {
            None
        }
    }

    /// Validate minimum value for numbers
    /// 验证数字的最小值
    pub fn require_min<T>(field: &str, value: &T, min: &T) -> Option<ValidationError>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value < min
        {
            Some(ValidationError::with_value(
                field,
                format!("Must be at least {}", min),
                value.to_string(),
            ))
        }
        else
        {
            None
        }
    }

    /// Validate maximum value for numbers
    /// 验证数字的最大值
    pub fn require_max<T>(field: &str, value: &T, max: &T) -> Option<ValidationError>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value > max
        {
            Some(ValidationError::with_value(
                field,
                format!("Must be at most {}", max),
                value.to_string(),
            ))
        }
        else
        {
            None
        }
    }

    /// Validate that a value matches a regex pattern
    /// 验证值匹配正则表达式模式
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::validation::ValidationHelpers;
    /// use hiver_http::ValidationError;
    ///
    /// let username = "user@123"; // Contains invalid character
    /// if let Some(err) = ValidationHelpers::require_pattern(
    ///     "username",
    ///     username,
    ///     r"^[a-zA-Z0-9_]+$"
    /// ) {
    ///     println!("Validation error: {:?}", err);
    /// }
    /// ```
    pub fn require_pattern(field: &str, value: &str, pattern: &str) -> Option<ValidationError>
    {
        match regex::Regex::new(pattern)
        {
            Ok(re) =>
            {
                if re.is_match(value)
                {
                    None
                }
                else
                {
                    Some(ValidationError::new(field, "Does not match required pattern"))
                }
            },
            Err(_) => Some(ValidationError::new(field, "Invalid regex pattern")),
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_validation_error_creation()
    {
        let error = ValidationError::new("username", "Username is required");
        assert_eq!(error.field, "username");
        assert_eq!(error.message, "Username is required");
    }

    #[test]
    fn test_validation_error_with_value()
    {
        let error = ValidationError::with_value("age", "Must be 18+", "17");
        assert_eq!(error.field, "age");
        assert_eq!(error.message, "Must be 18+");
        assert_eq!(error.value, Some("17".to_string()));
    }

    #[test]
    fn test_validation_errors()
    {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("field1", "Error 1"));
        errors.add(ValidationError::new("field2", "Error 2"));

        assert_eq!(errors.error_count(), 2);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_validated_wrapper()
    {
        let validated = Validated::new(42);
        assert_eq!(*validated, 42);
        assert_eq!(validated.into_inner(), 42);
    }

    #[test]
    fn test_require_non_empty()
    {
        assert!(ValidationHelpers::require_non_empty("field", "value").is_none());
        assert!(ValidationHelpers::require_non_empty("field", "").is_some());
    }

    #[test]
    fn test_require_min_length()
    {
        assert!(ValidationHelpers::require_min_length("field", "abc", 3).is_none());
        assert!(ValidationHelpers::require_min_length("field", "ab", 3).is_some());
    }

    #[test]
    fn test_require_max_length()
    {
        assert!(ValidationHelpers::require_max_length("field", "abc", 5).is_none());
        assert!(ValidationHelpers::require_max_length("field", "abcdef", 5).is_some());
    }

    #[test]
    fn test_require_email_format()
    {
        assert!(ValidationHelpers::require_email_format("email", "user@example.com").is_none());
        assert!(ValidationHelpers::require_email_format("email", "invalid").is_some());
    }

    #[test]
    fn test_require_min()
    {
        assert!(ValidationHelpers::require_min("age", &18u32, &18).is_none());
        assert!(ValidationHelpers::require_min("age", &17u32, &18).is_some());
    }

    #[test]
    fn test_require_max()
    {
        assert!(ValidationHelpers::require_max("age", &100u32, &100).is_none());
        assert!(ValidationHelpers::require_max("age", &101u32, &100).is_some());
    }

    #[test]
    fn test_require_pattern()
    {
        assert!(
            ValidationHelpers::require_pattern("username", "user123", r"^[a-zA-Z0-9_]+$").is_none()
        );

        assert!(
            ValidationHelpers::require_pattern("username", "user@123", r"^[a-zA-Z0-9_]+$")
                .is_some()
        );
    }
}
