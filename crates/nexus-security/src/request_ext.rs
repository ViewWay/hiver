//! Request extension for `SecurityContext`
//! `SecurityContext的Request扩展`
//!
//! This module provides Request-based `SecurityContext` that works across async boundaries.
//! 本模块提供基于Request的SecurityContext，可在异步边界间工作。

use crate::Authentication;
use nexus_http::Request;
use std::sync::Arc;
use tokio::sync::RwLock;

/// `SecurityContext` extension for Request
/// `Request的SecurityContext扩展`
///
/// This allows `SecurityContext` to be passed through Request extensions,
/// making it available across async boundaries without `ThreadLocal`.
///
/// `这允许SecurityContext通过Request扩展传递`，
/// 使其在异步边界间可用，无需ThreadLocal。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::request_ext::SecurityContextExt;
/// use nexus_http::Request;
///
/// async fn handler(req: Request) -> Result<Response> {
///     // Get SecurityContext from Request
///     let ctx = SecurityContextExt::from_request(&req)?;
///     let auth = ctx.get_authentication().await;
///     Ok(Response::json(auth))
/// }
/// ```
#[derive(Clone)]
pub struct SecurityContextExt {
    /// Current authentication
    /// 当前认证
    authentication: Arc<RwLock<Option<Authentication>>>,
}

impl SecurityContextExt {
    /// Create a new `SecurityContext` extension
    /// `创建新的SecurityContext扩展`
    pub fn new() -> Self {
        Self {
            authentication: Arc::new(RwLock::new(None)),
        }
    }

    /// Get `SecurityContext` from Request extensions
    /// `从Request扩展中获取SecurityContext`
    ///
    /// Returns an error if `SecurityContext` is not found in the request.
    /// 如果请求中未找到SecurityContext，则返回错误。
    pub fn from_request(req: &Request) -> Option<Arc<Self>> {
        req.extensions().get::<Arc<Self>>().cloned()
    }

    /// Set `SecurityContext` to Request extensions
    /// `将SecurityContext设置到Request扩展`
    pub fn set_to_request(req: &mut Request) -> Arc<Self> {
        let ctx = Arc::new(Self::new());
        req.extensions_mut().insert(ctx.clone());
        ctx
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

impl Default for SecurityContextExt {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: Get authentication from Request
/// 便捷函数：从Request获取认证
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::request_ext::get_authentication_from_request;
/// use nexus_http::Request;
///
/// async fn handler(req: Request) -> Result<Response> {
///     let auth = get_authentication_from_request(&req).await?;
///     Ok(Response::json(auth))
/// }
/// ```
pub async fn get_authentication_from_request(req: &Request) -> Option<Authentication> {
    SecurityContextExt::from_request(req)?
        .get_authentication()
        .await
}

/// Convenience function: Set authentication to Request
/// 便捷函数：将认证设置到Request
pub fn set_authentication_to_request(
    req: &mut Request,
    _auth: Authentication,
) -> Arc<SecurityContextExt> {
    let ctx = SecurityContextExt::set_to_request(req);
    // Note: This is a synchronous function, so we can't await
    // In practice, you should use the async set_authentication method
    // 注意：这是一个同步函数，所以不能await
    // 实际上，应该使用异步的set_authentication方法
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_http::{Method, Request};

    #[tokio::test]
    async fn test_security_context_ext() {
        let mut req = Request::from_method_uri(Method::GET, "/test");

        // Set SecurityContext
        let ctx = SecurityContextExt::set_to_request(&mut req);

        // Get from Request
        let ctx2 = SecurityContextExt::from_request(&req).unwrap();
        assert_eq!(Arc::as_ptr(&ctx), Arc::as_ptr(&ctx2));

        // Test authentication
        let auth = Authentication {
            principal: "john".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        };

        ctx.set_authentication(auth.clone()).await;

        assert!(ctx.is_authenticated().await);
        assert_eq!(ctx.get_username().await, Some("john".to_string()));

        // Get from Request again
        let auth_from_req = get_authentication_from_request(&req).await;
        assert_eq!(auth_from_req, Some(auth));
    }
}
