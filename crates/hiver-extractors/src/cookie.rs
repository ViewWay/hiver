//! Cookie extractor module
//! Cookie提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Cookie<T>` - `@CookieValue`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::Cookie;
//!
//! // GET /resource
//! // Cookie: session_id=abc123
//! async fn get_resource(Cookie(session_id): Cookie<String>) -> String {
//!     format!("Session: {}", session_id)
//! }
//! ```

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};
use std::collections::HashMap;

/// Cookie extractor
/// Cookie提取器
///
/// Equivalent to Spring's `@CookieValue`.
/// 等价于Spring的`@CookieValue`。
///
/// Extracts the first cookie value from the request.
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to convert the cookie value to.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::Cookie;
///
/// async fn handler(Cookie(session): Cookie<String>) -> String {
///     format!("Session: {}", session)
/// }
///
/// // With default value
/// use hiver_extractors::CookieOption;
///
/// async fn handler(CookieOption(session): CookieOption<String>) -> String {
///     format!("Session: {:?}", session)
/// }
/// ```
pub struct Cookie<T>(pub T);

impl<T> Cookie<T> {
    /// Consume the cookie extractor and get the inner value
    /// 消耗cookie提取器并获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Debug for Cookie<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cookie").field(&self.0).finish()
    }
}

impl<T> Clone for Cookie<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement FromRequest for String - extracts first cookie
impl FromRequest for Cookie<String> {
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let cookies = parse_cookies(req);

        Box::pin(async move {
            cookies
                .values()
                .next()
                .map(|v| Cookie(v.clone()))
                .ok_or_else(|| ExtractorError::Missing("cookie".to_string()))
        })
    }
}

/// Optional cookie extractor
/// 可选cookie提取器
///
/// Equivalent to Spring's `@CookieValue(required = false)`.
/// 等价于Spring的`@CookieValue(required = false)`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::CookieOption;
///
/// async fn handler(CookieOption(session): CookieOption<String>) -> String {
///     match session {
///         Some(s) => format!("Session: {}", s),
///         None => "No session".to_string(),
///     }
/// }
/// ```
pub struct CookieOption<T>(pub Option<T>);

impl<T> CookieOption<T> {
    /// Consume the cookie extractor and get the inner value
    /// 消耗cookie提取器并获取内部值
    pub fn into_inner(self) -> Option<T> {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }
}

impl<T> std::fmt::Debug for CookieOption<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CookieOption").field(&self.0).finish()
    }
}

impl<T> Clone for CookieOption<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl FromRequest for CookieOption<String> {
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let cookies = parse_cookies(req);

        Box::pin(async move { Ok(CookieOption(cookies.values().next().cloned())) })
    }
}

/// Parse cookies from request
/// 从请求解析cookies
fn parse_cookies(req: &Request) -> HashMap<String, String> {
    let mut cookies = HashMap::new();

    if let Some(cookie_header) = req.headers().get("cookie")
        && let Ok(cookie_str) = cookie_header.to_str() {
            for pair in cookie_str.split(';') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') {
                    cookies.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

    cookies
}

/// Cookie value with name
/// 带名称的cookie值
///
/// Equivalent to Spring's `@CookieValue(value = "session_id")`.
/// 等价于Spring的`@CookieValue(value = "session_id")`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::NamedCookie;
///
/// async fn handler(NamedCookie(session): NamedCookie<String>) -> String {
///     format!("Session: {}", session.value)
/// }
/// ```
pub struct NamedCookie<T> {
    /// Cookie name
    /// Cookie名称
    pub name: String,

    /// Cookie value
    /// Cookie值
    pub value: T,
}

impl<T> NamedCookie<T> {
    /// Create a new named cookie
    /// 创建新的命名cookie
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

impl<T> std::fmt::Debug for NamedCookie<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedCookie")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl<T> Clone for NamedCookie<T>
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

/// Get a specific cookie by name
/// 按名称获取特定cookie
///
/// Equivalent to Spring's `@CookieValue("session_id")`.
/// 等价于Spring的`@CookieValue("session_id")`。
pub fn get_cookie(req: &Request, name: &str) -> Option<String> {
    let cookies = parse_cookies(req);
    cookies.get(name).cloned()
}

/// Get all cookies
/// 获取所有cookies
pub fn get_all_cookies(req: &Request) -> HashMap<String, String> {
    parse_cookies(req)
}

/// Check if a cookie exists
/// 检查cookie是否存在
pub fn has_cookie(req: &Request, name: &str) -> bool {
    get_cookie(req, name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cookies() {
        let cookie_str = "session=abc123; user=john; theme=dark";
        let mut cookies = HashMap::new();
        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if let Some((key, value)) = pair.split_once('=') {
                cookies.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        assert_eq!(cookies.get("session"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user"), Some(&"john".to_string()));
        assert_eq!(cookies.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_cookie_into_inner() {
        let cookie: Cookie<String> = Cookie("test".to_string());
        assert_eq!(cookie.into_inner(), "test");
    }

    #[test]
    fn test_named_cookie() {
        let named = NamedCookie::new("session", "abc123");
        assert_eq!(named.name, "session");
        assert_eq!(named.value, "abc123");
    }
}
