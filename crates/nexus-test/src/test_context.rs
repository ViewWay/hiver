//! Test context for managing test application state
//! 用于管理测试应用状态的测试上下文

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test application context
/// 测试应用上下文
///
/// Manages beans and configuration for tests.
/// 管理测试的bean和配置。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootTest
/// public class MyTests {
///     @Autowired
///     private ApplicationContext context;
/// }
/// ```
#[derive(Clone)]
pub struct TestApplicationContext {
    /// Beans registry
    /// Bean注册表
    beans: Arc<RwLock<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>>,

    /// Configuration properties
    /// 配置属性
    config: Arc<RwLock<HashMap<String, String>>>,
}

impl TestApplicationContext {
    /// Create a new test application context
    /// 创建新的测试应用上下文
    pub fn new() -> Self {
        Self {
            beans: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a bean
    /// 注册bean
    pub async fn register_bean<T: 'static + Send + Sync>(
        &self,
        name: impl Into<String>,
        bean: T,
    ) {
        let name = name.into();
        let mut beans = self.beans.write().await;
        beans.insert(name, Arc::new(bean));
    }

    /// Get a bean by type
    /// 按类型获取bean
    pub async fn get_bean<T: 'static + Send + Sync + Clone>(&self) -> Option<T> {
        let beans = self.beans.read().await;
        for (_, bean) in beans.iter() {
            if let Some(b) = bean.downcast_ref::<T>() {
                return Some(b.clone());
            }
        }
        None
    }

    /// Get a bean by name and type
    /// 按名称和类型获取bean
    pub async fn get_bean_by_name<T: 'static + Send + Sync + Clone>(
        &self,
        name: &str,
    ) -> Option<T> {
        let beans = self.beans.read().await;
        if let Some(bean) = beans.get(name) {
            bean.downcast_ref::<T>().cloned()
        } else {
            None
        }
    }

    /// Check if a bean exists
    /// 检查bean是否存在
    pub async fn contains_bean(&self, name: &str) -> bool {
        let beans = self.beans.read().await;
        beans.contains_key(name)
    }

    /// Get all bean names
    /// 获取所有bean名称
    pub async fn bean_names(&self) -> Vec<String> {
        let beans = self.beans.read().await;
        beans.keys().cloned().collect()
    }

    /// Set a configuration property
    /// 设置配置属性
    pub async fn set_property(&self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        let mut config = self.config.write().await;
        config.insert(key, value);
    }

    /// Get a configuration property
    /// 获取配置属性
    pub async fn get_property(&self, key: &str) -> Option<String> {
        let config = self.config.read().await;
        config.get(key).cloned()
    }

    /// Get all configuration properties
    /// 获取所有配置属性
    pub async fn properties(&self) -> HashMap<String, String> {
        let config = self.config.read().await;
        config.clone()
    }

    /// Clear all beans
    /// 清除所有bean
    pub async fn clear_beans(&self) {
        let mut beans = self.beans.write().await;
        beans.clear();
    }

    /// Clear all configuration
    /// 清除所有配置
    pub async fn clear_config(&self) {
        let mut config = self.config.write().await;
        config.clear();
    }
}

impl Default for TestApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Test context holder
/// 测试上下文持有者
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @DirtiesContext
/// @TestMethodOrder(OrderAnnotation.class)
/// public class MyTests {
///     @BeforeEach
///     public void setup() {
///         // Setup test context
///     }
/// }
/// ```
#[derive(Clone)]
pub struct TestContext {
    /// Application context
    /// 应用上下文
    pub app_context: TestApplicationContext,

    /// Test name
    /// 测试名称
    pub test_name: String,

    /// Test index
    /// 测试索引
    pub test_index: usize,
}

impl TestContext {
    /// Create a new test context
    /// 创建新的测试上下文
    pub fn new() -> Self {
        Self {
            app_context: TestApplicationContext::new(),
            test_name: String::new(),
            test_index: 0,
        }
    }

    /// Create with test name
    /// 使用测试名称创建
    pub fn with_name(test_name: impl Into<String>) -> Self {
        Self {
            app_context: TestApplicationContext::new(),
            test_name: test_name.into(),
            test_index: 0,
        }
    }

    /// Set test name
    /// 设置测试名称
    pub fn set_test_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.test_name = name.into();
        self
    }

    /// Set test index
    /// 设置测试索引
    pub fn set_test_index(&mut self, index: usize) -> &mut Self {
        self.test_index = index;
        self
    }

    /// Mark test as dirty (context should be reset)
    /// 标记测试为dirty（上下文应重置）
    pub async fn mark_dirty(&self) {
        self.app_context.clear_beans().await;
        self.app_context.clear_config().await;
    }

    /// Reset the context
    /// 重置上下文
    pub async fn reset(&self) {
        self.mark_dirty().await;
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Global test context registry
/// 全局测试上下文注册表
///
/// Manages test contexts across multiple tests.
/// 管理多个测试之间的测试上下文。
pub struct TestContextRegistry {
    contexts: Arc<RwLock<HashMap<String, TestContext>>>,
    current_index: Arc<RwLock<usize>>,
}

impl TestContextRegistry {
    /// Create a new registry
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Get or create a context for a test
    /// 获取或创建测试的上下文
    pub async fn get_context(&self, test_name: &str) -> TestContext {
        let mut contexts = self.contexts.write().await;
        let _index = {
            let mut idx = self.current_index.write().await;
            *idx += 1;
            *idx - 1
        };

        contexts
            .entry(test_name.to_string())
            .or_insert_with(|| TestContext::with_name(test_name))
            .clone()
    }

    /// Remove a context
    /// 移除上下文
    pub async fn remove_context(&self, test_name: &str) {
        let mut contexts = self.contexts.write().await;
        contexts.remove(test_name);
    }

    /// Clear all contexts
    /// 清除所有上下文
    pub async fn clear_all(&self) {
        let mut contexts = self.contexts.write().await;
        contexts.clear();
        let mut index = self.current_index.write().await;
        *index = 0;
    }
}

impl Default for TestContextRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global test context instance
/// 全局测试上下文实例
pub fn global_test_registry() -> &'static TestContextRegistry {
    use once_cell::sync::Lazy;
    static REGISTRY: std::sync::LazyLock<TestContextRegistry> = std::sync::LazyLock::new(TestContextRegistry::new);
    &REGISTRY
}
