//! JSON extractor module
//! JSON提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Json<T>` - `@RequestBody`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::Json;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! // POST /users
//! // Content-Type: application/json
//! // Body: {"name": "John", "email": "john@example.com"}
//! async fn create_user(Json(user): Json<CreateUser>) -> String {
//!     format!("Created user: {}", user.name)
//! }
//! ```

use hiver_http::HttpBody;
use serde::Deserialize;

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};

/// JSON body extractor
/// JSON body提取器
///
/// Equivalent to Spring's `@RequestBody` with JSON content.
/// 等价于Spring的`@RequestBody`与JSON内容。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to deserialize from JSON. Must implement `Deserialize`. 要从 JSON
///   反序列化的类型。必须实现 `Deserialize`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// async fn create_user(Json(user): Json<User>) -> String {
///     format!("User: {}", user.name)
/// }
/// ```
pub struct Json<T>(pub T);

impl<T> Json<T>
{
    /// Consume the JSON extractor and get the inner value
    /// 消耗JSON提取器并获取内部值
    pub fn into_inner(self) -> T
    {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.0
    }

    /// Get mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T
    {
        &mut self.0
    }
}

impl<T> std::fmt::Debug for Json<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_tuple("Json").field(&self.0).finish()
    }
}

impl<T> Clone for Json<T>
where
    T: Clone,
{
    fn clone(&self) -> Self
    {
        Self(self.0.clone())
    }
}

// Implement FromRequest for deserializable types
impl<T> FromRequest for Json<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self>
    {
        let body_bytes = req.body().as_bytes().map(<[u8]>::to_vec);
        let content_type = req.header("content-type").unwrap_or("").to_string();

        Box::pin(async move {
            // Validate content type
            if !content_type.starts_with("application/json")
                && !content_type.starts_with("application/")
                && !content_type.is_empty()
            {
                return Err(ExtractorError::Invalid(format!(
                    "Expected JSON content type, got: {}",
                    content_type
                )));
            }

            let body = body_bytes.ok_or_else(|| {
                ExtractorError::Invalid("Request body is not available".to_string())
            })?;

            // Parse JSON
            serde_json::from_slice::<T>(&body)
                .map(Json)
                .map_err(ExtractorError::from)
        })
    }
}

/// Get content type from request
/// 从请求获取content type
pub fn get_content_type(req: &Request) -> String
{
    req.header("content-type").unwrap_or("").to_string()
}

/// Maximum JSON body size (default: 10MB)
/// 最大JSON body大小（默认：10MB）
pub const DEFAULT_JSON_LIMIT: usize = 10 * 1024 * 1024;

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_json_into_inner()
    {
        let json: Json<String> = Json("test".to_string());
        assert_eq!(json.into_inner(), "test");
    }
}
