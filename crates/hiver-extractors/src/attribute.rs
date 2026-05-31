//! Request attribute extractor module
//! 请求属性提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `RequestAttribute<T>` - `@RequestAttribute`
//!
//! # Description / 描述
//!
//! In Spring Boot, `@RequestAttribute` is used to access attributes that were
//! previously stored in the request, typically by filters, interceptors, or middleware.
//!
//! 在 Spring Boot 中，`@RequestAttribute` 用于访问之前存储在请求中的属性，
//! 通常由过滤器、拦截器或中间件设置。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::RequestAttribute;
//! use hiver_http::Request;
//! use std::sync::Arc;
//!
//! // In middleware or filter:
//! // req.extensions_mut().insert("userId", "user123");
//!
//! // In handler:
//! async fn get_profile(RequestAttribute(userId): RequestAttribute<String>) -> String {
//!     format!("User ID: {}", userId)
//! }
//! ```

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};
use std::sync::Arc;

/// Request attribute extractor
/// 请求属性提取器
///
/// Equivalent to Spring's `@RequestAttribute` annotation.
/// Retrieves values from the request's extensions.
/// 等价于Spring的`@RequestAttribute`注解。从请求的扩展中检索值。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type of the attribute to extract. Must be `Clone + Send + Sync + 'static`.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::Request;
///
/// // Setting an attribute (typically in middleware):
/// fn add_user_id(req: &mut Request) {
///     req.extensions_mut().insert("user_id".to_string(), "user123");
/// }
///
/// // Extracting in a handler:
/// use hiver_extractors::RequestAttribute;
///
/// async fn handler(RequestAttribute(user_id): RequestAttribute<String>) -> String {
///     format!("User: {}", user_id)
/// }
/// ```
pub struct RequestAttribute<T>(pub T);

impl<T> RequestAttribute<T> {
    /// Consume the extractor and get the inner value
    /// 消耗提取器并获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> std::fmt::Debug for RequestAttribute<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RequestAttribute").field(&self.0).finish()
    }
}

impl<T> Clone for RequestAttribute<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement FromRequest for types that can be retrieved from extensions
impl<T> FromRequest for RequestAttribute<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        // Try to get the value from extensions
        if let Some(value) = req.extensions().get::<T>() {
            let cloned = value.clone();
            return Box::pin(async move { Ok(RequestAttribute(cloned)) });
        }

        // Also try to get it wrapped in Arc (common pattern)
        if let Some(value) = req.extensions().get::<Arc<T>>() {
            let cloned = (**value).clone();
            return Box::pin(async move { Ok(RequestAttribute(cloned)) });
        }

        Box::pin(async move {
            Err(ExtractorError::Missing(format!(
                "Request attribute not found: {}",
                std::any::type_name::<T>()
            )))
        })
    }
}

/// Named request attribute extractor
/// 命名的请求属性提取器
///
/// Like `RequestAttribute` but with an explicit name/key.
/// Useful when storing values with string keys rather than type-based lookup.
/// 类似于`RequestAttribute`，但具有显式名称/键。当使用字符串键而不是基于类型的查找存储值时很有用。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::Request;
/// use hiver_extractors::NamedRequestAttribute;
///
/// // Setting an attribute:
/// req.extensions_mut().insert("user_id", "user123");
///
/// // Extracting by name:
/// async fn handler(NamedRequestAttribute(user_id): NamedRequestAttribute<String>) -> String {
///     format!("User: {}", user_id)
/// }
/// ```
pub struct NamedRequestAttribute<T>(pub T);

impl<T> NamedRequestAttribute<T> {
    /// Consume the extractor and get the inner value
    /// 消耗提取器并获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> std::fmt::Debug for NamedRequestAttribute<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NamedRequestAttribute")
            .field(&self.0)
            .finish()
    }
}

impl<T> Clone for NamedRequestAttribute<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_attribute_consume() {
        let attr = RequestAttribute("test_value".to_string());
        assert_eq!(attr.into_inner(), "test_value");
    }

    #[test]
    fn test_request_attribute_get() {
        let attr = RequestAttribute("test_value".to_string());
        assert_eq!(attr.get(), "test_value");
    }

    #[test]
    fn test_named_request_attribute_consume() {
        let attr = NamedRequestAttribute::<String>("user123".to_string());
        assert_eq!(attr.into_inner(), "user123");
    }
}
