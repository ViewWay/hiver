//! JWT (JSON Web Token) authentication module
//! JWT (JSON Web Token) 认证模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `JwtUtil` - JWT utility class
//! - `JwtAuthenticationFilter` - JWT authentication filter
//! - `JwtTokenProvider` - JWT token provider
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_security::jwt::{JwtUtil, JwtClaims};
//! use nexus_security::User;
//!
//! // Create JWT token for user
//! let user = User::with_roles("alice", "password", &[Role::User]);
//! let token = JwtUtil::create_token(user.id, &user.username, &user.authorities)?;
//!
//! // Verify JWT token
//! let claims = JwtUtil::verify_token(&token)?;
//! println!("User ID: {}", claims.sub);
//! ```

use crate::{Authority, Role, SecurityError, SecurityResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

/// JWT claims
/// JWT 声明
///
/// Contains the standard JWT claims plus custom fields.
/// 包含标准JWT声明加自定义字段。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class JwtClaims {
///     private String sub;      // Subject (user ID)
///     private String username; // Username
///     private List<String> authorities; // Roles/permissions
///     private Date iat;       // Issued at
///     private Date exp;       // Expiration
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    /// 主体（用户ID）
    pub sub: String,

    /// Username
    /// 用户名
    pub username: String,

    /// Authorities/roles
    /// 权限/角色
    pub authorities: Vec<String>,

    /// Issued at
    /// 签发时间
    pub iat: i64,

    /// Expiration
    /// 过期时间
    pub exp: i64,

    /// Issuer
    /// 签发者
    pub iss: Option<String>,
}

impl JwtClaims {
    /// Create new JWT claims
    /// 创建新的JWT声明
    pub fn new(
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
        expiration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expiration_hours);

        Self {
            sub: user_id.into(),
            username: username.into(),
            authorities: authorities.iter().map(std::string::ToString::to_string).collect(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            iss: Some("nexus-security".to_string()),
        }
    }

    /// Check if token is expired
    /// 检查token是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Get time until expiration
    /// 获取剩余有效时间
    pub fn time_until_expiration(&self) -> Duration {
        let now = Utc::now().timestamp();
        let seconds_left = self.exp - now;
        Duration::seconds(seconds_left)
    }

    /// Convert authorities to Authority enum
    /// 将authorities转换为Authority枚举
    pub fn get_authorities(&self) -> Vec<Authority> {
        self.authorities
            .iter()
            .filter_map(|a| Authority::from_string(a))
            .collect()
    }

    /// Check if has authority
    /// 检查是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool {
        self.get_authorities().contains(authority)
    }

    /// Check if has role
    /// 检查是否有角色
    pub fn has_role(&self, role: &Role) -> bool {
        self.get_authorities()
            .contains(&Authority::Role(role.clone()))
    }
}

/// JWT utility
/// JWT 工具类
///
/// Equivalent to Spring's `JwtUtil` class.
/// `等价于Spring的JwtUtil类`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class JwtUtil {
///     public static String createJWT(String subject) { ... }
///     public static Claims parseJWT(String jwt) { ... }
/// }
/// ```
pub struct JwtUtil;

impl JwtUtil {
    /// Get JWT secret key from environment or use default
    /// 从环境变量获取JWT密钥或使用默认值
    fn get_secret() -> String {
        env::var("JWT_SECRET")
            .unwrap_or_else(|_| "nexus-jwt-secret-key-change-in-production-2024".to_string())
    }

    /// Get default token expiration in hours
    /// 获取默认token过期时间（小时）
    fn get_default_expiration() -> i64 {
        env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24) // Default: 24 hours
    }

    /// Create JWT token for user
    /// 为用户创建JWT token
    ///
    /// # Arguments / 参数
    ///
    /// * `user_id` - User ID / 用户ID
    /// * `username` - Username / 用户名
    /// * `authorities` - User authorities / 用户权限
    ///
    /// # Returns / 返回
    ///
    /// JWT token string / JWT token字符串
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// let token = JwtUtil::create_token(
    ///     "123",
    ///     "alice",
    ///     &[Authority::Role(Role::User)]
    /// )?;
    /// ```
    pub fn create_token(
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
    ) -> SecurityResult<String> {
        let expiration_hours = Self::get_default_expiration();
        Self::create_token_with_expiration(user_id, username, authorities, expiration_hours)
    }

    /// Create JWT token with custom expiration
    /// 创建带自定义过期时间的JWT token
    ///
    /// # Arguments / 参数
    ///
    /// * `user_id` - User ID / 用户ID
    /// * `username` - Username / 用户名
    /// * `authorities` - User authorities / 用户权限
    /// * `expiration_hours` - Token expiration in hours / token过期时间（小时）
    pub fn create_token_with_expiration(
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
        expiration_hours: i64,
    ) -> SecurityResult<String> {
        let claims = JwtClaims::new(user_id, username, authorities, expiration_hours);

        let secret = Self::get_secret();
        let encoding_key = EncodingKey::from_secret(secret.as_ref());

        encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| SecurityError::TokenError(format!("Failed to encode token: {}", e)))
    }

    /// Verify and parse JWT token
    /// 验证并解析JWT token
    ///
    /// # Arguments / 参数
    ///
    /// * `token` - JWT token string / JWT token字符串
    ///
    /// # Returns / 返回
    ///
    /// Parsed JWT claims / 解析后的JWT声明
    ///
    /// # Errors / 错误
    ///
    /// Returns error if token is invalid or expired / 如果token无效或过期则返回错误
    pub fn verify_token(token: &str) -> SecurityResult<JwtClaims> {
        let secret = Self::get_secret();
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        decode::<JwtClaims>(token, &decoding_key, &validation)
            .map(|data| {
                let claims = data.claims;

                // Check expiration manually for better error messages
                if claims.is_expired() {
                    return Err(SecurityError::TokenExpired("Token has expired".to_string()));
                }

                Ok(claims)
            })
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    SecurityError::TokenExpired("Token signature has expired".to_string())
                },
                _ => SecurityError::InvalidToken(format!("Invalid token: {}", e)),
            })?
    }

    /// Refresh JWT token
    /// 刷新JWT token
    ///
    /// Creates a new token with the same user information but extended expiration.
    /// 创建具有相同用户信息但延长过期时间的新token。
    ///
    /// # Arguments / 参数
    ///
    /// * `token` - Old JWT token / 旧的JWT token
    pub fn refresh_token(token: &str) -> SecurityResult<String> {
        let claims = Self::verify_token(token)?;

        // Parse authorities back from strings
        let authorities: Vec<Authority> = claims
            .authorities
            .iter()
            .filter_map(|s| Authority::from_string(s))
            .collect();

        Self::create_token(&claims.sub, &claims.username, &authorities)
    }

    /// Parse token without verification (for debugging/testing only)
    /// 解析token但不验证（仅用于调试/测试）
    ///
    /// # Warning / 警告
    ///
    /// This should NOT be used in production for authentication.
    /// 这不应该在生产环境中用于身份验证。
    #[cfg(test)]
    pub fn parse_token_unsafe(token: &str) -> SecurityResult<JwtClaims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(SecurityError::InvalidToken("Invalid token format".to_string()));
        }

        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| SecurityError::InvalidToken("Failed to decode token".to_string()))?;

        let claims: JwtClaims = serde_json::from_slice(&decoded)
            .map_err(|_| SecurityError::InvalidToken("Failed to parse claims".to_string()))?;

        Ok(claims)
    }
}

/// JWT token provider
/// JWT token 提供者
///
/// Equivalent to Spring's `JwtTokenProvider`.
/// `等价于Spring的JwtTokenProvider`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class JwtTokenProvider {
///     public String generateToken(Authentication authentication) { ... }
///     public boolean validateToken(String token) { ... }
///     public Authentication getAuthentication(String token) { ... }
/// }
/// ```
#[derive(Clone)]
pub struct JwtTokenProvider {
    /// Secret key for signing tokens
    /// 签名token的密钥
    secret: String,

    /// Token expiration in hours
    /// Token过期时间（小时）
    expiration_hours: i64,
}

impl JwtTokenProvider {
    /// Create new JWT token provider
    /// 创建新的JWT token提供者
    pub fn new() -> Self {
        Self {
            secret: JwtUtil::get_secret(),
            expiration_hours: JwtUtil::get_default_expiration(),
        }
    }

    /// Create with custom settings
    /// 使用自定义设置创建
    pub fn with_settings(secret: impl Into<String>, expiration_hours: i64) -> Self {
        Self {
            secret: secret.into(),
            expiration_hours,
        }
    }

    /// Generate token from authentication
    /// 从认证生成token
    pub fn generate_token(
        &self,
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
    ) -> SecurityResult<String> {
        let claims = JwtClaims::new(user_id, username, authorities, self.expiration_hours);

        let encoding_key = EncodingKey::from_secret(self.secret.as_ref());
        encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| SecurityError::TokenError(format!("Failed to encode token: {}", e)))
    }

    /// Validate token
    /// 验证token
    pub fn validate_token(&self, token: &str) -> SecurityResult<bool> {
        match JwtUtil::verify_token(token) {
            Ok(_) => Ok(true),
            Err(SecurityError::TokenExpired(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// Get authentication from token
    /// 从token获取认证
    pub fn get_authentication(&self, token: &str) -> SecurityResult<JwtClaims> {
        JwtUtil::verify_token(token)
    }

    /// Refresh token
    /// 刷新token
    pub fn refresh_token(&self, token: &str) -> SecurityResult<String> {
        JwtUtil::refresh_token(token)
    }
}

impl Default for JwtTokenProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// JWT authentication result
/// JWT认证结果
#[derive(Debug, Clone)]
pub struct JwtAuthentication {
    /// User ID
    pub user_id: String,

    /// Username
    pub username: String,

    /// Authorities
    pub authorities: Vec<Authority>,
}

impl JwtAuthentication {
    /// Create from claims
    /// 从声明创建
    pub fn from_claims(claims: &JwtClaims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            username: claims.username.clone(),
            authorities: claims.get_authorities(),
        }
    }

    /// Check if has authority
    /// 检查是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool {
        self.authorities.contains(authority)
    }

    /// Check if has role
    /// 检查是否有角色
    pub fn has_role(&self, role: &Role) -> bool {
        self.authorities.contains(&Authority::Role(role.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let authorities = vec![
            Authority::Role(Role::User),
            Authority::Permission("user:read".to_string()),
        ];

        let token = JwtUtil::create_token("123", "alice", &authorities).unwrap();
        assert!(!token.is_empty());

        let claims = JwtUtil::verify_token(&token).unwrap();
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.authorities.len(), 2);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_token_authorities() {
        let authorities = vec![Authority::Role(Role::Admin), Authority::Role(Role::User)];

        let token = JwtUtil::create_token("123", "admin", &authorities).unwrap();
        let claims = JwtUtil::verify_token(&token).unwrap();

        assert!(claims.has_role(&Role::Admin));
        assert!(claims.has_role(&Role::User));
        assert!(!claims.has_role(&Role::Guest));
    }

    #[test]
    fn test_token_provider() {
        let provider = JwtTokenProvider::new();
        let authorities = vec![Authority::Role(Role::User)];

        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        assert!(provider.validate_token(&token).unwrap());

        let auth = provider.get_authentication(&token).unwrap();
        assert_eq!(auth.username, "alice");
    }

    #[test]
    fn test_refresh_token() {
        let authorities = vec![Authority::Role(Role::User)];
        let old_token = JwtUtil::create_token("123", "alice", &authorities).unwrap();

        // Sleep briefly to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_secs(2));

        let new_token = JwtUtil::refresh_token(&old_token).unwrap();
        assert_ne!(old_token, new_token);

        let claims = JwtUtil::verify_token(&new_token).unwrap();
        assert_eq!(claims.sub, "123");
    }

    #[test]
    fn test_jwt_authentication_from_claims() {
        let authorities = vec![Authority::Role(Role::Admin)];
        let token = JwtUtil::create_token("123", "admin", &authorities).unwrap();
        let claims = JwtUtil::verify_token(&token).unwrap();

        let auth = JwtAuthentication::from_claims(&claims);
        assert_eq!(auth.user_id, "123");
        assert_eq!(auth.username, "admin");
        assert!(auth.has_role(&Role::Admin));
    }

    #[test]
    fn test_token_with_custom_expiration() {
        let authorities = vec![Authority::Role(Role::User)];
        let token =
            JwtUtil::create_token_with_expiration("123", "alice", &authorities, 48).unwrap();

        let claims = JwtUtil::verify_token(&token).unwrap();
        // Should expire in ~48 hours
        let time_left = claims.time_until_expiration();
        assert!(time_left.num_hours() > 47);
        assert!(time_left.num_hours() <= 48);
    }

    #[test]
    fn test_invalid_token() {
        let result = JwtUtil::verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_token_unsafe() {
        let authorities = vec![Authority::Role(Role::User)];
        let token = JwtUtil::create_token("123", "alice", &authorities).unwrap();

        let claims = JwtUtil::parse_token_unsafe(&token).unwrap();
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.username, "alice");
    }
}
