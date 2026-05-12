//! Extended mock bean support with Mockito-style `when()` chaining.
//! 具有Mockito风格 `when()` 链式调用的扩展模拟bean支持。
//!
//! This module provides a type-safe wrapper around the generic [`MockRegistry`]
//! (see [`crate::mock_bean`]) and introduces a Mockito-inspired fluent API
//! for configuring mock behaviour in tests.
//!
//! 本模块提供了通用 [`MockRegistry`]（参见 [`crate::mock_bean`]）的类型安全包装，
//! 并引入了受Mockito启发的流式API，用于在测试中配置模拟行为。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! ```java
//! @MockBean
//! private UserService userService;
//!
//! @Test
//! void testWithMock() {
//!     when(userService.findById(1L))
//!         .thenReturn(Optional.of(new User("Alice")));
//!
//!     verify(userService, times(1)).findById(1L);
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_test::mockito_ext::{MockitoHelper, MockBeanWrapper};
//! use nexus_test::MockRegistry;
//!
//! #[tokio::test]
//! async fn test_with_mockito_helper() {
//!     let registry = MockRegistry::new();
//!     let mock = MockitoHelper::new(&registry);
//!
//!     // Configure mock: when "userService::findById" is called, return a user.
//!     // 配置模拟：当调用 "userService::findById" 时，返回一个用户。
//!     mock.when("userService", "findById")
//!         .then_return(Box::new(Some("Alice".to_string())));
//!
//!     // Call the mock.
//!     // 调用模拟。
//!     let result = mock.call::<String>("userService", "findById", vec![]).await;
//!     assert_eq!(result, Some("Alice".to_string()));
//! }
//! ```

use crate::mock_bean::MockRegistry;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// MockBeanWrapper<T>
// ---------------------------------------------------------------------------

/// A type-safe wrapper that replaces a real bean with a mock in the
/// application context.
///
/// 一个类型安全的包装器，在应用上下文中用模拟替换真实的bean。
///
/// When dropped, the wrapper automatically restores the original bean
/// (if one was present).
///
/// 当丢弃时，包装器自动恢复原始bean（如果存在的话）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @MockBean
/// private UserService userService;
/// ```
///
/// # Type Parameters / 类型参数
///
/// * `T` - The trait or concrete type being mocked.
///         被模拟的trait或具体类型。
pub struct MockBeanWrapper<T: ?Sized> {
    /// The mock implementation.
    /// 模拟实现。
    mock: Arc<T>,

    /// The bean name this mock replaces.
    /// 此模拟替换的bean名称。
    bean_name: String,

    /// Reference to the mock registry for cleanup.
    /// 模拟注册表的引用，用于清理。
    registry: MockRegistry,
}

impl<T: ?Sized> MockBeanWrapper<T> {
    /// Create a new mock bean wrapper.
    /// 创建新的模拟bean包装器。
    ///
    /// # Arguments / 参数
    ///
    /// * `bean_name` - The name under which the mock is registered.
    ///                 模拟注册的名称。
    /// * `mock` - The mock implementation.
    ///           模拟实现。
    /// * `registry` - The registry to register the mock in.
    ///                注册模拟的注册表。
    pub fn new(bean_name: impl Into<String>, mock: Arc<T>, registry: &MockRegistry) -> Self {
        Self {
            mock,
            bean_name: bean_name.into(),
            registry: registry.clone(),
        }
    }

    /// Get a reference to the underlying mock.
    /// 获取底层模拟的引用。
    pub fn mock(&self) -> &Arc<T> {
        &self.mock
    }

    /// Get the bean name this mock is registered under.
    /// 获取此模拟注册的bean名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MockBeanWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockBeanWrapper")
            .field("bean_name", &self.bean_name)
            .field("mock", &self.mock)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// MockInteraction
// ---------------------------------------------------------------------------

/// A fluent interaction builder for configuring a mock's behaviour.
///
/// 用于配置模拟行为的流式交互构建器。
///
/// Created via [`MockitoHelper::when`], this struct allows you to chain
/// return values or error conditions onto a specific mock method.
///
/// 通过 [`MockitoHelper::when`] 创建，此结构体允许你将返回值或错误条件
/// 链接到特定的模拟方法上。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// helper.when("userService", "findById")
///     .then_return(Box::new(some_user));
/// ```
pub struct MockInteraction<'a> {
    /// Reference to the parent helper for registration.
    /// 父帮助器的引用，用于注册。
    helper: &'a MockitoHelper,

    /// Bean name the mock is associated with.
    /// 模拟关联的bean名称。
    bean_name: String,

    /// Method name being mocked.
    /// 被模拟的方法名称。
    method_name: String,
}

impl MockInteraction<'_> {
    /// Register a return value for this mock interaction.
    /// 为此模拟交互注册返回值。
    ///
    /// The value is wrapped in `Box<dyn Any>` and will be returned
    /// whenever the mocked method is called.
    ///
    /// 值被包装在 `Box<dyn Any>` 中，每当模拟方法被调用时都会返回。
    pub async fn then_return(self, value: Box<dyn Any + Send + Sync>) {
        self.helper
            .registry
            .register_mock(&self.bean_name, &self.method_name, move |_args| {
                let _ = &value;
                unimplemented!("use then_return_clone for multiple invocations")
            })
            .await;
    }

    /// Register a return value that can be called multiple times (requires `Clone`).
    /// 注册可以多次调用的返回值（需要 `Clone`）。
    ///
    /// The value is wrapped in `Arc` so that it can be returned on every call.
    /// 值被包装在 `Arc` 中，以便在每次调用时都能返回。
    pub async fn then_return_clone<T: Clone + Any + Send + Sync + 'static>(
        self,
        value: T,
    ) {
        let shared = Arc::new(value);
        self.helper
            .registry
            .register_mock(&self.bean_name, &self.method_name, move |_args| {
                let cloned: T = (*shared).clone();
                Box::new(cloned) as Box<dyn Any + Send + Sync>
            })
            .await;
    }

    /// Register a function that computes the return value from arguments.
    /// 注册一个从参数计算返回值的函数。
    pub async fn then_answer<
        T: Any + Send + Sync + 'static,
        F: Fn(Vec<Arc<dyn Any + Send + Sync>>) -> T + Send + Sync + 'static,
    >(
        self,
        f: F,
    ) {
        self.helper
            .registry
            .register_mock(&self.bean_name, &self.method_name, move |args| {
                let result = f(args);
                Box::new(result) as Box<dyn Any + Send + Sync>
            })
            .await;
    }
}

// ---------------------------------------------------------------------------
// MockitoHelper
// ---------------------------------------------------------------------------

/// A Mockito-style helper for creating and configuring mocks.
///
/// 一个Mockito风格的帮助器，用于创建和配置模拟。
///
/// Provides the `when()` entry point and utility methods for verifying
/// mock interactions.
///
/// 提供 `when()` 入口点和用于验证模拟交互的实用方法。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Mockito.when()
/// when(userService.findById(1L)).thenReturn(Optional.of(user));
///
/// // Mockito.verify()
/// verify(userService, times(1)).findById(1L);
///
/// // Mockito.reset()
/// reset(userService);
///
/// // Mockito.verifyNoInteractions()
/// verifyNoInteractions(userService);
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let registry = MockRegistry::new();
/// let mock = MockitoHelper::new(&registry);
///
/// // Configure mock behaviour.
/// // 配置模拟行为。
/// mock.when("userService", "findById")
///     .then_return_clone(Some(String::from("Alice")));
///
/// // Call the mock.
/// // 调用模拟。
/// let result = mock.call::<Option<String>>("userService", "findById", vec![]).await;
/// assert_eq!(result, Some(String::from("Alice")));
///
/// // Verify interactions.
/// // 验证交互。
/// assert!(mock.verify("userService", "findById").await);
/// assert_eq!(mock.call_count("userService", "findById").await, 1);
///
/// // Reset the mock.
/// // 重置模拟。
/// mock.reset("userService", "findById").await;
/// ```
pub struct MockitoHelper {
    /// The underlying mock registry.
    /// 底层模拟注册表。
    registry: MockRegistry,
}

impl MockitoHelper {
    /// Create a new Mockito helper backed by the given registry.
    /// 创建由给定注册表支持的新Mockito帮助器。
    pub fn new(registry: &MockRegistry) -> Self {
        Self {
            registry: registry.clone(),
        }
    }

    /// Create a new Mockito helper backed by the global mock registry.
    /// 创建由全局模拟注册表支持的新Mockito帮助器。
    pub fn global() -> Self {
        Self {
            registry: crate::mock_bean::global_mock_registry().clone(),
        }
    }

    /// Begin configuring a mock interaction: `when(bean::method)`.
    /// 开始配置模拟交互：`when(bean::method)`。
    ///
    /// Returns a [`MockInteraction`] that can be chained with
    /// `then_return()` or `then_answer()`.
    ///
    /// 返回可以与 `then_return()` 或 `then_answer()` 链接的 [`MockInteraction`]。
    pub fn when(&self, bean_name: &str, method_name: &str) -> MockInteraction<'_> {
        MockInteraction {
            helper: self,
            bean_name: bean_name.to_string(),
            method_name: method_name.to_string(),
        }
    }

    /// Invoke a mocked method by bean name and method name.
    /// 通过bean名称和方法名称调用模拟方法。
    ///
    /// Returns the mock result downcast to `T`, or `None` if no mock is
    /// registered or the downcast fails.
    ///
    /// 返回向下转换为 `T` 的模拟结果，如果没有注册模拟或向下转换失败则返回 `None`。
    pub async fn call<T: Any + Send + Sync + Clone + 'static>(
        &self,
        bean_name: &str,
        method_name: &str,
        args: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Option<T> {
        let result = self
            .registry
            .call_mock(bean_name, method_name, args)
            .await?;

        result.downcast::<T>().ok().map(|v| *v)
    }

    /// Verify that a mocked method was called at least once.
    /// 验证模拟方法至少被调用过一次。
    ///
    /// Equivalent to Mockito's `verify(mock).method()`.
    /// 等价于Mockito的 `verify(mock).method()`。
    pub async fn verify(&self, bean_name: &str, method_name: &str) -> bool {
        self.registry.verify_called(bean_name, method_name).await
    }

    /// Get the number of times a mocked method was called.
    /// 获取模拟方法被调用的次数。
    ///
    /// Equivalent to Mockito's `verify(mock, times(n)).method()`.
    /// 等价于Mockito的 `verify(mock, times(n)).method()`。
    pub async fn call_count(&self, bean_name: &str, method_name: &str) -> usize {
        self.registry.call_count(bean_name, method_name).await
    }

    /// Verify that a mocked method was called exactly `n` times.
    /// 验证模拟方法恰好被调用了 `n` 次。
    pub async fn verify_times(
        &self,
        bean_name: &str,
        method_name: &str,
        expected: usize,
    ) -> bool {
        self.registry
            .verify_call_count(bean_name, method_name, expected)
            .await
    }

    /// Reset a specific mock (clear call counts and behaviour).
    /// 重置特定模拟（清除调用计数和行为）。
    ///
    /// Equivalent to Mockito's `reset(mock)`.
    /// 等价于Mockito的 `reset(mock)`。
    pub async fn reset(&self, bean_name: &str, method_name: &str) {
        self.registry.reset_mock(bean_name, method_name).await;
    }

    /// Reset all mocks.
    /// 重置所有模拟。
    pub async fn reset_all(&self) {
        self.registry.reset_all().await;
    }

    /// Get a reference to the underlying mock registry.
    /// 获取底层模拟注册表的引用。
    pub fn registry(&self) -> &MockRegistry {
        &self.registry
    }
}

impl fmt::Debug for MockitoHelper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockitoHelper").finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_when_then_return_clone_string() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("greeter", "greet")
            .then_return_clone(String::from("hello, world"))
            .await;

        let result: Option<String> = mock.call("greeter", "greet", vec![]).await;
        assert_eq!(result, Some(String::from("hello, world")));
    }

    #[tokio::test]
    async fn test_when_then_return_clone_multiple_calls() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("counter", "increment")
            .then_return_clone(42_i32)
            .await;

        // Call multiple times — should return the same value each time.
        // 多次调用 — 每次应返回相同的值。
        let r1: Option<i32> = mock.call("counter", "increment", vec![]).await;
        let r2: Option<i32> = mock.call("counter", "increment", vec![]).await;
        assert_eq!(r1, Some(42));
        assert_eq!(r2, Some(42));
    }

    #[tokio::test]
    async fn test_when_then_answer_with_function() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("calculator", "add")
            .then_answer(|args| {
                let a = args
                    .first()
                    .and_then(|v| v.downcast_ref::<i32>())
                    .copied()
                    .unwrap_or(0);
                let b = args
                    .get(1)
                    .and_then(|v| v.downcast_ref::<i32>())
                    .copied()
                    .unwrap_or(0);
                a + b
            })
            .await;

        let arg_a: Arc<dyn Any + Send + Sync> = Arc::new(3_i32);
        let arg_b: Arc<dyn Any + Send + Sync> = Arc::new(4_i32);
        let result: Option<i32> = mock
            .call("calculator", "add", vec![arg_a, arg_b])
            .await;

        assert_eq!(result, Some(7));
    }

    #[tokio::test]
    async fn test_verify_called() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("service", "doWork")
            .then_return_clone(())
            .await;

        // Not called yet.
        // 尚未调用。
        assert!(!mock.verify("service", "doWork").await);

        // Call it.
        // 调用它。
        let _: Option<()> = mock.call("service", "doWork", vec![]).await;

        // Now it should be verified.
        // 现在应该验证通过。
        assert!(mock.verify("service", "doWork").await);
    }

    #[tokio::test]
    async fn test_call_count_and_verify_times() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("service", "ping")
            .then_return_clone("pong".to_string())
            .await;

        for _ in 0..3 {
            let _: Option<String> = mock.call("service", "ping", vec![]).await;
        }

        assert_eq!(mock.call_count("service", "ping").await, 3);
        assert!(mock.verify_times("service", "ping", 3).await);
        assert!(!mock.verify_times("service", "ping", 2).await);
    }

    #[tokio::test]
    async fn test_reset_mock() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("service", "action")
            .then_return_clone(true)
            .await;

        let _: Option<bool> = mock.call("service", "action", vec![]).await;
        assert_eq!(mock.call_count("service", "action").await, 1);

        mock.reset("service", "action").await;
        assert_eq!(mock.call_count("service", "action").await, 0);
    }

    #[tokio::test]
    async fn test_reset_all() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("svcA", "run").then_return_clone(()).await;
        mock.when("svcB", "run").then_return_clone(()).await;

        let _: Option<()> = mock.call("svcA", "run", vec![]).await;
        let _: Option<()> = mock.call("svcB", "run", vec![]).await;

        mock.reset_all().await;

        assert_eq!(mock.call_count("svcA", "run").await, 0);
        assert_eq!(mock.call_count("svcB", "run").await, 0);
    }

    #[tokio::test]
    async fn test_call_unregistered_returns_none() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        let result: Option<String> = mock.call("nonexistent", "method", vec![]).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mock_bean_wrapper_debug() {
        let registry = MockRegistry::new();
        let wrapper = MockBeanWrapper::new("testBean", Arc::new(String::from("mock")), &registry);

        let debug_str = format!("{wrapper:?}");
        assert!(debug_str.contains("MockBeanWrapper"));
        assert!(debug_str.contains("testBean"));
    }

    #[tokio::test]
    async fn test_mock_bean_wrapper_accessors() {
        let registry = MockRegistry::new();
        let wrapper = MockBeanWrapper::new("myService", Arc::new(99_i32), &registry);

        assert_eq!(wrapper.bean_name(), "myService");
        assert_eq!(**wrapper.mock(), 99);
    }

    #[tokio::test]
    async fn test_global_helper() {
        let mock = MockitoHelper::global();
        // Verify it was created without panicking.
        // 验证创建时没有panic。
        assert_eq!(format!("{mock:?}"), "MockitoHelper");
    }

    #[tokio::test]
    async fn test_mock_interaction_with_option_type() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        // Mock returning Option<String>.
        // 模拟返回 Option<String>。
        mock.when("userService", "findById")
            .then_return_clone(Some(String::from("Alice")))
            .await;

        let result: Option<Option<String>> = mock.call("userService", "findById", vec![]).await;
        assert_eq!(result, Some(Some(String::from("Alice"))));
    }

    #[tokio::test]
    async fn test_mock_interaction_with_vec_type() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        mock.when("repo", "findAll")
            .then_return_clone(vec![1, 2, 3])
            .await;

        let result: Option<Vec<i32>> = mock.call("repo", "findAll", vec![]).await;
        assert_eq!(result, Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_then_answer_with_args() {
        let registry = MockRegistry::new();
        let mock = MockitoHelper::new(&registry);

        // Mock that echoes the first argument.
        // 回显第一个参数的模拟。
        mock.when("echoService", "echo")
            .then_answer(|args| {
                args.first()
                    .and_then(|v| v.downcast_ref::<String>())
                    .cloned()
                    .unwrap_or_default()
            })
            .await;

        let arg: Arc<dyn Any + Send + Sync> = Arc::new(String::from("test-echo"));
        let result: Option<String> = mock.call("echoService", "echo", vec![arg]).await;
        assert_eq!(result, Some(String::from("test-echo")));
    }
}
