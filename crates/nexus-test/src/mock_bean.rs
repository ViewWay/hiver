//! Mock bean support for tests
//! 测试的模拟bean支持

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock bean marker trait
/// 模拟bean标记trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @MockBean
/// private UserService userService;
///
/// @Test
/// void testWithMock() {
///     when(userService.findById(1L)).thenReturn(Optional.of(user));
/// }
/// ```
pub trait MockBean: Any + Send + Sync {}

/// Mock registry for storing mock beans
/// 用于存储模拟bean的模拟注册表
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let registry = MockRegistry::new();
///
/// // Register a mock function
/// registry.register_mock("userService", "findById", |args: Vec<Arc<dyn Any>>| {
///     // Return mock user
///     Box::new(Some(user)) as Box<dyn Any>
/// });
///
/// // Use the mock
/// let result = registry.call_mock("userService", "findById", vec![user_id]).await;
/// ```
#[derive(Clone)]
pub struct MockRegistry {
    /// Registered mocks
    /// 注册的模拟
    mocks: Arc<RwLock<HashMap<String, MockDefinition>>>,

    /// Mock call counts
    /// 模拟调用计数
    call_counts: Arc<RwLock<HashMap<String, usize>> >,
}

/// Mock definition
/// 模拟定义
#[derive(Clone)]
struct MockDefinition {
    /// Mock function
    /// 模拟函数
    func: Arc<dyn Fn(Vec<Arc<dyn Any + Send + Sync>>) -> Box<dyn Any + Send + Sync> + Send + Sync>,

    /// Expected arguments (for verification)
    /// 预期参数（用于验证）
    expected_args: Option<Vec<String>>,

    /// Return type name
    /// 返回类型名称
    return_type: String,
}

impl MockDefinition {
    /// Create a new mock definition
    /// 创建新的模拟定义
    pub(crate) fn new<
        F: Fn(Vec<Arc<dyn Any + Send + Sync>>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static,
    >(
        func: F,
        return_type: &str,
    ) -> Self {
        Self {
            func: Arc::new(func),
            expected_args: None,
            return_type: return_type.to_string(),
        }
    }

    /// Create with expected arguments
    /// 使用预期参数创建
    pub(crate) fn with_expected_args<
        F: Fn(Vec<Arc<dyn Any + Send + Sync>>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static,
    >(
        func: F,
        return_type: &str,
        args: Vec<String>,
    ) -> Self {
        Self {
            func: Arc::new(func),
            expected_args: Some(args),
            return_type: return_type.to_string(),
        }
    }
}

impl MockRegistry {
    /// Create a new mock registry
    /// 创建新的模拟注册表
    pub fn new() -> Self {
        Self {
            mocks: Arc::new(RwLock::new(HashMap::new())),
            call_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a mock
    /// 注册模拟
    pub async fn register_mock<
        F: Fn(Vec<Arc<dyn Any + Send + Sync>>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static,
    >(
        &self,
        bean_name: impl Into<String>,
        method_name: impl Into<String>,
        func: F,
    ) {
        let key = format!("{}::{}", bean_name.into(), method_name.into());
        let mock = MockDefinition::new(func, "Any");
        let mut mocks = self.mocks.write().await;
        mocks.insert(key, mock);
    }

    /// Register a mock with expected arguments
    /// 注册带有预期参数的模拟
    pub async fn register_mock_with_args<
        F: Fn(Vec<Arc<dyn Any + Send + Sync>>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static,
    >(
        &self,
        bean_name: impl Into<String>,
        method_name: impl Into<String>,
        func: F,
        expected_args: Vec<String>,
    ) {
        let key = format!("{}::{}", bean_name.into(), method_name.into());
        let mock = MockDefinition::with_expected_args(func, "Any", expected_args);
        let mut mocks = self.mocks.write().await;
        mocks.insert(key, mock);
    }

    /// Call a mock
    /// 调用模拟
    pub async fn call_mock(
        &self,
        bean_name: &str,
        method_name: &str,
        args: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Option<Box<dyn Any + Send + Sync>> {
        let key = format!("{}::{}", bean_name, method_name);

        // Increment call count
        // 增加调用计数
        {
            let mut counts = self.call_counts.write().await;
            *counts.entry(key.clone()).or_insert(0) += 1;
        }

        // Get mock
        // 获取模拟
        let mocks = self.mocks.read().await;
        if let Some(mock) = mocks.get(&key) {
            Some((mock.func)(args))
        } else {
            None
        }
    }

    /// Get call count for a mock
    /// 获取模拟的调用计数
    pub async fn call_count(&self, bean_name: &str, method_name: &str) -> usize {
        let key = format!("{}::{}", bean_name, method_name);
        let counts = self.call_counts.read().await;
        *counts.get(&key).unwrap_or(&0)
    }

    /// Verify mock was called
    /// 验证模拟被调用
    pub async fn verify_called(&self, bean_name: &str, method_name: &str) -> bool {
        self.call_count(bean_name, method_name).await > 0
    }

    /// Verify mock was called exactly n times
    /// 验证模拟被调用恰好n次
    pub async fn verify_call_count(&self, bean_name: &str, method_name: &str, expected: usize) -> bool {
        self.call_count(bean_name, method_name).await == expected
    }

    /// Reset a mock
    /// 重置模拟
    pub async fn reset_mock(&self, bean_name: &str, method_name: &str) {
        let key = format!("{}::{}", bean_name, method_name);
        let mut counts = self.call_counts.write().await;
        counts.remove(&key);
    }

    /// Reset all mocks
    /// 重置所有模拟
    pub async fn reset_all(&self) {
        let mut counts = self.call_counts.write().await;
        counts.clear();
    }

    /// Clear all mocks
    /// 清除所有模拟
    pub async fn clear_all(&self) {
        let mut mocks = self.mocks.write().await;
        mocks.clear();
        let mut counts = self.call_counts.write().await;
        counts.clear();
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global mock registry
/// 全局模拟注册表
pub fn global_mock_registry() -> &'static MockRegistry {
    
    static REGISTRY: std::sync::LazyLock<MockRegistry> = std::sync::LazyLock::new(MockRegistry::new);
    &REGISTRY
}
