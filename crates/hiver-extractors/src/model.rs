//! Model attribute extractor module
//! 模型属性提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `ModelAttribute<T>` - `@ModelAttribute`
//!
//! # Description / 描述
//!
//! In Spring Boot, `@ModelAttribute` can bind data from multiple sources:
//! 1. Query parameters (URL parameters)
//! 2. Form data (application/x-www-form-urlencoded)
//! 3. Multipart form data
//!
//! This extractor combines both query parameters and form data,
//! with form data taking precedence over query parameters.
//!
//! 在 Spring Boot 中，`@ModelAttribute` 可以从多个来源绑定数据：
//! 1. 查询参数（URL参数）
//! 2. 表单数据（application/x-www-form-urlencoded）
//! 3. 多部分表单数据
//!
//! 该提取器结合了查询参数和表单数据，表单数据优先于查询参数。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::ModelAttribute;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct SearchForm {
//!     query: String,
//!     page: Option<u32>,
//!     sort: Option<String>,
//! }
//!
//! // GET /search?query=rust&page=1
//! // or POST /search with body: query=rust&page=1
//! async fn search(ModelAttribute(form): ModelAttribute<SearchForm>) -> String {
//!     format!("Searching for: {}", form.query)
//! }
//! ```

use std::collections::HashMap;

use hiver_http::HttpBody;
use serde::Deserialize;

use crate::{
    ExtractorError, ExtractorFuture, FromRequest, Request,
    form::{parse_form_data, url_decode},
};

/// Model attribute extractor
/// 模型属性提取器
///
/// Equivalent to Spring's `@ModelAttribute` annotation.
/// Combines data from query parameters and form body.
/// 等价于Spring的`@ModelAttribute`注解。结合来自查询参数和表单主体的数据。
///
/// # Binding Priority / 绑定优先级
///
/// 1. Form body data (if present) / 表单主体数据（如果存在）
/// 2. Query parameters / 查询参数
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to deserialize. Must implement `Deserialize`. 要反序列化的类型。必须实现
///   `Deserialize`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UserForm {
///     name: String,
///     email: String,
///     age: Option<u32>,
/// }
///
/// async fn create_user(ModelAttribute(form): ModelAttribute<UserForm>) -> String {
///     format!("Creating user: {}", form.name)
/// }
/// ```
pub struct ModelAttribute<T>(pub T);

impl<T> ModelAttribute<T> {
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

impl<T> std::fmt::Debug for ModelAttribute<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ModelAttribute").field(&self.0).finish()
    }
}

impl<T> Clone for ModelAttribute<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement FromRequest for deserializable types
impl<T> FromRequest for ModelAttribute<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        // Extract query parameters
        let uri = req.uri().to_string();
        let query_params = parse_query_params(&uri);

        // Extract form data from body (if present)
        let body_bytes = req.body().as_bytes().map(<[u8]>::to_vec);
        let content_type = req.header("content-type").unwrap_or("").to_string();
        let has_form_body = content_type.starts_with("application/x-www-form-urlencoded");

        Box::pin(async move {
            let mut merged_params = query_params;

            // Merge form data if present (form data takes precedence)
            if has_form_body && let Some(body) = body_bytes {
                let body_str = String::from_utf8(body)
                    .map_err(|_| ExtractorError::Invalid("Invalid UTF-8 in body".to_string()))?;

                let form_params = parse_form_data(&body_str);
                for (key, value) in form_params {
                    merged_params.insert(key, value);
                }
            }

            // Deserialize merged parameters
            serde_json::to_value(&merged_params)
                .and_then(|v| serde_json::from_value(v))
                .map(ModelAttribute)
                .map_err(ExtractorError::from)
        })
    }
}

/// Parse query parameters from URI
/// 从URI解析查询参数
pub fn parse_query_params(uri: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if let Some(query_start) = uri.find('?') {
        let query = &uri[query_start + 1..];
        // Strip fragment if present
        let query = query.split('#').next().unwrap_or(query);

        for pair in query.split('&') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };

            let key = url_decode(key);
            let value = url_decode(value);

            params.insert(key, value);
        }
    }

    params
}

/// Model attribute that binds only from query parameters.
/// 仅从查询参数绑定的模型属性。
///
/// Similar to `ModelAttribute` but only considers query parameters,
/// not the request body.
/// 类似于 `ModelAttribute`，但仅考虑查询参数，不考虑请求主体。
///
/// # Equivalent to Spring Boot / 等价于 Spring Boot
///
/// - `@RequestParam` with `required = false` for individual fields
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to deserialize from query parameters. Must implement `Deserialize`.
///   要从查询参数反序列化的类型。必须实现 `Deserialize`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SearchQuery {
///     q: String,
///     page: Option<u32>,
/// }
///
/// async fn search(QueryParams(query): QueryParams<SearchQuery>) -> String {
///     format!("Searching for: {}", query.q)
/// }
/// ```
pub struct QueryParams<T>(pub T);

impl<T> QueryParams<T> {
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

impl<T> std::fmt::Debug for QueryParams<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("QueryParams").field(&self.0).finish()
    }
}

impl<T> Clone for QueryParams<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> FromRequest for QueryParams<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let uri = req.uri().to_string();

        Box::pin(async move {
            let params = parse_query_params(&uri);

            serde_json::to_value(&params)
                .and_then(|v| serde_json::from_value(v))
                .map(QueryParams)
                .map_err(ExtractorError::from)
        })
    }
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
    fn test_parse_query_params() {
        let uri = "/search?q=rust&page=1&sort=desc";
        let params = parse_query_params(uri);

        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_parse_query_params_empty() {
        let uri = "/search";
        let params = parse_query_params(uri);

        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_query_params_with_fragment() {
        let uri = "/search?q=rust#section";
        let params = parse_query_params(uri);

        assert_eq!(params.get("q"), Some(&"rust".to_string()));
    }

    #[test]
    fn test_parse_query_params_encoded() {
        let uri = "/search?q=hello%20world&email=user%40example.com";
        let params = parse_query_params(uri);

        assert_eq!(params.get("q"), Some(&"hello world".to_string()));
        assert_eq!(params.get("email"), Some(&"user@example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params_empty_value() {
        let uri = "/search?q=&page=1";
        let params = parse_query_params(uri);

        assert_eq!(params.get("q"), Some(&String::new()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
    }

    #[test]
    fn test_parse_query_params_no_value() {
        let uri = "/search?q&flag";
        let params = parse_query_params(uri);

        assert_eq!(params.get("q"), Some(&String::new()));
        assert_eq!(params.get("flag"), Some(&String::new()));
    }
}
