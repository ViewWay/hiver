//! Form extractor module
//! 表单提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Form<T>` - `@ModelAttribute`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::Form;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct LoginForm {
//!     username: String,
//!     password: String,
//! }
//!
//! // POST /login
//! // Content-Type: application/x-www-form-urlencoded
//! // Body: username=john&password=secret
//! async fn login(Form(form): Form<LoginForm>) -> String {
//!     format!("Login attempt: {}", form.username)
//! }
//! ```

use std::collections::HashMap;

use hiver_http::HttpBody;
use serde::Deserialize;

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};

/// Form data extractor
/// 表单数据提取器
///
/// Equivalent to Spring's `@ModelAttribute` for form data.
/// 等价于Spring的表单数据`@ModelAttribute`。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to deserialize from form data. Must implement `Deserialize`.
///   要从表单数据反序列化的类型。必须实现 `Deserialize`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
/// }
///
/// async fn login(Form(form): Form<LoginForm>) -> String {
///     format!("User: {}", form.username)
/// }
/// ```
pub struct Form<T>(pub T);

impl<T> Form<T> {
    /// Consume the form extractor and get the inner value
    /// 消耗表单提取器并获取内部值
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

impl<T> std::fmt::Debug for Form<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Form").field(&self.0).finish()
    }
}

impl<T> Clone for Form<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement FromRequest for deserializable types
impl<T> FromRequest for Form<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let body_bytes = req.body().as_bytes().map(<[u8]>::to_vec);
        let content_type = req.header("content-type").unwrap_or("").to_string();

        Box::pin(async move {
            // Validate content type
            if !content_type.starts_with("application/x-www-form-urlencoded")
                && !content_type.starts_with("multipart/form-data")
                && !content_type.is_empty()
            {
                return Err(ExtractorError::Invalid(format!(
                    "Expected form content type, got: {}",
                    content_type
                )));
            }

            let body = body_bytes.ok_or_else(|| {
                ExtractorError::Invalid("Request body is not available".to_string())
            })?;

            let body_str = String::from_utf8(body)
                .map_err(|_| ExtractorError::Invalid("Invalid UTF-8 in body".to_string()))?;

            // Parse form data
            let form_params = parse_form_data(&body_str);

            serde_json::to_value(&form_params)
                .and_then(|v| serde_json::from_value(v))
                .map(Form)
                .map_err(ExtractorError::from)
        })
    }
}

/// Parse form data into a map
/// 将表单数据解析为映射
pub fn parse_form_data(body: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in body.split('&') {
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
                && let Some(decoded) = char::from_u32(byte as u32)
            {
                result.push(decoded);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Get content type from request
/// 从请求获取content type
pub fn get_content_type(req: &Request) -> String {
    req.header("content-type").unwrap_or("").to_string()
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
    fn test_parse_form_data() {
        let form = "username=John&password=secret&remember=on";
        let params = parse_form_data(form);

        assert_eq!(params.get("username"), Some(&"John".to_string()));
        assert_eq!(params.get("password"), Some(&"secret".to_string()));
        assert_eq!(params.get("remember"), Some(&"on".to_string()));
    }

    #[test]
    fn test_url_decode_form() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("user%40email.com"), "user@email.com");
    }
}
