//! HTTP request logging middleware
//! HTTP 请求日志中间件
//!
//! This middleware provides structured logging for HTTP requests and responses.
//! 本中间件为 HTTP 请求和响应提供结构化日志。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use hiver_http::{Request, Response, Result};
use hiver_router::{Middleware, Next};

/// Logger middleware for HTTP requests
/// HTTP 请求日志中间件
///
/// Logs incoming requests and outgoing responses with timing information.
/// 对传入的请求和传出的响应进行日志记录，包含时间信息。
///
/// # Output format / 输出格式
///
/// ```text
/// 2025-01-24 19:15:30.123 INFO  4838 [main] n.middleware.http : GET /api/users
/// 2025-01-24 19:15:30.456 INFO  4838 [main] n.middleware.http : GET /api/users 200 45ms
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_router::Router;
/// use hiver_middleware::LoggerMiddleware;
/// use std::sync::Arc;
///
/// let logger = Arc::new(LoggerMiddleware::new());
/// let router = Router::new()
///     .middleware(logger)
///     .get("/", handler);
/// ```
#[derive(Clone)]
pub struct LoggerMiddleware {
    /// Log request headers
    /// 记录请求headers
    pub log_headers: bool,

    /// Include query string in path
    /// 路径中包含查询字符串
    pub include_query: bool,

    /// Log level for successful requests
    /// 成功请求的日志级别
    pub success_level: LogLevel,

    /// Log level for failed requests
    /// 失败请求的日志级别
    pub error_level: LogLevel,
}

/// Log level for middleware output
/// 中间件输出的日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Debug level
    /// DEBUG 级别
    Debug,
    /// Info level
    /// INFO 级别
    Info,
    /// Warn level
    /// WARN 级别
    Warn,
}

impl LoggerMiddleware {
    /// Create a new logger middleware
    /// 创建新的日志中间件
    pub fn new() -> Self {
        Self {
            log_headers: false,
            include_query: true,
            success_level: LogLevel::Info,
            error_level: LogLevel::Warn,
        }
    }

    /// Enable header logging
    /// 启用header日志
    pub fn log_headers(mut self, log: bool) -> Self {
        self.log_headers = log;
        self
    }

    /// Include query string in path
    /// 路径中包含查询字符串
    pub fn include_query(mut self, include: bool) -> Self {
        self.include_query = include;
        self
    }

    /// Set log level for successful requests
    /// 设置成功请求的日志级别
    pub fn success_level(mut self, level: LogLevel) -> Self {
        self.success_level = level;
        self
    }

    /// Set log level for failed requests
    /// 设置失败请求的日志级别
    pub fn error_level(mut self, level: LogLevel) -> Self {
        self.error_level = level;
        self
    }
}

impl Default for LoggerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Middleware<S> for LoggerMiddleware
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let log_headers = self.log_headers;
        let include_query = self.include_query;
        let success_level = self.success_level;
        let error_level = self.error_level;

        Box::pin(async move {
            let method = req.method();
            let path = if include_query {
                req.uri().to_string()
            } else {
                req.path().to_string()
            };

            // Extract client IP and user agent for logging
            let client_ip = req
                .header("X-Forwarded-For")
                .or_else(|| req.header("X-Real-IP"))
                .map(ToString::to_string);

            let _user_agent = req.header("User-Agent").map(ToString::to_string);

            let start = Instant::now();

            // Log request with structured fields
            // 使用结构化字段记录请求
            if log_headers {
                tracing::info!(
                    target: "hiver.middleware.http",
                    method = %method,
                    uri = %path,
                    client = ?client_ip,
                    headers = ?req.headers(),
                    "Request started"
                );
            } else {
                tracing::info!(
                    target: "hiver.middleware.http",
                    method = %method,
                    uri = %path,
                    client = ?client_ip,
                    "Request"
                );
            }

            let response = next.call(req, state).await;
            let duration = start.elapsed();
            let duration_ms = duration.as_millis();

            // Log response with status and timing
            // 记录响应状态和时间
            match &response {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let level = match success_level {
                        LogLevel::Debug => tracing::Level::DEBUG,
                        LogLevel::Info => tracing::Level::INFO,
                        LogLevel::Warn => tracing::Level::WARN,
                    };

                    // Color-coded logging based on status code
                    // 根据状态码进行颜色编码的日志记录
                    if status >= 500 {
                        tracing::error!(
                            target: "hiver.middleware.http",
                            method = %method,
                            uri = %path,
                            status = status,
                            duration_ms = duration_ms,
                            client = ?client_ip,
                            "Server error"
                        );
                    } else if status >= 400 {
                        tracing::warn!(
                            target: "hiver.middleware.http",
                            method = %method,
                            uri = %path,
                            status = status,
                            duration_ms = duration_ms,
                            client = ?client_ip,
                            "Client error"
                        );
                    } else {
                        match level {
                            tracing::Level::DEBUG => {
                                tracing::debug!(
                                    target: "hiver.middleware.http",
                                    method = %method,
                                    uri = %path,
                                    status = status,
                                    duration_ms = duration_ms,
                                    "Completed"
                                );
                            },
                            tracing::Level::INFO => {
                                tracing::info!(
                                    target: "hiver.middleware.http",
                                    method = %method,
                                    uri = %path,
                                    status = status,
                                    duration_ms = duration_ms,
                                    "Completed"
                                );
                            },
                            tracing::Level::WARN => {
                                tracing::warn!(
                                    target: "hiver.middleware.http",
                                    method = %method,
                                    uri = %path,
                                    status = status,
                                    duration_ms = duration_ms,
                                    "Completed"
                                );
                            },
                            _ => {},
                        }
                    }
                },
                Err(e) => {
                    let level = match error_level {
                        LogLevel::Debug => tracing::Level::DEBUG,
                        LogLevel::Info => tracing::Level::INFO,
                        LogLevel::Warn => tracing::Level::WARN,
                    };

                    match level {
                        tracing::Level::DEBUG => {
                            tracing::debug!(
                                target: "hiver.middleware.http",
                                method = %method,
                                uri = %path,
                                duration_ms = duration_ms,
                                client = ?client_ip,
                                error = %e,
                                "Failed"
                            );
                        },
                        tracing::Level::INFO => {
                            tracing::info!(
                                target: "hiver.middleware.http",
                                method = %method,
                                uri = %path,
                                duration_ms = duration_ms,
                                client = ?client_ip,
                                error = %e,
                                "Failed"
                            );
                        },
                        tracing::Level::WARN => {
                            tracing::warn!(
                                target: "hiver.middleware.http",
                                method = %method,
                                uri = %path,
                                duration_ms = duration_ms,
                                client = ?client_ip,
                                error = %e,
                                "Failed"
                            );
                        },
                        _ => {},
                    }
                },
            }

            response
        })
    }
}

/// MDC (Mapped Diagnostic Context) utility
/// MDC（映射诊断上下文）工具
///
/// Provides thread-local context for logging.
/// 为日志记录提供线程本地上下文。
pub struct Mdc;

impl Mdc {
    /// Put a value into MDC
    /// 向MDC中放入值
    pub fn put(key: impl Into<String>, value: impl Into<String>) {
        let _key = key.into();
        let value = value.into();
        tracing::info_span!("mdc", key = &value);
    }

    /// Get a value from MDC
    /// 从MDC中获取值
    pub fn get(_key: &str) -> Option<String> {
        // Note: MDC requires async-local storage for proper async context
        // 注意：MDC 需要异步本地存储来支持正确的异步上下文
        None
    }

    /// Remove a value from MDC
    /// 从MDC中移除值
    pub fn remove(_key: &str) {
        // TODO: Implement with async-local storage
        // TODO：使用异步本地存储实现
    }

    /// Clear all MDC values
    /// 清除所有MDC值
    pub fn clear() {
        // TODO: Implement with async-local storage
        // TODO：使用异步本地存储实现
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = LoggerMiddleware::new();
        assert!(!logger.log_headers);
        assert!(logger.include_query);
    }

    #[test]
    fn test_logger_builder() {
        let logger = LoggerMiddleware::new()
            .log_headers(true)
            .include_query(false)
            .success_level(LogLevel::Debug);

        assert!(logger.log_headers);
        assert!(!logger.include_query);
        assert_eq!(logger.success_level, LogLevel::Debug);
    }

    #[test]
    fn test_log_level() {
        assert_eq!(LoggerMiddleware::new().success_level, LogLevel::Info);
        assert_eq!(LoggerMiddleware::new().error_level, LogLevel::Warn);
    }
}
