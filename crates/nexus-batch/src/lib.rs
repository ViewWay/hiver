//! Nexus Batch Framework
//! Nexus批处理框架
//!
//! A batch processing framework equivalent to Spring Batch.
//! 等价于Spring Batch的批处理框架。
//!
//! # Overview / 概述
//!
//! This module provides a comprehensive batch processing framework with:
//! 本模块提供完整的批处理框架，具有：
//!
//! - **Job**: Batch job container containing multiple steps
//!   **Job**: 包含多个步骤的批处理作业容器
//! - **Step**: Independent phase of execution within a job
//!   **Step**: 作业中独立的执行阶段
//! - **ItemReader**: Reads items for processing
//!   **ItemReader**: 读取待处理项目
//! - **ItemProcessor**: Transforms items (optional)
//!   **ItemProcessor**: 转换项目（可选）
//! - **ItemWriter**: Writes processed items
//!   **ItemWriter**: 写入处理后的项目
//! - **JobRepository**: Stores job metadata and execution history
//!   **JobRepository**: 存储作业元数据和执行历史
//! - **JobLauncher**: Launches and manages job execution
//!   **JobLauncher**: 启动和管理作业执行
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! // Spring Batch Job Configuration
//! @Bean
//! public Job importUserJob(JobRepository jobRepository, Step step1) {
//!     return new JobBuilder("importUserJob", jobRepository)
//!         .start(step1)
//!         .build();
//! }
//!
//! @Bean
//! public Step step1(JobRepository jobRepository,
//!                   PlatformTransactionManager transactionManager,
//!                   ItemReader<User> reader,
//!                   ItemWriter<User> writer) {
//!     return new StepBuilder("step1", jobRepository)
//!         .<User, User>chunk(100)
//!         .reader(reader)
//!         .writer(writer)
//!         .transactionManager(transactionManager)
//!         .build();
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use nexus_batch::prelude::*;
//! use async_trait::async_trait;
//!
//! // Define reader
//! struct UserReader;
//!
//! #[async_trait]
//! impl ItemReader for UserReader {
//!     type Item = User;
//!
//!     async fn read(&mut self) -> BatchResult<Option<User>> {
//!         // Read user from source
//!         Ok(Some(User::default()))
//!     }
//! }
//!
//! // Define writer
//! struct UserWriter;
//!
//! #[async_trait]
//! impl ItemWriter for UserWriter {
//!     type Item = User;
//!
//!     async fn write(&mut self, items: Vec<User>) -> BatchResult<()> {
//!         // Write users to destination
//!         Ok(())
//!     }
//! }
//!
//! // Create and run job
//! #[tokio::main]
//! async fn main() -> BatchResult<()> {
//!     let job = Job::new("import-users")
//!         .add_step(Step::new("process-users")
//!             .with_chunk_size(100)
//!             .with_reader(UserReader)
//!             .with_writer(UserWriter))
//!         .build()?;
//!
//!     let launcher = JobLauncher::new(JobRepository::new());
//!     let execution = launcher.run(job).await?;
//!
//!     println!("Job completed with status: {:?}", execution.status);
//!     Ok(())
//! }
//! ```

#[cfg(test)]
mod tests;

pub mod error;
pub mod job;
pub mod step;
pub mod reader;
pub mod processor;
pub mod writer;
pub mod repository;
pub mod launcher;
pub mod execution;
pub mod context;
pub mod partition;
pub mod flow;

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{BatchError, BatchResult};
    pub use crate::job::{Job, JobBuilder};
    pub use crate::step::{Step, StepBuilder};
    pub use crate::reader::{ItemReader, ItemStreamReader};
    pub use crate::processor::{ItemProcessor, PassThroughProcessor};
    pub use crate::writer::{ItemWriter, ItemStreamWriter};
    pub use crate::repository::JobRepository;
    pub use crate::launcher::JobLauncher;
    pub use crate::execution::{JobExecution, StepExecution, JobStatus, BatchStatus};
    pub use crate::context::{JobContext, StepContext};
}

/// Re-exports for convenience
/// 便捷重导出
pub use error::{BatchError, BatchResult};
pub use job::{Job, JobBuilder};
pub use step::{Step, StepBuilder};
pub use reader::{ItemReader, ItemStreamReader};
pub use processor::{ItemProcessor, PassThroughProcessor};
pub use writer::{ItemWriter, ItemStreamWriter};
pub use repository::JobRepository;
pub use launcher::JobLauncher;
pub use execution::{JobExecution, StepExecution, JobStatus, BatchStatus};
pub use context::{JobContext, StepContext};
