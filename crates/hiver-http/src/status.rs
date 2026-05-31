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

impl StatusCode {
    // 1xx Informational / 1xx 信息响应

    /// 100 Continue
    pub const CONTINUE: StatusCode = StatusCode(100);
    /// 101 Switching Protocols
    pub const SWITCHING_PROTOCOLS: StatusCode = StatusCode(101);
    /// 102 Processing
    pub const PROCESSING: StatusCode = StatusCode(102);

    // 2xx Success / 2xx 成功响应

    /// 200 OK
    pub const OK: StatusCode = StatusCode(200);
    /// 201 Created
    pub const CREATED: StatusCode = StatusCode(201);
    /// 202 Accepted
    pub const ACCEPTED: StatusCode = StatusCode(202);
    /// 203 Non-Authoritative Information
    pub const NON_AUTHORITATIVE_INFORMATION: StatusCode = StatusCode(203);
    /// 204 No Content
    pub const NO_CONTENT: StatusCode = StatusCode(204);
    /// 205 Reset Content
    pub const RESET_CONTENT: StatusCode = StatusCode(205);
    /// 206 Partial Content
    pub const PARTIAL_CONTENT: StatusCode = StatusCode(206);

    // 3xx Redirection / 3xx 重定向

    /// 300 Multiple Choices
    pub const MULTIPLE_CHOICES: StatusCode = StatusCode(300);
    /// 301 Moved Permanently
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode(301);
    /// 302 Found
    pub const FOUND: StatusCode = StatusCode(302);
    /// 303 See Other
    pub const SEE_OTHER: StatusCode = StatusCode(303);
    /// 304 Not Modified
    pub const NOT_MODIFIED: StatusCode = StatusCode(304);
    /// 305 Use Proxy
    pub const USE_PROXY: StatusCode = StatusCode(305);
    /// 307 Temporary Redirect
    pub const TEMPORARY_REDIRECT: StatusCode = StatusCode(307);
    /// 308 Permanent Redirect
    pub const PERMANENT_REDIRECT: StatusCode = StatusCode(308);

    // 4xx Client Error / 4xx 客户端错误

    /// 400 Bad Request
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    /// 401 Unauthorized
    pub const UNAUTHORIZED: StatusCode = StatusCode(401);
    /// 402 Payment Required
    pub const PAYMENT_REQUIRED: StatusCode = StatusCode(402);
    /// 403 Forbidden
    pub const FORBIDDEN: StatusCode = StatusCode(403);
    /// 404 Not Found
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    /// 405 Method Not Allowed
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode(405);
    /// 406 Not Acceptable
    pub const NOT_ACCEPTABLE: StatusCode = StatusCode(406);
    /// 407 Proxy Authentication Required
    pub const PROXY_AUTHENTICATION_REQUIRED: StatusCode = StatusCode(407);
    /// 408 Request Timeout
    pub const REQUEST_TIMEOUT: StatusCode = StatusCode(408);
    /// 409 Conflict
    pub const CONFLICT: StatusCode = StatusCode(409);
    /// 410 Gone
    pub const GONE: StatusCode = StatusCode(410);
    /// 411 Length Required
    pub const LENGTH_REQUIRED: StatusCode = StatusCode(411);
    /// 412 Precondition Failed
    pub const PRECONDITION_FAILED: StatusCode = StatusCode(412);
    /// 413 Payload Too Large
    pub const PAYLOAD_TOO_LARGE: StatusCode = StatusCode(413);
    /// 414 URI Too Long
    pub const URI_TOO_LONG: StatusCode = StatusCode(414);
    /// 415 Unsupported Media Type
    pub const UNSUPPORTED_MEDIA_TYPE: StatusCode = StatusCode(415);
    /// 416 Range Not Satisfiable
    pub const RANGE_NOT_SATISFIABLE: StatusCode = StatusCode(416);
    /// 417 Expectation Failed
    pub const EXPECTATION_FAILED: StatusCode = StatusCode(417);
    /// 418 I'm a teapot
    pub const IM_A_TEAPOT: StatusCode = StatusCode(418);
    /// 422 Unprocessable Entity
    pub const UNPROCESSABLE_ENTITY: StatusCode = StatusCode(422);
    /// 425 Too Early
    pub const TOO_EARLY: StatusCode = StatusCode(425);
    /// 426 Upgrade Required
    pub const UPGRADE_REQUIRED: StatusCode = StatusCode(426);
    /// 428 Precondition Required
    pub const PRECONDITION_REQUIRED: StatusCode = StatusCode(428);
    /// 429 Too Many Requests
    pub const TOO_MANY_REQUESTS: StatusCode = StatusCode(429);

    // 5xx Server Error / 5xx 服务器错误

    /// 500 Internal Server Error
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);
    /// 501 Not Implemented
    pub const NOT_IMPLEMENTED: StatusCode = StatusCode(501);
    /// 502 Bad Gateway
    pub const BAD_GATEWAY: StatusCode = StatusCode(502);
    /// 503 Service Unavailable
    pub const SERVICE_UNAVAILABLE: StatusCode = StatusCode(503);
    /// 504 Gateway Timeout
    pub const GATEWAY_TIMEOUT: StatusCode = StatusCode(504);
    /// 505 HTTP Version Not Supported
    pub const HTTP_VERSION_NOT_SUPPORTED: StatusCode = StatusCode(505);
    /// 511 Network Authentication Required
    pub const NETWORK_AUTHENTICATION_REQUIRED: StatusCode = StatusCode(511);

    /// Create a `StatusCode` from a u16
    /// 从u16创建状态码
    pub const fn from_u16(code: u16) -> StatusCode {
        StatusCode(code)
    }

    /// Get the status code as u16
    /// 获取u16格式的状态码
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    /// Check if this is a 1xx informational response
    /// 检查是否为1xx信息响应
    pub const fn is_informational(self) -> bool {
        self.0 >= 100 && self.0 < 200
    }

    /// Check if this is a 2xx success response
    /// 检查是否为2xx成功响应
    pub const fn is_success(self) -> bool {
        self.0 >= 200 && self.0 < 300
    }

    /// Check if this is a 3xx redirection
    /// 检查是否为3xx重定向
    pub const fn is_redirection(self) -> bool {
        self.0 >= 300 && self.0 < 400
    }

    /// Check if this is a 4xx client error
    /// 检查是否为4xx客户端错误
    pub const fn is_client_error(self) -> bool {
        self.0 >= 400 && self.0 < 500
    }

    /// Check if this is a 5xx server error
    /// 检查是否为5xx服务器错误
    pub const fn is_server_error(self) -> bool {
        self.0 >= 500 && self.0 < 600
    }

    /// Get the canonical reason phrase for this status code
    /// 获取此状态码的标准原因短语
    pub fn canonical_reason(self) -> Option<&'static str> {
        match self {
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

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode::OK
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(reason) = self.canonical_reason() {
            write!(f, "{} {}", self.0, reason)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        StatusCode(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_categories() {
        assert!(StatusCode::OK.is_success());
        assert!(StatusCode::CREATED.is_success());
        assert!(StatusCode::NOT_FOUND.is_client_error());
        assert!(StatusCode::INTERNAL_SERVER_ERROR.is_server_error());
        assert!(StatusCode::FOUND.is_redirection());
        assert!(StatusCode::CONTINUE.is_informational());
    }

    #[test]
    fn test_status_code_display() {
        assert_eq!(StatusCode::OK.to_string(), "200 OK");
        assert_eq!(StatusCode::NOT_FOUND.to_string(), "404 Not Found");
    }
}
