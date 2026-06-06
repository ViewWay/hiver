//! Hiver Retry - Enhanced retry framework with annotations and template pattern
//! Hiver 重试 - 带注解和模板模式的增强重试框架
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
//! - Integration with `hiver-resilience` retry policies
//!
//! # Example / 示例
//!
//! ## Using #[retry] attribute / 使用 #[retry] 属性宏
//!
//! ```rust,ignore
//! use hiver_retry::retry;
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
//! use hiver_retry::RetryTemplate;
//!
//! let template = RetryTemplate::fixed(3);
//! let result = template
//!     .execute(|| async { fetch_data().await })
//!     .await?;
//! ```

pub use hiver_retry_macros::{recover, retry};

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

mod template;

// Re-export from hiver-resilience
pub use hiver_resilience::retry::{BackoffType, RetryAll, RetryError, RetryPolicy, ShouldRetry};
pub use template::{
    NoOpCallback, RetryCallback, RetryContext, RetryTemplate, RetryTemplateBuilder,
};
