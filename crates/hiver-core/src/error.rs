//! Error types
//! 错误类型
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`ResponseStatus`
//! - `ResponseEntityExceptionHandler`
//! - @`ExceptionHandler`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

/// Framework error type
/// 框架错误类型
#[derive(Debug)]
pub struct Error
{
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error
{
    /// Create a new error
    /// 创建新错误
    pub fn new(kind: ErrorKind) -> Self
    {
        Self {
            kind,
            message: String::new(),
            source: None,
        }
    }

    /// Create a new error with a message
    /// 创建带消息的新错误
    pub fn with_message(kind: ErrorKind, message: impl Into<String>) -> Self
    {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    /// Create a new error from `ErrorKind`
    /// `从ErrorKind创建新错误`
    ///
    /// When `ErrorKind` already contains a message (like `NotFound` or Internal),
    /// that message is extracted and used as the error message.
    pub fn from_kind(kind: ErrorKind) -> Self
    {
        let message = match &kind
        {
            ErrorKind::NotFound(s) | ErrorKind::Internal(s) => Some(s.clone()),
            _ => None,
        };
        Self {
            kind,
            message: message.unwrap_or_default(),
            source: None,
        }
    }

    /// Create an internal error with a message
    /// 创建带消息的内部错误
    pub fn internal(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Internal(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Create a not found error with a message
    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::NotFound(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    // ── Domain-layer convenience constructors ─────────────────────────────
    // ── 领域层便捷构造函数 ──────────────────────────────────────────────────

    /// Create a data-layer error (DB query, entity not found, duplicate key).
    /// 创建数据层错误（DB 查询、实体未找到、重复键）。
    pub fn data(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Data(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Create an infrastructure error (connection, pool, network, driver).
    /// 创建基础设施错误（连接、连接池、网络、驱动）。
    pub fn infra(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Infra(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Create a configuration error (invalid value, missing key, parse error).
    /// 创建配置错误（无效值、缺失键、解析错误）。
    pub fn config(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Config(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Create a messaging error (kafka, amqp, bus, stream).
    /// 创建消息传递错误（kafka、amqp、总线、流）。
    pub fn messaging(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Messaging(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Create a security error (auth, authz, token, crypto).
    /// 创建安全错误（认证、授权、令牌、加密）。
    pub fn security(msg: impl Into<String>) -> Self
    {
        Self {
            kind: ErrorKind::Security(String::new()),
            message: msg.into(),
            source: None,
        }
    }

    /// Get the error kind
    /// 获取错误类型
    pub fn kind(&self) -> &ErrorKind
    {
        &self.kind
    }

    /// Get the error message
    /// 获取错误消息
    pub fn message(&self) -> &str
    {
        &self.message
    }

    /// Attach a source error (error chain).
    /// 附加源错误（错误链）。
    ///
    /// # Example / 示例
    pub fn caused_by(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the source error in the chain.
    /// 获取错误链中的源错误。
    pub fn source_error(&self) -> Option<&(dyn std::error::Error + Send + Sync)>
    {
        self.source.as_ref().map(AsRef::as_ref)
    }

    /// Get the full error chain as a vector of messages.
    /// 获取完整错误链作为消息向量。
    pub fn chain(&self) -> Vec<String>
    {
        let mut chain = vec![self.to_string()];
        let mut source: Option<&(dyn std::error::Error + 'static)> =
            self.source.as_ref().map(|b| b.as_ref() as _);
        while let Some(err) = source
        {
            chain.push(err.to_string());
            source = err.source();
        }
        chain
    }
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.message.is_empty()
        {
            write!(f, "{:?}", self.kind)
        }
        else
        {
            write!(f, "{}: {}", self.kind, self.message)
        }
    }
}

impl std::error::Error for Error
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        self.source
            .as_ref()
            .map(|b| b.as_ref() as &(dyn std::error::Error + 'static))
    }
}

// ── From conversions for error chaining ────────────────────────────────

impl From<anyhow::Error> for Error
{
    fn from(err: anyhow::Error) -> Self
    {
        Self::internal(format!("{}", err))
    }
}

impl From<std::io::Error> for Error
{
    fn from(err: std::io::Error) -> Self
    {
        Self::internal(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error
{
    fn from(err: serde_json::Error) -> Self
    {
        Self::internal(format!("JSON error: {}", err))
    }
}

/// Error kind
/// 错误类型
///
/// Combines HTTP-status-shaped variants (for the web layer) with domain
/// variants (for data/infra/config layers). This lets every crate in the
/// workspace funnel its failures through a single `hiver_core::Error`
/// instead of defining 75 parallel error enums.
/// 合并 HTTP 状态码变体（Web 层）与领域变体（数据/基础设施/配置层）。
/// 使工作区中的每个 crate 都能通过单一 `hiver_core::Error` 汇聚失败，
/// 而非定义 75 个并行错误枚举。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind
{
    // ── HTTP-layer kinds (map to status codes) ───────────────────────────
    // ── HTTP 层类型（映射到状态码）───────────────────────────────────────
    /// Bad request (400)
    /// 错误请求 (400)
    BadRequest,

    /// Unauthorized (401)
    /// 未授权 (401)
    Unauthorized,

    /// Forbidden (403)
    /// 禁止访问 (403)
    Forbidden,

    /// Not found (404)
    /// 未找到 (404)
    NotFound(String),

    /// Method not allowed (405)
    /// 方法不允许 (405)
    MethodNotAllowed,

    /// Conflict (409)
    /// 冲突 (409)
    Conflict,

    /// Internal server error (500)
    /// 内部服务器错误 (500)
    Internal(String),

    /// Service unavailable (503)
    /// 服务不可用 (503)
    ServiceUnavailable,

    /// Custom error with code
    /// 带代码的自定义错误
    Custom(u16, String),

    // ── Domain kinds (layer-specific failures, not directly HTTP-shaped) ─
    // ── 领域类型（分层特定失败，不直接映射 HTTP）──────────────────────────
    /// Data-layer failure (DB query, entity not found, duplicate key, etc.)
    /// 数据层失败（DB 查询、实体未找到、重复键等）
    Data(String),

    /// Infrastructure failure (connection, pool, network, driver)
    /// 基础设施失败（连接、连接池、网络、驱动）
    Infra(String),

    /// Configuration failure (invalid value, missing key, parse error)
    /// 配置失败（无效值、缺失键、解析错误）
    Config(String),

    /// Caching failure (miss, eviction, serialization)
    /// 缓存失败（未命中、驱逐、序列化）
    Cache(String),

    /// Messaging failure (kafka, amqp, bus, stream)
    /// 消息传递失败（kafka、amqp、总线、流）
    Messaging(String),

    /// Security failure (auth, authz, token, crypto)
    /// 安全失败（认证、授权、令牌、加密）
    Security(String),
}

impl fmt::Display for ErrorKind
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ErrorKind::BadRequest => write!(f, "Bad Request"),
            ErrorKind::Unauthorized => write!(f, "Unauthorized"),
            ErrorKind::Forbidden => write!(f, "Forbidden"),
            ErrorKind::NotFound(s) => write!(f, "Not Found: {}", s),
            ErrorKind::MethodNotAllowed => write!(f, "Method Not Allowed"),
            ErrorKind::Conflict => write!(f, "Conflict"),
            ErrorKind::Internal(s) => write!(f, "Internal Server Error: {}", s),
            ErrorKind::ServiceUnavailable => write!(f, "Service Unavailable"),
            ErrorKind::Custom(code, msg) => write!(f, "Error {}: {}", code, msg),
            ErrorKind::Data(s) => write!(f, "Data error: {}", s),
            ErrorKind::Infra(s) => write!(f, "Infrastructure error: {}", s),
            ErrorKind::Config(s) => write!(f, "Configuration error: {}", s),
            ErrorKind::Cache(s) => write!(f, "Cache error: {}", s),
            ErrorKind::Messaging(s) => write!(f, "Messaging error: {}", s),
            ErrorKind::Security(s) => write!(f, "Security error: {}", s),
        }
    }
}

impl ErrorKind
{
    /// Get the HTTP status code for this error
    /// 获取此错误的HTTP状态码
    pub fn status_code(&self) -> u16
    {
        match self
        {
            ErrorKind::BadRequest => 400,
            ErrorKind::Unauthorized => 401,
            ErrorKind::Forbidden => 403,
            ErrorKind::NotFound(_) => 404,
            ErrorKind::MethodNotAllowed => 405,
            ErrorKind::Conflict => 409,
            ErrorKind::Internal(_) => 500,
            ErrorKind::ServiceUnavailable => 503,
            ErrorKind::Custom(code, _) => *code,
            // Domain kinds: map to the most appropriate HTTP status when they
            // surface at the web layer. Data/Config/Cache → 500 (server-side),
            // Infra → 503 (downstream), Messaging → 503, Security → 403.
            // 领域类型：当浮到 Web 层时映射到最合适的 HTTP 状态码。
            // Data/Config/Cache → 500（服务端），Infra → 503（下游），
            // Messaging → 503，Security → 403。
            ErrorKind::Data(_) | ErrorKind::Config(_) | ErrorKind::Cache(_) => 500,
            ErrorKind::Infra(_) | ErrorKind::Messaging(_) => 503,
            ErrorKind::Security(_) => 403,
        }
    }
}

/// Result type alias
/// Result类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// Canonical unified error type for the entire hiver workspace.
/// 整个 hiver 工作区的规范统一错误类型。
///
/// This is the single funnel point: every crate should convert its
/// domain-specific errors into `HiverError` (via `From` impls or the
/// convenience constructors `Error::data()` / `Error::infra()` / etc.)
/// rather than defining 75 parallel error enums.
/// 这是唯一的汇聚点：每个 crate 应将其领域特定错误转换为 `HiverError`
/// （通过 `From` impl 或便捷构造函数 `Error::data()` / `Error::infra()` 等），
/// 而非定义 75 个并行错误枚举。
pub type HiverError = Error;

/// Canonical unified Result alias for the entire hiver workspace.
/// 整个 hiver 工作区的规范统一 Result 别名。
pub type HiverResult<T> = std::result::Result<T, HiverError>;
