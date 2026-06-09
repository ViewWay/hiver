//! CSRF (Cross-Site Request Forgery) protection module
//! CSRF（跨站请求伪造）防护模块
//!
//! Provides token generation, validation, and repository abstractions
//! for protecting state-changing requests against CSRF attacks.
//! 提供令牌生成、验证和存储库抽象，用于保护状态变更请求免受CSRF攻击。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! ```java
//! // Spring Security CSRF configuration
//! http.csrf(csrf -> csrf
//!     .csrfTokenRepository(CookieCsrfTokenRepository.withHttpOnlyFalse())
//!     .ignoringRequestMatchers("/api/public/**")
//! );
//! ```
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_security::csrf::{CsrfProtectionConfig, CsrfValidator, InMemoryCsrfTokenRepository};
//!
//! let config = CsrfProtectionConfig::default();
//! let repository = InMemoryCsrfTokenRepository::new();
//! let validator = CsrfValidator::new(config, repository);
//!
//! // Generate a token for a session
//! let token = validator.generate_token("session-123").await.unwrap();
//!
//! // Validate a submitted token
//! let valid = validator.validate_token("session-123", &token.token).await.unwrap();
//! ```

use std::collections::HashMap;

use rand::RngCore;
use tokio::sync::RwLock;

use crate::SecurityError;

// ---------------------------------------------------------------------------
// CsrfToken
// ---------------------------------------------------------------------------

/// A CSRF token carrying its raw value and the header/parameter names
/// used to transmit it.
/// CSRF令牌，携带其原始值以及用于传输它的header/参数名称。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// CsrfToken csrfToken = (CsrfToken) request.getAttribute(CsrfToken.class.getName());
/// String token = csrfToken.getToken();
/// ```
#[derive(Debug, Clone)]
pub struct CsrfToken {
    /// The raw token string.
    /// 原始令牌字符串。
    pub token: String,

    /// HTTP header name used to transmit the token (e.g. `X-CSRF-TOKEN`).
    /// 用于传输令牌的HTTP头部名称（如 `X-CSRF-TOKEN`）。
    pub header_name: String,

    /// Form/query parameter name used to transmit the token (e.g. `_csrf`).
    /// 用于传输令牌的表单/查询参数名称（如 `_csrf`）。
    pub parameter_name: String,
}

impl CsrfToken {
    /// Create a new CSRF token with the given values.
    /// 使用给定值创建新的CSRF令牌。
    pub fn new(
        token: impl Into<String>,
        header_name: impl Into<String>,
        parameter_name: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            header_name: header_name.into(),
            parameter_name: parameter_name.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// CsrfTokenRepository
// ---------------------------------------------------------------------------

/// Trait for CSRF token persistence.
/// CSRF令牌持久化trait。
///
/// Implementations decide how tokens are stored and retrieved
/// (in-memory, cookie-based, session-backed, etc.).
/// 实现者决定令牌如何存储和检索（内存、基于cookie、会话等）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface CsrfTokenRepository {
///     CsrfToken generateToken(HttpServletRequest request);
///     void saveToken(CsrfToken token, HttpServletRequest request, HttpServletResponse response);
///     CsrfToken loadToken(HttpServletRequest request);
/// }
/// ```
#[async_trait::async_trait]
pub trait CsrfTokenRepository: Send + Sync {
    /// Generate a new random CSRF token for the given identifier (e.g. session id).
    /// 为给定标识符（如会话ID）生成新的随机CSRF令牌。
    async fn generate_token(&self, identifier: &str) -> CsrfToken;

    /// Persist a token for the given identifier.
    /// 为给定标识符持久化令牌。
    async fn save_token(&self, identifier: &str, token: &CsrfToken);

    /// Load the stored token for the given identifier, if any.
    /// 加载给定标识符的存储令牌（如果有）。
    async fn load_token(&self, identifier: &str) -> Option<CsrfToken>;

    /// Remove the stored token for the given identifier.
    /// 移除给定标识符的存储令牌。
    async fn remove_token(&self, identifier: &str);
}

// ---------------------------------------------------------------------------
// InMemoryCsrfTokenRepository
// ---------------------------------------------------------------------------

/// In-memory implementation of [`CsrfTokenRepository`].
/// [`CsrfTokenRepository`] 的内存实现。
///
/// Tokens are stored in a `HashMap` protected by an async `RwLock`.
/// Suitable for single-node deployments and testing.
/// 令牌存储在由异步 `RwLock` 保护的 `HashMap` 中。
/// 适用于单节点部署和测试。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Not a direct Spring equivalent -- Spring provides
/// // HttpSessionCsrfTokenRepository and CookieCsrfTokenRepository.
/// // Spring没有直接等价物——Spring提供
/// // HttpSessionCsrfTokenRepository 和 CookieCsrfTokenRepository。
/// ```
#[derive(Debug, Clone)]
pub struct InMemoryCsrfTokenRepository {
    /// Header name used for generated tokens.
    /// 生成的令牌使用的头部名称。
    header_name: String,

    /// Parameter name used for generated tokens.
    /// 生成的令牌使用的参数名称。
    parameter_name: String,

    /// Token length in bytes (hex-encoded, so actual string length is double).
    /// 令牌长度（字节，十六进制编码后实际字符串长度为其两倍）。
    token_length: usize,

    /// Internal storage: identifier -> CsrfToken.
    /// 内部存储：标识符 -> CsrfToken。
    store: std::sync::Arc<RwLock<HashMap<String, CsrfToken>>>,
}

impl InMemoryCsrfTokenRepository {
    /// Create a new repository with default settings.
    /// 使用默认设置创建新的存储库。
    ///
    /// Defaults:
    /// - header_name: `"X-CSRF-TOKEN"`
    /// - parameter_name: `"_csrf"`
    /// - token_length: `32` bytes (64 hex chars)
    pub fn new() -> Self {
        Self {
            header_name: "X-CSRF-TOKEN".to_string(),
            parameter_name: "_csrf".to_string(),
            token_length: 32,
            store: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the header name used for generated tokens.
    /// 设置生成的令牌使用的头部名称。
    pub fn header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    /// Set the parameter name used for generated tokens.
    /// 设置生成的令牌使用的参数名称。
    pub fn parameter_name(mut self, name: impl Into<String>) -> Self {
        self.parameter_name = name.into();
        self
    }

    /// Set the random token length in bytes.
    /// 设置随机令牌长度（字节）。
    pub fn token_length(mut self, length: usize) -> Self {
        self.token_length = length;
        self
    }

    /// Generate a random hex token of the configured length.
    /// 生成配置长度的随机十六进制令牌。
    fn random_token(&self) -> String {
        let mut buf = vec![0u8; self.token_length];
        rand::rng().fill_bytes(&mut buf);
        hex::encode(&buf)
    }
}

impl Default for InMemoryCsrfTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CsrfTokenRepository for InMemoryCsrfTokenRepository {
    async fn generate_token(&self, identifier: &str) -> CsrfToken {
        let token_value = self.random_token();
        let csrf_token =
            CsrfToken::new(token_value, self.header_name.clone(), self.parameter_name.clone());
        self.save_token(identifier, &csrf_token).await;
        csrf_token
    }

    async fn save_token(&self, identifier: &str, token: &CsrfToken) {
        let mut store = self.store.write().await;
        store.insert(identifier.to_string(), token.clone());
    }

    async fn load_token(&self, identifier: &str) -> Option<CsrfToken> {
        let store = self.store.read().await;
        store.get(identifier).cloned()
    }

    async fn remove_token(&self, identifier: &str) {
        let mut store = self.store.write().await;
        store.remove(identifier);
    }
}

// ---------------------------------------------------------------------------
// CookieCsrfTokenRepository
// ---------------------------------------------------------------------------

/// Cookie-based CSRF token repository.
/// 基于Cookie的CSRF令牌存储库。
///
/// Tokens are stored in cookies. The repository itself does not directly
/// interact with HTTP cookies; instead it stores token data in memory
/// keyed by a cookie-derived identifier, ready for middleware integration.
/// 令牌存储在cookie中。存储库本身不直接与HTTP cookie交互；
/// 而是将令牌数据存储在内存中，以cookie派生的标识符为键，
/// 便于中间件集成。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// CookieCsrfTokenRepository.withHttpOnlyFalse();
/// ```
#[derive(Debug, Clone)]
pub struct CookieCsrfTokenRepository {
    /// Cookie name used to read/write the CSRF token.
    /// 用于读取/写入CSRF令牌的cookie名称。
    pub cookie_name: String,

    /// Header name expected on state-changing requests.
    /// 状态变更请求中期望的头部名称。
    pub header_name: String,

    /// Parameter name as fallback.
    /// 备用参数名称。
    pub parameter_name: String,

    /// Whether the cookie is HttpOnly.
    /// cookie是否为HttpOnly。
    pub cookie_http_only: bool,

    /// Whether the cookie requires Secure (HTTPS only).
    /// cookie是否要求Secure（仅HTTPS）。
    pub cookie_secure: bool,

    /// Cookie path.
    /// Cookie路径。
    pub cookie_path: String,

    /// Max age of the cookie in seconds. `None` means session cookie.
    /// cookie的最大存活时间（秒）。`None` 表示会话cookie。
    pub cookie_max_age: Option<u64>,

    /// SameSite attribute for the cookie.
    /// cookie的SameSite属性。
    pub cookie_same_site: String,

    /// Internal storage: cookie_value -> CsrfToken.
    /// 内部存储：cookie_value -> CsrfToken。
    store: std::sync::Arc<RwLock<HashMap<String, CsrfToken>>>,
}

impl CookieCsrfTokenRepository {
    /// Create a new cookie-based repository with default settings.
    /// 使用默认设置创建新的基于cookie的存储库。
    ///
    /// Defaults:
    /// - cookie_name: `"XSRF-TOKEN"`
    /// - header_name: `"X-XSRF-TOKEN"`
    /// - parameter_name: `"_csrf"`
    /// - cookie_http_only: `false` (JavaScript must be able to read it)
    /// - cookie_secure: `false`
    /// - cookie_path: `"/"`
    pub fn new() -> Self {
        Self {
            cookie_name: "XSRF-TOKEN".to_string(),
            header_name: "X-XSRF-TOKEN".to_string(),
            parameter_name: "_csrf".to_string(),
            cookie_http_only: false,
            cookie_secure: false,
            cookie_path: "/".to_string(),
            cookie_max_age: None,
            cookie_same_site: "Lax".to_string(),
            store: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a repository with `http_only` set to `false` so that
    /// front-end JavaScript (e.g. Angular) can read the cookie.
    /// 创建一个 `http_only` 设置为 `false` 的存储库，
    /// 以便前端JavaScript（如Angular）可以读取cookie。
    ///
    /// Equivalent to Spring's `CookieCsrfTokenRepository.withHttpOnlyFalse()`.
    /// 等价于Spring的 `CookieCsrfTokenRepository.withHttpOnlyFalse()`。
    pub fn with_http_only_false() -> Self {
        Self::new()
    }

    /// Set the cookie name.
    /// 设置cookie名称。
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set the header name.
    /// 设置头部名称。
    pub fn header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    /// Set the parameter name.
    /// 设置参数名称。
    pub fn parameter_name(mut self, name: impl Into<String>) -> Self {
        self.parameter_name = name.into();
        self
    }

    /// Set whether the cookie is HttpOnly.
    /// 设置cookie是否为HttpOnly。
    pub fn cookie_http_only(mut self, http_only: bool) -> Self {
        self.cookie_http_only = http_only;
        self
    }

    /// Set whether the cookie requires Secure.
    /// 设置cookie是否要求Secure。
    pub fn cookie_secure(mut self, secure: bool) -> Self {
        self.cookie_secure = secure;
        self
    }

    /// Set the cookie path.
    /// 设置cookie路径。
    pub fn cookie_path(mut self, path: impl Into<String>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set the cookie max age in seconds. `None` means session cookie.
    /// 设置cookie最大存活时间（秒）。`None` 表示会话cookie。
    pub fn cookie_max_age(mut self, max_age: Option<u64>) -> Self {
        self.cookie_max_age = max_age;
        self
    }

    /// Set the SameSite attribute.
    /// 设置SameSite属性。
    pub fn cookie_same_site(mut self, same_site: impl Into<String>) -> Self {
        self.cookie_same_site = same_site.into();
        self
    }

    /// Generate a random hex token (32 bytes / 64 hex chars).
    /// 生成随机十六进制令牌（32字节/64个十六进制字符）。
    fn random_token() -> String {
        let mut buf = vec![0u8; 32];
        rand::rng().fill_bytes(&mut buf);
        hex::encode(&buf)
    }
}

impl Default for CookieCsrfTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CsrfTokenRepository for CookieCsrfTokenRepository {
    async fn generate_token(&self, identifier: &str) -> CsrfToken {
        let token_value = Self::random_token();
        let csrf_token =
            CsrfToken::new(token_value, self.header_name.clone(), self.parameter_name.clone());
        self.save_token(identifier, &csrf_token).await;
        csrf_token
    }

    async fn save_token(&self, identifier: &str, token: &CsrfToken) {
        let mut store = self.store.write().await;
        store.insert(identifier.to_string(), token.clone());
    }

    async fn load_token(&self, identifier: &str) -> Option<CsrfToken> {
        let store = self.store.read().await;
        store.get(identifier).cloned()
    }

    async fn remove_token(&self, identifier: &str) {
        let mut store = self.store.write().await;
        store.remove(identifier);
    }
}

// ---------------------------------------------------------------------------
// CsrfProtectionConfig
// ---------------------------------------------------------------------------

/// Configuration for CSRF protection behaviour.
/// CSRF保护行为配置。
///
/// Controls which HTTP methods are ignored, the token header/parameter
/// names, and whether CSRF protection is enabled at all.
/// 控制哪些HTTP方法被忽略、令牌头部/参数名称，以及是否完全启用CSRF保护。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// http.csrf(csrf -> csrf
///     .csrfTokenRepository(repo)
///     .ignoringRequestMatchers("/api/public/**")
/// );
/// ```
#[derive(Debug, Clone)]
pub struct CsrfProtectionConfig {
    /// Whether CSRF protection is enabled.
    /// 是否启用CSRF保护。
    pub enabled: bool,

    /// HTTP methods that are safe and do not require CSRF validation.
    /// 安全的、不需要CSRF验证的HTTP方法。
    ///
    /// Defaults: GET, HEAD, OPTIONS, TRACE.
    pub ignored_methods: Vec<http::Method>,

    /// Header name expected to carry the CSRF token on state-changing requests.
    /// 状态变更请求中期望携带CSRF令牌的头部名称。
    pub token_header_name: String,

    /// Query/form parameter name as a fallback for the CSRF token.
    /// CSRF令牌的备用查询/表单参数名称。
    pub token_param_name: String,
}

impl Default for CsrfProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ignored_methods: vec![
                http::Method::GET,
                http::Method::HEAD,
                http::Method::OPTIONS,
                http::Method::TRACE,
            ],
            token_header_name: "X-CSRF-TOKEN".to_string(),
            token_param_name: "_csrf".to_string(),
        }
    }
}

impl CsrfProtectionConfig {
    /// Create a new configuration with CSRF protection disabled.
    /// 创建一个禁用CSRF保护的新配置。
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    /// Set whether CSRF protection is enabled.
    /// 设置是否启用CSRF保护。
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add an HTTP method to the ignore list.
    /// 添加HTTP方法到忽略列表。
    pub fn ignore_method(mut self, method: http::Method) -> Self {
        if !self.ignored_methods.contains(&method) {
            self.ignored_methods.push(method);
        }
        self
    }

    /// Set the token header name.
    /// 设置令牌头部名称。
    pub fn token_header_name(mut self, name: impl Into<String>) -> Self {
        self.token_header_name = name.into();
        self
    }

    /// Set the token parameter name.
    /// 设置令牌参数名称。
    pub fn token_param_name(mut self, name: impl Into<String>) -> Self {
        self.token_param_name = name.into();
        self
    }

    /// Check if the given HTTP method should be ignored (safe / read-only).
    /// 检查给定的HTTP方法是否应被忽略（安全/只读）。
    pub fn is_method_ignored(&self, method: &http::Method) -> bool {
        self.ignored_methods.contains(method)
    }
}

// ---------------------------------------------------------------------------
// CsrfValidator
// ---------------------------------------------------------------------------

/// CSRF token validator.
/// CSRF令牌验证器。
///
/// Wraps a [`CsrfTokenRepository`] and a [`CsrfProtectionConfig`] to
/// provide token generation and validation for request processing.
/// 封装 [`CsrfTokenRepository`] 和 [`CsrfProtectionConfig`]，
/// 为请求处理提供令牌生成和验证。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_security::csrf::{CsrfValidator, CsrfProtectionConfig, InMemoryCsrfTokenRepository};
/// use http::Method;
///
/// let config = CsrfProtectionConfig::default();
/// let repo = InMemoryCsrfTokenRepository::new();
/// let validator = CsrfValidator::new(config, repo);
///
/// // In a request handler:
/// // 1. Generate a token and return it to the client
/// let token = validator.generate_token("session-id").await.unwrap();
///
/// // 2. On subsequent state-changing requests, validate the submitted token
/// let result = validator.validate_token("session-id", "submitted-token").await;
/// ```
pub struct CsrfValidator<R: CsrfTokenRepository> {
    /// CSRF protection configuration.
    /// CSRF保护配置。
    config: CsrfProtectionConfig,

    /// Token repository.
    /// 令牌存储库。
    repository: R,
}

impl<R: CsrfTokenRepository> CsrfValidator<R> {
    /// Create a new CSRF validator.
    /// 创建新的CSRF验证器。
    pub fn new(config: CsrfProtectionConfig, repository: R) -> Self {
        Self { config, repository }
    }

    /// Generate a new CSRF token for the given identifier.
    /// 为给定标识符生成新的CSRF令牌。
    pub async fn generate_token(&self, identifier: &str) -> crate::SecurityResult<CsrfToken> {
        if !self.config.enabled {
            return Err(SecurityError::CsrfValidationFailed(
                "CSRF protection is disabled".to_string(),
            ));
        }
        Ok(self.repository.generate_token(identifier).await)
    }

    /// Validate a submitted token against the stored token for the identifier.
    /// 将提交的令牌与标识符的存储令牌进行验证。
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    /// 使用恒定时间比较以防止时序攻击。
    pub async fn validate_token(
        &self,
        identifier: &str,
        submitted_token: &str,
    ) -> crate::SecurityResult<()> {
        if !self.config.enabled {
            // CSRF protection disabled -- always pass.
            // CSRF保护已禁用——始终通过。
            return Ok(());
        }

        let stored = self
            .repository
            .load_token(identifier)
            .await
            .ok_or_else(|| {
                SecurityError::CsrfValidationFailed("No CSRF token found for session".to_string())
            })?;

        // Constant-time comparison to prevent timing attacks.
        // 恒定时间比较以防止时序攻击。
        if subtle::ConstantTimeEq::ct_eq(stored.token.as_bytes(), submitted_token.as_bytes()).into()
        {
            Ok(())
        } else {
            Err(SecurityError::CsrfValidationFailed("CSRF token mismatch".to_string()))
        }
    }

    /// Convenience: check whether the given HTTP method requires validation.
    /// 便捷方法：检查给定的HTTP方法是否需要验证。
    ///
    /// Returns `true` when validation is **not** required (safe methods).
    /// 当**不需要**验证时返回 `true`（安全方法）。
    pub fn is_method_ignored(&self, method: &http::Method) -> bool {
        self.config.is_method_ignored(method)
    }

    /// Full validation pipeline: check method, then validate token.
    /// 完整验证流程：检查方法，然后验证令牌。
    ///
    /// Returns `Ok(())` when the method is in the ignore list
    /// **or** the token is valid.
    /// 当方法在忽略列表中**或**令牌有效时返回 `Ok(())`。
    pub async fn validate(
        &self,
        method: &http::Method,
        identifier: &str,
        submitted_token: &str,
    ) -> crate::SecurityResult<()> {
        if self.is_method_ignored(method) {
            return Ok(());
        }
        self.validate_token(identifier, submitted_token).await
    }

    /// Remove the stored token for the given identifier (e.g. on logout).
    /// 移除给定标识符的存储令牌（如注销时）。
    pub async fn remove_token(&self, identifier: &str) {
        self.repository.remove_token(identifier).await;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    // -- CsrfToken -----------------------------------------------------------

    #[test]
    fn test_csrf_token_new() {
        let token = CsrfToken::new("abc123", "X-CSRF-TOKEN", "_csrf");
        assert_eq!(token.token, "abc123");
        assert_eq!(token.header_name, "X-CSRF-TOKEN");
        assert_eq!(token.parameter_name, "_csrf");
    }

    // -- CsrfProtectionConfig ------------------------------------------------

    #[test]
    fn test_default_config() {
        let config = CsrfProtectionConfig::default();
        assert!(config.enabled);
        assert!(config.is_method_ignored(&http::Method::GET));
        assert!(config.is_method_ignored(&http::Method::HEAD));
        assert!(config.is_method_ignored(&http::Method::OPTIONS));
        assert!(config.is_method_ignored(&http::Method::TRACE));
        assert!(!config.is_method_ignored(&http::Method::POST));
        assert!(!config.is_method_ignored(&http::Method::PUT));
        assert!(!config.is_method_ignored(&http::Method::DELETE));
    }

    #[test]
    fn test_disabled_config() {
        let config = CsrfProtectionConfig::disabled();
        assert!(!config.enabled);
        // Ignored methods should still be populated.
        assert!(config.is_method_ignored(&http::Method::GET));
    }

    #[test]
    fn test_config_builder() {
        let config = CsrfProtectionConfig::default()
            .enabled(false)
            .token_header_name("X-MY-CSRF")
            .token_param_name("csrf_field");

        assert!(!config.enabled);
        assert_eq!(config.token_header_name, "X-MY-CSRF");
        assert_eq!(config.token_param_name, "csrf_field");
    }

    #[test]
    fn test_config_ignore_method() {
        let config = CsrfProtectionConfig::default().ignore_method(http::Method::POST);
        assert!(config.is_method_ignored(&http::Method::POST));
    }

    #[test]
    fn test_config_ignore_method_no_duplicates() {
        let config = CsrfProtectionConfig::default()
            .ignore_method(http::Method::GET)
            .ignore_method(http::Method::GET);
        assert_eq!(
            config
                .ignored_methods
                .iter()
                .filter(|m| **m == http::Method::GET)
                .count(),
            1
        );
    }

    // -- InMemoryCsrfTokenRepository ------------------------------------------

    #[tokio::test]
    async fn test_in_memory_generate_and_load() {
        let repo = InMemoryCsrfTokenRepository::new();
        let token = repo.generate_token("session-1").await;

        assert!(!token.token.is_empty());
        assert_eq!(token.header_name, "X-CSRF-TOKEN");
        assert_eq!(token.parameter_name, "_csrf");

        let loaded = repo.load_token("session-1").await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().token, token.token);
    }

    #[tokio::test]
    async fn test_in_memory_save_and_load() {
        let repo = InMemoryCsrfTokenRepository::new();
        let token = CsrfToken::new("my-token", "X-CSRF", "csrf");

        repo.save_token("session-2", &token).await;

        let loaded = repo.load_token("session-2").await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().token, "my-token");
    }

    #[tokio::test]
    async fn test_in_memory_load_missing() {
        let repo = InMemoryCsrfTokenRepository::new();
        let loaded = repo.load_token("nonexistent").await;
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_remove() {
        let repo = InMemoryCsrfTokenRepository::new();
        repo.generate_token("session-3").await;

        repo.remove_token("session-3").await;
        let loaded = repo.load_token("session-3").await;
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_custom_settings() {
        let repo = InMemoryCsrfTokenRepository::new()
            .header_name("X-MY-CSRF")
            .parameter_name("my_csrf")
            .token_length(16);

        let token = repo.generate_token("s").await;
        // 16 bytes => 32 hex chars
        assert_eq!(token.token.len(), 32);
        assert_eq!(token.header_name, "X-MY-CSRF");
        assert_eq!(token.parameter_name, "my_csrf");
    }

    // -- CookieCsrfTokenRepository --------------------------------------------

    #[test]
    fn test_cookie_repo_default_settings() {
        let repo = CookieCsrfTokenRepository::new();
        assert_eq!(repo.cookie_name, "XSRF-TOKEN");
        assert_eq!(repo.header_name, "X-XSRF-TOKEN");
        assert_eq!(repo.parameter_name, "_csrf");
        assert!(!repo.cookie_http_only);
        assert!(!repo.cookie_secure);
        assert_eq!(repo.cookie_path, "/");
        assert_eq!(repo.cookie_same_site, "Lax");
        assert!(repo.cookie_max_age.is_none());
    }

    #[test]
    fn test_cookie_repo_with_http_only_false() {
        let repo = CookieCsrfTokenRepository::with_http_only_false();
        assert!(!repo.cookie_http_only);
    }

    #[test]
    fn test_cookie_repo_builder() {
        let repo = CookieCsrfTokenRepository::new()
            .cookie_name("MY-CSRF-COOKIE")
            .header_name("X-MY-CSRF")
            .parameter_name("my_csrf")
            .cookie_http_only(true)
            .cookie_secure(true)
            .cookie_path("/api")
            .cookie_max_age(Some(3600))
            .cookie_same_site("Strict");

        assert_eq!(repo.cookie_name, "MY-CSRF-COOKIE");
        assert_eq!(repo.header_name, "X-MY-CSRF");
        assert!(repo.cookie_http_only);
        assert!(repo.cookie_secure);
        assert_eq!(repo.cookie_path, "/api");
        assert_eq!(repo.cookie_max_age, Some(3600));
        assert_eq!(repo.cookie_same_site, "Strict");
    }

    #[tokio::test]
    async fn test_cookie_repo_generate_and_load() {
        let repo = CookieCsrfTokenRepository::new();
        let token = repo.generate_token("cookie-session-1").await;

        assert!(!token.token.is_empty());

        let loaded = repo.load_token("cookie-session-1").await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().token, token.token);
    }

    // -- CsrfValidator -------------------------------------------------------

    #[tokio::test]
    async fn test_validator_generate_and_validate() {
        let config = CsrfProtectionConfig::default();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        let token = validator.generate_token("sess-1").await.unwrap();

        // Valid token should pass.
        let result = validator.validate_token("sess-1", &token.token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validator_invalid_token() {
        let config = CsrfProtectionConfig::default();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        validator.generate_token("sess-2").await.unwrap();

        let result = validator.validate_token("sess-2", "wrong-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validator_missing_session() {
        let config = CsrfProtectionConfig::default();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        let result = validator
            .validate_token("no-such-session", "any-token")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validator_disabled() {
        let config = CsrfProtectionConfig::disabled();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        // Should always pass when disabled.
        let result = validator.validate_token("any", "any").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validator_generate_when_disabled() {
        let config = CsrfProtectionConfig::disabled();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        let result = validator.generate_token("sess").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validator_validate_safe_methods() {
        let config = CsrfProtectionConfig::default();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        // Safe methods should pass without a token.
        for method in &[
            http::Method::GET,
            http::Method::HEAD,
            http::Method::OPTIONS,
            http::Method::TRACE,
        ] {
            let result = validator.validate(method, "no-token-needed", "").await;
            assert!(result.is_ok(), "Method {} should be ignored", method);
        }
    }

    #[tokio::test]
    async fn test_validator_validate_unsafe_methods() {
        let config = CsrfProtectionConfig::default();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        let token = validator.generate_token("sess-unsafe").await.unwrap();

        // POST with valid token should pass.
        let result = validator
            .validate(&http::Method::POST, "sess-unsafe", &token.token)
            .await;
        assert!(result.is_ok());

        // PUT with wrong token should fail.
        let result = validator
            .validate(&http::Method::PUT, "sess-unsafe", "bad")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validator_remove_token() {
        let config = CsrfProtectionConfig::default();
        let repo = InMemoryCsrfTokenRepository::new();
        let validator = CsrfValidator::new(config, repo);

        validator.generate_token("sess-rm").await.unwrap();
        validator.remove_token("sess-rm").await;

        let result = validator.validate_token("sess-rm", "any").await;
        assert!(result.is_err());
    }
}
