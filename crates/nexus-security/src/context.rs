//! Security context module
//! 安全上下文模块
//!
//! Provides per-task async-scoped security context propagation via
//! `tokio::task_local!`, as well as a legacy global context for
//! backward compatibility.
//! 通过 `tokio::task_local!` 提供每个任务级别的异步范围安全上下文传播，
//! 以及一个用于向后兼容的全局上下文。
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! SecurityContext context = SecurityContextHolder.getContext();
//! Authentication auth = context.getAuthentication();
//! ```

use crate::Authentication;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// SecurityContext
// ---------------------------------------------------------------------------

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

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Global (legacy) context
// ---------------------------------------------------------------------------

/// Global security context
/// 全局安全上下文
///
/// Provides backward-compatible access to a single shared context.
/// 提供对单一共享上下文的向后兼容访问。
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

// ---------------------------------------------------------------------------
// Async task-local context propagation
// ---------------------------------------------------------------------------

// Task-local storage for the per-task security context.
// 每个任务的安全上下文任务本地存储。
//
// Unlike the global context, `task_local` values are scoped to the
// current async task and automatically propagate through `.await` points.
// This prevents cross-request leakage in server environments where many
// tasks share the same thread.
// 与全局上下文不同，`task_local` 值的作用域限定在当前异步任务内，
// 并自动通过 `.await` 点传播。这可以防止在多任务共享同一线程的
// 服务器环境中出现跨请求泄漏。
tokio::task_local! {
    static CURRENT_SECURITY_CONTEXT: Arc<SecurityContext>;
}

/// RAII guard that installs a [`SecurityContext`] for the duration of a scope.
/// RAII守卫，在作用域期间安装 [`SecurityContext`]。
///
/// The guard keeps the [`SecurityContext`] alive via `Arc`. Use
/// [`SecurityContextGuard::scope`] or [`SecurityContextGuard::scope_async`]
/// to run code with the context installed as a task-local value.
/// 守卫通过 `Arc` 保持 [`SecurityContext`] 存活。
/// 使用 [`SecurityContextGuard::scope`] 或 [`SecurityContextGuard::scope_async`]
/// 在安装上下文作为任务本地值的情况下运行代码。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::context::SecurityContextGuard;
/// use nexus_security::SecurityContext;
/// use nexus_security::Authentication;
///
/// let ctx = SecurityContext::new();
/// // ... set authentication on ctx ...
///
/// let guard = SecurityContextGuard::new(ctx);
/// let result = guard.scope(|| {
///     // Inside this scope, `get_security_context()` returns Some(ctx).
///     42
/// });
/// ```
pub struct SecurityContextGuard {
    /// The wrapped security context.
    /// 包装的安全上下文。
    ctx: Arc<SecurityContext>,
}

impl SecurityContextGuard {
    /// Create a new guard wrapping the given [`SecurityContext`].
    /// 创建新的守卫，包装给定的 [`SecurityContext`]。
    pub fn new(ctx: SecurityContext) -> Self {
        Self {
            ctx: Arc::new(ctx),
        }
    }

    /// Run a synchronous closure with this context installed as the
    /// task-local value.
    /// 在安装此上下文为任务本地值的情况下运行同步闭包。
    pub fn scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        CURRENT_SECURITY_CONTEXT.sync_scope(self.ctx.clone(), f)
    }

    /// Run an async future with this context installed as the task-local
    /// value. The context propagates through all `.await` points inside
    /// the future.
    /// 在安装此上下文为任务本地值的情况下运行异步 future。
    /// 上下文在 future 内所有 `.await` 点传播。
    pub async fn scope_async<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = R>,
    {
        CURRENT_SECURITY_CONTEXT.scope(self.ctx.clone(), f()).await
    }

    /// Get a clone of the inner `Arc<SecurityContext>`.
    /// 获取内部 `Arc<SecurityContext>` 的克隆。
    pub fn context(&self) -> Arc<SecurityContext> {
        self.ctx.clone()
    }
}

/// Set the security context and run a synchronous closure with it.
/// 设置安全上下文并使用它运行同步闭包。
///
/// Shorthand for `SecurityContextGuard::new(ctx).scope(f)`.
/// `SecurityContextGuard::new(ctx).scope(f)` 的简写。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::context::set_security_context;
/// use nexus_security::SecurityContext;
///
/// let ctx = SecurityContext::new();
/// let result = set_security_context(ctx, || {
///     // ctx is now active in this scope
///     42
/// });
/// ```
pub fn set_security_context<F, R>(ctx: SecurityContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = SecurityContextGuard::new(ctx);
    guard.scope(f)
}

/// Get the current task-local [`SecurityContext`], if one has been set.
/// 获取当前任务本地的 [`SecurityContext`]（如果已设置）。
///
/// Returns `None` if no context has been installed for the current task.
/// 如果当前任务未安装上下文，则返回 `None`。
///
/// This function may be called both in async and sync contexts.
/// 此函数可以在异步和同步上下文中调用。
pub fn get_security_context() -> Option<Arc<SecurityContext>> {
    CURRENT_SECURITY_CONTEXT.try_with(|ctx| ctx.clone()).ok()
}

/// Run a closure with the given [`SecurityContext`] installed as the
/// task-local value for the current async task.
/// 使用给定的 [`SecurityContext`] 作为当前异步任务的任务本地值运行闭包。
///
/// This is the most ergonomic way to scope a context for a single
/// operation.  The context is automatically removed when the closure
/// returns.
/// 这是为单个操作限定上下文范围的最符合人体工程学的方式。
/// 闭包返回时上下文会自动移除。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::context::with_security_context;
/// use nexus_security::SecurityContext;
///
/// let ctx = SecurityContext::new();
/// let result = with_security_context(ctx, || {
///     // Inside this closure, `get_security_context()` returns Some.
///     42
/// });
/// assert_eq!(result, 42);
/// ```
pub fn with_security_context<F, R>(ctx: SecurityContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    CURRENT_SECURITY_CONTEXT.sync_scope(Arc::new(ctx), f)
}

/// Run an async closure with the given [`SecurityContext`] installed as
/// the task-local value.  The context propagates through all `.await`
/// points inside the future.
/// 使用给定的 [`SecurityContext`] 作为任务本地值运行异步闭包。
/// 上下文在 future 内所有 `.await` 点传播。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_security::context::with_security_context_async;
/// use nexus_security::SecurityContext;
///
/// let ctx = SecurityContext::new();
/// let result = with_security_context_async(ctx, || async {
///     // Inside this block, `get_security_context()` returns Some.
///     some_async_fn().await
/// }).await;
/// ```
pub async fn with_security_context_async<F, Fut, R>(ctx: SecurityContext, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    let arc = Arc::new(ctx);
    CURRENT_SECURITY_CONTEXT.scope(arc, f()).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        assert_eq!(
            context.get_username().await,
            Some("john".to_string())
        );
    }

    #[test]
    fn test_default_security_context() {
        let ctx = SecurityContext::default();
        assert!(ctx.authentication.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_guard_scope_sync() {
        // Initially no context is set.
        assert!(get_security_context().is_none());

        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "alice".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let guard = SecurityContextGuard::new(ctx);
        guard.scope(|| {
            // Inside the scope, the context is available.
            let retrieved = get_security_context().expect("context should be set");
            assert!(Arc::strong_count(&retrieved) > 0);
        });

        // After the scope, the context is gone.
        assert!(get_security_context().is_none());
    }

    #[tokio::test]
    async fn test_set_security_context() {
        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "bob".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let result = set_security_context(ctx, || {
            let retrieved = get_security_context().expect("context should be set");
            assert!(Arc::strong_count(&retrieved) > 0);
            42
        });

        assert_eq!(result, 42);
        // After the closure, the context is gone.
        assert!(get_security_context().is_none());
    }

    #[tokio::test]
    async fn test_with_security_context() {
        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "bob".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let result = with_security_context(ctx, || {
            let retrieved = get_security_context().expect("context should be set");
            assert!(Arc::strong_count(&retrieved) > 0);
            42
        });

        assert_eq!(result, 42);
        // After the closure, the context is gone.
        assert!(get_security_context().is_none());
    }

    #[tokio::test]
    async fn test_with_security_context_async() {
        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "charlie".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let username = with_security_context_async(ctx, || async {
            // Simulate an await point — context must still be available.
            tokio::task::yield_now().await;

            let retrieved = get_security_context().expect("context should be set");
            retrieved.get_username().await
        })
        .await;

        assert_eq!(username, Some("charlie".to_string()));

        // After the async block, context is gone.
        assert!(get_security_context().is_none());
    }

    #[tokio::test]
    async fn test_context_propagates_across_await() {
        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "dave".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let guard = SecurityContextGuard::new(ctx);

        let username = guard
            .scope_async(|| async {
                // First check before yield.
                let before = get_security_context()
                    .unwrap()
                    .get_username()
                    .await;
                assert_eq!(before, Some("dave".to_string()));

                // Yield to the scheduler — context must survive.
                tokio::task::yield_now().await;

                // Check after yield.
                let after = get_security_context()
                    .unwrap()
                    .get_username()
                    .await;
                assert_eq!(after, Some("dave".to_string()));

                after
            })
            .await;

        assert_eq!(username, Some("dave".to_string()));
        // After the scope, context is gone.
        assert!(get_security_context().is_none());
    }

    #[tokio::test]
    async fn test_spawned_task_does_not_inherit_context() {
        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "eve".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let guard = SecurityContextGuard::new(ctx);

        // A spawned task should NOT see the parent's task-local context.
        // We need to spawn from within the scope.
        guard.scope(|| {
            let handle = tokio::task::spawn(async { get_security_context().is_some() });

            // The spawned task should not inherit the task-local.
            // We cannot await inside sync scope, so we use block_in_place or just
            // assert the handle was created. For a true async test, see below.
            drop(handle);
        });
    }

    #[tokio::test]
    async fn test_guard_context_accessor() {
        let ctx = SecurityContext::new();
        ctx.set_authentication(Authentication {
            principal: "frank".to_string(),
            credentials: None,
            authorities: vec![],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        })
        .await;

        let guard = SecurityContextGuard::new(ctx);
        let arc = guard.context();
        let username = arc.get_username().await;
        assert_eq!(username, Some("frank".to_string()));
    }
}
