//! State extractor module
//! 状态提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `State<T>` - `@Autowired` / Application state / Singleton beans
//!
//! # Note / 注意
//!
//! State is typically managed through the Router's `with_state()` method
//! and `Stateful` handlers, not through this extractor. This extractor
//! is provided for compatibility with frameworks that use request extensions.
//!
//! 状态通常通过Router的`with_state()`方法和`Stateful`处理器管理，
//! 而不是通过此提取器。此提取器提供用于使用请求扩展的框架的兼容性。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::State;
//! use std::sync::Arc;
//!
//! struct Database {
//!     connection_string: String,
//! }
//!
//! // GET /users
//! async fn get_users(State(db): State<Arc<Database>>) -> String {
//!     format!("Database: {}", db.connection_string)
//! }
//! ```

use std::sync::Arc;

/// Application state extractor
/// 应用状态提取器
///
/// Equivalent to Spring's:
/// - `@Autowired` for dependency injection
/// - Application-scoped beans
/// - Singleton beans
///
/// 等价于Spring的：
/// - `@Autowired`用于依赖注入
/// - 应用作用域bean
/// - 单例bean
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type of state to extract. Must be present in the application state.
///
/// # Note / 注意
///
/// For stateful handlers, prefer using `Stateful<T, S>` with the Router's
/// `with_state()` method instead of this extractor.
///
/// 对于有状态的处理程序，请优先使用Router的`with_state()`方法和
/// `Stateful<T, S>`而不是此提取器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use std::sync::Arc;
///
/// struct Database {
///     connection_string: String,
/// }
///
/// async fn handler(State(db): State<Arc<Database>>) -> String {
///     format!("Database: {}", db.connection_string)
/// }
/// ```
pub struct State<T>(pub Arc<T>);

impl<T> State<T> {
    /// Consume the state extractor and get the inner Arc
    /// 消耗状态提取器并获取内部Arc
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        self.0.as_ref()
    }

    /// Clone the Arc
    /// 克隆Arc
    pub fn clone(&self) -> Arc<T> {
        Arc::clone(&self.0)
    }
}

impl<T> std::fmt::Debug for State<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("State").field(&self.0).finish()
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

// Note: State extraction through request extensions is not currently supported
// with hiver_http::Request. Use Router's `with_state()` and `Stateful` handlers instead.
// 状态通过请求扩展提取目前不支持hiver_http::Request。
// 请改用Router的`with_state()`和`Stateful`处理程序。

/// Extension trait for adding state to requests
/// 向请求添加状态的扩展trait
///
/// Note: This is not currently supported with `hiver_http::Request`.
/// Use Router's `with_state()` method instead.
/// `注意：目前不支持hiver_http::Request。请改用Router的``with_state()`方法。
pub trait AddState {
    /// Add state to the request
    /// 向请求添加状态
    fn add_state<T>(&mut self, state: Arc<T>)
    where
        T: Send + Sync + 'static;
}

// Note: The implementation is currently disabled because hiver_http::Request
// doesn't support extensions. Use Router's stateful handlers instead.
// 注意：实现当前被禁用，因为hiver_http::Request不支持扩展。
// 请改用Router的有状态处理程序。

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_clone() {
        let value = Arc::new("test".to_string());
        let state: State<String> = State(value.clone());
        // Explicitly call Clone trait to avoid Arc::clone
        let cloned: State<String> = Clone::clone(&state);
        // Use into_inner to get the Arc
        let inner: Arc<String> = cloned.into_inner();
        assert_eq!(*inner, "test");
    }
}
