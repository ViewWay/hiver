#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::doc_overindented_list_items,
    clippy::missing_fields_in_debug
)]
//! Hiver Test - Testing framework module
//! Hiver测试 - 测试框架模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@SpringBootTest` - `HiverTest`
//! - `@WebMvcTest` - `WebTest`
//! - `@MockBean` - `MockBean`
//! - `TestRestTemplate` - `TestClient`
//! - `MockMvc` - `TestClient`
//! - `@TestConfiguration` - `TestConfig`
//! - `@TestMethodOrder` - `TestOrder`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_test::{HiverTest, TestClient};
//! use hiver_http::{Request, Response};
//!
//! #[tokio::test]
//! async fn test_user_endpoint() {
//!     let client = TestClient::new().await;
//!
//!     let response = client.get("/api/users/1")
//!         .send()
//!         .await;
//!
//!     assert_eq!(response.status(), 200);
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

pub mod containers;
mod listener;
mod mock_bean;
mod mockito_ext;
mod property_source;
mod test_application;
mod test_client;
mod test_config;
mod test_context;
mod web_test_client;

pub use containers::{ContainerSet, KafkaContainer, PostgresContainer, RedisContainer};
pub use listener::{
    LoggingTestListener, TestExecutionListener, TestLifecycleContext, TestListenerRegistry,
};
pub use mock_bean::{MockBean, MockRegistry, global_mock_registry};
pub use mockito_ext::{MockBeanWrapper, MockInteraction, MockitoHelper};
pub use property_source::TestPropertySource;
pub use test_application::{
    TestAppResult, TestApplication, TestApplicationBuilder, TestApplicationError,
};
pub use test_client::{TestClient, TestRequest, TestResponse};
pub use test_config::{
    DatabaseConfig, ServerConfig, TestConfig, TestConfigHolder, TestMode, global_test_config,
};
pub use test_context::{
    TestApplicationContext, TestContext, TestContextRegistry, global_test_registry,
};
pub use web_test_client::{RequestSpec, ResponseSpec, WebTestClient};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    pub use super::{
        MockBeanWrapper, MockRegistry, MockitoHelper, TestApplication, TestApplicationBuilder,
        TestApplicationError, TestClient, TestConfig, TestContext, WebTestClient,
    };
}

/// Version of the test module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default test timeout in seconds
/// 默认测试超时时间（秒）
pub const DEFAULT_TEST_TIMEOUT_SECS: u64 = 30;

/// Default test server port (0 for random available port)
/// 默认测试服务器端口（0表示随机可用端口）
pub const DEFAULT_TEST_PORT: u16 = 0;

/// `HiverTest` marker trait
/// `HiverTest` 标记 trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @SpringBootTest
/// @AutoConfigureMockMvc
/// public class MyTests {
///     @Autowired
///     private MockMvc mockMvc;
/// }
/// ```
pub trait HiverTest {}
