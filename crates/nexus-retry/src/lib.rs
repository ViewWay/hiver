//! Nexus Retry - Enhanced retry framework with annotations and template pattern
//! Nexus 重试 - 带注解和模板模式的增强重试框架
//!
//! # Spring Equivalent / Spring等价物
//!
//! - Spring Retry @Retryable
//! - Spring Retry @Recover
//! - Spring Retry RetryTemplate
//!
//! # Features / 功能特性
//!
//! - `#[retry]` attribute macro for automatic retry logic generation
//! - `RetryTemplate` for programmatic retry with callbacks
//! - Integration with `nexus-resilience` retry policies
//!
//! # Example / 示例
//!
//! ## Using #[retry] attribute / 使用 #[retry] 属性宏
//!
//! ```rust,ignore
//! use nexus_retry::retry;
//!
//! #[retry]
//! async fn fetch_data() -> Result<String, std::io::Error> {
//!     Ok("data".to_string())
//! }
//!
//! // With custom configuration / 带自定义配置
//! #[retry(max_attempts = 5, backoff = "exponential", initial_delay = 100)]
//! async fn call_api() -> Result<String, reqwest::Error> {
//!     Ok(reqwest::get("https://api.example.com").await?.text().await?)
//! }
//! ```
//!
//! ## Using RetryTemplate / 使用 RetryTemplate
//!
//! ```rust,ignore
//! use nexus_retry::RetryTemplate;
//!
//! let template = RetryTemplate::fixed(3);
//! let result = template
//!     .execute(|| async { fetch_data().await })
//!     .await?;
//! ```

pub use nexus_retry_macros::{retry, recover};

mod template;

pub use template::{
    RetryTemplate,
    RetryTemplateBuilder,
    RetryCallback,
    RetryContext,
    NoOpCallback,
};

// Re-export from nexus-resilience
pub use nexus_resilience::retry::{
    RetryPolicy,
    RetryError,
    ShouldRetry,
    RetryAll,
    BackoffType,
};
