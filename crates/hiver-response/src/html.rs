//! HTML response module
//! HTML响应模块
//!
//! # Overview / 概述
//!
//! This module provides HTML response types.
//! 本模块提供HTML响应类型。

use crate::response::{IntoResponse, Response};

/// HTML response wrapper
/// HTML响应包装器
///
/// Wraps a value that can be converted to a string for HTML responses.
/// 包装可转换为字符串以用于HTML响应的值。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_response::Html;
///
/// let html = Html(r#"
/// <!DOCTYPE html>
/// <html>
///     <body>
///         <h1>Hello, World!</h1>
///     </body>
/// </html>
/// "#);
///
/// // Sets content-type header to text/html; charset=utf-8
/// ```
pub struct Html<T>(pub T);

impl<T: AsRef<str>> IntoResponse for Html<T> {
    fn into_response(self) -> Response {
        // Convert the inner value to a string for the response body
        // 将内部值转换为字符串用于响应体
        //
        // We need to convert to an owned string because the response builder
        // requires 'static lifetime for the body content.
        // 我们需要转换为拥有的字符串，因为响应构建器要求主体内容具有 'static 生命周期。
        let body_str = self.0.as_ref();
        #[allow(clippy::unwrap_used)]
        Response::builder()
            .header_static("content-type", "text/html; charset=utf-8")
            .body(body_str.to_string())
            .unwrap()
    }
}

impl<T> Html<T> {
    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get a mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Unwrap the inner value
    /// 解包内部值
    pub fn into_inner(self) -> T {
        self.0
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
    fn test_html_response() {
        let html = Html("<h1>Hello</h1>");
        let response = html.into_response();

        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.body(), "<h1>Hello</h1>");
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");
    }
}
