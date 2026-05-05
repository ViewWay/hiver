//! Query extractor module
//! 查询提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Query<T>` - `@RequestParam` / `@RequestParam`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_extractors::Query;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct SearchParams {
//!     q: String,
//!     page: Option<u32>,
//!     size: Option<u32>,
//! }
//!
//! // GET /search?q=rust&page=1&size=10
//! async fn search(Query(params): Query<SearchParams>) -> String {
//!     format!("Search: {}, Page: {:?}", params.q, params.page)
//! }
//! ```

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};
use serde::Deserialize;
use std::collections::HashMap;

/// Query parameter extractor
/// 查询参数提取器
///
/// Equivalent to Spring's `@RequestParam`.
/// 等价于Spring的`@RequestParam`。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to extract. Can be:
///   - A single value: `Query<String>`, `Query<u32>`, etc.
///   - A struct that implements `Deserialize` (most common)
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Params {
///     name: String,
///     age: Option<u32>,
/// }
///
/// async fn handler(Query(params): Query<Params>) -> String {
///     format!("Name: {}, Age: {:?}", params.name, params.age)
/// }
/// ```
pub struct Query<T>(pub T);

impl<T> Query<T> {
    /// Consume the query extractor and get the inner value
    /// 消耗查询提取器并获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Debug for Query<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Query").field(&self.0).finish()
    }
}

impl<T> Clone for Query<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement FromRequest for deserializable types
impl<T> FromRequest for Query<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let query_params = req.params().clone();

        Box::pin(async move {
            serde_json::to_value(&query_params)
                .and_then(|v| serde_json::from_value(v))
                .map(Query)
                .map_err(ExtractorError::from)
        })
    }
}

/// Parse query string into a map
/// 将查询字符串解析为映射
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };

        // URL decode the key and value
        let key = url_decode(key);
        let value = url_decode(value);

        params.insert(key, value);
    }

    params
}

/// Simple URL decode
/// 简单的URL解码
pub fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '+' {
            result.push(' ');
        } else if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2
                && let Ok(byte) = u8::from_str_radix(&hex, 16)
                    && let Some(decoded) = char::from_u32(byte as u32) {
                        result.push(decoded);
                    }
        } else {
            result.push(c);
        }
    }

    result
}

/// Single query parameter extractor
/// 单个查询参数提取器
///
/// Equivalent to Spring's `@RequestParam("name")`.
/// 等价于Spring的`@RequestParam("name")`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_extractors::query::Param;
///
/// async fn handler(Param(name): Param<String>) -> String {
///     format!("Name: {}", name)
/// }
/// ```
pub struct Param<T>(pub T);

impl<T> Param<T> {
    /// Consume and get inner value
    /// 消耗并获取内部值
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Debug for Param<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Param").field(&self.0).finish()
    }
}

impl<T> Clone for Param<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Optional query parameter
/// 可选查询参数
///
/// Equivalent to Spring's `@RequestParam(required = false)`.
/// 等价于Spring的`@RequestParam(required = false)`。
pub struct ParamOption<T>(pub Option<T>);

impl<T> ParamOption<T> {
    /// Consume and get inner value
    /// 消耗并获取内部值
    pub fn into_inner(self) -> Option<T> {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }
}

impl<T> std::fmt::Debug for ParamOption<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParamOption").field(&self.0).finish()
    }
}

impl<T> Clone for ParamOption<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Get a specific query parameter by name
/// 按名称获取特定查询参数
///
/// Equivalent to Spring's `@RequestParam("name")`.
/// 等价于Spring的`@RequestParam("name")`。
pub fn get_param(req: &Request, name: &str) -> Option<String> {
    req.param(name).map(std::string::ToString::to_string)
}

/// Get all query parameters
/// 获取所有查询参数
pub fn get_all_params(req: &Request) -> HashMap<String, String> {
    req.params().clone()
}

/// Check if a query parameter exists
/// 检查查询参数是否存在
pub fn has_param(req: &Request, name: &str) -> bool {
    req.param(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=John&age=30&city=";
        let params = parse_query_string(query);

        assert_eq!(params.get("name"), Some(&"John".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"".to_string()));
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a%2Bb"), "a+b");
        assert_eq!(url_decode("test%3F"), "test?");
    }
}
