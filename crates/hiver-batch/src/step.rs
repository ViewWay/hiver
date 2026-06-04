//! Step configuration and execution
//! 步骤配置和执行

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use crate::{
    context::StepContext,
    error::{BatchError, BatchResult},
    execution::{BatchStatus, ExitStatus, StepExecution},
    processor::{ItemProcessor, PassThroughProcessor},
    reader::ItemReader,
    writer::ItemWriter,
};

/// Step execution configuration
/// 步骤执行配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Step step1(JobRepository jobRepository,
///                   PlatformTransactionManager transactionManager,
///                   ItemReader<I> reader,
///                   ItemWriter<O> writer) {
///     return new StepBuilder("step1", jobRepository)
///         .<I, O>chunk(100)
///         .reader(reader)
///         .writer(writer)
///         .faultTolerant()
///         .retry(ConstraintViolationException.class)
///         .retryLimit(3)
///         .build();
/// }
/// ```
pub struct Step<R, P, W>
where
    R: ItemReader,
    R::Item: Send + Sync,
    P: ItemProcessor<Input = R::Item>,
    W: ItemWriter<Item = P::Output>,
{
    /// Step name
    /// 步骤名称
    pub name: String,

    /// Reader
    /// 读取器
    pub reader: Arc<RwLock<R>>,

    /// Processor
    /// 处理器
    pub processor: Arc<RwLock<P>>,

    /// Writer
    /// 写入器
    pub writer: Arc<RwLock<W>>,

    /// Chunk size
    /// 块大小
    pub chunk_size: usize,

    /// Allow restart
    /// 允许重启
    pub allow_restart: bool,

    /// Start limit
    /// 启动限制
    pub start_limit: usize,

    /// Skip limit
    /// 跳过限制
    pub skip_limit: usize,

    /// Retry limit
    /// 重试限制
    pub retry_limit: usize,

    /// Timeout in seconds
    /// 超时时间（秒）
    pub timeout_secs: Option<u64>,

    /// Listener
    /// 监听器
    pub listener: Option<Arc<dyn StepListener>>,
}

impl<R, P, W> Step<R, P, W>
where
    R: ItemReader,
    R::Item: Send + Sync,
    P: ItemProcessor<Input = R::Item>,
    W: ItemWriter<Item = P::Output>,
{
    /// Create step with reader and writer (no processor)
    /// 使用读取器和写入器创建步骤（无处理器）
    pub fn new(name: impl Into<String>, reader: R, writer: W) -> Self
    where
        P: Default,
    {
        Self {
            name: name.into(),
            reader: Arc::new(RwLock::new(reader)),
            processor: Arc::new(RwLock::new(P::default())),
            writer: Arc::new(RwLock::new(writer)),
            chunk_size: 100,
            allow_restart: true,
            start_limit: usize::MAX,
            skip_limit: 0,
            retry_limit: 0,
            timeout_secs: None,
            listener: None,
        }
    }

    /// Set chunk size
    /// 设置块大小
    pub fn with_chunk_size(mut self, size: usize) -> Self
    {
        self.chunk_size = size;
        self
    }

    /// Set processor
    /// 设置处理器
    pub fn with_processor(mut self, processor: P) -> Self
    {
        self.processor = Arc::new(RwLock::new(processor));
        self
    }

    /// Set skip limit
    /// 设置跳过限制
    pub fn with_skip_limit(mut self, limit: usize) -> Self
    {
        self.skip_limit = limit;
        self
    }

    /// Set retry limit
    /// 设置重试限制
    pub fn with_retry_limit(mut self, limit: usize) -> Self
    {
        self.retry_limit = limit;
        self
    }

    /// Set timeout
    /// 设置超时
    pub fn with_timeout(mut self, secs: u64) -> Self
    {
        self.timeout_secs = Some(secs);
        self
    }

    /// Set listener
    /// 设置监听器
    pub fn with_listener(mut self, listener: Arc<dyn StepListener>) -> Self
    {
        self.listener = Some(listener);
        self
    }

    /// Execute the step
    /// 执行步骤
    pub async fn execute(
        &self,
        step_execution: &mut StepExecution,
        context: &StepContext,
    ) -> BatchResult<ExitStatus>
    {
        step_execution.set_status(BatchStatus::Started);
        step_execution.set_start_time(Utc::now());

        if let Some(listener) = &self.listener
        {
            listener.before_step(step_execution, context).await?;
        }

        let result = self.execute_internal(step_execution, context).await;

        step_execution.set_end_time(Utc::now());

        match &result
        {
            Ok(_) =>
            {
                step_execution.set_status(BatchStatus::Completed);
                step_execution.set_exit_status(ExitStatus::completed());
            },
            Err(_e) =>
            {
                step_execution.set_status(BatchStatus::Failed);
                step_execution.set_exit_status(ExitStatus::failed());
            },
        }

        if let Some(listener) = &self.listener
        {
            listener.after_step(step_execution, context).await?;
        }

        result
    }

    async fn execute_internal(
        &self,
        step_execution: &mut StepExecution,
        _context: &StepContext,
    ) -> BatchResult<ExitStatus>
    {
        // Open reader and writer
        self.reader.write().await.open().await?;
        self.writer.write().await.open().await?;
        self.processor.write().await.before_process().await?;

        let mut skip_count = 0;

        loop
        {
            // Read chunk
            let mut items = Vec::with_capacity(self.chunk_size);
            let mut processed = Vec::with_capacity(self.chunk_size);

            for _ in 0..self.chunk_size
            {
                let item = match self.reader.write().await.read().await
                {
                    Ok(Some(item)) => item,
                    Ok(None) => break,
                    Err(_e) =>
                    {
                        skip_count += 1;
                        if skip_count > self.skip_limit
                        {
                            return Err(BatchError::SkipLimitExceeded {
                                limit: self.skip_limit,
                                count: skip_count,
                            });
                        }
                        step_execution.increment_skip_count();
                        continue;
                    },
                };
                step_execution.increment_read_count();
                items.push(item);
            }

            if items.is_empty()
            {
                break;
            }

            // Process items
            for item in items
            {
                let result = match self.processor.write().await.process(item).await
                {
                    Ok(Some(processed_item)) => Ok(Some(processed_item)),
                    Ok(None) =>
                    {
                        // Filtered out
                        Ok(None)
                    },
                    Err(e) =>
                    {
                        step_execution.increment_rollback_count();
                        Err(e)
                    },
                };

                match result
                {
                    Ok(Some(processed_item)) =>
                    {
                        processed.push(processed_item);
                        step_execution.increment_process_count();
                    },
                    Ok(None) =>
                    {
                        step_execution.increment_filter_count();
                    },
                    Err(_e) =>
                    {
                        skip_count += 1;
                        if skip_count > self.skip_limit
                        {
                            return Err(BatchError::SkipLimitExceeded {
                                limit: self.skip_limit,
                                count: skip_count,
                            });
                        }
                        step_execution.increment_skip_count();
                    },
                }
            }

            // Write processed items
            if !processed.is_empty()
            {
                self.writer.write().await.write(processed).await?;
                step_execution.increment_write_count(step_execution.process_count());
            }
        }

        self.processor.write().await.after_process().await?;
        self.reader.write().await.close().await?;
        self.writer.write().await.close().await?;

        Ok(ExitStatus::completed())
    }
}

/// Step builder
/// 步骤构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// StepBuilder builder = new StepBuilder("stepName", jobRepository);
/// ```
pub struct StepBuilder
{
    name: String,
    chunk_size: usize,
    skip_limit: usize,
    retry_limit: usize,
    timeout_secs: Option<u64>,
    allow_restart: bool,
    listener: Option<Arc<dyn StepListener>>,
}

impl StepBuilder
{
    /// Create new step builder
    /// 创建新步骤构建器
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            chunk_size: 100,
            skip_limit: 0,
            retry_limit: 0,
            timeout_secs: None,
            allow_restart: true,
            listener: None,
        }
    }

    /// Set chunk size
    /// 设置块大小
    pub fn with_chunk_size(mut self, size: usize) -> Self
    {
        self.chunk_size = size;
        self
    }

    /// Set skip limit
    /// 设置跳过限制
    pub fn with_skip_limit(mut self, limit: usize) -> Self
    {
        self.skip_limit = limit;
        self
    }

    /// Set retry limit
    /// 设置重试限制
    pub fn with_retry_limit(mut self, limit: usize) -> Self
    {
        self.retry_limit = limit;
        self
    }

    /// Set timeout
    /// 设置超时
    pub fn with_timeout(mut self, secs: u64) -> Self
    {
        self.timeout_secs = Some(secs);
        self
    }

    /// Set listener
    /// 设置监听器
    pub fn with_listener(mut self, listener: Arc<dyn StepListener>) -> Self
    {
        self.listener = Some(listener);
        self
    }

    /// Build step with reader and writer
    /// 使用读取器和写入器构建步骤
    pub fn build<R, W>(self, reader: R, writer: W) -> Step<R, PassThroughProcessor<R::Item>, W>
    where
        R: ItemReader + Send + Sync,
        R::Item: Send + Sync,
        W: ItemWriter<Item = R::Item> + Send + Sync,
    {
        Step {
            name: self.name,
            reader: Arc::new(RwLock::new(reader)),
            processor: Arc::new(RwLock::new(PassThroughProcessor::new())),
            writer: Arc::new(RwLock::new(writer)),
            chunk_size: self.chunk_size,
            allow_restart: self.allow_restart,
            start_limit: usize::MAX,
            skip_limit: self.skip_limit,
            retry_limit: self.retry_limit,
            timeout_secs: self.timeout_secs,
            listener: self.listener,
        }
    }

    /// Build step with reader, processor, and writer
    /// 使用读取器、处理器和写入器构建步骤
    pub fn build_with_processor<R, P, W>(self, reader: R, processor: P, writer: W) -> Step<R, P, W>
    where
        R: ItemReader + Send + Sync,
        R::Item: Send + Sync,
        P: ItemProcessor<Input = R::Item> + Send + Sync,
        W: ItemWriter<Item = P::Output> + Send + Sync,
    {
        Step {
            name: self.name,
            reader: Arc::new(RwLock::new(reader)),
            processor: Arc::new(RwLock::new(processor)),
            writer: Arc::new(RwLock::new(writer)),
            chunk_size: self.chunk_size,
            allow_restart: self.allow_restart,
            start_limit: usize::MAX,
            skip_limit: self.skip_limit,
            retry_limit: self.retry_limit,
            timeout_secs: self.timeout_secs,
            listener: self.listener,
        }
    }
}

/// Step listener
/// 步骤监听器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface StepExecutionListener {
///     void beforeStep(StepExecution stepExecution);
///     ExitStatus afterStep(StepExecution stepExecution);
/// }
/// ```
#[async_trait]
pub trait StepListener: Send + Sync
{
    /// Called before step execution
    /// 步骤执行前调用
    async fn before_step(
        &self,
        step_execution: &StepExecution,
        context: &StepContext,
    ) -> BatchResult<()>
    {
        let _ = (step_execution, context);
        Ok(())
    }

    /// Called after step execution
    /// 步骤执行后调用
    async fn after_step(
        &self,
        step_execution: &StepExecution,
        context: &StepContext,
    ) -> BatchResult<()>
    {
        let _ = (step_execution, context);
        Ok(())
    }

    /// Called on step error
    /// 步骤错误时调用
    async fn on_error(&self, step_execution: &StepExecution, error: &BatchError)
    -> BatchResult<()>
    {
        let _ = (step_execution, error);
        Ok(())
    }
}

/// Simple step listener for logging
/// 用于日志记录的简单步骤监听器
pub struct LoggingStepListener
{
    pub log_prefix: String,
}

impl LoggingStepListener
{
    pub fn new(prefix: impl Into<String>) -> Self
    {
        Self {
            log_prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl StepListener for LoggingStepListener
{
    async fn before_step(
        &self,
        step_execution: &StepExecution,
        _context: &StepContext,
    ) -> BatchResult<()>
    {
        tracing::info!("[{}] Starting step: {}", self.log_prefix, step_execution.step_name);
        Ok(())
    }

    async fn after_step(
        &self,
        step_execution: &StepExecution,
        _context: &StepContext,
    ) -> BatchResult<()>
    {
        tracing::info!(
            "[{}] Completed step: {} - read: {}, write: {}, skip: {}",
            self.log_prefix,
            step_execution.step_name,
            step_execution.read_count(),
            step_execution.write_count(),
            step_execution.skip_count()
        );
        Ok(())
    }

    async fn on_error(&self, step_execution: &StepExecution, error: &BatchError)
    -> BatchResult<()>
    {
        tracing::error!(
            "[{}] Error in step {}: {}",
            self.log_prefix,
            step_execution.step_name,
            error
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use uuid::Uuid;

    use super::*;
    use crate::{reader::ItemStreamReader, writer::ItemStreamWriter};

    #[tokio::test]
    async fn test_step_builder()
    {
        let builder = StepBuilder::new("test-step")
            .with_chunk_size(50)
            .with_skip_limit(10);

        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        let step = builder.build(reader, writer);
        assert_eq!(step.name, "test-step");
        assert_eq!(step.chunk_size, 50);
        assert_eq!(step.skip_limit, 10);
    }

    #[tokio::test]
    async fn test_step_execution()
    {
        let reader = ItemStreamReader::new(vec![1, 2, 3, 4, 5]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("test-step")
            .with_chunk_size(2)
            .build(reader, writer);

        let job_exec_id = Uuid::new_v4();
        let mut step_execution = StepExecution::new("test-step", job_exec_id);
        let context = StepContext::standalone();

        let _exit_status = step.execute(&mut step_execution, &context).await.unwrap();

        assert_eq!(step_execution.status, BatchStatus::Completed);
        assert_eq!(step_execution.read_count(), 5);
        assert_eq!(step_execution.process_count(), 5);
    }

    #[tokio::test]
    async fn test_logging_listener()
    {
        let listener = Arc::new(LoggingStepListener::new("TEST"));

        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("test-step")
            .with_listener(listener.clone())
            .build(reader, writer);

        let job_exec_id = Uuid::new_v4();
        let mut step_execution = StepExecution::new("test-step", job_exec_id);
        let context = StepContext::standalone();

        step.execute(&mut step_execution, &context).await.unwrap();

        assert_eq!(step_execution.status, BatchStatus::Completed);
    }
}
