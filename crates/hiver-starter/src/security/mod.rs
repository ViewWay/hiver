//! Security 自动配置模块 / Security Auto-Configuration Module
//!
//! 自动配置安全功能（认证、授权、JWT）。
//! Auto-configures security features (authentication, authorization, JWT).

// Re-export security types
// 重新导出安全类型
pub use hiver_security::{
    // Authentication & Authorization
    Authentication,
    AuthenticationManager,
    // Authorities & Roles
    Authority,
    BcryptPasswordEncoder,
    JwtAuthentication,
    JwtClaims,
    // JWT
    JwtTokenProvider,
    JwtUtil,
    // Password encoding
    PasswordEncoder,
    Permission,
    // Security annotations
    PreAuthorize,
    Role,
    Secured,
    SecurityContext,
    // Errors
    SecurityError,
    SecurityResult,
    // Users
    User,
    UserDetails,
    UserService,
};

use crate::core::{ApplicationContext, AutoConfiguration};

// ============================================================================
// SecurityAutoConfiguration / 安全自动配置
// ============================================================================

/// Security 自动配置
/// Security auto-configuration
///
/// 参考 Spring Boot 的 `SecurityAutoConfiguration`。
/// Based on Spring Boot's `SecurityAutoConfiguration`.
#[derive(Debug)]
pub struct SecurityAutoConfiguration
{
    /// 是否启用安全
    pub enabled: bool,

    /// JWT 密钥
    pub jwt_secret: Option<String>,

    /// JWT 过期时间（秒）
    pub jwt_expiration: u64,
}

impl SecurityAutoConfiguration
{
    /// 创建新的安全自动配置
    pub fn new() -> Self
    {
        Self {
            enabled: true,
            jwt_secret: None,
            jwt_expiration: 3600,
        }
    }

    /// 从配置创建
    pub fn from_config(ctx: &ApplicationContext) -> Self
    {
        Self {
            enabled: ctx
                .get_property("security.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(true),
            jwt_secret: ctx.get_property("security.jwt_secret"),
            jwt_expiration: ctx
                .get_property("security.jwt_expiration")
                .and_then(|p| p.parse().ok())
                .unwrap_or(3600),
        }
    }
}

impl Default for SecurityAutoConfiguration
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoConfiguration for SecurityAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "SecurityAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        50 // 在服务器配置之后
    }

    fn condition(&self) -> bool
    {
        self.enabled
    }

    fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        tracing::info!("Configuring Security");

        // Create JwtTokenProvider with configured settings
        // 使用配置的设置创建 JwtTokenProvider
        let secret = self.jwt_secret.clone().unwrap_or_else(|| {
            tracing::warn!("JWT secret not configured, using default (insecure!)");
            "hiver-jwt-secret-key-change-in-production-2024".to_string()
        });

        // Convert expiration from seconds to hours
        // 将过期时间从秒转换为小时
        let expiration_hours = (self.jwt_expiration / 3600) as i64;

        let provider = JwtTokenProvider::with_settings(secret, expiration_hours);
        ctx.register_bean(provider);
        tracing::info!("Registered JwtTokenProvider bean (expiration: {}s)", self.jwt_expiration);

        Ok(())
    }
}

// ============================================================================
// JwtAutoConfiguration / JWT 自动配置
// ============================================================================

/// JWT 自动配置
/// JWT auto-configuration
///
/// 配置 JWT 相关的 Bean。
/// Configures JWT-related beans.
#[derive(Debug)]
pub struct JwtAutoConfiguration;

impl AutoConfiguration for JwtAutoConfiguration
{
    fn name(&self) -> &'static str
    {
        "JwtAutoConfiguration"
    }

    fn order(&self) -> i32
    {
        60 // 在 SecurityAutoConfiguration 之后
    }

    fn configure(&self, _ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        // JWT is configured in SecurityAutoConfiguration
        // This is a placeholder for additional JWT-specific configuration
        // JWT 在 SecurityAutoConfiguration 中配置
        // 这是额外的 JWT 特定配置的占位符
        tracing::info!("JWT additional configuration completed");
        Ok(())
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_security_auto_config()
    {
        let config = SecurityAutoConfiguration::new();
        assert!(config.enabled);
        assert_eq!(config.jwt_expiration, 3600);
        assert!(config.jwt_secret.is_none());
    }

    #[test]
    fn test_security_auto_config_with_secret()
    {
        let config = SecurityAutoConfiguration {
            enabled: true,
            jwt_secret: Some("my-secret-key".to_string()),
            jwt_expiration: 7200,
        };
        assert_eq!(config.jwt_secret, Some("my-secret-key".to_string()));
        assert_eq!(config.jwt_expiration, 7200);
    }

    #[test]
    fn test_security_auto_config_registers_provider()
    {
        let config = SecurityAutoConfiguration {
            enabled: true,
            jwt_secret: Some("test-secret".to_string()),
            jwt_expiration: 3600,
        };

        let mut ctx = ApplicationContext::new();
        config.configure(&mut ctx).unwrap();

        // Verify JwtTokenProvider was registered
        // 验证 JwtTokenProvider 已注册
        assert!(ctx.contains_bean::<JwtTokenProvider>());
    }

    #[test]
    fn test_security_auto_config_with_default_secret()
    {
        let config = SecurityAutoConfiguration {
            enabled: true,
            jwt_secret: None,
            jwt_expiration: 3600,
        };

        let mut ctx = ApplicationContext::new();
        config.configure(&mut ctx).unwrap();

        // Should still register with default secret
        // 应该仍然使用默认密钥注册
        assert!(ctx.contains_bean::<JwtTokenProvider>());
    }
}
