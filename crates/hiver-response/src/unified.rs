//! Unified API response wrapper module.
//! 统一 API 响应包装器模块。
//!
//! # Overview / 概述
//!
//! Provides a global response wrapping mechanism similar to Spring Boot's
//! `ResponseBodyAdvice`. All API responses can be automatically wrapped in a
//! consistent structure with code, message, data, and timestamp fields.
//!
//! 提供类似 Spring Boot `ResponseBodyAdvice` 的全局响应包装机制。
//! 所有 API 响应可以自动包装为包含 code、message、data 和 timestamp 的统一结构。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_response::unified::{ApiResponse, ResponseResult};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User { name: String }
//!
//! // Direct construction / 直接构建
//! let resp = ApiResponse::success(User { name: "Alice".into() });
//!
//! // Via ResponseResult type alias / 通过 ResponseResult 类型别名
//! type HandlerResult<T> = ResponseResult<T>;
//! ```

use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{IntoResponse, Response};

/// Unified API response wrapper.
/// 统一 API 响应包装器。
///
/// All API responses are automatically wrapped in this structure:
/// ```json
/// {"code": 200, "message": "success", "data": {...}, "timestamp": 1234567890}
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize>
{
    /// Business status code. 200 for success, 4xx/5xx for errors.
    /// 业务状态码。200 表示成功，4xx/5xx 表示错误。
    pub code: u16,
    /// Human-readable message.
    /// 可读消息。
    pub message: String,
    /// Response payload, `None` for error responses.
    /// 响应数据，错误响应时为 `None`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Unix timestamp (seconds).
    /// Unix 时间戳（秒）。
    pub timestamp: u64,
}

impl<T: Serialize> ApiResponse<T>
{
    /// Create a success response with data.
    /// 创建带数据的成功响应。
    pub fn success(data: T) -> Self
    {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
            timestamp: current_timestamp(),
        }
    }

    /// Create a success response with a custom message and data.
    /// 创建带自定义消息和数据的成功响应。
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self
    {
        Self {
            code: 200,
            message: message.into(),
            data: Some(data),
            timestamp: current_timestamp(),
        }
    }

    /// Create an error response (returns `ApiResponse<()>` to discard data).
    /// 创建错误响应（返回 `ApiResponse<()>` 以丢弃数据）。
    pub fn error(code: u16, message: impl Into<String>) -> ApiResponse<()>
    {
        ApiResponse {
            code,
            message: message.into(),
            data: None,
            timestamp: current_timestamp(),
        }
    }

    /// Create an error response from a `StatusCode`.
    /// 从 `StatusCode` 创建错误响应。
    pub fn from_status(status: StatusCode) -> ApiResponse<()>
    {
        Self::error(status.as_u16(), status.canonical_reason().unwrap_or("Unknown"))
    }

    /// Override the message on this response.
    /// 覆盖此响应的消息。
    pub fn with_message(mut self, message: impl Into<String>) -> Self
    {
        self.message = message.into();
        self
    }

    /// Override the code on this response.
    /// 覆盖此响应的状态码。
    pub fn with_code(mut self, code: u16) -> Self
    {
        self.code = code;
        self
    }

    /// Check whether this response represents a success.
    /// 检查此响应是否表示成功。
    pub fn is_success(&self) -> bool
    {
        self.code >= 200 && self.code < 300
    }
}

impl ApiResponse<()>
{
    /// Create a success response with no data.
    /// 创建无数据的成功响应。
    pub fn ok() -> Self
    {
        Self {
            code: 200,
            message: "success".to_string(),
            data: None,
            timestamp: current_timestamp(),
        }
    }

    /// Create a 400 Bad Request error.
    /// 创建 400 错误请求。
    pub fn bad_request(message: impl Into<String>) -> Self
    {
        Self::error(400, message)
    }

    /// Create a 401 Unauthorized error.
    /// 创建 401 未认证错误。
    pub fn unauthorized(message: impl Into<String>) -> Self
    {
        Self::error(401, message)
    }

    /// Create a 403 Forbidden error.
    /// 创建 403 禁止访问错误。
    pub fn forbidden(message: impl Into<String>) -> Self
    {
        Self::error(403, message)
    }

    /// Create a 404 Not Found error.
    /// 创建 404 未找到错误。
    pub fn not_found(message: impl Into<String>) -> Self
    {
        Self::error(404, message)
    }

    /// Create a 409 Conflict error.
    /// 创建 409 冲突错误。
    pub fn conflict(message: impl Into<String>) -> Self
    {
        Self::error(409, message)
    }

    /// Create a 500 Internal Server Error.
    /// 创建 500 服务器内部错误。
    pub fn internal_error(message: impl Into<String>) -> Self
    {
        Self::error(500, message)
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T>
{
    fn into_response(self) -> Response
    {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        match serde_json::to_vec(&self)
        {
            Ok(body) => Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(body)
                .unwrap_or_else(|_| Response::new()),
            Err(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to serialize ApiResponse")
                .unwrap_or_else(|_| Response::new()),
        }
    }
}

/// Response advice trait for automatic wrapping.
/// 自动包装的响应通知 trait。
///
/// Implement this trait to provide custom wrapping logic. The default
/// implementation simply wraps data into `ApiResponse::success`.
/// The framework can call `ResponseAdvice::wrap` in middleware to
/// automatically envelope handler return values.
///
/// 实现此 trait 以提供自定义包装逻辑。默认实现将数据包装到
/// `ApiResponse::success`。框架可以在中间件中调用 `ResponseAdvice::wrap`
/// 来自动包装处理器返回值。
pub trait ResponseAdvice
{
    /// Wrap a value into an `ApiResponse`.
    /// 将值包装为 `ApiResponse`。
    fn wrap<T: Serialize>(data: T) -> ApiResponse<T>;
}

/// Default advice that wraps with `ApiResponse::success`.
/// 使用 `ApiResponse::success` 包装的默认通知。
pub struct DefaultResponseAdvice;

impl ResponseAdvice for DefaultResponseAdvice
{
    fn wrap<T: Serialize>(data: T) -> ApiResponse<T>
    {
        ApiResponse::success(data)
    }
}

/// A convenience result wrapper for handlers that return wrapped API responses.
/// 用于返回包装 API 响应的处理器的便捷结果包装器。
///
/// Wraps `std::result::Result<ApiResponse<T>, ApiResponse<()>>` as a newtype so that
/// inherent methods and trait impls are permitted by the orphan rules.
///
/// `Ok(T)` is mapped to `ApiResponse::success(T)`.
/// `Err(ApiResponse<()>)` is passed through directly.
///
/// 将 `std::result::Result<ApiResponse<T>, ApiResponse<()>>` 包装为新类型，
/// 以便孤儿规则允许固有方法和 trait 实现。
///
/// `Ok(T)` 映射为 `ApiResponse::success(T)`。
/// `Err(ApiResponse<()>)` 直接传递。
pub struct ResponseResult<T: Serialize>(pub Result<ApiResponse<T>, ApiResponse<()>>);

impl<T: Serialize> ResponseResult<T>
{
    /// Create an `Ok` variant with success data.
    /// 创建包含成功数据的 `Ok` 变体。
    pub fn ok(data: T) -> Self
    {
        Self(Ok(ApiResponse::success(data)))
    }

    /// Create an `Err` variant with the given code and message.
    /// 使用给定的代码和消息创建 `Err` 变体。
    pub fn err(code: u16, message: impl Into<String>) -> Self
    {
        Self(Err(ApiResponse::<()>::error(code, message)))
    }
}

impl<T: Serialize> IntoResponse for ResponseResult<T>
{
    fn into_response(self) -> Response
    {
        match self.0
        {
            Ok(api_resp) => api_resp.into_response(),
            Err(err_resp) => err_resp.into_response(),
        }
    }
}

/// Return the current Unix timestamp in seconds.
/// 返回当前 Unix 时间戳（秒）。
fn current_timestamp() -> u64
{
    chrono::Utc::now().timestamp() as u64
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct User
    {
        name: String,
    }

    #[test]
    fn test_api_response_success()
    {
        let resp = ApiResponse::success(User {
            name: "Alice".to_string(),
        });
        assert!(resp.is_success());
        assert_eq!(resp.code, 200);
        assert_eq!(resp.message, "success");
        assert!(resp.data.is_some());
    }

    #[test]
    fn test_api_response_error()
    {
        let resp = ApiResponse::<()>::error(404, "Not found");
        assert!(!resp.is_success());
        assert_eq!(resp.code, 404);
        assert_eq!(resp.message, "Not found");
        assert!(resp.data.is_none());
    }

    #[test]
    fn test_api_response_with_message()
    {
        let resp = ApiResponse::success(42u32).with_message("custom message");
        assert_eq!(resp.message, "custom message");
    }

    #[test]
    fn test_api_response_ok()
    {
        let resp = ApiResponse::ok();
        assert!(resp.is_success());
        assert!(resp.data.is_none());
    }

    #[test]
    fn test_api_response_convenience_errors()
    {
        let e = ApiResponse::bad_request("invalid input");
        assert_eq!(e.code, 400);

        let e = ApiResponse::unauthorized("token expired");
        assert_eq!(e.code, 401);

        let e = ApiResponse::forbidden("no access");
        assert_eq!(e.code, 403);

        let e = ApiResponse::not_found("resource missing");
        assert_eq!(e.code, 404);

        let e = ApiResponse::conflict("duplicate key");
        assert_eq!(e.code, 409);

        let e = ApiResponse::internal_error("db error");
        assert_eq!(e.code, 500);
    }

    #[test]
    fn test_api_response_from_status()
    {
        let resp = ApiResponse::<()>::from_status(StatusCode::NOT_FOUND);
        assert_eq!(resp.code, 404);
        assert_eq!(resp.message, "Not Found");
    }

    #[test]
    fn test_api_response_into_response_success()
    {
        let resp = ApiResponse::success("hello").into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = std::str::from_utf8(resp.body()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["code"], 200);
        assert_eq!(parsed["data"], "hello");
    }

    #[test]
    fn test_api_response_into_response_error()
    {
        let resp = ApiResponse::<()>::error(404, "gone").into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = std::str::from_utf8(resp.body()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["code"], 404);
        assert_eq!(parsed["message"], "gone");
    }

    #[test]
    fn test_response_result_ok()
    {
        let result: ResponseResult<User> = ResponseResult::ok(User { name: "Bob".into() });
        let resp = result.into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = std::str::from_utf8(resp.body()).unwrap();
        assert!(body.contains("Bob"));
    }

    #[test]
    fn test_response_result_err()
    {
        let result: ResponseResult<User> = ResponseResult::err(403, "nope");
        let resp = result.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_default_response_advice()
    {
        let wrapped = DefaultResponseAdvice::wrap(42u32);
        assert_eq!(wrapped.code, 200);
        assert_eq!(wrapped.data, Some(42));
    }

    #[test]
    fn test_api_response_serde_roundtrip()
    {
        let resp = ApiResponse::success(User {
            name: "Carol".to_string(),
        });
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<User> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, 200);
        assert_eq!(deserialized.data.as_ref().unwrap().name, "Carol");
    }
}
