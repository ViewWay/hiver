//! Async task execution framework
//! 异步任务执行框架
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `AsyncTaskExecutor` | `TaskExecutor` / `AsyncTaskExecutor` |
//! | `@Async` | `@Async` |
//! | `TaskExecutor` | `ThreadPoolTaskExecutor` |
//! | `AsyncTask` | `@Async` method |
//!
//! # Examples / 示例
//!
//! ```rust,ignore
//! use hiver_async::{AsyncTaskExecutor, AsyncTask};
//! use tokio::time::{sleep, Duration};
//!
//! struct MyTask {
//!     name: String,
//! }
//!
//! #[async_trait::async_trait]
//! impl AsyncTask for MyTask {
//!     async fn run(&self) -> Result<(), String> {
//!         sleep(Duration::from_millis(100)).await;
//!         println!("Task {} executed", self.name);
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let executor = AsyncTaskExecutor::new();
//!
//!     // Submit task
//!     let handle = executor.submit(MyTask {
//!         name: "test".to_string(),
//!     })?;
//!
//!     // Wait for completion
//!     handle.await?;
//!
//!     Ok(())
//! }
//! ```

#[cfg(test)]
mod tests;

mod config;
mod error;
mod executor;
mod task;

pub use config::{ExecutionMode, RejectionPolicy, TaskExecutorConfig};
pub use error::{AsyncError, AsyncResult};
pub use executor::{AsyncTaskExecutor, TaskExecutor};
pub use task::{AsyncTask, AsyncTaskHandle, RunnableTask};
