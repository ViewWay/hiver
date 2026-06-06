//! HTTP Method type
//! HTTP 方法类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `RequestMethod`, `HttpMethod`
//! - @`GetMapping`, @`PostMapping`, etc.

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{fmt, str::FromStr};

/// HTTP Methods
/// HTTP 方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Method
{
    /// GET method - retrieve a resource
    /// GET 方法 - 获取资源
    #[default]
    GET,
    /// POST method - create a resource
    /// POST 方法 - 创建资源
    POST,
    /// PUT method - update/replace a resource
    /// PUT 方法 - 更新/替换资源
    PUT,
    /// PATCH method - partially update a resource
    /// PATCH 方法 - 部分更新资源
    PATCH,
    /// DELETE method - delete a resource
    /// DELETE 方法 - 删除资源
    DELETE,
    /// HEAD method - get headers only
    /// HEAD 方法 - 仅获取头信息
    HEAD,
    /// OPTIONS method - get allowed methods
    /// OPTIONS 方法 - 获取允许的方法
    OPTIONS,
    /// TRACE method - echo the request
    /// TRACE 方法 - 回显请求
    TRACE,
    /// CONNECT method - establish a tunnel
    /// CONNECT 方法 - 建立隧道
    CONNECT,
}

impl Method
{
    /// All standard HTTP methods
    /// 所有标准HTTP方法
    pub const ALL: [Method; 9] = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::HEAD,
        Method::OPTIONS,
        Method::TRACE,
        Method::CONNECT,
    ];

    /// Check if this method is safe (idempotent and doesn't modify state)
    /// 检查此方法是否安全（幂等且不修改状态）
    pub fn is_safe(&self) -> bool
    {
        matches!(self, Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE)
    }

    /// Check if this method is idempotent
    /// 检查此方法是否幂等
    pub fn is_idempotent(&self) -> bool
    {
        matches!(
            self,
            Method::GET
                | Method::HEAD
                | Method::PUT
                | Method::DELETE
                | Method::OPTIONS
                | Method::TRACE
        )
    }
}

/// Formats the method as an uppercase string (e.g. `"GET"`, `"POST"`).
/// 将方法格式化为大写字符串（例如 `"GET"`、`"POST"`）。
impl fmt::Display for Method
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::PATCH => write!(f, "PATCH"),
            Method::DELETE => write!(f, "DELETE"),
            Method::HEAD => write!(f, "HEAD"),
            Method::OPTIONS => write!(f, "OPTIONS"),
            Method::TRACE => write!(f, "TRACE"),
            Method::CONNECT => write!(f, "CONNECT"),
        }
    }
}

/// Parses a method string (case-insensitive). Unknown methods default to GET.
/// 解析方法字符串（不区分大小写）。未知方法默认为 GET。
impl FromStr for Method
{
    type Err = MethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s.to_uppercase().as_str()
        {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "PATCH" => Ok(Method::PATCH),
            "DELETE" => Ok(Method::DELETE),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "CONNECT" => Ok(Method::CONNECT),
            _ => Err(MethodError::InvalidMethod(s.to_string())),
        }
    }
}

/// Method parsing error
/// 方法解析错误
#[derive(Debug, Clone, PartialEq)]
pub enum MethodError
{
    /// Invalid HTTP method
    InvalidMethod(String),
}

impl fmt::Display for MethodError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            MethodError::InvalidMethod(m) => write!(f, "Invalid HTTP method: {}", m),
        }
    }
}

impl std::error::Error for MethodError {}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_method_from_str()
    {
        assert_eq!(Method::from_str("GET"), Ok(Method::GET));
        assert_eq!(Method::from_str("get"), Ok(Method::GET));
        assert_eq!(Method::from_str("POST"), Ok(Method::POST));
        assert!(Method::from_str("INVALID").is_err());
    }

    #[test]
    fn test_method_is_safe()
    {
        assert!(Method::GET.is_safe());
        assert!(Method::HEAD.is_safe());
        assert!(!Method::POST.is_safe());
        assert!(!Method::DELETE.is_safe());
    }

    #[test]
    fn test_method_is_idempotent()
    {
        assert!(Method::GET.is_idempotent());
        assert!(Method::PUT.is_idempotent());
        assert!(Method::DELETE.is_idempotent());
        assert!(!Method::POST.is_idempotent());
    }
}
