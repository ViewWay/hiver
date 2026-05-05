//! Test configuration support
//! 测试配置支持

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test configuration
/// 测试配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @TestConfiguration
/// public class TestConfig {
///     @Bean
///     public UserService testUserService() {
///         return new TestUserService();
///     }
/// }
///
/// @TestPropertySource(locations = "classpath:test.properties")
/// @SpringBootTest
/// public class MyTests { }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestConfig {
    /// Test properties
    /// 测试属性
    #[serde(default)]
    pub properties: HashMap<String, String>,

    /// Active profiles
    /// 活动配置文件
    #[serde(default)]
    pub profiles: Vec<String>,

    /// Test mode (unit, integration, e2e)
    /// 测试模式
    #[serde(default)]
    pub test_mode: TestMode,

    /// Server configuration
    /// 服务器配置
    #[serde(default)]
    pub server: ServerConfig,

    /// Database configuration
    /// 数据库配置
    #[serde(default)]
    pub database: DatabaseConfig,
}

/// Test mode
/// 测试模式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum TestMode {
    /// Unit test (no external dependencies)
    /// 单元测试（无外部依赖）
    #[default]
    Unit,

    /// Integration test (some external dependencies)
    /// 集成测试（部分外部依赖）
    Integration,

    /// End-to-end test (full environment)
    /// 端到端测试（完整环境）
    E2E,
}


/// Server configuration for tests
/// 测试的服务器配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    /// 绑定主机
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to bind to (0 for random)
    /// 绑定端口（0表示随机）
    #[serde(default)]
    pub port: u16,

    /// Enable SSL
    /// 启用SSL
    #[serde(default)]
    pub ssl: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: 0,
            ssl: false,
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

/// Database configuration for tests
/// 测试的数据库配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Use in-memory database
    /// 使用内存数据库
    #[serde(default)]
    pub in_memory: bool,

    /// Database URL
    /// 数据库URL
    #[serde(default)]
    pub url: Option<String>,

    /// Auto-migrate schema
    /// 自动迁移schema
    #[serde(default = "default_auto_migrate")]
    pub auto_migrate: bool,

    /// Clean database after each test
    /// 每次测试后清理数据库
    #[serde(default = "default_auto_cleanup")]
    pub auto_cleanup: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            in_memory: true,
            url: None,
            auto_migrate: default_auto_migrate(),
            auto_cleanup: default_auto_cleanup(),
        }
    }
}

fn default_auto_migrate() -> bool {
    true
}

fn default_auto_cleanup() -> bool {
    true
}

impl TestConfig {
    /// Create a new test configuration
    /// 创建新的测试配置
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            profiles: vec!["test".to_string()],
            test_mode: TestMode::Unit,
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
        }
    }

    /// Set test mode
    /// 设置测试模式
    pub fn with_test_mode(mut self, mode: TestMode) -> Self {
        self.test_mode = mode;
        self
    }

    /// Add a property
    /// 添加属性
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Add a profile
    /// 添加配置文件
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profiles.push(profile.into());
        self
    }

    /// Set server port
    /// 设置服务器端口
    pub fn with_port(mut self, port: u16) -> Self {
        self.server.port = port;
        self
    }

    /// Set database URL
    /// 设置数据库URL
    pub fn with_database_url(mut self, url: impl Into<String>) -> Self {
        self.database.url = Some(url.into());
        self.database.in_memory = false;
        self
    }

    /// Enable in-memory database
    /// 启用内存数据库
    pub fn with_in_memory_db(mut self) -> Self {
        self.database.in_memory = true;
        self.database.url = None;
        self
    }

    /// Get a property value
    /// 获取属性值
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// Check if profile is active
    /// 检查配置文件是否活动
    pub fn has_profile(&self, profile: &str) -> bool {
        self.profiles.iter().any(|p| p == profile)
    }

    /// Load from environment variables
    /// 从环境变量加载
    pub fn load_from_env(&mut self) {
        // Load common test configuration from environment
        // 从环境变量加载常用测试配置
        if let Ok(port) = std::env::var("TEST_SERVER_PORT")
            && let Ok(p) = port.parse::<u16>() {
                self.server.port = p;
            }

        if let Ok(db_url) = std::env::var("TEST_DATABASE_URL") {
            self.database.url = Some(db_url);
            self.database.in_memory = false;
        }

        if let Ok(mode) = std::env::var("TEST_MODE") {
            match mode.to_lowercase().as_str() {
                "unit" => self.test_mode = TestMode::Unit,
                "integration" => self.test_mode = TestMode::Integration,
                "e2e" => self.test_mode = TestMode::E2E,
                _ => {}
            }
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Test configuration holder
/// 测试配置持有者
#[derive(Clone)]
pub struct TestConfigHolder {
    config: Arc<RwLock<TestConfig>>,
}

impl TestConfigHolder {
    /// Create a new test configuration holder
    /// 创建新的测试配置持有者
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(TestConfig::new())),
        }
    }

    /// Set the configuration
    /// 设置配置
    pub async fn set(&self, config: TestConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Get the configuration
    /// 获取配置
    pub async fn get(&self) -> TestConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    /// 更新配置
    pub async fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut TestConfig),
    {
        let mut cfg = self.config.write().await;
        f(&mut cfg);
    }

    /// Load from environment
    /// 从环境加载
    pub async fn load_from_env(&self) {
        let mut cfg = self.config.write().await;
        cfg.load_from_env();
    }
}

impl Default for TestConfigHolder {
    fn default() -> Self {
        Self::new()
    }
}

/// Global test configuration holder
/// 全局测试配置持有者
pub fn global_test_config() -> &'static TestConfigHolder {
    use once_cell::sync::Lazy;
    static CONFIG: std::sync::LazyLock<TestConfigHolder> = std::sync::LazyLock::new(TestConfigHolder::new);
    &CONFIG
}

/// `test_config` attribute macro
/// `test_config` 属性宏
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_test::test_config;
///
/// #[test_config]
/// fn test_config() -> TestConfig {
///     TestConfig::new()
///         .with_test_mode(TestMode::Integration)
///         .with_port(8080)
/// }
/// ```
// Note: test_config would be a proc-macro in a full implementation
// 注：test_config 在完整实现中将是过程宏
pub(crate) type TestConfigFn = fn() -> TestConfig;
