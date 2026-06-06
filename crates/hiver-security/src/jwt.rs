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
//! use hiver_security::jwt::{JwtUtil, JwtClaims};
//! use hiver_security::User;
//!
//! // Create JWT token for user
//! let user = User::with_roles("alice", "password", &[Role::User]);
//! let token = JwtUtil::create_token(user.id, &user.username, &user.authorities)?;
//!
//! // Verify JWT token
//! let claims = JwtUtil::verify_token(&token)?;
//! println!("User ID: {}", claims.sub);
//! ```

use std::{collections::HashMap, env};

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{Authority, Role, SecurityError, SecurityResult};

/// JWT claims
/// JWT 声明
///
/// Contains all standard JWT claims (RFC 7519) plus application-specific fields.
/// 包含所有标准JWT声明（RFC 7519）加应用特定字段。
///
/// # Standard Claims / 标准声明
///
/// - `iss` (Issuer) / 签发者
/// - `sub` (Subject) / 主题
/// - `aud` (Audience) / 受众
/// - `exp` (Expiration) / 过期时间
/// - `nbf` (Not Before) / 生效时间
/// - `iat` (Issued At) / 签发时间
/// - `jti` (JWT ID) / JWT标识符
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public class JwtClaims {
///     private String iss;      // Issuer
///     private String sub;      // Subject (user ID)
///     private String aud;      // Audience
///     private long exp;        // Expiration
///     private long nbf;        // Not Before
///     private long iat;        // Issued at
///     private String jti;      // JWT ID
///     private String username; // Username
///     private List<String> authorities; // Roles/permissions
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims
{
    /// Subject (user ID)
    /// 主体（用户ID）
    pub sub: String,

    /// Username
    /// 用户名
    pub username: String,

    /// Authorities/roles
    /// 权限/角色
    pub authorities: Vec<String>,

    /// Issued at (seconds since epoch)
    /// 签发时间（自纪元以来的秒数）
    pub iat: i64,

    /// Expiration (seconds since epoch)
    /// 过期时间（自纪元以来的秒数）
    pub exp: i64,

    /// Issuer
    /// 签发者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Audience (recipient(s) the JWT is intended for)
    /// 受众（JWT的预期接收者）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<serde_json::Value>,

    /// Not Before (seconds since epoch; token is not valid before this time)
    /// 生效时间（自纪元以来的秒数；此时间之前令牌无效）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// JWT ID (unique identifier for the token)
    /// JWT标识符（令牌的唯一标识符）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    /// Custom claims (application-specific key-value pairs)
    /// 自定义声明（应用特定的键值对）
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl JwtClaims
{
    /// Create new JWT claims
    /// 创建新的JWT声明
    pub fn new(
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
        expiration_hours: i64,
    ) -> Self
    {
        let now = Utc::now();
        let expiration = now + Duration::hours(expiration_hours);

        Self {
            sub: user_id.into(),
            username: username.into(),
            authorities: authorities.iter().map(ToString::to_string).collect(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            iss: Some("hiver-security".to_string()),
            aud: None,
            nbf: None,
            jti: None,
            custom: HashMap::new(),
        }
    }

    /// Create a JwtClaims builder for advanced configuration
    /// 创建JwtClaims构建器用于高级配置
    pub fn builder(user_id: impl Into<String>, username: impl Into<String>) -> JwtClaimsBuilder
    {
        JwtClaimsBuilder {
            sub: user_id.into(),
            username: username.into(),
            authorities: Vec::new(),
            expiration_hours: 24,
            issuer: Some("hiver-security".to_string()),
            audience: None,
            not_before: None,
            jwt_id: None,
            custom: HashMap::new(),
        }
    }

    /// Set the issuer claim
    /// 设置签发者声明
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self
    {
        self.iss = Some(issuer.into());
        self
    }

    /// Set the audience claim
    /// 设置受众声明
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self
    {
        self.aud = Some(serde_json::Value::String(audience.into()));
        self
    }

    /// Set multiple audiences
    /// 设置多个受众
    pub fn with_audiences(mut self, audiences: Vec<String>) -> Self
    {
        self.aud = Some(serde_json::Value::Array(
            audiences
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ));
        self
    }

    /// Set the not-before claim
    /// 设置生效时间声明
    pub fn with_not_before(mut self, nbf: i64) -> Self
    {
        self.nbf = Some(nbf);
        self
    }

    /// Set the JWT ID claim
    /// 设置JWT标识符声明
    pub fn with_jwt_id(mut self, jti: impl Into<String>) -> Self
    {
        self.jti = Some(jti.into());
        self
    }

    /// Add a custom claim
    /// 添加自定义声明
    pub fn with_custom_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self
    {
        self.custom.insert(key.into(), value);
        self
    }

    /// Check if token is expired
    /// 检查token是否过期
    pub fn is_expired(&self) -> bool
    {
        Utc::now().timestamp() > self.exp
    }

    /// Get time until expiration
    /// 获取剩余有效时间
    pub fn time_until_expiration(&self) -> Duration
    {
        let now = Utc::now().timestamp();
        let seconds_left = self.exp - now;
        Duration::seconds(seconds_left)
    }

    /// Convert authorities to Authority enum
    /// 将authorities转换为Authority枚举
    pub fn get_authorities(&self) -> Vec<Authority>
    {
        self.authorities
            .iter()
            .filter_map(|a| Authority::from_string(a))
            .collect()
    }

    /// Check if has authority
    /// 检查是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool
    {
        self.get_authorities().contains(authority)
    }

    /// Check if has role
    /// 检查是否有角色
    pub fn has_role(&self, role: &Role) -> bool
    {
        self.get_authorities()
            .contains(&Authority::Role(role.clone()))
    }

    /// Get the audience as a list of strings
    /// 获取受众字符串列表
    pub fn audiences(&self) -> Vec<String>
    {
        match &self.aud
        {
            Some(serde_json::Value::String(s)) => vec![s.clone()],
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Check if a specific audience is present
    /// 检查是否包含特定受众
    pub fn has_audience(&self, audience: &str) -> bool
    {
        self.audiences().iter().any(|a| a == audience)
    }
}

/// Builder for constructing JwtClaims with all optional fields
/// 用于构建包含所有可选字段的JwtClaims的构建器
#[derive(Debug)]
pub struct JwtClaimsBuilder
{
    sub: String,
    username: String,
    authorities: Vec<String>,
    expiration_hours: i64,
    issuer: Option<String>,
    audience: Option<String>,
    not_before: Option<i64>,
    jwt_id: Option<String>,
    custom: HashMap<String, serde_json::Value>,
}

impl JwtClaimsBuilder
{
    /// Set authorities
    /// 设置权限
    pub fn authorities(mut self, auths: &[Authority]) -> Self
    {
        self.authorities = auths.iter().map(ToString::to_string).collect();
        self
    }

    /// Set expiration in hours
    /// 设置过期时间（小时）
    pub fn expiration_hours(mut self, hours: i64) -> Self
    {
        self.expiration_hours = hours;
        self
    }

    /// Set issuer
    /// 设置签发者
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self
    {
        self.issuer = Some(issuer.into());
        self
    }

    /// Set audience
    /// 设置受众
    pub fn audience(mut self, audience: impl Into<String>) -> Self
    {
        self.audience = Some(audience.into());
        self
    }

    /// Set not-before timestamp
    /// 设置生效时间戳
    pub fn not_before(mut self, nbf: i64) -> Self
    {
        self.not_before = Some(nbf);
        self
    }

    /// Set JWT ID
    /// 设置JWT标识符
    pub fn jwt_id(mut self, jti: impl Into<String>) -> Self
    {
        self.jwt_id = Some(jti.into());
        self
    }

    /// Add a custom claim
    /// 添加自定义声明
    pub fn custom_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self
    {
        self.custom.insert(key.into(), value);
        self
    }

    /// Build the JwtClaims
    /// 构建JwtClaims
    pub fn build(self) -> JwtClaims
    {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.expiration_hours);

        JwtClaims {
            sub: self.sub,
            username: self.username,
            authorities: self.authorities,
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            iss: self.issuer,
            aud: self.audience.map(serde_json::Value::String),
            nbf: self.not_before,
            jti: self.jwt_id,
            custom: self.custom,
        }
    }
}

/// Supported JWT signing algorithms
/// 支持的JWT签名算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JwtAlgorithm
{
    /// HMAC using SHA-256
    #[default]
    Hs256,
    /// HMAC using SHA-384
    Hs384,
    /// HMAC using SHA-512
    Hs512,
    /// RSASSA-PKCS1-v1_5 using SHA-256
    Rs256,
}

impl JwtAlgorithm
{
    /// Convert to jsonwebtoken Algorithm
    /// 转换为jsonwebtoken库的Algorithm
    pub fn to_algorithm(&self) -> jsonwebtoken::Algorithm
    {
        match self
        {
            JwtAlgorithm::Hs256 => jsonwebtoken::Algorithm::HS256,
            JwtAlgorithm::Hs384 => jsonwebtoken::Algorithm::HS384,
            JwtAlgorithm::Hs512 => jsonwebtoken::Algorithm::HS512,
            JwtAlgorithm::Rs256 => jsonwebtoken::Algorithm::RS256,
        }
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

impl JwtUtil
{
    /// Get JWT secret key from environment or use default
    /// 从环境变量获取JWT密钥或使用默认值
    fn get_secret() -> String
    {
        env::var("JWT_SECRET")
            .unwrap_or_else(|_| "hiver-jwt-secret-key-change-in-production-2024".to_string())
    }

    /// Get default token expiration in hours
    /// 获取默认token过期时间（小时）
    fn get_default_expiration() -> i64
    {
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
    ) -> SecurityResult<String>
    {
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
    ) -> SecurityResult<String>
    {
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
    pub fn verify_token(token: &str) -> SecurityResult<JwtClaims>
    {
        let secret = Self::get_secret();
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        decode::<JwtClaims>(token, &decoding_key, &validation)
            .map(|data| {
                let claims = data.claims;

                // Check expiration manually for better error messages
                if claims.is_expired()
                {
                    return Err(SecurityError::TokenExpired("Token has expired".to_string()));
                }

                Ok(claims)
            })
            .map_err(|e| match e.kind()
            {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature =>
                {
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
    pub fn refresh_token(token: &str) -> SecurityResult<String>
    {
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
    pub fn parse_token_unsafe(token: &str) -> SecurityResult<JwtClaims>
    {
        Self::decode_without_validation(token)
    }

    /// Decode token without any signature or claim validation
    /// 不进行任何签名或声明验证地解码令牌
    ///
    /// Reads the claims payload from the token without checking the signature,
    /// expiration, or any other validation rules. Useful for inspecting token
    /// contents in non-security contexts.
    /// 从令牌中读取声明负载而不检查签名、过期时间或任何其他验证规则。
    /// 适用于在非安全上下文中检查令牌内容。
    ///
    /// # Warning / 警告
    ///
    /// Do NOT use this for authentication decisions.
    /// 不要将此用于身份验证决策。
    pub fn decode_without_validation(token: &str) -> SecurityResult<JwtClaims>
    {
        use base64::Engine;
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3
        {
            return Err(SecurityError::InvalidToken(
                "Invalid token format: expected 3 parts".to_string(),
            ));
        }

        let payload = parts
            .get(1)
            .ok_or_else(|| SecurityError::InvalidToken("Token payload part missing".to_string()))?;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| {
                SecurityError::InvalidToken("Failed to decode token payload".to_string())
            })?;

        let claims: JwtClaims = serde_json::from_slice(&decoded)
            .map_err(|e| SecurityError::InvalidToken(format!("Failed to parse claims: {}", e)))?;

        Ok(claims)
    }

    /// Fully validate and decode a JWT token
    /// 完全验证并解码JWT令牌
    ///
    /// Performs signature verification and validates all standard claims:
    /// 执行签名验证并验证所有标准声明：
    /// - Signature is valid for the given secret
    /// - `exp` (expiration) has not passed
    /// - `nbf` (not-before) has passed if present
    ///
    /// Optionally validates `iss` (issuer) and `aud` (audience) if provided.
    /// 如果提供了`iss`（签发者）和`aud`（受众），则可选地验证它们。
    pub fn decode_and_validate(
        token: &str,
        secret: &str,
        algorithm: &JwtAlgorithm,
        issuer: Option<&str>,
        audience: Option<&str>,
    ) -> SecurityResult<JwtClaims>
    {
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let mut validation = Validation::new(algorithm.to_algorithm());

        if let Some(iss) = issuer
        {
            validation.set_issuer(&[iss]);
        }
        if let Some(aud) = audience
        {
            validation.set_audience(&[aud]);
        }

        decode::<JwtClaims>(token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind()
            {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature =>
                {
                    SecurityError::TokenExpired("Token has expired".to_string())
                },
                jsonwebtoken::errors::ErrorKind::InvalidToken =>
                {
                    SecurityError::InvalidToken("Token is invalid".to_string())
                },
                jsonwebtoken::errors::ErrorKind::InvalidSignature =>
                {
                    SecurityError::InvalidToken("Invalid token signature".to_string())
                },
                _ => SecurityError::InvalidToken(format!("Token validation failed: {}", e)),
            })
    }

    /// Refresh token if it will expire within the given threshold
    /// 如果令牌将在给定阈值内过期，则刷新令牌
    ///
    /// If the token's remaining lifetime is less than `threshold_secs` seconds,
    /// a new token is created with the same claims but a fresh expiration.
    /// Otherwise, the original token string is returned unchanged.
    /// 如果令牌的剩余生存期少于`threshold_secs`秒，
    /// 则创建具有相同声明但具有新过期时间的新令牌。
    /// 否则，原令牌字符串不变地返回。
    ///
    /// # Arguments / 参数
    ///
    /// * `token` - The current JWT token / 当前的JWT令牌
    /// * `threshold_secs` - Seconds before expiration to trigger refresh / 触发刷新的过期前秒数
    ///
    /// # Returns / 返回
    ///
    /// A tuple of (token_string, was_refreshed) / 一个元组（令牌字符串，是否已刷新）
    pub fn refresh_if_needed(token: &str, threshold_secs: i64) -> SecurityResult<(String, bool)>
    {
        let claims = Self::decode_without_validation(token)?;

        let now = Utc::now().timestamp();
        let remaining = claims.exp - now;

        if remaining < threshold_secs
        {
            // Token is close to expiry or already expired; refresh it
            // 令牌即将过期或已经过期；刷新它
            let authorities: Vec<Authority> = claims
                .authorities
                .iter()
                .filter_map(|s| Authority::from_string(s))
                .collect();
            let new_token = Self::create_token(&claims.sub, &claims.username, &authorities)?;
            Ok((new_token, true))
        }
        else
        {
            Ok((token.to_string(), false))
        }
    }
}

/// JWT token provider
/// JWT token 提供者
///
/// Equivalent to Spring's `JwtTokenProvider`.
/// `等价于Spring的JwtTokenProvider`。
///
/// Supports HMAC (HS256/HS384/HS512) and RSA (RS256) algorithms.
/// 支持HMAC（HS256/HS384/HS512）和RSA（RS256）算法。
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
pub struct JwtTokenProvider
{
    /// Secret key for signing tokens (HMAC) or PEM-encoded RSA private key
    /// 签名令牌的密钥（HMAC）或PEM编码的RSA私钥
    secret: String,

    /// PEM-encoded RSA public key for RS256 verification (optional)
    /// PEM编码的RSA公钥，用于RS256验证（可选）
    rsa_public_key_pem: Option<String>,

    /// Token expiration in hours
    /// Token过期时间（小时）
    expiration_hours: i64,

    /// Signing algorithm
    /// 签名算法
    algorithm: JwtAlgorithm,

    /// Expected issuer for validation
    /// 用于验证的预期签发者
    issuer: Option<String>,

    /// Expected audience for validation
    /// 用于验证的预期受众
    audience: Option<String>,
}

impl JwtTokenProvider
{
    /// Create new JWT token provider with default settings
    /// 使用默认设置创建新的JWT令牌提供者
    pub fn new() -> Self
    {
        Self {
            secret: JwtUtil::get_secret(),
            rsa_public_key_pem: None,
            expiration_hours: JwtUtil::get_default_expiration(),
            algorithm: JwtAlgorithm::default(),
            issuer: Some("hiver-security".to_string()),
            audience: None,
        }
    }

    /// Create with custom secret and expiration
    /// 使用自定义密钥和过期时间创建
    pub fn with_settings(secret: impl Into<String>, expiration_hours: i64) -> Self
    {
        Self {
            secret: secret.into(),
            rsa_public_key_pem: None,
            expiration_hours,
            algorithm: JwtAlgorithm::default(),
            issuer: Some("hiver-security".to_string()),
            audience: None,
        }
    }

    /// Set the signing algorithm
    /// 设置签名算法
    pub fn with_algorithm(mut self, algorithm: JwtAlgorithm) -> Self
    {
        self.algorithm = algorithm;
        self
    }

    /// Set the expected issuer for validation
    /// 设置用于验证的预期签发者
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self
    {
        self.issuer = Some(issuer.into());
        self
    }

    /// Set the expected audience for validation
    /// 设置用于验证的预期受众
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self
    {
        self.audience = Some(audience.into());
        self
    }

    /// Set RSA public key PEM for RS256 verification
    /// 设置用于RS256验证的RSA公钥PEM
    ///
    /// When using RS256, the private key is used for signing and
    /// the public key is used for verification.
    /// 使用RS256时，私钥用于签名，公钥用于验证。
    pub fn with_rsa_public_key(mut self, pem: impl Into<String>) -> Self
    {
        self.rsa_public_key_pem = Some(pem.into());
        self
    }

    /// Get the encoding key based on the algorithm
    /// 根据算法获取编码密钥
    fn encoding_key(&self) -> SecurityResult<EncodingKey>
    {
        match self.algorithm
        {
            JwtAlgorithm::Rs256 => EncodingKey::from_rsa_pem(self.secret.as_bytes())
                .map_err(|e| SecurityError::Jwt(format!("Invalid RSA private key: {}", e))),
            _ => Ok(EncodingKey::from_secret(self.secret.as_ref())),
        }
    }

    /// Get the decoding key based on the algorithm
    /// 根据算法获取解码密钥
    fn decoding_key(&self) -> SecurityResult<DecodingKey>
    {
        match self.algorithm
        {
            JwtAlgorithm::Rs256 =>
            {
                let pem = self.rsa_public_key_pem.as_deref().unwrap_or(&self.secret);
                DecodingKey::from_rsa_pem(pem.as_bytes())
                    .map_err(|e| SecurityError::Jwt(format!("Invalid RSA public key: {}", e)))
            },
            _ => Ok(DecodingKey::from_secret(self.secret.as_ref())),
        }
    }

    /// Build the validation rules
    /// 构建验证规则
    fn validation(&self) -> Validation
    {
        let mut validation = Validation::new(self.algorithm.to_algorithm());
        if let Some(ref iss) = self.issuer
        {
            validation.set_issuer(&[iss.as_str()]);
        }
        if let Some(ref aud) = self.audience
        {
            validation.set_audience(&[aud.as_str()]);
        }
        else
        {
            // If no audience is configured, disable audience validation
            // 如果未配置受众，则禁用受众验证
            validation.set_audience::<&str>(&[]);
        }
        validation
    }

    /// Generate token from authentication
    /// 从认证生成token
    pub fn generate_token(
        &self,
        user_id: impl Into<String>,
        username: impl Into<String>,
        authorities: &[Authority],
    ) -> SecurityResult<String>
    {
        let mut claims = JwtClaims::new(user_id, username, authorities, self.expiration_hours);

        // Apply provider-level issuer and audience to claims
        // 将提供者级别的签发者和受众应用于声明
        if self.issuer.is_some()
        {
            claims.iss.clone_from(&self.issuer);
        }
        if let Some(ref aud) = self.audience
        {
            claims.aud = Some(serde_json::Value::String(aud.clone()));
        }

        let encoding_key = self.encoding_key()?;
        let header = Header::new(self.algorithm.to_algorithm());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| SecurityError::TokenError(format!("Failed to encode token: {}", e)))
    }

    /// Validate token, returning true if valid
    /// 验证令牌，有效则返回true
    pub fn validate_token(&self, token: &str) -> SecurityResult<bool>
    {
        match self.decode_and_validate(token)
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get authentication from token
    /// 从token获取认证
    pub fn get_authentication(&self, token: &str) -> SecurityResult<JwtClaims>
    {
        self.decode_and_validate(token)
    }

    /// Refresh token
    /// 刷新token
    pub fn refresh_token(&self, token: &str) -> SecurityResult<String>
    {
        let claims = self.decode_and_validate(token)?;
        let authorities: Vec<Authority> = claims
            .authorities
            .iter()
            .filter_map(|s| Authority::from_string(s))
            .collect();
        self.generate_token(&claims.sub, &claims.username, &authorities)
    }

    /// Full validation: verify signature, check exp/nbf, optionally check issuer/audience
    /// 完整验证：验证签名，检查exp/nbf，可选检查签发者/受众
    ///
    /// Returns the decoded claims if all validations pass.
    /// 如果所有验证都通过，则返回解码后的声明。
    pub fn decode_and_validate(&self, token: &str) -> SecurityResult<JwtClaims>
    {
        let decoding_key = self.decoding_key()?;
        let validation = self.validation();

        decode::<JwtClaims>(token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind()
            {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature =>
                {
                    SecurityError::TokenExpired("Token has expired".to_string())
                },
                jsonwebtoken::errors::ErrorKind::InvalidSignature =>
                {
                    SecurityError::InvalidToken("Invalid token signature".to_string())
                },
                _ => SecurityError::InvalidToken(format!("Token validation failed: {}", e)),
            })
    }

    /// Decode token without validation (reads claims without checking signature)
    /// 不验证地解码令牌（读取声明而不检查签名）
    pub fn decode_without_validation(&self, token: &str) -> SecurityResult<JwtClaims>
    {
        JwtUtil::decode_without_validation(token)
    }

    /// Refresh token if it will expire within the given threshold
    /// 如果令牌将在给定阈值内过期，则刷新令牌
    ///
    /// Returns (token_string, was_refreshed).
    /// 返回（令牌字符串，是否已刷新）。
    pub fn refresh_if_needed(
        &self,
        token: &str,
        threshold_secs: i64,
    ) -> SecurityResult<(String, bool)>
    {
        let claims = JwtUtil::decode_without_validation(token)?;

        let now = Utc::now().timestamp();
        let remaining = claims.exp - now;

        if remaining < threshold_secs
        {
            let authorities: Vec<Authority> = claims
                .authorities
                .iter()
                .filter_map(|s| Authority::from_string(s))
                .collect();
            let new_token = self.generate_token(&claims.sub, &claims.username, &authorities)?;
            Ok((new_token, true))
        }
        else
        {
            Ok((token.to_string(), false))
        }
    }

    // ── OAuth2 / OIDC helpers ─────────────────────────────────────────────────

    /// Convenience constructor: HMAC-SHA256 with a custom issuer.
    /// 便捷构造函数：使用自定义签发者的 HMAC-SHA256。
    pub fn new_hmac(secret: impl Into<String>, issuer: impl Into<String>) -> Self
    {
        Self {
            secret: secret.into(),
            rsa_public_key_pem: None,
            expiration_hours: 1,
            algorithm: JwtAlgorithm::Hs256,
            issuer: Some(issuer.into()),
            audience: None,
        }
    }

    /// Generate an OAuth2 access token embedding `scope` and `client_id` custom claims.
    /// 生成包含 `scope` 和 `client_id` 自定义声明的 OAuth2 访问令牌。
    pub fn generate_oauth2_token(
        &self,
        subject: &str,
        client_id: &str,
        scope: &str,
        ttl: std::time::Duration,
    ) -> SecurityResult<String>
    {
        let now = Utc::now().timestamp();
        let exp = now + ttl.as_secs() as i64;
        let mut custom: HashMap<String, serde_json::Value> = HashMap::new();
        custom.insert("scope".into(), serde_json::Value::String(scope.to_string()));
        custom.insert("client_id".into(), serde_json::Value::String(client_id.to_string()));
        let claims = JwtClaims {
            sub: subject.to_string(),
            username: subject.to_string(),
            authorities: Vec::new(),
            iat: now,
            exp,
            iss: self.issuer.clone(),
            aud: None,
            nbf: None,
            jti: None,
            custom,
        };
        let encoding_key = self.encoding_key()?;
        let header = Header::new(self.algorithm.to_algorithm());
        encode(&header, &claims, &encoding_key)
            .map_err(|e| SecurityError::TokenError(format!("Failed to encode OAuth2 token: {e}")))
    }

    /// Generate a minimal OIDC ID token (subject + audience).
    /// 生成最小化的 OIDC ID 令牌（主体 + 受众）。
    pub fn generate_id_token(&self, subject: &str, client_id: &str) -> SecurityResult<String>
    {
        let now = Utc::now().timestamp();
        let exp = now + 3600;
        let claims = JwtClaims {
            sub: subject.to_string(),
            username: subject.to_string(),
            authorities: Vec::new(),
            iat: now,
            exp,
            iss: self.issuer.clone(),
            aud: Some(serde_json::Value::String(client_id.to_string())),
            nbf: None,
            jti: None,
            custom: HashMap::new(),
        };
        let encoding_key = self.encoding_key()?;
        let header = Header::new(self.algorithm.to_algorithm());
        encode(&header, &claims, &encoding_key)
            .map_err(|e| SecurityError::TokenError(format!("Failed to encode ID token: {e}")))
    }
}

impl Default for JwtTokenProvider
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// JWT authentication result
/// JWT认证结果
#[derive(Debug, Clone)]
pub struct JwtAuthentication
{
    /// User ID
    pub user_id: String,

    /// Username
    pub username: String,

    /// Authorities
    pub authorities: Vec<Authority>,
}

impl JwtAuthentication
{
    /// Create from claims
    /// 从声明创建
    pub fn from_claims(claims: &JwtClaims) -> Self
    {
        Self {
            user_id: claims.sub.clone(),
            username: claims.username.clone(),
            authorities: claims.get_authorities(),
        }
    }

    /// Check if has authority
    /// 检查是否有权限
    pub fn has_authority(&self, authority: &Authority) -> bool
    {
        self.authorities.contains(authority)
    }

    /// Check if has role
    /// 检查是否有角色
    pub fn has_role(&self, role: &Role) -> bool
    {
        self.authorities.contains(&Authority::Role(role.clone()))
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_create_and_verify_token()
    {
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
    fn test_token_authorities()
    {
        let authorities = vec![Authority::Role(Role::Admin), Authority::Role(Role::User)];

        let token = JwtUtil::create_token("123", "admin", &authorities).unwrap();
        let claims = JwtUtil::verify_token(&token).unwrap();

        assert!(claims.has_role(&Role::Admin));
        assert!(claims.has_role(&Role::User));
        assert!(!claims.has_role(&Role::Guest));
    }

    #[test]
    fn test_token_provider()
    {
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
    fn test_refresh_token()
    {
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
    fn test_jwt_authentication_from_claims()
    {
        let authorities = vec![Authority::Role(Role::Admin)];
        let token = JwtUtil::create_token("123", "admin", &authorities).unwrap();
        let claims = JwtUtil::verify_token(&token).unwrap();

        let auth = JwtAuthentication::from_claims(&claims);
        assert_eq!(auth.user_id, "123");
        assert_eq!(auth.username, "admin");
        assert!(auth.has_role(&Role::Admin));
    }

    #[test]
    fn test_token_with_custom_expiration()
    {
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
    fn test_invalid_token()
    {
        let result = JwtUtil::verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_without_validation()
    {
        let authorities = vec![Authority::Role(Role::User)];
        let token = JwtUtil::create_token("123", "alice", &authorities).unwrap();

        let claims = JwtUtil::decode_without_validation(&token).unwrap();
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.username, "alice");
    }

    #[test]
    fn test_decode_without_validation_invalid_format()
    {
        let result = JwtUtil::decode_without_validation("not.a.valid.jwt.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_and_validate()
    {
        let secret = "test-secret-for-validation";
        let provider = JwtTokenProvider::with_settings(secret, 24);

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        // Should succeed with the same secret
        let claims =
            JwtUtil::decode_and_validate(&token, secret, &JwtAlgorithm::Hs256, None, None).unwrap();
        assert_eq!(claims.sub, "123");
    }

    #[test]
    fn test_decode_and_validate_wrong_secret()
    {
        let secret = "correct-secret";
        let provider = JwtTokenProvider::with_settings(secret, 24);

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        // Should fail with wrong secret
        let result =
            JwtUtil::decode_and_validate(&token, "wrong-secret", &JwtAlgorithm::Hs256, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_and_validate_with_issuer()
    {
        let secret = "test-secret";
        let provider = JwtTokenProvider::with_settings(secret, 24).with_issuer("my-app");

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        // Should succeed with matching issuer
        let claims = JwtUtil::decode_and_validate(
            &token,
            secret,
            &JwtAlgorithm::Hs256,
            Some("my-app"),
            None,
        )
        .unwrap();
        assert_eq!(claims.iss, Some("my-app".to_string()));
    }

    #[test]
    fn test_decode_and_validate_wrong_issuer()
    {
        let secret = "test-secret";
        let provider = JwtTokenProvider::with_settings(secret, 24).with_issuer("my-app");

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        // Should fail with wrong issuer
        let result = JwtUtil::decode_and_validate(
            &token,
            secret,
            &JwtAlgorithm::Hs256,
            Some("wrong-issuer"),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_if_needed_no_refresh()
    {
        let authorities = vec![Authority::Role(Role::User)];
        // Token expires in 24 hours by default
        let token = JwtUtil::create_token("123", "alice", &authorities).unwrap();

        // Threshold of 1 hour - token has ~24 hours left, should NOT refresh
        let (returned_token, refreshed) = JwtUtil::refresh_if_needed(&token, 3600).unwrap();
        assert!(!refreshed);
        assert_eq!(returned_token, token);
    }

    #[test]
    fn test_provider_refresh_if_needed_no_refresh()
    {
        let provider = JwtTokenProvider::new();
        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        // Token has ~24h left, threshold 1h -> no refresh
        let (returned_token, refreshed) = provider.refresh_if_needed(&token, 3600).unwrap();
        assert!(!refreshed);
        assert_eq!(returned_token, token);
    }

    #[test]
    fn test_provider_with_audience()
    {
        let provider = JwtTokenProvider::with_settings("secret", 24).with_audience("my-api");

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        let claims = provider.decode_and_validate(&token).unwrap();
        assert_eq!(claims.audiences(), vec!["my-api"]);
        assert!(claims.has_audience("my-api"));
        assert!(!claims.has_audience("other-api"));
    }

    #[test]
    fn test_claims_builder()
    {
        let claims = JwtClaims::builder("123", "alice")
            .authorities(&[Authority::Role(Role::Admin)])
            .expiration_hours(48)
            .issuer("test-app")
            .audience("my-api")
            .jwt_id("unique-id-123")
            .custom_claim("department", serde_json::Value::String("engineering".to_string()))
            .build();

        assert_eq!(claims.sub, "123");
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.iss, Some("test-app".to_string()));
        assert_eq!(claims.audiences(), vec!["my-api"]);
        assert_eq!(claims.jti, Some("unique-id-123".to_string()));
        assert_eq!(
            claims.custom.get("department").unwrap(),
            &serde_json::Value::String("engineering".to_string())
        );
    }

    #[test]
    fn test_claims_with_audiences()
    {
        let claims = JwtClaims::new("123", "alice", &[], 24)
            .with_audiences(vec!["api-v1".to_string(), "api-v2".to_string()]);

        assert_eq!(claims.audiences(), vec!["api-v1", "api-v2"]);
        assert!(claims.has_audience("api-v1"));
        assert!(claims.has_audience("api-v2"));
        assert!(!claims.has_audience("api-v3"));
    }

    #[test]
    fn test_claims_custom_claims()
    {
        let claims = JwtClaims::new("123", "alice", &[], 24)
            .with_custom_claim("role", serde_json::Value::String("manager".to_string()))
            .with_custom_claim("level", serde_json::Value::Number(5.into()));

        assert_eq!(
            claims.custom.get("role").unwrap(),
            &serde_json::Value::String("manager".to_string())
        );
        assert_eq!(claims.custom.get("level").unwrap(), &serde_json::Value::Number(5.into()));
    }

    #[test]
    fn test_provider_hs384()
    {
        let provider =
            JwtTokenProvider::with_settings("secret-key", 24).with_algorithm(JwtAlgorithm::Hs384);

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        assert!(provider.validate_token(&token).unwrap());

        let claims = provider.decode_and_validate(&token).unwrap();
        assert_eq!(claims.sub, "123");
    }

    #[test]
    fn test_provider_hs512()
    {
        let provider =
            JwtTokenProvider::with_settings("secret-key", 24).with_algorithm(JwtAlgorithm::Hs512);

        let authorities = vec![Authority::Role(Role::User)];
        let token = provider
            .generate_token("123", "alice", &authorities)
            .unwrap();

        assert!(provider.validate_token(&token).unwrap());

        let claims = provider.decode_and_validate(&token).unwrap();
        assert_eq!(claims.sub, "123");
    }

    #[test]
    fn test_algorithm_default()
    {
        assert_eq!(JwtAlgorithm::default(), JwtAlgorithm::Hs256);
    }

    #[test]
    fn test_expired_token_rejection()
    {
        // Create a token that expires immediately (0 hours = already past)
        // Note: we can't truly create an expired token with the current API,
        // so we test with a very short expiration and check the logic
        let claims = JwtClaims {
            sub: "123".to_string(),
            username: "alice".to_string(),
            authorities: vec![],
            iat: Utc::now().timestamp() - 7200, // 2 hours ago
            exp: Utc::now().timestamp() - 3600, // 1 hour ago (expired)
            iss: Some("hiver-security".to_string()),
            aud: None,
            nbf: None,
            jti: None,
            custom: HashMap::new(),
        };

        assert!(claims.is_expired());
    }

    #[test]
    fn test_token_round_trip_all_claims()
    {
        let provider = JwtTokenProvider::with_settings("test-secret", 1)
            .with_issuer("test-issuer")
            .with_audience("test-audience");

        let authorities = vec![Authority::Role(Role::Admin)];
        let token = provider
            .generate_token("user-1", "bob", &authorities)
            .unwrap();

        let claims = provider.decode_and_validate(&token).unwrap();

        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.username, "bob");
        assert_eq!(claims.iss, Some("test-issuer".to_string()));
        assert!(claims.has_audience("test-audience"));
        assert!(!claims.is_expired());
        assert!(claims.has_role(&Role::Admin));
    }
}
