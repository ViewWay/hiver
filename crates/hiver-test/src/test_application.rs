//! Test application support for booting a test instance of the Hiver application.
//! 测试应用支持，用于启动Hiver应用的测试实例。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! ```java
//! @SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
//! @ActiveProfiles("test")
//! @TestPropertySource(properties = "server.port=0")
//! public class MyTests {
//!     @Autowired
//!     private ApplicationContext context;
//!
//!     @LocalServerPort
//!     private int port;
//!
//!     @Test
//!     void contextLoads() {
//!         assertThat(context).isNotNull();
//!     }
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_test::TestApplication;
//!
//! #[tokio::test]
//! async fn test_application_startup() {
//!     let app = TestApplication::builder()
//!         .with_profile("test")
//!         .with_property("server.port", "0")
//!         .with_bean("myService", || MyService::new())
//!         .start()
//!         .await
//!         .expect("application should start");
//!
//!     let port = app.port();
//!     assert!(port > 0);
//!
//!     // app automatically shuts down on drop
//! }
//! ```

use std::{any::Any, collections::HashMap, sync::Arc};

use thiserror::Error;
use tokio::sync::RwLock;

use crate::{test_config::TestConfig, test_context::TestApplicationContext};

/// Errors that can occur when starting or managing a test application.
/// 启动或管理测试应用时可能发生的错误。
#[derive(Debug, Error)]
pub enum TestApplicationError
{
    /// The application failed to start.
    /// 应用启动失败。
    #[error("failed to start test application: {0}")]
    StartupFailed(String),

    /// A required bean was not found.
    /// 未找到所需的bean。
    #[error("bean not found: {0}")]
    BeanNotFound(String),

    /// A bean with the given name already exists.
    /// 具有给定名称的bean已存在。
    #[error("bean already registered: {0}")]
    BeanAlreadyExists(String),

    /// The application has been shut down.
    /// 应用已关闭。
    #[error("application is shut down")]
    ShutDown,
}

/// Result type for test application operations.
/// 测试应用操作的结果类型。
pub type TestAppResult<T> = Result<T, TestApplicationError>;

/// A test application instance that boots a lightweight Hiver context.
///
/// 测试应用实例，启动轻量级的Hiver上下文。
///
/// Automatically shuts down when dropped (equivalent to Spring's
/// `@DirtiesContext` cleanup).
///
/// 在丢弃时自动关闭（等价于Spring的 `@DirtiesContext` 清理）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootTest
/// @AutoConfigureMockMvc
/// public class ApplicationTests {
///     @Autowired private ApplicationContext context;
///     @LocalServerPort private int port;
/// }
/// ```
pub struct TestApplication
{
    /// Test application context holding beans and config.
    /// 持有bean和配置的测试应用上下文。
    context: TestApplicationContext,

    /// The resolved configuration for this test run.
    /// 本次测试运行的解析配置。
    config: TestConfig,

    /// The actual port the server is listening on (0 if not started).
    /// 服务器实际监听的端口（未启动时为0）。
    port: u16,

    /// Whether the application has been started.
    /// 应用是否已启动。
    started: bool,

    /// Whether the application has been shut down.
    /// 应用是否已关闭。
    shut_down: bool,

    /// Registered bean factory closures (stored for deferred initialisation).
    /// 注册的bean工厂闭包（存储用于延迟初始化）。
    bean_factories:
        Arc<RwLock<HashMap<String, Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + Send + Sync>>>>,

    /// Pre-registered bean instances (stored for deferred registration).
    /// 预注册的bean实例（存储用于延迟注册）。
    pre_registered: Vec<(String, Arc<dyn Any + Send + Sync>)>,
}

impl TestApplication
{
    /// Create a new default test application builder.
    /// 创建新的默认测试应用构建器。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let app = TestApplication::builder()
    ///     .with_profile("test")
    ///     .start()
    ///     .await?;
    /// ```
    pub fn builder() -> TestApplicationBuilder
    {
        TestApplicationBuilder::new()
    }

    /// Create a test application with default configuration.
    /// 使用默认配置创建测试应用。
    ///
    /// Equivalent to `@SpringBootTest` with no customisation.
    /// 等价于没有自定义的 `@SpringBootTest`。
    pub fn new() -> Self
    {
        Self {
            context: TestApplicationContext::new(),
            config: TestConfig::new(),
            port: 0,
            started: false,
            shut_down: false,
            bean_factories: Arc::new(RwLock::new(HashMap::new())),
            pre_registered: Vec::new(),
        }
    }

    /// Build the application with the given configuration.
    /// 使用给定配置构建应用。
    pub fn with_config(mut self, config: TestConfig) -> Self
    {
        self.config = config;
        self
    }

    /// Activate the given Spring-style profile (e.g. `"test"`, `"integration"`).
    /// 激活给定的Spring风格配置文件（例如 `"test"`、`"integration"`）。
    ///
    /// Equivalent to `@ActiveProfiles("test")`.
    /// 等价于 `@ActiveProfiles("test")`。
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self
    {
        self.config.profiles.push(profile.into());
        self
    }

    /// Override or register a named bean in the test context.
    /// 在测试上下文中覆盖或注册命名bean。
    ///
    /// Equivalent to `@TestConfiguration @Bean`.
    /// 等价于 `@TestConfiguration @Bean`。
    pub fn with_bean<T: Any + Send + Sync + 'static>(self, name: impl Into<String>, bean: T)
    -> Self
    {
        let rt = tokio::runtime::Handle::current();
        let name = name.into();
        rt.block_on(self.context.register_bean(name.as_str(), bean));
        self
    }

    /// Start the test application.
    /// 启动测试应用。
    ///
    /// Initialises the application context, registers all bean factories,
    /// and marks the application as running.
    ///
    /// 初始化应用上下文，注册所有bean工厂，并将应用标记为运行中。
    pub async fn start(mut self) -> TestAppResult<Self>
    {
        if self.shut_down
        {
            return Err(TestApplicationError::ShutDown);
        }

        // Load configuration from environment overrides.
        // 从环境覆盖加载配置。
        {
            let mut cfg = self.config.clone();
            cfg.load_from_env();
            self.config = cfg;
        }

        // Apply bean factories that were registered via the builder.
        // 应用通过构建器注册的bean工厂。
        {
            let factories = self.bean_factories.read().await;
            for (name, factory) in factories.iter()
            {
                let bean = factory();
                self.context.register_boxed_bean(name.as_str(), bean).await;
            }
        }

        // Apply pre-registered bean instances.
        // 应用预注册的bean实例。
        {
            for (name, bean) in self.pre_registered.drain(..)
            {
                let mut beans = self.context.beans_mut().await;
                beans.insert(name, bean);
            }
        }

        // Resolve port from config.
        // 从配置解析端口。
        self.port = self.config.server.port;

        // In a full implementation this is where the HTTP server,
        // database connections, and other infrastructure would be
        // bootstrapped.
        // 在完整实现中，这里会引导HTTP服务器、数据库连接和其他基础设施。

        self.started = true;
        tracing::info!(
            port = self.port,
            profiles = ?self.config.profiles,
            "TestApplication started"
        );
        Ok(self)
    }

    /// Shut down the test application, releasing all resources.
    /// 关闭测试应用，释放所有资源。
    ///
    /// Equivalent to Spring context close.
    /// 等价于Spring上下文关闭。
    pub async fn shutdown(&mut self)
    {
        if !self.shut_down
        {
            self.context.clear_beans().await;
            self.context.clear_config().await;
            self.started = false;
            self.shut_down = true;
            tracing::info!("TestApplication shut down");
        }
    }

    /// Returns the port the test server is listening on.
    /// 返回测试服务器监听的端口。
    ///
    /// Equivalent to `@LocalServerPort`.
    /// 等价于 `@LocalServerPort`。
    pub fn port(&self) -> u16
    {
        self.port
    }

    /// Returns a reference to the underlying test application context.
    /// 返回底层测试应用上下文的引用。
    pub fn context(&self) -> &TestApplicationContext
    {
        &self.context
    }

    /// Returns whether the application has been started.
    /// 返回应用是否已启动。
    pub fn is_started(&self) -> bool
    {
        self.started && !self.shut_down
    }

    /// Retrieve a bean by type from the application context.
    /// 从应用上下文按类型检索bean。
    ///
    /// # Errors
    ///
    /// Returns [`TestApplicationError::BeanNotFound`] if no bean of the
    /// requested type is registered.
    pub async fn get_bean<T: 'static + Send + Sync + Clone>(&self) -> TestAppResult<T>
    {
        self.context.get_bean::<T>().await.ok_or_else(|| {
            TestApplicationError::BeanNotFound(std::any::type_name::<T>().to_string())
        })
    }

    /// Retrieve a bean by name and type from the application context.
    /// 从应用上下文按名称和类型检索bean。
    ///
    /// # Errors
    ///
    /// Returns [`TestApplicationError::BeanNotFound`] if no bean with the
    /// given name exists.
    pub async fn get_bean_by_name<T: 'static + Send + Sync + Clone>(
        &self,
        name: &str,
    ) -> TestAppResult<T>
    {
        self.context
            .get_bean_by_name::<T>(name)
            .await
            .ok_or_else(|| TestApplicationError::BeanNotFound(name.to_string()))
    }

    /// Get a configuration property.
    /// 获取配置属性。
    pub async fn get_property(&self, key: &str) -> Option<String>
    {
        self.context.get_property(key).await
    }

    /// Set a configuration property at runtime.
    /// 在运行时设置配置属性。
    pub async fn set_property(&self, key: impl Into<String>, value: impl Into<String>)
    {
        self.context.set_property(key, value).await;
    }

    /// Get the active profiles.
    /// 获取活动配置文件。
    pub fn profiles(&self) -> &[String]
    {
        &self.config.profiles
    }

    /// Get the test mode.
    /// 获取测试模式。
    pub fn test_mode(&self) -> crate::test_config::TestMode
    {
        self.config.test_mode
    }
}

impl Default for TestApplication
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Drop for TestApplication
{
    fn drop(&mut self)
    {
        if self.started && !self.shut_down
        {
            // Spawn a task to perform async cleanup.
            // 生成一个任务来执行异步清理。
            let context = self.context.clone();
            tokio::spawn(async move {
                context.clear_beans().await;
                context.clear_config().await;
            });
            tracing::info!("TestApplication dropped — cleanup scheduled");
        }
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for constructing a [`TestApplication`] with custom configuration.
///
/// 用于构建具有自定义配置的 [`TestApplication`] 的构建器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootTest(
///     webEnvironment = WebEnvironment.RANDOM_PORT,
///     properties = { "server.port=0", "spring.datasource.url=jdbc:h2:mem:test" }
/// )
/// @ActiveProfiles({ "test", "security" })
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// let app = TestApplication::builder()
///     .with_profile("test")
///     .with_property("server.port", "0")
///     .with_property("spring.datasource.url", "jdbc:h2:mem:test")
///     .start()
///     .await?;
/// ```
pub struct TestApplicationBuilder
{
    /// Resolved configuration for this build.
    /// 本次构建的解析配置。
    config: TestConfig,

    /// Bean factory closures keyed by bean name.
    /// 按bean名称键控的bean工厂闭包。
    bean_factories: HashMap<String, Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + Send + Sync>>,

    /// Pre-registered bean instances (name, Arc<dyn Any + Send + Sync>).
    /// 预注册的bean实例（名称, Arc<dyn Any + Send + Sync>）。
    pre_registered: Vec<(String, Arc<dyn Any + Send + Sync>)>,
}

impl TestApplicationBuilder
{
    /// Create a new builder with default test configuration.
    /// 使用默认测试配置创建新的构建器。
    pub fn new() -> Self
    {
        Self {
            config: TestConfig::new(),
            bean_factories: HashMap::new(),
            pre_registered: Vec::new(),
        }
    }

    /// Set the test configuration directly.
    /// 直接设置测试配置。
    pub fn with_config(mut self, config: TestConfig) -> Self
    {
        self.config = config;
        self
    }

    /// Activate an additional profile (can be called multiple times).
    /// 激活额外的配置文件（可以多次调用）。
    ///
    /// Equivalent to `@ActiveProfiles({ "test", "security" })`.
    /// 等价于 `@ActiveProfiles({ "test", "security" })`。
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self
    {
        self.config.profiles.push(profile.into());
        self
    }

    /// Set the test mode (unit, integration, e2e).
    /// 设置测试模式（unit、integration、e2e）。
    pub fn with_test_mode(mut self, mode: crate::test_config::TestMode) -> Self
    {
        self.config.test_mode = mode;
        self
    }

    /// Add a configuration property.
    /// 添加配置属性。
    ///
    /// Equivalent to `@TestPropertySource(properties = "...")`.
    /// 等价于 `@TestPropertySource(properties = "...")`。
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.config.properties.insert(key.into(), value.into());
        self
    }

    /// Override or register a named bean instance.
    /// 覆盖或注册命名bean实例。
    ///
    /// The bean is stored as-is and registered during [`start()`](TestApplication::start).
    /// bean按原样存储，并在 [`start()`](TestApplication::start) 期间注册。
    pub fn with_bean<T: Any + Send + Sync + 'static>(
        mut self,
        name: impl Into<String>,
        bean: T,
    ) -> Self
    {
        self.pre_registered.push((name.into(), Arc::new(bean)));
        self
    }

    /// Register a bean factory function that will be invoked during
    /// [`start()`](TestApplication::start). 注册一个bean工厂函数，将在
    /// [`start()`](TestApplication::start) 期间调用。
    pub fn with_bean_factory<T: Any + Send + Sync + 'static, F>(
        mut self,
        name: impl Into<String>,
        factory: F,
    ) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let name = name.into();
        let factory = Arc::new(move || -> Box<dyn Any + Send + Sync> { Box::new(factory()) });
        self.bean_factories.insert(name, factory);
        self
    }

    /// Set the server port (0 for random).
    /// 设置服务器端口（0表示随机）。
    pub fn with_port(mut self, port: u16) -> Self
    {
        self.config.server.port = port;
        self
    }

    /// Set the database URL.
    /// 设置数据库URL。
    pub fn with_database_url(mut self, url: impl Into<String>) -> Self
    {
        self.config.database.url = Some(url.into());
        self.config.database.in_memory = false;
        self
    }

    /// Use an in-memory database.
    /// 使用内存数据库。
    pub fn with_in_memory_db(mut self) -> Self
    {
        self.config.database.in_memory = true;
        self.config.database.url = None;
        self
    }

    /// Build the [`TestApplication`] without starting it.
    /// 构建但不启动 [`TestApplication`]。
    ///
    /// Call [`TestApplication::start()`] to boot the application.
    /// 调用 [`TestApplication::start()`] 启动应用。
    pub fn build(self) -> TestAppResult<TestApplication>
    {
        let mut app = TestApplication::new().with_config(self.config);

        // Store pre-registered beans directly on the app for later registration during start().
        // 将预注册的bean直接存储在app上，以便在start()期间注册。
        app.pre_registered = self.pre_registered;

        for (name, factory) in self.bean_factories
        {
            app.bean_factories
                .try_write()
                .map_err(|_| {
                    TestApplicationError::StartupFailed("failed to acquire write lock".to_string())
                })?
                .insert(name, factory);
        }

        Ok(app)
    }

    /// Build and immediately start the test application.
    /// 构建并立即启动测试应用。
    pub async fn start(self) -> TestAppResult<TestApplication>
    {
        let app = self.build()?;
        app.start().await
    }
}

impl Default for TestApplicationBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_default_builder_creates_app()
    {
        let app = TestApplicationBuilder::new()
            .build()
            .expect("build should succeed");
        assert!(!app.is_started());
        assert_eq!(app.port(), 0);
    }

    #[tokio::test]
    async fn test_builder_with_profile()
    {
        let app = TestApplicationBuilder::new()
            .with_profile("integration")
            .build()
            .expect("build should succeed");
        assert!(app.profiles().contains(&"integration".to_string()));
    }

    #[tokio::test]
    async fn test_builder_with_properties()
    {
        let app = TestApplicationBuilder::new()
            .with_property("custom.key", "custom-value")
            .build()
            .expect("build should succeed");
        assert_eq!(app.config.properties.get("custom.key"), Some(&"custom-value".to_string()));
    }

    #[tokio::test]
    async fn test_start_and_shutdown()
    {
        let app = TestApplicationBuilder::new()
            .with_port(0)
            .start()
            .await
            .expect("start should succeed");
        assert!(app.is_started());

        let mut app = app;
        app.shutdown().await;
        assert!(!app.is_started());
    }

    #[tokio::test]
    async fn test_with_bean_factory()
    {
        let app = TestApplicationBuilder::new()
            .with_bean_factory("counter", || 42_i32)
            .start()
            .await
            .expect("start should succeed");

        let value: i32 = app
            .get_bean_by_name("counter")
            .await
            .expect("bean should exist");
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_get_bean_not_found()
    {
        let app = TestApplicationBuilder::new()
            .start()
            .await
            .expect("start should succeed");

        let result = app.get_bean::<String>().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_profiles()
    {
        let app = TestApplicationBuilder::new()
            .with_profile("test")
            .with_profile("security")
            .start()
            .await
            .expect("start should succeed");

        assert!(app.profiles().contains(&"test".to_string()));
        assert!(app.profiles().contains(&"security".to_string()));
    }

    #[tokio::test]
    async fn test_with_test_mode()
    {
        let app = TestApplicationBuilder::new()
            .with_test_mode(crate::test_config::TestMode::Integration)
            .build()
            .expect("build should succeed");

        assert_eq!(app.test_mode(), crate::test_config::TestMode::Integration);
    }

    #[tokio::test]
    async fn test_set_and_get_property_at_runtime()
    {
        let app = TestApplicationBuilder::new()
            .start()
            .await
            .expect("start should succeed");

        app.set_property("runtime.key", "runtime-value").await;
        let value = app.get_property("runtime.key").await;
        assert_eq!(value, Some("runtime-value".to_string()));
    }

    #[tokio::test]
    async fn test_builder_with_database_url()
    {
        let app = TestApplicationBuilder::new()
            .with_database_url("jdbc:h2:mem:testdb")
            .build()
            .expect("build should succeed");

        assert_eq!(app.config.database.url.as_deref(), Some("jdbc:h2:mem:testdb"));
        assert!(!app.config.database.in_memory);
    }

    #[tokio::test]
    async fn test_drop_triggers_cleanup()
    {
        {
            let _app = TestApplicationBuilder::new()
                .start()
                .await
                .expect("start should succeed");
            // _app is dropped here; cleanup is scheduled.
            // _app 在此处被丢弃；清理被调度。
        }
        // Give the async cleanup a moment to run.
        // 给异步清理一点时间运行。
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_with_bean_instance()
    {
        let app = TestApplicationBuilder::new()
            .with_bean("greeting", String::from("hello, world"))
            .start()
            .await
            .expect("start should succeed");

        let value: String = app
            .get_bean_by_name("greeting")
            .await
            .expect("bean should exist");
        assert_eq!(value, "hello, world");
    }

    #[tokio::test]
    async fn test_builder_with_in_memory_db()
    {
        let app = TestApplicationBuilder::new()
            .with_in_memory_db()
            .build()
            .expect("build should succeed");

        assert!(app.config.database.in_memory);
        assert!(app.config.database.url.is_none());
    }
}
