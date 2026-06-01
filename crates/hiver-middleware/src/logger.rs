//! HTTP request logging middleware
//! HTTP 请求日志中间件
//!
//! This middleware provides structured logging for HTTP requests and responses.
//! 本中间件为 HTTP 请求和响应提供结构化日志。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;
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
/// Provides async-local context for logging using `tokio::task_local`.
/// Values stored in MDC are automatically propagated across `.await` points
/// within the same task, but NOT to spawned child tasks.
///
/// 使用 `tokio::task_local` 为日志记录提供异步本地上下文。
/// 存储在 MDC 中的值会在同一任务的 `.await` 点之间自动传播，
/// 但不会传播到生成（spawned）的子任务。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_middleware::logger::Mdc;
///
/// Mdc::put("user_id", "123");
/// Mdc::put("request_id", "abc-123");
///
/// // Later, after .await points:
/// if let Some(user) = Mdc::get("user_id") {
///     tracing::info!("Processing user: {}", user);
/// }
///
/// Mdc::remove("user_id");
/// Mdc::clear();
/// ```
pub struct Mdc;

// Task-local storage for MDC context.
// Uses a RefCell so we can mutate within an async context.
tokio::task_local! {
    static MDC_MAP: std::cell::RefCell<HashMap<String, String>>;
}

impl Mdc {
    /// Put a value into MDC.
    /// The value will be visible in log output for the current async task.
    /// 向MDC中放入值。该值将显示在当前异步任务的日志输出中。
    ///
    /// This is a no-op if called outside a tokio task context (e.g., before
    /// the async runtime has started).
    /// 如果在 tokio 任务上下文之外调用（例如异步运行时启动前），则为空操作。
    pub fn put(key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        MDC_MAP.try_with(|cell| {
            cell.borrow_mut().insert(key.clone(), value);
        }).ok();
    }

    /// Get a value from MDC by key.
    /// Returns `None` if the key does not exist or if called outside a task context.
    /// 按键从MDC中获取值。如果键不存在或在任务上下文之外调用，则返回 `None`。
    pub fn get(key: &str) -> Option<String> {
        MDC_MAP.try_with(|cell| {
            cell.borrow().get(key).cloned()
        }).ok().flatten()
    }

    /// Remove a value from MDC by key.
    /// 按键从MDC中移除值。
    pub fn remove(key: &str) {
        MDC_MAP.try_with(|cell| {
            cell.borrow_mut().remove(key);
        }).ok();
    }

    /// Clear all MDC values for the current async task.
    /// 清除当前异步任务的所有MDC值。
    pub fn clear() {
        MDC_MAP.try_with(|cell| {
            cell.borrow_mut().clear();
        }).ok();
    }

    /// Initialize the MDC context for the current async task.
    /// Must be called at the start of an async task that wants to use MDC.
    /// Creates an empty context if one doesn't exist yet.
    /// 为当前异步任务初始化 MDC 上下文。
    /// 必须在需要使用 MDC 的异步任务开始时调用。
    /// 如果上下文尚不存在，则创建一个空上下文。
    pub async fn init() {
        MDC_MAP.scope(
            std::cell::RefCell::new(HashMap::new()),
            async { /* context is now available */ },
        ).await;
    }

    /// Run the given future with an initialized MDC context.
    /// All MDC operations within the future (and its `.await` points)
    /// will operate on this isolated context.
    /// 使用初始化的 MDC 上下文运行给定的 future。
    /// future 内（及其中所有 `.await` 点）的所有 MDC 操作
    /// 都将在此隔离上下文上运行。
    pub async fn with_scope<F, T>(f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        MDC_MAP.scope(
            std::cell::RefCell::new(HashMap::new()),
            f,
        ).await
    }

    /// Run the given future with a pre-populated MDC context.
    /// The initial values will be available to MDC::get().
    /// 使用预填充的 MDC 上下文运行给定的 future。
    /// 初始值将可用于 MDC::get()。
    pub async fn with_initial_values<F, T>(values: HashMap<String, String>, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        MDC_MAP.scope(
            std::cell::RefCell::new(values),
            f,
        ).await
    }

    /// Check if the MDC has a value for the given key.
    /// 检查 MDC 是否存在给定键的值。
    pub fn has(key: &str) -> bool {
        MDC_MAP.try_with(|cell| {
            cell.borrow().contains_key(key)
        }).ok().unwrap_or(false)
    }

    /// Return the number of entries in the current MDC context.
    /// 返回当前 MDC 上下文中的条目数。
    pub fn len() -> usize {
        MDC_MAP.try_with(|cell| {
            cell.borrow().len()
        }).ok().unwrap_or(0)
    }

    /// Return all entries as a snapshot HashMap.
    /// 返回所有条目的快照 HashMap。
    pub fn snapshot() -> HashMap<String, String> {
        MDC_MAP.try_with(|cell| {
            cell.borrow().clone()
        }).ok().unwrap_or_default()
    }

    /// Check if the MDC context is empty.
    /// 检查 MDC 上下文是否为空。
    pub fn is_empty() -> bool {
        Self::len() == 0
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

    #[tokio::test]
    async fn test_mdc_put_get() {
        Mdc::with_scope(async {
            Mdc::put("key1", "value1");
            assert_eq!(Mdc::get("key1"), Some("value1".to_string()));
            assert!(Mdc::has("key1"));
            assert!(!Mdc::is_empty());
            assert_eq!(Mdc::len(), 1);
        }).await;
    }

    #[tokio::test]
    async fn test_mdc_remove() {
        Mdc::with_scope(async {
            Mdc::put("key1", "value1");
            assert!(Mdc::has("key1"));
            Mdc::remove("key1");
            assert!(!Mdc::has("key1"));
            assert_eq!(Mdc::get("key1"), None);
            assert!(Mdc::is_empty());
        }).await;
    }

    #[tokio::test]
    async fn test_mdc_clear() {
        Mdc::with_scope(async {
            Mdc::put("a", "1");
            Mdc::put("b", "2");
            assert_eq!(Mdc::len(), 2);
            Mdc::clear();
            assert!(Mdc::is_empty());
        }).await;
    }

    #[tokio::test]
    async fn test_mdc_snapshot() {
        Mdc::with_scope(async {
            Mdc::put("x", "10");
            Mdc::put("y", "20");
            let snapshot = Mdc::snapshot();
            assert_eq!(snapshot.get("x"), Some(&"10".to_string()));
            assert_eq!(snapshot.get("y"), Some(&"20".to_string()));
            assert_eq!(snapshot.len(), 2);
        }).await;
    }

    #[tokio::test]
    async fn test_mdc_isolation_between_scopes() {
        Mdc::with_scope(async {
            Mdc::put("scope1_key", "scope1_val");
            assert_eq!(Mdc::get("scope1_key"), Some("scope1_val".to_string()));
        }).await;

        // Second scope should be empty
        Mdc::with_scope(async {
            assert!(Mdc::get("scope1_key").is_none());
            assert!(Mdc::is_empty());
        }).await;
    }

    #[tokio::test]
    async fn test_mdc_with_initial_values() {
        let mut initial = std::collections::HashMap::new();
        initial.insert("pre_set".to_string(), "value".to_string());

        Mdc::with_initial_values(initial, async {
            assert_eq!(Mdc::get("pre_set"), Some("value".to_string()));
            Mdc::put("after_set", "after_value");
            assert_eq!(Mdc::get("after_set"), Some("after_value".to_string()));
        }).await;
    }

    #[tokio::test]
    async fn test_mdc_persists_across_await() {
        Mdc::with_scope(async {
            Mdc::put("persist_key", "persist_value");
            // Simulate an await point
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            assert_eq!(Mdc::get("persist_key"), Some("persist_value".to_string()));
        }).await;
    }

    #[test]
    fn test_mdc_outside_task_context() {
        // Calling MDC methods outside of a tokio task context should not panic
        Mdc::put("key", "value");
        assert_eq!(Mdc::get("key"), None);
        assert!(!Mdc::has("key"));
        assert!(Mdc::is_empty());
        Mdc::remove("key");
        Mdc::clear();
    }
}
