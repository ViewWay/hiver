//! Unified API Response Structure / 统一 API 响应结构
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `ResponseEntity<T>` - Wrapper for API responses
//! - `@ResponseBody` + `ResponseAdvice` - Automatic response wrapping
//! - `Result<T>` / `ApiResponse<T>` - Standardized response format
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::ApiResponse;
//! use hiver_http::response::Response;
//!
//! #[hiver_macros::get("/users/{id}")]
//! fn get_user(id: u64) -> Response {
//!     let user = user_service.find_by_id(id)?;
//!     ApiResponse::success(user).into_response()
//! }
//!
//! // Error handling
//! #[hiver_macros::get("/users/{id}")]
//! fn get_user(id: u64) -> Response {
//!     ApiResponse::not_found()
//!         .code("USER_NOT_FOUND")
//!         .message(&format!("User {} not found", id))
//!         .into_response()
//! }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{StatusCode, body::Body, response::Response};

// ============================================================================
// Result Code Enum / 结果码枚举
// ============================================================================

/// Standard result codes
/// 标准结果码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCode
{
    // Success / 成功
    /// 200 OK - Request succeeded / 请求成功
    Success = 200,
    /// 201 Created - Resource created / 资源已创建
    Created = 201,
    /// 202 Accepted - Request accepted / 请求已接受
    Accepted = 202,
    /// 204 No Content - Success with no response body / 成功但无响应体
    NoContent = 204,

    // Client errors / 客户端错误
    /// 400 Bad Request - Invalid request / 无效请求
    BadRequest = 400,
    /// 401 Unauthorized - Authentication required / 需要认证
    Unauthorized = 401,
    /// 403 Forbidden - Access denied / 访问被拒绝
    Forbidden = 403,
    /// 404 Not Found - Resource not found / 资源未找到
    NotFound = 404,
    /// 405 Method Not Allowed - Method not supported / 方法不支持
    MethodNotAllowed = 405,
    /// 409 Conflict - Resource conflict / 资源冲突
    Conflict = 409,
    /// 422 Unprocessable Entity - Invalid data / 无效数据
    UnprocessableEntity = 422,
    /// 429 Too Many Requests - Rate limit exceeded / 超过速率限制
    TooManyRequests = 429,

    // Server errors / 服务端错误
    /// 500 Internal Server Error - Server error / 服务器错误
    InternalError = 500,
    /// 501 Not Implemented - Feature not implemented / 功能未实现
    NotImplemented = 501,
    /// 502 Bad Gateway - Invalid gateway response / 网关响应无效
    BadGateway = 502,
    /// 503 Service Unavailable - Service unavailable / 服务不可用
    ServiceUnavailable = 503,
}

impl ResultCode
{
    /// Get the numeric code
    /// 获取数字代码
    pub fn code(&self) -> u16
    {
        *self as u16
    }

    /// Get the default message
    /// 获取默认消息
    pub fn message(&self) -> &'static str
    {
        match self
        {
            ResultCode::Success => "Success",
            ResultCode::Created => "Created",
            ResultCode::Accepted => "Accepted",
            ResultCode::NoContent => "No Content",

            ResultCode::BadRequest => "Bad Request",
            ResultCode::Unauthorized => "Unauthorized",
            ResultCode::Forbidden => "Forbidden",
            ResultCode::NotFound => "Not Found",
            ResultCode::MethodNotAllowed => "Method Not Allowed",
            ResultCode::Conflict => "Conflict",
            ResultCode::UnprocessableEntity => "Unprocessable Entity",
            ResultCode::TooManyRequests => "Too Many Requests",

            ResultCode::InternalError => "Internal Server Error",
            ResultCode::NotImplemented => "Not Implemented",
            ResultCode::BadGateway => "Bad Gateway",
            ResultCode::ServiceUnavailable => "Service Unavailable",
        }
    }

    /// Get Chinese message
    /// 获取中文消息
    pub fn message_zh(&self) -> &'static str
    {
        match self
        {
            ResultCode::Success => "操作成功",
            ResultCode::Created => "创建成功",
            ResultCode::Accepted => "请求已接受",
            ResultCode::NoContent => "无内容",

            ResultCode::BadRequest => "请求参数错误",
            ResultCode::Unauthorized => "未认证",
            ResultCode::Forbidden => "无权限",
            ResultCode::NotFound => "资源不存在",
            ResultCode::MethodNotAllowed => "方法不允许",
            ResultCode::Conflict => "资源冲突",
            ResultCode::UnprocessableEntity => "无法处理的实体",
            ResultCode::TooManyRequests => "请求过于频繁",

            ResultCode::InternalError => "服务器内部错误",
            ResultCode::NotImplemented => "未实现",
            ResultCode::BadGateway => "网关错误",
            ResultCode::ServiceUnavailable => "服务不可用",
        }
    }

    /// Get the HTTP status code
    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> StatusCode
    {
        StatusCode::from_u16(self.code())
    }
}

// ============================================================================
// Unified API Response / 统一 API 响应
// ============================================================================

/// Unified API response structure
/// 统一 API 响应结构
///
/// This is equivalent to Spring's `ResponseEntity<T>` or a custom `Result<T>` wrapper.
/// 这等价于 Spring 的 `ResponseEntity<T>` 或自定义 `Result<T>` 包装器。
///
/// # JSON Format / JSON 格式
///
/// Success response / 成功响应:
/// ```json
/// {
///   "code": 200,
///   "message": "Success",
///   "data": { "id": 1, "name": "Alice" },
///   "timestamp": "2024-01-29T10:30:45Z"
/// }
/// ```
///
/// Error response / 错误响应:
/// ```json
/// {
///   "code": 404,
///   "message": "Not Found",
///   "error": "USER_NOT_FOUND",
///   "timestamp": "2024-01-29T10:30:45Z"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T>
{
    /// Response code / 响应码
    pub code: u16,

    /// Response message / 响应消息
    pub message: String,

    /// Response data / 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error code (for error responses) / 错误码（用于错误响应）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Request path / 请求路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Response timestamp / 响应时间戳
    pub timestamp: String,

    /// Additional errors (for validation errors) / 额外错误（用于校验错误）
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub errors: HashMap<String, String>,
}

impl<T> ApiResponse<T>
{
    /// Create a new API response
    /// 创建新的 API 响应
    pub fn new(code: u16, message: impl Into<String>) -> Self
    {
        Self {
            code,
            message: message.into(),
            data: None,
            error: None,
            path: None,
            timestamp: format_timestamp(),
            errors: HashMap::new(),
        }
    }

    /// Create a success response with data
    /// 创建带数据的成功响应
    pub fn success_data(data: T) -> Self
    {
        Self {
            code: ResultCode::Success.code(),
            message: ResultCode::Success.message().to_string(),
            data: Some(data),
            error: None,
            path: None,
            timestamp: format_timestamp(),
            errors: HashMap::new(),
        }
    }

    /// Create a 201 Created response
    /// 创建 201 Created 响应
    pub fn created(data: T) -> Self
    {
        Self {
            code: ResultCode::Created.code(),
            message: ResultCode::Created.message().to_string(),
            data: Some(data),
            error: None,
            path: None,
            timestamp: format_timestamp(),
            errors: HashMap::new(),
        }
    }

    /// Set the response code
    /// 设置响应码
    pub fn code(mut self, code: u16) -> Self
    {
        self.code = code;
        self
    }

    /// Set the response message
    /// 设置响应消息
    pub fn message(mut self, message: impl Into<String>) -> Self
    {
        self.message = message.into();
        self
    }

    /// Set the error code
    /// 设置错误码
    pub fn error(mut self, error: impl Into<String>) -> Self
    {
        self.error = Some(error.into());
        self
    }

    /// Set the request path
    /// 设置请求路径
    pub fn path(mut self, path: impl Into<String>) -> Self
    {
        self.path = Some(path.into());
        self
    }

    /// Add an error field
    /// 添加错误字段
    pub fn add_error(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.errors.insert(key.into(), value.into());
        self
    }

    /// Convert to HTTP Response
    /// 转换为 HTTP Response
    pub fn into_response(self) -> Response
    where
        T: Serialize,
    {
        let json_body = serde_json::to_string(&self).unwrap_or_default();
        let status = StatusCode::from_u16(self.code);

        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap_or_else(|_| Response::internal_server_error())
    }

    /// Check if this is a success response (2xx)
    /// 检查是否为成功响应 (2xx)
    pub fn is_success(&self) -> bool
    {
        (200..300).contains(&self.code)
    }

    /// Check if this is a client error (4xx)
    /// 检查是否为客户端错误 (4xx)
    pub fn is_client_error(&self) -> bool
    {
        (400..500).contains(&self.code)
    }

    /// Check if this is a server error (5xx)
    /// 检查是否为服务端错误 (5xx)
    pub fn is_server_error(&self) -> bool
    {
        (500..600).contains(&self.code)
    }
}

/// Methods that don't depend on type parameter T
/// 不依赖类型参数 T 的方法
impl ApiResponse<()>
{
    /// Create a success response without data
    /// 创建无数据的成功响应
    pub fn success() -> Self
    {
        Self {
            code: ResultCode::Success.code(),
            message: ResultCode::Success.message().to_string(),
            data: None,
            error: None,
            path: None,
            timestamp: format_timestamp(),
            errors: HashMap::new(),
        }
    }

    /// Create a 204 No Content response
    /// 创建 204 No Content 响应
    pub fn no_content() -> Self
    {
        Self {
            code: ResultCode::NoContent.code(),
            message: ResultCode::NoContent.message().to_string(),
            data: None,
            error: None,
            path: None,
            timestamp: format_timestamp(),
            errors: HashMap::new(),
        }
    }

    /// Create a 400 Bad Request response
    /// 创建 400 Bad Request 响应
    pub fn bad_request() -> Self
    {
        Self::error_code(ResultCode::BadRequest)
    }

    /// Create a 401 Unauthorized response
    /// 创建 401 Unauthorized 响应
    pub fn unauthorized() -> Self
    {
        Self::error_code(ResultCode::Unauthorized)
    }

    /// Create a 403 Forbidden response
    /// 创建 403 Forbidden 响应
    pub fn forbidden() -> Self
    {
        Self::error_code(ResultCode::Forbidden)
    }

    /// Create a 404 Not Found response
    /// 创建 404 Not Found 响应
    pub fn not_found() -> Self
    {
        Self::error_code(ResultCode::NotFound)
    }

    /// Create a 409 Conflict response
    /// 创建 409 Conflict 响应
    pub fn conflict() -> Self
    {
        Self::error_code(ResultCode::Conflict)
    }

    /// Create a 422 Unprocessable Entity response
    /// 创建 422 Unprocessable Entity 响应
    pub fn unprocessable_entity() -> Self
    {
        Self::error_code(ResultCode::UnprocessableEntity)
    }

    /// Create a 500 Internal Server Error response
    /// 创建 500 Internal Server Error 响应
    pub fn internal_error() -> Self
    {
        Self::error_code(ResultCode::InternalError)
    }

    /// Create an error response from a result code
    /// 从结果码创建错误响应
    pub fn error_code(code: ResultCode) -> Self
    {
        Self {
            code: code.code(),
            message: code.message().to_string(),
            data: None,
            error: Some(code.message().to_string()),
            path: None,
            timestamp: format_timestamp(),
            errors: HashMap::new(),
        }
    }
}

// ============================================================================
// Page Response / 分页响应
// ============================================================================

/// Paginated response structure
/// 分页响应结构
///
/// Equivalent to Spring's `Page<T>` or `PageResult<T>`.
/// 等价于 Spring 的 `Page<T>` 或 `PageResult<T>`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T>
{
    /// Page content / 页面内容
    pub content: Vec<T>,

    /// Current page number (0-based) / 当前页码（从0开始）
    pub page: u32,

    /// Page size / 页面大小
    pub size: u32,

    /// Total number of elements / 总元素数
    pub total_elements: u64,

    /// Total number of pages / 总页数
    pub total_pages: u32,

    /// Whether this is the first page / 是否为第一页
    pub first: bool,

    /// Whether this is the last page / 是否为最后一页
    pub last: bool,
}

impl<T> PageResponse<T>
{
    /// Create a new page response
    /// 创建新的分页响应
    pub fn new(content: Vec<T>, page: u32, size: u32, total_elements: u64) -> Self
    {
        #[allow(clippy::cast_precision_loss)]
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
            first: page == 0,
            last: page + 1 >= total_pages,
        }
    }

    /// Create an empty page response
    /// 创建空的分页响应
    pub fn empty() -> Self
    {
        Self::new(vec![], 0, 10, 0)
    }

    /// Wrap the page content in an `ApiResponse`
    /// 将分页内容包装为 `ApiResponse`
    pub fn into_api_response(self) -> ApiResponse<Vec<T>>
    {
        ApiResponse::success_data(self.content)
            .add_error("page", self.page.to_string())
            .add_error("size", self.size.to_string())
            .add_error("total_elements", self.total_elements.to_string())
            .add_error("total_pages", self.total_pages.to_string())
    }
}

// ============================================================================
// Utility Functions / 工具函数
// ============================================================================

/// Format current timestamp in ISO 8601 format
/// 格式化当前时间戳为 ISO 8601 格式
fn format_timestamp() -> String
{
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        (1970 + secs / 31_536_000) % 10000,
        ((secs % 31_536_000) / 2_592_000 + 1) % 13,
        ((secs % 2_592_000) / 86_400 + 1) % 32,
        (secs % 86_400) / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

// ============================================================================
// Conversion Traits / 转换 Trait
// ============================================================================

/// Trait for converting types to `ApiResponse`
/// 将类型转换为 `ApiResponse` 的 Trait
pub trait IntoApiResponse<T>
{
    /// Convert self into an `ApiResponse`
    /// 将 self 转换为 `ApiResponse`
    fn into_api_response(self) -> ApiResponse<T>;
}

impl<T: Serialize> IntoApiResponse<T> for T
{
    fn into_api_response(self) -> ApiResponse<T>
    {
        ApiResponse::success_data(self)
    }
}

// ============================================================================
// IntoResponse Implementation / IntoResponse 实现
// ============================================================================

impl<T: Serialize> crate::IntoResponse for ApiResponse<T>
{
    fn into_response(self) -> Response
    {
        self.into_response()
    }
}

impl<T: Serialize> crate::IntoResponse for PageResponse<T>
{
    fn into_response(self) -> Response
    {
        let api_response = ApiResponse::success_data(self.content)
            .add_error("page", self.page.to_string())
            .add_error("size", self.size.to_string())
            .add_error("total_elements", self.total_elements.to_string())
            .add_error("total_pages", self.total_pages.to_string());

        api_response.into_response()
    }
}

// ============================================================================
// Tests / 测试
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_result_code_messages()
    {
        assert_eq!(ResultCode::Success.message(), "Success");
        assert_eq!(ResultCode::Success.message_zh(), "操作成功");
        assert_eq!(ResultCode::NotFound.message(), "Not Found");
        assert_eq!(ResultCode::NotFound.message_zh(), "资源不存在");
    }

    #[test]
    fn test_api_response_success()
    {
        #[derive(Serialize)]
        struct User
        {
            id: u64,
            name: String,
        }

        let user = User {
            id: 1,
            name: "Alice".to_string(),
        };

        let response = ApiResponse::success_data(user);
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "Success");
        assert!(response.data.is_some());
        assert!(response.is_success());
        assert!(!response.is_client_error());
        assert!(!response.is_server_error());
    }

    #[test]
    fn test_api_response_error()
    {
        let response = ApiResponse::not_found();
        assert_eq!(response.code, 404);
        assert_eq!(response.message, "Not Found");
        assert!(response.error.is_some());
        assert!(!response.is_success());
        assert!(response.is_client_error());
        assert!(!response.is_server_error());
    }

    #[test]
    fn test_api_response_builder()
    {
        let response = ApiResponse::bad_request()
            .code(400)
            .message("Invalid input")
            .error("VALIDATION_ERROR")
            .path("/api/users");

        assert_eq!(response.code, 400);
        assert_eq!(response.message, "Invalid input");
        assert_eq!(response.error, Some("VALIDATION_ERROR".to_string()));
        assert_eq!(response.path, Some("/api/users".to_string()));
    }

    #[test]
    fn test_page_response()
    {
        let content = vec![1, 2, 3, 4, 5];
        let page = PageResponse::new(content, 0, 5, 12);

        assert_eq!(page.page, 0);
        assert_eq!(page.size, 5);
        assert_eq!(page.total_elements, 12);
        assert_eq!(page.total_pages, 3);
        assert!(page.first);
        assert!(!page.last);
    }

    #[test]
    fn test_page_response_empty()
    {
        let page = PageResponse::<i32>::empty();
        assert!(page.content.is_empty());
        assert_eq!(page.total_elements, 0);
        assert!(page.first);
        assert!(page.last);
    }
}
