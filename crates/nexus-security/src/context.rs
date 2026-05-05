//! Security context module
//! 安全上下文模块

use crate::Authentication;
use std::sync::Arc;

/// Security context
/// 安全上下文
///
/// Holds the current authentication and security information.
/// 保存当前认证和安全信息。
///
/// Equivalent to Spring's `SecurityContext`.
/// `等价于Spring的SecurityContext`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// SecurityContext context = SecurityContextHolder.getContext();
/// Authentication auth = context.getAuthentication();
/// ```
pub struct SecurityContext {
    /// Current authentication
    /// 当前认证
    authentication: Arc<tokio::sync::RwLock<Option<Authentication>>>,
}

impl SecurityContext {
    /// Create a new security context
    /// 创建新的安全上下文
    pub fn new() -> Self {
        Self {
            authentication: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Get current authentication
    /// 获取当前认证
    pub async fn get_authentication(&self) -> Option<Authentication> {
        self.authentication.read().await.clone()
    }

    /// Set authentication
    /// 设置认证
    pub async fn set_authentication(&self, auth: Authentication) {
        let mut auth_guard = self.authentication.write().await;
        *auth_guard = Some(auth);
    }

    /// Clear authentication
    /// 清除认证
    pub async fn clear(&self) {
        let mut auth_guard = self.authentication.write().await;
        *auth_guard = None;
    }

    /// Check if authenticated
    /// 检查是否已认证
    pub async fn is_authenticated(&self) -> bool {
        self.authentication
            .read()
            .await
            .as_ref()
            .is_some_and(|a| a.authenticated)
    }

    /// Get current username
    /// 获取当前用户名
    pub async fn get_username(&self) -> Option<String> {
        self.authentication
            .read()
            .await
            .as_ref()
            .map(|a| a.principal.clone())
    }

    /// Check if user has authority
    /// 检查用户是否有权限
    pub async fn has_authority(&self, authority: &crate::Authority) -> bool {
        self.authentication
            .read()
            .await
            .as_ref()
            .is_some_and(|a| a.has_authority(authority))
    }

    /// Check if user has role
    /// 检查用户是否有角色
    pub async fn has_role(&self, role: &crate::Role) -> bool {
        self.authentication
            .read()
            .await
            .as_ref()
            .is_some_and(|a| a.has_role(role))
    }
}

/// Global security context
/// 全局安全上下文
static GLOBAL_CONTEXT: std::sync::LazyLock<SecurityContext> =
    std::sync::LazyLock::new(SecurityContext::new);

/// Get global security context
/// 获取全局安全上下文
pub fn context() -> &'static SecurityContext {
    &GLOBAL_CONTEXT
}

/// Get current authentication from global context
/// 从全局上下文获取当前认证
pub async fn get_authentication() -> Option<Authentication> {
    context().get_authentication().await
}

/// Set authentication in global context
/// 在全局上下文中设置认证
pub async fn set_authentication(auth: Authentication) {
    context().set_authentication(auth).await;
}

/// Clear global context
/// 清除全局上下文
pub async fn clear_context() {
    context().clear().await;
}

/// Check if current user is authenticated
/// 检查当前用户是否已认证
pub async fn is_authenticated() -> bool {
    context().is_authenticated().await
}

/// Get current username
/// 获取当前用户名
pub async fn get_username() -> Option<String> {
    context().get_username().await
}

/// Check if current user has authority
/// 检查当前用户是否有权限
pub async fn has_authority(authority: &crate::Authority) -> bool {
    context().has_authority(authority).await
}

/// Check if current user has role
/// 检查当前用户是否有角色
pub async fn has_role(role: &crate::Role) -> bool {
    context().has_role(role).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_context() {
        let context = SecurityContext::new();

        assert!(!context.is_authenticated().await);
        assert!(context.get_username().await.is_none());

        let auth = Authentication {
            principal: "john".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        };

        context.set_authentication(auth).await;

        assert!(context.is_authenticated().await);
        assert_eq!(context.get_username().await, Some("john".to_string()));
    }
}
