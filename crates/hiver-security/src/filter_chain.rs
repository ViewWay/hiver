//! Security filter chain
//! 安全过滤器链
//!
//! Equivalent to Spring Security's `SecurityFilterChain`.
//! 等价于 Spring Security 的 `SecurityFilterChain`。
//!
//! Provides a composable chain of security filters that process requests
//! in order: authentication, authorization, CSRF, CORS, etc.
//!
//! 提供可组合的安全过滤器链，按顺序处理请求：
//! 认证、授权、CSRF、CORS 等。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::sync::Arc;

/// Security context carrying authentication state through the filter chain.
/// 安全上下文，在过滤器链中传递认证状态。
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// The authenticated principal (user identity).
    /// 已认证的主体（用户身份）。
    pub principal: Option<String>,

    /// Granted authorities / roles.
    /// 授予的权限/角色。
    pub authorities: Vec<String>,

    /// Whether the request is authenticated.
    /// 请求是否已认证。
    pub authenticated: bool,

    /// Additional attributes.
    /// 附加属性。
    pub attributes: std::collections::HashMap<String, String>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            principal: None,
            authorities: Vec::new(),
            authenticated: false,
            attributes: std::collections::HashMap::new(),
        }
    }
}

impl SecurityContext {
    /// Create a new empty security context.
    /// 创建新的空安全上下文。
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the principal.
    /// 设置主体。
    pub fn with_principal(mut self, principal: impl Into<String>) -> Self {
        self.principal = Some(principal.into());
        self
    }

    /// Set the authorities.
    /// 设置权限。
    pub fn with_authorities(mut self, authorities: Vec<String>) -> Self {
        self.authorities = authorities;
        self
    }

    /// Mark as authenticated.
    /// 标记为已认证。
    pub fn authenticated(mut self) -> Self {
        self.authenticated = true;
        self
    }

    /// Check if the context has a specific authority.
    /// 检查上下文是否拥有特定权限。
    pub fn has_authority(&self, authority: &str) -> bool {
        self.authorities.iter().any(|a| a == authority)
    }

    /// Check if the context has any of the given authorities.
    /// 检查上下文是否拥有给定权限中的任何一个。
    pub fn has_any_authority(&self, authorities: &[&str]) -> bool {
        authorities.iter().any(|a| self.has_authority(a))
    }
}

/// Result of a security filter processing.
/// 安全过滤器处理结果。
#[derive(Debug, Clone)]
pub enum FilterResult {
    /// Continue to the next filter.
    /// 继续下一个过滤器。
    Continue(SecurityContext),

    /// Stop the chain and deny the request.
    /// 停止链并拒绝请求。
    Deny {
        /// HTTP status code.
        status: u16,
        /// Reason message.
        reason: String,
    },
}

/// A single security filter in the chain.
/// 链中的单个安全过滤器。
///
/// Equivalent to Spring Security's `Filter` interface.
/// 等价于 Spring Security 的 `Filter` 接口。
pub trait SecurityFilter: Send + Sync {
    /// Process the security context.
    /// 处理安全上下文。
    fn do_filter(&self, context: SecurityContext, request_path: &str) -> FilterResult;

    /// Filter ordering (lower = earlier). Default is 100.
    /// 过滤器排序（越小越先执行）。默认 100。
    fn order(&self) -> i32 {
        100
    }

    /// Filter name for diagnostics.
    /// 过滤器名称，用于诊断。
    fn name(&self) -> &str;
}

/// A chain of security filters, processed in order.
/// 安全过滤器链，按顺序处理。
///
/// Equivalent to Spring Security's `SecurityFilterChain`.
/// 等价于 Spring Security 的 `SecurityFilterChain`。
pub struct SecurityFilterChain {
    filters: Vec<Arc<dyn SecurityFilter>>,
}

impl SecurityFilterChain {
    /// Create a new empty filter chain.
    /// 创建新的空过滤器链。
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Add a filter to the chain. Filters are sorted by `order()`.
    /// 添加过滤器到链中。过滤器按 `order()` 排序。
    pub fn add_filter<F: SecurityFilter + 'static>(mut self, filter: F) -> Self {
        self.filters.push(Arc::new(filter));
        self.filters.sort_by_key(|f| f.order());
        self
    }

    /// Execute the filter chain.
    /// 执行过滤器链。
    pub fn execute(&self, context: SecurityContext, request_path: &str) -> FilterResult {
        let mut ctx = context;
        for filter in &self.filters {
            match filter.do_filter(ctx, request_path) {
                FilterResult::Continue(new_ctx) => ctx = new_ctx,
                deny @ FilterResult::Deny { .. } => return deny,
            }
        }
        FilterResult::Continue(ctx)
    }

    /// Get the number of filters.
    /// 获取过滤器数量。
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if the chain is empty.
    /// 检查链是否为空。
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl Default for SecurityFilterChain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in filters / 内置过滤器
// ============================================================================

/// Filter that denies unauthenticated requests.
/// 拒绝未认证请求的过滤器。
pub struct AuthenticationFilter;

impl SecurityFilter for AuthenticationFilter {
    fn do_filter(&self, context: SecurityContext, _request_path: &str) -> FilterResult {
        if context.authenticated {
            FilterResult::Continue(context)
        } else {
            FilterResult::Deny {
                status: 401,
                reason: "Unauthorized: authentication required".to_string(),
            }
        }
    }

    fn order(&self) -> i32 {
        200
    }

    fn name(&self) -> &str {
        "AuthenticationFilter"
    }
}

/// Filter that checks for required roles/authorities.
/// 检查所需角色/权限的过滤器。
pub struct RoleAuthorizationFilter {
    required_roles: Vec<String>,
}

impl RoleAuthorizationFilter {
    /// Create a new filter requiring any of the given roles.
    /// 创建需要给定角色之一的新过滤器。
    pub fn new(roles: &[&str]) -> Self {
        Self {
            required_roles: roles.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl SecurityFilter for RoleAuthorizationFilter {
    fn do_filter(&self, context: SecurityContext, _request_path: &str) -> FilterResult {
        if self.required_roles.is_empty() {
            return FilterResult::Continue(context);
        }

        let has_role = self
            .required_roles
            .iter()
            .any(|role| context.has_authority(role));

        if has_role {
            FilterResult::Continue(context)
        } else {
            FilterResult::Deny {
                status: 403,
                reason: format!("Forbidden: requires one of roles {:?}", self.required_roles),
            }
        }
    }

    fn order(&self) -> i32 {
        300
    }

    fn name(&self) -> &str {
        "RoleAuthorizationFilter"
    }
}

/// Filter that allows path-based public access (skip authentication).
/// 允许基于路径的公开访问（跳过认证）的过滤器。
pub struct PublicPathFilter {
    public_paths: Vec<String>,
}

impl PublicPathFilter {
    /// Create a filter that marks certain paths as public.
    /// 创建标记某些路径为公开的过滤器。
    pub fn new(paths: &[&str]) -> Self {
        Self {
            public_paths: paths.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl SecurityFilter for PublicPathFilter {
    fn do_filter(&self, context: SecurityContext, request_path: &str) -> FilterResult {
        if self
            .public_paths
            .iter()
            .any(|p| request_path.starts_with(p.as_str()))
        {
            FilterResult::Continue(context.authenticated())
        } else {
            FilterResult::Continue(context)
        }
    }

    fn order(&self) -> i32 {
        50
    }

    fn name(&self) -> &str {
        "PublicPathFilter"
    }
}

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

    #[test]
    fn test_security_context_authorities() {
        let ctx = SecurityContext::new()
            .with_principal("admin")
            .with_authorities(vec!["ADMIN".into(), "USER".into()])
            .authenticated();

        assert!(ctx.has_authority("ADMIN"));
        assert!(ctx.has_any_authority(&["ADMIN", "SUPER"]));
        assert!(!ctx.has_authority("SUPER"));
    }

    #[test]
    fn test_filter_chain_allows_authenticated() {
        let chain = SecurityFilterChain::new().add_filter(AuthenticationFilter);

        let ctx = SecurityContext::new()
            .with_principal("user")
            .authenticated();
        let result = chain.execute(ctx, "/api/data");
        assert!(matches!(result, FilterResult::Continue(_)));
    }

    #[test]
    fn test_filter_chain_denies_unauthenticated() {
        let chain = SecurityFilterChain::new().add_filter(AuthenticationFilter);

        let result = chain.execute(SecurityContext::new(), "/api/data");
        assert!(matches!(result, FilterResult::Deny { status: 401, .. }));
    }

    #[test]
    fn test_role_filter_allows() {
        let chain = SecurityFilterChain::new()
            .add_filter(AuthenticationFilter)
            .add_filter(RoleAuthorizationFilter::new(&["ADMIN"]));

        let ctx = SecurityContext::new()
            .with_principal("admin")
            .with_authorities(vec!["ADMIN".into()])
            .authenticated();

        let result = chain.execute(ctx, "/api/admin");
        assert!(matches!(result, FilterResult::Continue(_)));
    }

    #[test]
    fn test_role_filter_denies() {
        let chain = SecurityFilterChain::new()
            .add_filter(AuthenticationFilter)
            .add_filter(RoleAuthorizationFilter::new(&["ADMIN"]));

        let ctx = SecurityContext::new()
            .with_principal("user")
            .with_authorities(vec!["USER".into()])
            .authenticated();

        let result = chain.execute(ctx, "/api/admin");
        assert!(matches!(result, FilterResult::Deny { status: 403, .. }));
    }

    #[test]
    fn test_public_path_filter() {
        let chain = SecurityFilterChain::new()
            .add_filter(PublicPathFilter::new(&["/public", "/health"]))
            .add_filter(AuthenticationFilter);

        let result = chain.execute(SecurityContext::new(), "/health");
        assert!(matches!(result, FilterResult::Continue(_)));
    }
}
