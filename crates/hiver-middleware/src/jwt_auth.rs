//! JWT Authentication Middleware
//! JWT 认证中间件
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `JwtAuthenticationFilter` - JWT authentication filter
//! - `OncePerRequestFilter` - Execute once per request
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_middleware::JwtAuthenticationMiddleware;
//! use hiver_router::Router;
//! use std::sync::Arc;
//!
//! let jwt_middleware = Arc::new(JwtAuthenticationMiddleware::new());
//!
//! let app = Router::new()
//!     .middleware(jwt_middleware)
//!     .get("/api/users", get_users);
//! ```

use std::{future::Future, pin::Pin, sync::Arc};

use hiver_router::{Middleware, Next};
use hiver_security::{JwtAuthentication, JwtClaims, JwtUtil, SecurityError};

use crate::{Error, Request, Response, Result};

/// JWT authentication middleware
/// JWT 认证中间件
///
/// Extracts and validates JWT tokens from the Authorization header.
/// `从Authorization头中提取并验证JWT` token。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class JwtAuthenticationTokenFilter extends OncePerRequestFilter {
///     @Override
///     protected void doFilterInternal(HttpServletRequest request,
///                                     HttpServletResponse response,
///                                     FilterChain chain) {
///         String jwt = resolveToken(request);
///         if (jwt != null && jwtProvider.validateToken(jwt)) {
///             Authentication auth = jwtProvider.getAuthentication(jwt);
///             SecurityContextHolder.getContext().setAuthentication(auth);
///         }
///         chain.doFilter(request, response);
///     }
/// }
/// ```
///
/// # Header Format / 头格式
///
/// The middleware expects the JWT token in the Authorization header:
/// `中间件期望在Authorization头中有JWT` token：
///
/// ```text
/// Authorization: Bearer <token>
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// let middleware = JwtAuthenticationMiddleware::new();
/// ```
#[derive(Clone)]
pub struct JwtAuthenticationMiddleware
{
    /// Token header name (default: "authorization")
    /// Token头名称（默认："authorization"）
    token_header: String,

    /// Token prefix (default: "Bearer ")
    /// Token前缀（默认："Bearer "）
    token_prefix: String,

    /// Skip authentication for these paths
    /// 跳过这些路径的认证
    skip_paths: Vec<String>,
}

impl JwtAuthenticationMiddleware
{
    /// Create a new JWT authentication middleware
    /// 创建新的JWT认证中间件
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let middleware = JwtAuthenticationMiddleware::new();
    /// ```
    pub fn new() -> Self
    {
        Self {
            token_header: "authorization".to_string(),
            token_prefix: "Bearer ".to_string(),
            skip_paths: vec![
                "/api/auth/login".to_string(),
                "/api/auth/register".to_string(),
                "/health".to_string(),
            ],
        }
    }

    /// Set custom token header name
    /// 设置自定义token头名称
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let middleware = JwtAuthenticationMiddleware::new()
    ///     .with_token_header("x-auth-token");
    /// ```
    pub fn with_token_header(mut self, header: impl Into<String>) -> Self
    {
        self.token_header = header.into().to_lowercase();
        self
    }

    /// Set custom token prefix
    /// 设置自定义token前缀
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let middleware = JwtAuthenticationMiddleware::new()
    ///     .with_token_prefix("Token ");
    /// ```
    pub fn with_token_prefix(mut self, prefix: impl Into<String>) -> Self
    {
        self.token_prefix = prefix.into();
        self
    }

    /// Add a path to skip authentication
    /// 添加跳过认证的路径
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let middleware = JwtAuthenticationMiddleware::new()
    ///     .skip_path("/api/public")
    ///     .skip_path("/api/docs");
    /// ```
    pub fn skip_path(mut self, path: impl Into<String>) -> Self
    {
        self.skip_paths.push(path.into());
        self
    }

    /// Set paths to skip authentication
    /// 设置跳过认证的路径
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let middleware = JwtAuthenticationMiddleware::new()
    ///     .with_skip_paths(&["/api/auth/login", "/api/auth/register"]);
    /// ```
    pub fn with_skip_paths(mut self, paths: &[&str]) -> Self
    {
        self.skip_paths = paths.iter().map(ToString::to_string).collect();
        self
    }

    /// Extract JWT token from request headers
    /// 从请求头中提取JWT token
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// private String resolveToken(HttpServletRequest request) {
    ///     String bearerToken = request.getHeader("Authorization");
    ///     if (bearerToken != null && bearerToken.startsWith("Bearer ")) {
    ///         return bearerToken.substring(7);
    ///     }
    ///     return null;
    /// }
    /// ```
    fn extract_token<'a>(&'a self, headers: &'a http::HeaderMap) -> Option<&'a str>
    {
        headers
            .get(&self.token_header)
            .and_then(|value: &http::header::HeaderValue| value.to_str().ok())
            .and_then(|auth_header: &str| {
                if auth_header.starts_with(&self.token_prefix)
                {
                    Some(&auth_header[self.token_prefix.len()..])
                }
                else
                {
                    None
                }
            })
    }

    /// Check if request path should skip authentication
    /// 检查请求路径是否应该跳过认证
    fn should_skip_auth(&self, path: &str) -> bool
    {
        self.skip_paths
            .iter()
            .any(|skip_path| path == skip_path || path.starts_with(&format!("{}/", skip_path)))
    }
}

impl Default for JwtAuthenticationMiddleware
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl<S> Middleware<S> for JwtAuthenticationMiddleware
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>
    {
        let token_header = self.token_header.clone();
        let token_prefix = self.token_prefix.clone();
        let skip_paths = self.skip_paths.clone();

        Box::pin(async move {
            let path = req.path();

            // Skip authentication for certain paths
            if skip_paths
                .iter()
                .any(|p| path == p || path.starts_with(&format!("{}/", p)))
            {
                tracing::debug!("Skipping authentication for path: {}", path);
                return next.call(req, state).await;
            }

            // Extract JWT token from headers
            let token = match req
                .headers()
                .get(&token_header)
                .and_then(|v| v.to_str().ok())
            {
                Some(auth_header) if auth_header.starts_with(&token_prefix) =>
                {
                    Some(auth_header[token_prefix.len()..].to_string())
                },
                Some(_) => None,
                None => None,
            };

            let token = if let Some(t) = token
            {
                t
            }
            else
            {
                tracing::warn!("Missing JWT token for path: {}", path);
                return Err(Error::unauthorized());
            };

            // Verify and parse JWT token
            let _claims: JwtClaims = match JwtUtil::verify_token(&token)
            {
                Ok(claims) =>
                {
                    tracing::debug!("JWT verified for user: {}", claims.username);
                    claims
                },
                Err(SecurityError::TokenExpired(msg)) =>
                {
                    tracing::warn!("JWT token expired: {}", msg);
                    return Err(Error::unauthorized());
                },
                Err(SecurityError::InvalidToken(msg)) =>
                {
                    tracing::warn!("Invalid JWT token: {}", msg);
                    return Err(Error::unauthorized());
                },
                Err(e) =>
                {
                    tracing::error!("JWT verification error: {:?}", e);
                    return Err(Error::internal("Authentication error"));
                },
            };

            // Note: JWT claims extraction successful
            // In a full implementation, authentication would be stored in request extensions
            let _ = _claims;

            // Continue with the request
            next.call(req, state).await
        })
    }
}

/// Extension trait to get JWT authentication from request
/// `从请求获取JWT认证的扩展trait`
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// UsernamePasswordAuthenticationToken authentication =
///     (UsernamePasswordAuthenticationToken) SecurityContextHolder
///         .getContext()
///         .getAuthentication();
/// UserDetailsImpl loginUser = (UserDetailsImpl) authentication.getPrincipal();
/// ```
pub trait JwtRequestExt
{
    /// Get JWT authentication from request
    /// 从请求获取JWT认证
    ///
    /// # Returns / 返回
    ///
    /// `Some(JwtAuthentication)` if authenticated, `None` otherwise
    /// 如果已认证则返回`Some(JwtAuthentication)`，否则返回`None`
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// use hiver_middleware::JwtRequestExt;
    ///
    /// async fn get_current_user(req: &Request) -> Result<User> {
    ///     let auth = req.get_jwt_auth()
    ///         .ok_or(Error::Unauthorized)?;
    ///
    ///     let user = load_user(&auth.user_id).await?;
    ///     Ok(user)
    /// }
    /// ```
    fn get_jwt_auth(&self) -> Option<&JwtAuthentication>;

    /// Get current user ID
    /// 获取当前用户ID
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let user_id = req.get_current_user_id()
    ///     .ok_or(Error::Unauthorized)?;
    /// ```
    fn get_current_user_id(&self) -> Option<&str>;

    /// Get current username
    /// 获取当前用户名
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let username = req.get_current_username()
    ///     .ok_or(Error::Unauthorized)?;
    /// ```
    fn get_current_username(&self) -> Option<&str>;
}

impl JwtRequestExt for Request
{
    fn get_jwt_auth(&self) -> Option<&JwtAuthentication>
    {
        self.extensions().get::<JwtAuthentication>()
    }

    fn get_current_user_id(&self) -> Option<&str>
    {
        self.get_jwt_auth().map(|auth| auth.user_id.as_str())
    }

    fn get_current_username(&self) -> Option<&str>
    {
        self.get_jwt_auth().map(|auth| auth.username.as_str())
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_extract_token()
    {
        let middleware = JwtAuthenticationMiddleware::new();

        // Test valid token
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test"
                .parse()
                .unwrap(),
        );

        let token = middleware.extract_token(&headers);
        assert!(token.is_some());
        assert_eq!(token.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test");

        // Test missing token
        let headers = http::HeaderMap::new();
        let token = middleware.extract_token(&headers);
        assert!(token.is_none());

        // Test invalid token format
        let mut headers = http::HeaderMap::new();
        headers.insert("authorization", "InvalidToken".parse().unwrap());
        let token = middleware.extract_token(&headers);
        assert!(token.is_none());
    }

    #[test]
    fn test_should_skip_auth()
    {
        let middleware = JwtAuthenticationMiddleware::new();

        assert!(middleware.should_skip_auth("/api/auth/login"));
        assert!(middleware.should_skip_auth("/api/auth/register"));
        assert!(middleware.should_skip_auth("/health"));

        assert!(!middleware.should_skip_auth("/api/users"));
        assert!(!middleware.should_skip_auth("/api/posts"));
    }

    #[test]
    fn test_custom_token_header()
    {
        let middleware = JwtAuthenticationMiddleware::new()
            .with_token_header("X-Auth-Token")
            .with_token_prefix("");

        let mut headers = http::HeaderMap::new();
        headers.insert("x-auth-token", "my-token".parse().unwrap());

        let token = middleware.extract_token(&headers);
        assert_eq!(token.unwrap(), "my-token");
    }

    #[test]
    fn test_with_skip_paths()
    {
        let middleware = JwtAuthenticationMiddleware::new()
            .skip_path("/api/public")
            .skip_path("/api/docs");

        assert!(middleware.should_skip_auth("/api/public"));
        assert!(middleware.should_skip_auth("/api/public/data"));
        assert!(middleware.should_skip_auth("/api/docs"));
        assert!(!middleware.should_skip_auth("/api/users"));
    }
}
