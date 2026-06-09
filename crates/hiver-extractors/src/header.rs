//! Header extractor module
//! Header提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Header<T>` - `@RequestHeader`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::Header;
//!
//! // GET /resource
//! // Header: Authorization: Bearer token123
//! async fn get_resource(Header(auth): Header<String>) -> String {
//!     format!("Auth: {}", auth)
//! }
//! ```

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};

/// Header extractor
/// Header提取器
///
/// Equivalent to Spring's `@RequestHeader`.
/// 等价于Spring的`@RequestHeader`。
///
/// Extracts the first header value from the request.
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to convert the header value to. 要将 header 值转换成的类型。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::Header;
///
/// async fn handler(Header(auth): Header<String>) -> String {
///     format!("Auth: {}", auth)
/// }
///
/// // With default value
/// use hiver_extractors::HeaderOption;
///
/// async fn handler(HeaderOption(auth): HeaderOption<String>) -> String {
///     format!("Auth: {:?}", auth)
/// }
/// ```
pub struct Header<T>(pub T);

impl<T> Header<T> {
    /// Consume the header extractor and get the inner value
    /// 消耗header提取器并获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Debug for Header<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Header").field(&self.0).finish()
    }
}

impl<T> Clone for Header<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement FromRequest for String - extracts first header
impl FromRequest for Header<String> {
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        // Get the first header value
        let value = req
            .headers()
            .iter()
            .next()
            .and_then(|(name, _)| req.headers().get(name))
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        Box::pin(async move {
            value
                .map(Header)
                .ok_or_else(|| ExtractorError::Missing("header".to_string()))
        })
    }
}

/// Optional header extractor
/// 可选header提取器
///
/// Equivalent to Spring's `@RequestHeader(required = false)`.
/// 等价于Spring的`@RequestHeader(required = false)`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::HeaderOption;
///
/// async fn handler(HeaderOption(auth): HeaderOption<String>) -> String {
///     match auth {
///         Some(a) => format!("Auth: {}", a),
///         None => "No auth".to_string(),
///     }
/// }
/// ```
pub struct HeaderOption<T>(pub Option<T>);

impl<T> HeaderOption<T> {
    /// Consume the header extractor and get the inner value
    /// 消耗header提取器并获取内部值
    pub fn into_inner(self) -> Option<T> {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }
}

impl<T> std::fmt::Debug for HeaderOption<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HeaderOption").field(&self.0).finish()
    }
}

impl<T> Clone for HeaderOption<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl FromRequest for HeaderOption<String> {
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let value = req
            .headers()
            .iter()
            .next()
            .and_then(|(name, _)| req.headers().get(name))
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        Box::pin(async move { Ok(HeaderOption(value)) })
    }
}

/// Header value with name
/// 带名称的header值
///
/// Equivalent to Spring's `@RequestHeader("Authorization")`.
/// 等价于Spring的`@RequestHeader("Authorization")`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::NamedHeader;
///
/// async fn handler(NamedHeader(auth): NamedHeader<String>) -> String {
///     format!("Header {}: {}", auth.name, auth.value)
/// }
/// ```
pub struct NamedHeader<T> {
    /// Header name
    /// Header名称
    pub name: String,

    /// Header value
    /// Header值
    pub value: T,
}

impl<T> NamedHeader<T> {
    /// Create a new named header
    /// 创建新的命名header
    pub fn new(name: impl Into<String>, value: T) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Get reference to the value
    /// 获取值的引用
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Consume and get the inner value
    /// 消耗并获取内部值
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> std::fmt::Debug for NamedHeader<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedHeader")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl<T> Clone for NamedHeader<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

/// Helper to extract a specific header
/// 提取特定header的助手
///
/// Equivalent to Spring's `@RequestHeader("Authorization")`.
/// 等价于Spring的`@RequestHeader("Authorization")`。
pub fn get_header(req: &Request, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
}

/// Get all header values for a name
/// 获取名称的所有header值
pub fn get_all_headers(req: &Request, name: &str) -> Vec<String> {
    req.headers()
        .get_all(name)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(ToString::to_string)
        .collect()
}

/// Check if a header exists
/// 检查header是否存在
pub fn has_header(req: &Request, name: &str) -> bool {
    req.headers().get(name).is_some()
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_header_into_inner() {
        let header: Header<String> = Header("test".to_string());
        assert_eq!(header.into_inner(), "test");
    }

    #[test]
    fn test_header_option() {
        let header: HeaderOption<String> = HeaderOption(Some("test".to_string()));
        assert_eq!(header.into_inner(), Some("test".to_string()));
    }

    #[test]
    fn test_named_header() {
        let named = NamedHeader::new("Authorization", "Bearer token");
        assert_eq!(named.name, "Authorization");
        assert_eq!(named.value, "Bearer token");
    }
}
