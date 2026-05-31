#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::doc_overindented_list_items, clippy::missing_fields_in_debug)]
//! Nexus Test - Testing framework module
//! Nexus测试 - 测试框架模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@SpringBootTest` - `NexusTest`
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
//! use hiver_test::{NexusTest, TestClient};
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
#![allow(dead_code)]

#[cfg(test)]
mod tests;

mod test_client;
mod test_context;
mod mock_bean;
mod mockito_ext;
mod test_application;
mod test_config;
mod web_test_client;
pub mod containers;
mod listener;
mod property_source;

pub use test_client::{TestClient, TestRequest, TestResponse};
pub use test_context::{TestContext, TestApplicationContext, TestContextRegistry, global_test_registry};
pub use mock_bean::{MockBean, MockRegistry, global_mock_registry};
pub use mockito_ext::{MockBeanWrapper, MockInteraction, MockitoHelper};
pub use test_application::{TestApplication, TestApplicationBuilder, TestApplicationError, TestAppResult};
pub use test_config::{TestConfig, TestConfigHolder, TestMode, ServerConfig, DatabaseConfig, global_test_config};
pub use web_test_client::{WebTestClient, RequestSpec, ResponseSpec};
pub use containers::{ContainerSet, KafkaContainer, PostgresContainer, RedisContainer};
pub use listener::{TestExecutionListener, TestLifecycleContext, TestListenerRegistry, LoggingTestListener};
pub use property_source::TestPropertySource;

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        MockBeanWrapper, MockitoHelper, TestApplicationBuilder, TestApplicationError,
        TestClient, TestConfig, TestContext, MockRegistry, TestApplication, WebTestClient,
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

/// `NexusTest` marker trait
/// `NexusTest` 标记 trait
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
pub trait NexusTest {}
