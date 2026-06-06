//! JSON response module
//! JSON响应模块
//!
//! # Overview / 概述
//!
//! This module provides JSON response types with automatic serialization.
//! 本模块提供具有自动序列化的JSON响应类型。

use serde::Serialize;

use crate::response::{IntoResponse, Response};

/// JSON response wrapper
/// JSON响应包装器
///
/// Wraps a value that will be serialized as JSON.
/// 包装将被序列化为JSON的值。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_response::Json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let user = User {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// let response = Json(user); // Automatically serializes to JSON
/// ```
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T>
{
    fn into_response(self) -> Response
    {
        match serde_json::to_vec(&self.0)
        {
            Ok(json) => Response::builder()
                .header("content-type", "application/json")
                .body(json)
                .unwrap_or_else(|_| Response::new()),
            Err(_) => Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to serialize JSON")
                .unwrap_or_else(|_| Response::new()),
        }
    }
}

impl<T> Json<T>
{
    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.0
    }

    /// Get a mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T
    {
        &mut self.0
    }

    /// Unwrap the inner value
    /// 解包内部值
    pub fn into_inner(self) -> T
    {
        self.0
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_json_response()
    {
        #[derive(Serialize)]
        struct TestData
        {
            message: String,
            count: u32,
        }

        let data = TestData {
            message: "Hello".to_string(),
            count: 42,
        };

        let response = Json(data).into_response();
        assert_eq!(response.status(), http::StatusCode::OK);

        let body_str = std::str::from_utf8(response.body()).unwrap();
        assert!(body_str.contains("\"message\":\"Hello\""));
        assert!(body_str.contains("\"count\":42"));
    }
}
