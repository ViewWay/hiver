//! 统一响应结构 / Unified response structure
//!
//! 等价于 Spring Boot 的 Result<T> / Equivalent to Spring Boot's Result<T>
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_response::Result;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//!
//! // Success response
//! let response: Result<User> = Result::success(user);
//!
//! // Error response
//! let error: Result<User> = Result::error(404, "User not found");
//! ```

use std::collections::HashMap;

use http::StatusCode;
use serde::Serialize;

use crate::{IntoResponse, Json, Response};

/// 响应码枚举 / Response code enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCode
{
    /// 成功 / Success
    Success = 200,
    /// 已创建 / Created
    Created = 201,
    /// 无内容 / No Content
    NoContent = 204,
    /// 错误请求 / Bad Request
    BadRequest = 400,
    /// 未认证 / Unauthorized
    Unauthorized = 401,
    /// 禁止访问 / Forbidden
    Forbidden = 403,
    /// 未找到 / Not Found
    NotFound = 404,
    /// 冲突 / Conflict
    Conflict = 409,
    /// 服务器错误 / Internal Server Error
    InternalError = 500,
}

impl ResultCode
{
    /// 获取HTTP状态码 / Get HTTP status code
    pub fn status_code(&self) -> StatusCode
    {
        match self
        {
            ResultCode::Success => StatusCode::OK,
            ResultCode::Created => StatusCode::CREATED,
            ResultCode::NoContent => StatusCode::NO_CONTENT,
            ResultCode::BadRequest => StatusCode::BAD_REQUEST,
            ResultCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ResultCode::Forbidden => StatusCode::FORBIDDEN,
            ResultCode::NotFound => StatusCode::NOT_FOUND,
            ResultCode::Conflict => StatusCode::CONFLICT,
            ResultCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取描述 / Get description
    pub fn description(&self) -> &'static str
    {
        match self
        {
            ResultCode::Success => "操作成功 / Operation successful",
            ResultCode::Created => "创建成功 / Created successfully",
            ResultCode::NoContent => "无内容 / No content",
            ResultCode::BadRequest => "请求参数错误 / Bad request",
            ResultCode::Unauthorized => "未认证 / Unauthorized",
            ResultCode::Forbidden => "无权限 / Forbidden",
            ResultCode::NotFound => "资源不存在 / Resource not found",
            ResultCode::Conflict => "资源冲突 / Resource conflict",
            ResultCode::InternalError => "服务器错误 / Internal server error",
        }
    }
}

impl From<ResultCode> for StatusCode
{
    fn from(code: ResultCode) -> Self
    {
        code.status_code()
    }
}

/// 统一响应结构 / Unified response structure
///
/// 等价于 Spring Boot 的 Result<T>
#[derive(Debug, Clone, Serialize)]
pub struct Result<T>
{
    /// 响应码 / Response code
    pub code: u16,
    /// 响应消息 / Response message
    pub message: String,
    /// 响应数据 / Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 时间戳 / Timestamp
    pub timestamp: i64,
    /// 请求路径 / Request path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 错误详情 / Error details
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub errors: HashMap<String, String>,
}

impl<T> Result<T>
{
    /// 创建成功响应（无数据） / Create success response without data
    pub fn ok() -> Self
    {
        Self::success_with_message("成功 / Success".to_string())
    }

    /// 创建成功响应（带数据） / Create success response with data
    pub fn success(data: T) -> Self
    {
        Self {
            code: ResultCode::Success as u16,
            message: ResultCode::Success.description().to_string(),
            data: Some(data),
            timestamp: chrono::Utc::now().timestamp(),
            path: None,
            errors: HashMap::new(),
        }
    }

    /// 创建成功响应（自定义消息） / Create success response with custom message
    pub fn success_with_message(message: String) -> Self
    {
        Self {
            code: ResultCode::Success as u16,
            message,
            data: None,
            timestamp: chrono::Utc::now().timestamp(),
            path: None,
            errors: HashMap::new(),
        }
    }

    /// 创建错误响应 / Create error response
    pub fn error(code: u16, message: impl Into<String>) -> Self
    {
        Self {
            code,
            message: message.into(),
            data: None,
            timestamp: chrono::Utc::now().timestamp(),
            path: None,
            errors: HashMap::new(),
        }
    }

    /// 创建错误响应（ResultCode） / Create error response with `ResultCode`
    pub fn from_code(code: ResultCode) -> Self
    {
        Self::error(code as u16, code.description())
    }

    /// 创建400错误 / Create 400 error
    pub fn bad_request(message: impl Into<String>) -> Self
    {
        Self::error(ResultCode::BadRequest as u16, message)
    }

    /// 创建401错误 / Create 401 error
    pub fn unauthorized(message: impl Into<String>) -> Self
    {
        Self::error(ResultCode::Unauthorized as u16, message)
    }

    /// 创建403错误 / Create 403 error
    pub fn forbidden(message: impl Into<String>) -> Self
    {
        Self::error(ResultCode::Forbidden as u16, message)
    }

    /// 创建404错误 / Create 404 error
    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self
    {
        Self::error(
            ResultCode::NotFound as u16,
            format!("{} {}: 不存在 / not found", resource.into(), id.into()),
        )
    }

    /// 创建409错误 / Create 409 error
    pub fn conflict(message: impl Into<String>) -> Self
    {
        Self::error(ResultCode::Conflict as u16, message)
    }

    /// 创建500错误 / Create 500 error
    pub fn internal_error(message: impl Into<String>) -> Self
    {
        Self::error(ResultCode::InternalError as u16, message)
    }

    /// 设置请求路径 / Set request path
    pub fn with_path(mut self, path: impl Into<String>) -> Self
    {
        self.path = Some(path.into());
        self
    }

    /// 设置错误详情 / Set error details
    pub fn with_errors(mut self, errors: HashMap<String, String>) -> Self
    {
        self.errors = errors;
        self
    }

    /// 添加单个错误详情 / Add single error detail
    pub fn add_error(mut self, field: impl Into<String>, message: impl Into<String>) -> Self
    {
        self.errors.insert(field.into(), message.into());
        self
    }

    /// 检查是否成功 / Check if successful
    pub fn is_success(&self) -> bool
    {
        self.code >= 200 && self.code < 300
    }

    /// 检查是否失败 / Check if failed
    pub fn is_error(&self) -> bool
    {
        !self.is_success()
    }
}

impl<T> IntoResponse for Result<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response
    {
        let _ = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Json(self).into_response()
    }
}

/// 分页结果 / Paginated result
#[derive(Debug, Clone, Serialize)]
pub struct PageResult<T>
{
    /// 数据列表 / Data list
    pub content: Vec<T>,
    /// 当前页 / Current page (0-indexed)
    pub page: u32,
    /// 每页数量 / Page size
    pub size: u32,
    /// 总元素数 / Total elements
    pub total_elements: u64,
    /// 总页数 / Total pages
    pub total_pages: u32,
}

impl<T> PageResult<T>
{
    /// 创建分页结果 / Create paginated result
    #[allow(clippy::cast_precision_loss)]
    pub fn new(content: Vec<T>, page: u32, size: u32, total_elements: u64) -> Self
    {
        let total_pages = if size == 0
        {
            0
        }
        else
        {
            ((total_elements as f64) / (size as f64)).ceil() as u32
        };

        Self {
            content,
            page,
            size,
            total_elements,
            total_pages,
        }
    }

    /// 创建空分页结果 / Create empty paginated result
    pub fn empty() -> Self
    {
        Self::new(vec![], 0, 10, 0)
    }

    /// 创建成功的分页响应 / Create success paginated response
    pub fn success(self) -> Result<Vec<T>>
    {
        Result::success(self.content)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_result_success()
    {
        let result = Result::<String>::success("test".to_string());
        assert!(result.is_success());
        assert_eq!(result.code, 200);
        assert_eq!(result.data, Some("test".to_string()));
    }

    #[test]
    fn test_result_error()
    {
        let result = Result::<String>::not_found("User", "123");
        assert!(result.is_error());
        assert_eq!(result.code, 404);
    }

    #[test]
    fn test_result_with_errors()
    {
        let result = Result::<()>::bad_request("Validation failed")
            .add_error("username", "Username is required")
            .add_error("email", "Email is invalid");

        assert_eq!(result.code, 400);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_page_result()
    {
        let items = vec!["a", "b", "c"];
        let page = PageResult::new(items.clone(), 0, 10, 3);

        assert_eq!(page.content, items);
        assert_eq!(page.page, 0);
        assert_eq!(page.size, 10);
        assert_eq!(page.total_elements, 3);
        assert_eq!(page.total_pages, 1);
    }

    #[test]
    fn test_page_result_empty()
    {
        let page: PageResult<String> = PageResult::empty();
        assert!(page.content.is_empty());
        assert_eq!(page.total_elements, 0);
    }
}
