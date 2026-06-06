//! HTTP Status Code type
//! HTTP 状态码类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `HttpStatus`, @`ResponseStatus`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

/// HTTP Status Codes
/// HTTP 状态码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusCode(u16);

impl StatusCode
{
    /// 202 Accepted / 202 已接受
    pub const ACCEPTED: StatusCode = StatusCode(202);
    /// 502 Bad Gateway / 502 网关错误
    pub const BAD_GATEWAY: StatusCode = StatusCode(502);
    // 4xx Client Error / 4xx 客户端错误

    /// 400 Bad Request / 400 错误请求
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    /// 409 Conflict / 409 冲突
    pub const CONFLICT: StatusCode = StatusCode(409);
    // 1xx Informational / 1xx 信息响应

    /// 100 Continue / 100 继续
    pub const CONTINUE: StatusCode = StatusCode(100);
    /// 201 Created / 201 已创建
    pub const CREATED: StatusCode = StatusCode(201);
    /// 417 Expectation Failed / 417 期望失败
    pub const EXPECTATION_FAILED: StatusCode = StatusCode(417);
    /// 403 Forbidden / 403 禁止访问
    pub const FORBIDDEN: StatusCode = StatusCode(403);
    /// 302 Found / 302 临时跳转
    pub const FOUND: StatusCode = StatusCode(302);
    /// 504 Gateway Timeout / 504 网关超时
    pub const GATEWAY_TIMEOUT: StatusCode = StatusCode(504);
    /// 410 Gone / 410 已消失
    pub const GONE: StatusCode = StatusCode(410);
    /// 505 HTTP Version Not Supported / 505 HTTP 版本不支持
    pub const HTTP_VERSION_NOT_SUPPORTED: StatusCode = StatusCode(505);
    /// 418 I'm a teapot / 418 我是一个茶壶
    pub const IM_A_TEAPOT: StatusCode = StatusCode(418);
    // 5xx Server Error / 5xx 服务器错误

    /// 500 Internal Server Error / 500 服务器内部错误
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);
    /// 411 Length Required / 411 需要内容长度
    pub const LENGTH_REQUIRED: StatusCode = StatusCode(411);
    /// 405 Method Not Allowed / 405 方法不允许
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode(405);
    /// 301 Moved Permanently / 301 永久移动
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode(301);
    // 3xx Redirection / 3xx 重定向

    /// 300 Multiple Choices / 300 多种选择
    pub const MULTIPLE_CHOICES: StatusCode = StatusCode(300);
    /// 511 Network Authentication Required / 511 需要网络认证
    pub const NETWORK_AUTHENTICATION_REQUIRED: StatusCode = StatusCode(511);
    /// 203 Non-Authoritative Information / 203 非权威信息
    pub const NON_AUTHORITATIVE_INFORMATION: StatusCode = StatusCode(203);
    /// 406 Not Acceptable / 406 不可接受
    pub const NOT_ACCEPTABLE: StatusCode = StatusCode(406);
    /// 404 Not Found / 404 未找到
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    /// 501 Not Implemented / 501 未实现
    pub const NOT_IMPLEMENTED: StatusCode = StatusCode(501);
    /// 304 Not Modified / 304 未修改
    pub const NOT_MODIFIED: StatusCode = StatusCode(304);
    /// 204 No Content / 204 无内容
    pub const NO_CONTENT: StatusCode = StatusCode(204);
    // 2xx Success / 2xx 成功响应

    /// 200 OK / 200 成功
    pub const OK: StatusCode = StatusCode(200);
    /// 206 Partial Content / 206 部分内容
    pub const PARTIAL_CONTENT: StatusCode = StatusCode(206);
    /// 413 Payload Too Large / 413 负载过大
    pub const PAYLOAD_TOO_LARGE: StatusCode = StatusCode(413);
    /// 402 Payment Required / 402 需要付款
    pub const PAYMENT_REQUIRED: StatusCode = StatusCode(402);
    /// 308 Permanent Redirect / 308 永久重定向
    pub const PERMANENT_REDIRECT: StatusCode = StatusCode(308);
    /// 412 Precondition Failed / 412 前置条件失败
    pub const PRECONDITION_FAILED: StatusCode = StatusCode(412);
    /// 428 Precondition Required / 428 需要前置条件
    pub const PRECONDITION_REQUIRED: StatusCode = StatusCode(428);
    /// 102 Processing / 102 处理中
    pub const PROCESSING: StatusCode = StatusCode(102);
    /// 407 Proxy Authentication Required / 407 需要代理认证
    pub const PROXY_AUTHENTICATION_REQUIRED: StatusCode = StatusCode(407);
    /// 416 Range Not Satisfiable / 416 范围不满足
    pub const RANGE_NOT_SATISFIABLE: StatusCode = StatusCode(416);
    /// 408 Request Timeout / 408 请求超时
    pub const REQUEST_TIMEOUT: StatusCode = StatusCode(408);
    /// 205 Reset Content / 205 重置内容
    pub const RESET_CONTENT: StatusCode = StatusCode(205);
    /// 303 See Other / 303 请参见其他
    pub const SEE_OTHER: StatusCode = StatusCode(303);
    /// 503 Service Unavailable / 503 服务不可用
    pub const SERVICE_UNAVAILABLE: StatusCode = StatusCode(503);
    /// 101 Switching Protocols / 101 切换协议
    pub const SWITCHING_PROTOCOLS: StatusCode = StatusCode(101);
    /// 307 Temporary Redirect / 307 临时重定向
    pub const TEMPORARY_REDIRECT: StatusCode = StatusCode(307);
    /// 425 Too Early / 425 过早
    pub const TOO_EARLY: StatusCode = StatusCode(425);
    /// 429 Too Many Requests / 429 请求过多
    pub const TOO_MANY_REQUESTS: StatusCode = StatusCode(429);
    /// 401 Unauthorized / 401 未授权
    pub const UNAUTHORIZED: StatusCode = StatusCode(401);
    /// 422 Unprocessable Entity / 422 无法处理的实体
    pub const UNPROCESSABLE_ENTITY: StatusCode = StatusCode(422);
    /// 415 Unsupported Media Type / 415 不支持的媒体类型
    pub const UNSUPPORTED_MEDIA_TYPE: StatusCode = StatusCode(415);
    /// 426 Upgrade Required / 426 需要升级
    pub const UPGRADE_REQUIRED: StatusCode = StatusCode(426);
    /// 414 URI Too Long / 414 URI 过长
    pub const URI_TOO_LONG: StatusCode = StatusCode(414);
    /// 305 Use Proxy / 305 使用代理
    pub const USE_PROXY: StatusCode = StatusCode(305);

    /// Create a `StatusCode` from a u16
    /// 从u16创建状态码
    pub const fn from_u16(code: u16) -> StatusCode
    {
        StatusCode(code)
    }

    /// Get the status code as u16
    /// 获取u16格式的状态码
    pub const fn as_u16(self) -> u16
    {
        self.0
    }

    /// Check if this is a 1xx informational response
    /// 检查是否为1xx信息响应
    pub const fn is_informational(self) -> bool
    {
        self.0 >= 100 && self.0 < 200
    }

    /// Check if this is a 2xx success response
    /// 检查是否为2xx成功响应
    pub const fn is_success(self) -> bool
    {
        self.0 >= 200 && self.0 < 300
    }

    /// Check if this is a 3xx redirection
    /// 检查是否为3xx重定向
    pub const fn is_redirection(self) -> bool
    {
        self.0 >= 300 && self.0 < 400
    }

    /// Check if this is a 4xx client error
    /// 检查是否为4xx客户端错误
    pub const fn is_client_error(self) -> bool
    {
        self.0 >= 400 && self.0 < 500
    }

    /// Check if this is a 5xx server error
    /// 检查是否为5xx服务器错误
    pub const fn is_server_error(self) -> bool
    {
        self.0 >= 500 && self.0 < 600
    }

    /// Get the canonical reason phrase for this status code
    /// 获取此状态码的标准原因短语
    pub fn canonical_reason(self) -> Option<&'static str>
    {
        match self
        {
            StatusCode::CONTINUE => Some("Continue"),
            StatusCode::SWITCHING_PROTOCOLS => Some("Switching Protocols"),
            StatusCode::PROCESSING => Some("Processing"),
            StatusCode::OK => Some("OK"),
            StatusCode::CREATED => Some("Created"),
            StatusCode::ACCEPTED => Some("Accepted"),
            StatusCode::NON_AUTHORITATIVE_INFORMATION => Some("Non-Authoritative Information"),
            StatusCode::NO_CONTENT => Some("No Content"),
            StatusCode::RESET_CONTENT => Some("Reset Content"),
            StatusCode::PARTIAL_CONTENT => Some("Partial Content"),
            StatusCode::MULTIPLE_CHOICES => Some("Multiple Choices"),
            StatusCode::MOVED_PERMANENTLY => Some("Moved Permanently"),
            StatusCode::FOUND => Some("Found"),
            StatusCode::SEE_OTHER => Some("See Other"),
            StatusCode::NOT_MODIFIED => Some("Not Modified"),
            StatusCode::USE_PROXY => Some("Use Proxy"),
            StatusCode::TEMPORARY_REDIRECT => Some("Temporary Redirect"),
            StatusCode::PERMANENT_REDIRECT => Some("Permanent Redirect"),
            StatusCode::BAD_REQUEST => Some("Bad Request"),
            StatusCode::UNAUTHORIZED => Some("Unauthorized"),
            StatusCode::PAYMENT_REQUIRED => Some("Payment Required"),
            StatusCode::FORBIDDEN => Some("Forbidden"),
            StatusCode::NOT_FOUND => Some("Not Found"),
            StatusCode::METHOD_NOT_ALLOWED => Some("Method Not Allowed"),
            StatusCode::NOT_ACCEPTABLE => Some("Not Acceptable"),
            StatusCode::PROXY_AUTHENTICATION_REQUIRED => Some("Proxy Authentication Required"),
            StatusCode::REQUEST_TIMEOUT => Some("Request Timeout"),
            StatusCode::CONFLICT => Some("Conflict"),
            StatusCode::GONE => Some("Gone"),
            StatusCode::LENGTH_REQUIRED => Some("Length Required"),
            StatusCode::PRECONDITION_FAILED => Some("Precondition Failed"),
            StatusCode::PAYLOAD_TOO_LARGE => Some("Payload Too Large"),
            StatusCode::URI_TOO_LONG => Some("URI Too Long"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => Some("Unsupported Media Type"),
            StatusCode::RANGE_NOT_SATISFIABLE => Some("Range Not Satisfiable"),
            StatusCode::EXPECTATION_FAILED => Some("Expectation Failed"),
            StatusCode::IM_A_TEAPOT => Some("I'm a teapot"),
            StatusCode::UNPROCESSABLE_ENTITY => Some("Unprocessable Entity"),
            StatusCode::TOO_EARLY => Some("Too Early"),
            StatusCode::UPGRADE_REQUIRED => Some("Upgrade Required"),
            StatusCode::PRECONDITION_REQUIRED => Some("Precondition Required"),
            StatusCode::TOO_MANY_REQUESTS => Some("Too Many Requests"),
            StatusCode::INTERNAL_SERVER_ERROR => Some("Internal Server Error"),
            StatusCode::NOT_IMPLEMENTED => Some("Not Implemented"),
            StatusCode::BAD_GATEWAY => Some("Bad Gateway"),
            StatusCode::SERVICE_UNAVAILABLE => Some("Service Unavailable"),
            StatusCode::GATEWAY_TIMEOUT => Some("Gateway Timeout"),
            StatusCode::HTTP_VERSION_NOT_SUPPORTED => Some("HTTP Version Not Supported"),
            StatusCode::NETWORK_AUTHENTICATION_REQUIRED => Some("Network Authentication Required"),
            _ => None,
        }
    }
}

/// Returns `StatusCode::OK` (200) as the default.
/// 返回 `StatusCode::OK` (200) 作为默认值。
impl Default for StatusCode
{
    fn default() -> Self
    {
        StatusCode::OK
    }
}

/// Formats the status code as `"CODE Reason"`, e.g. `"404 Not Found"`.
/// 将状态码格式化为 `"CODE 原因短语"`，例如 `"404 Not Found"`。
impl fmt::Display for StatusCode
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if let Some(reason) = self.canonical_reason()
        {
            write!(f, "{} {}", self.0, reason)
        }
        else
        {
            write!(f, "{}", self.0)
        }
    }
}

/// Converts a `u16` into a `StatusCode` without validation.
/// 将 `u16` 转换为 `StatusCode`，不进行验证。
impl From<u16> for StatusCode
{
    fn from(code: u16) -> Self
    {
        StatusCode(code)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_status_code_categories()
    {
        assert!(StatusCode::OK.is_success());
        assert!(StatusCode::CREATED.is_success());
        assert!(StatusCode::NOT_FOUND.is_client_error());
        assert!(StatusCode::INTERNAL_SERVER_ERROR.is_server_error());
        assert!(StatusCode::FOUND.is_redirection());
        assert!(StatusCode::CONTINUE.is_informational());
    }

    #[test]
    fn test_status_code_display()
    {
        assert_eq!(StatusCode::OK.to_string(), "200 OK");
        assert_eq!(StatusCode::NOT_FOUND.to_string(), "404 Not Found");
    }
}
