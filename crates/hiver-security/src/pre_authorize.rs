//! `PreAuthorize` module
//! PreAuthorize模块（@PreAuthorize等价物）

use std::{future::Future, pin::Pin};

use crate::{Authority, SecurityContext};

/// Security expression for @`PreAuthorize`
/// @`PreAuthorize的安全表达式`
///
/// Equivalent to Spring Security's `SpEL` expressions in @`PreAuthorize`.
/// 等价于Spring Security在@PreAuthorize中的SpEL表达式。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PreAuthorize("hasRole('ADMIN') and #username == authentication.name")
/// @PreAuthorize("hasAuthority('USER:READ')")
/// @PreAuthorize("#userId == authentication.principal.id")
/// ```
#[derive(Debug, Clone)]
pub enum SecurityExpression
{
    /// Has role
    /// 有角色
    HasRole(String),

    /// Has authority/permission
    /// 有权限/许可
    HasAuthority(String),

    /// Is authenticated
    /// 已认证
    IsAuthenticated,

    /// Is anonymous
    /// 是匿名
    IsAnonymous,

    /// Is fully authenticated (not remember-me)
    /// 完全认证（非记住我）
    IsFullyAuthenticated,

    /// Has permission on object
    /// 对对象有许可
    HasPermission(String, String),

    /// Custom expression
    /// 自定义表达式
    Custom(String),
}

impl SecurityExpression
{
    /// Evaluate the expression
    /// 评估表达式
    pub async fn evaluate(&self, context: &SecurityContext) -> bool
    {
        match self
        {
            SecurityExpression::IsAuthenticated => context.is_authenticated().await,
            SecurityExpression::IsAnonymous => !context.is_authenticated().await,
            SecurityExpression::IsFullyAuthenticated =>
            {
                // For now, same as authenticated
                context.is_authenticated().await
            },
            SecurityExpression::HasRole(role) =>
            {
                let role = crate::Role::from_str(role);
                context.has_role(&role).await
            },
            SecurityExpression::HasAuthority(auth) =>
            {
                let authority = Authority::Permission(auth.clone());
                context.has_authority(&authority).await
            },
            SecurityExpression::HasPermission(target, permission) =>
            {
                let auth = Authority::Permission(format!("{}:{}", target, permission));
                context.has_authority(&auth).await
            },
            SecurityExpression::Custom(expr) =>
            {
                // Custom expressions would need a full expression parser
                // For now, return false
                tracing::warn!("Custom security expression not implemented: {}", expr);
                false
            },
        }
    }

    /// Parse expression from string
    /// 从字符串解析表达式
    pub fn parse(input: &str) -> Vec<Self>
    {
        let mut expressions = Vec::new();

        // Simple parser for common patterns
        // In a full implementation, this would use a proper expression language

        // Try hasRole with single quotes first, then double quotes
        if input.contains("hasRole(")
        {
            if let Some(start) = input.find("hasRole('")
            {
                if let Some(end) = input[start..].find("')")
                {
                    let role = &input[start + 9..start + end];
                    expressions.push(SecurityExpression::HasRole(role.to_string()));
                }
            }
            else if let Some(start) = input.find("hasRole(\"")
                && let Some(end) = input[start..].find("\")")
            {
                let role = &input[start + 9..start + end];
                expressions.push(SecurityExpression::HasRole(role.to_string()));
            }
        }

        // Try hasAuthority with single quotes first, then double quotes
        if input.contains("hasAuthority(")
        {
            if let Some(start) = input.find("hasAuthority('")
            {
                if let Some(end) = input[start..].find("')")
                {
                    let auth = &input[start + 14..start + end];
                    expressions.push(SecurityExpression::HasAuthority(auth.to_string()));
                }
            }
            else if let Some(start) = input.find("hasAuthority(\"")
                && let Some(end) = input[start..].find("\")")
            {
                let auth = &input[start + 14..start + end];
                expressions.push(SecurityExpression::HasAuthority(auth.to_string()));
            }
        }

        if input.contains("isAuthenticated()")
        {
            expressions.push(SecurityExpression::IsAuthenticated);
        }

        if input.contains("isAnonymous()")
        {
            expressions.push(SecurityExpression::IsAnonymous);
        }

        if expressions.is_empty()
        {
            expressions.push(SecurityExpression::Custom(input.to_string()));
        }

        expressions
    }
}

/// `PreAuthorize` trait
/// `PreAuthorize` trait
///
/// Equivalent to Spring's @`PreAuthorize` annotation.
/// 等价于Spring的@PreAuthorize注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PreAuthorize("hasRole('ADMIN')")
/// public void deleteUser(Long id) { }
///
/// @PreAuthorize("hasAuthority('USER:WRITE') and #userId == authentication.principal.id")
/// public void updateUserProfile(Long userId, Profile profile) { }
/// ```
pub trait PreAuthorize
{
    /// Check if access should be granted
    /// 检查是否应授予访问
    fn check_authorization(
        &self,
        context: &SecurityContext,
    ) -> Pin<Box<dyn Future<Output = bool> + Send>>;
}

/// `PreAuthorize` options
/// `PreAuthorize选项`
#[derive(Debug, Clone)]
pub struct PreAuthorizeOptions
{
    /// Security expressions
    /// 安全表达式
    pub expressions: Vec<SecurityExpression>,

    /// All expressions must pass (AND logic)
    /// 所有表达式必须通过（AND逻辑）
    pub require_all: bool,
}

impl PreAuthorizeOptions
{
    /// Create new options
    /// 创建新选项
    pub fn new() -> Self
    {
        Self {
            expressions: Vec::new(),
            require_all: true,
        }
    }

    /// Add expression
    /// 添加表达式
    pub fn add_expression(mut self, expr: SecurityExpression) -> Self
    {
        self.expressions.push(expr);
        self
    }

    /// Parse and add expression string
    /// 解析并添加表达式字符串
    pub fn add_expression_string(mut self, expr: impl Into<String>) -> Self
    {
        let parsed = SecurityExpression::parse(&expr.into());
        self.expressions.extend(parsed);
        self
    }

    /// Set require all (AND) or require any (OR)
    /// 设置需要全部（AND）或需要任一（OR）
    pub fn require_all(mut self, require_all: bool) -> Self
    {
        self.require_all = require_all;
        self
    }

    /// Evaluate all expressions
    /// 评估所有表达式
    pub async fn evaluate(&self, context: &SecurityContext) -> bool
    {
        if self.expressions.is_empty()
        {
            return true;
        }

        if self.require_all
        {
            // All must pass
            for expr in &self.expressions
            {
                if !expr.evaluate(context).await
                {
                    return false;
                }
            }
            true
        }
        else
        {
            // Any can pass
            for expr in &self.expressions
            {
                if expr.evaluate(context).await
                {
                    return true;
                }
            }
            false
        }
    }
}

impl Default for PreAuthorizeOptions
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Helper function to check pre-authorize
/// 检查pre-authorize的助手函数
pub async fn check_pre_authorize(
    context: &SecurityContext,
    expression: &str,
) -> Result<bool, crate::SecurityError>
{
    let options = PreAuthorizeOptions::new().add_expression_string(expression);
    Ok(options.evaluate(context).await)
}

/// Common security expressions
/// 常用安全表达式
pub struct Expressions;

impl Expressions
{
    /// Has role expression
    /// 有角色表达式
    pub fn has_role(role: impl Into<String>) -> SecurityExpression
    {
        SecurityExpression::HasRole(role.into())
    }

    /// Has authority expression
    /// 有权限表达式
    pub fn has_authority(auth: impl Into<String>) -> SecurityExpression
    {
        SecurityExpression::HasAuthority(auth.into())
    }

    /// Is authenticated expression
    /// 已认证表达式
    pub fn is_authenticated() -> SecurityExpression
    {
        SecurityExpression::IsAuthenticated
    }

    /// Is anonymous expression
    /// 是匿名表达式
    pub fn is_anonymous() -> SecurityExpression
    {
        SecurityExpression::IsAnonymous
    }

    /// Has permission expression
    /// 有许可表达式
    pub fn has_permission(
        target: impl Into<String>,
        permission: impl Into<String>,
    ) -> SecurityExpression
    {
        SecurityExpression::HasPermission(target.into(), permission.into())
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_security_expression_parse()
    {
        let exprs = SecurityExpression::parse("hasRole('ADMIN')");
        assert_eq!(exprs.len(), 1);
        match &exprs[0]
        {
            SecurityExpression::HasRole(role) => assert_eq!(role, "ADMIN"),
            _ => panic!("Expected HasRole"),
        }
    }

    #[tokio::test]
    async fn test_pre_authorize_options()
    {
        let context = SecurityContext::new();

        let options = PreAuthorizeOptions::new()
            .add_expression(SecurityExpression::IsAuthenticated)
            .add_expression_string("hasRole('ADMIN')");

        // Should return false because context is empty (no auth)
        assert!(!options.evaluate(&context).await);
    }
}
