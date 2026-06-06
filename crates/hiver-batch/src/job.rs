//! Job configuration and execution
//! 作业配置和执行

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;

use crate::{
    context::{JobContext, StepContext},
    error::{BatchError, BatchResult},
    execution::{ExitStatus, JobExecution, JobStatus, StepExecution},
    repository::{InMemoryJobRepository, JobRepository},
    step::Step,
};

/// Job configuration
/// 作业配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Job importUserJob(JobRepository jobRepository, Step step1) {
///     return new JobBuilder("importUserJob", jobRepository)
///         .start(step1)
///         .build();
/// }
/// ```
pub struct Job
{
    /// Job name
    /// 作业名称
    pub name: String,

    /// Job steps
    /// 作业步骤
    pub steps: Vec<Arc<dyn AnyStep>>,

    /// Allow restart
    /// 允许重启
    pub allow_restart: bool,

    /// Job parameters
    /// 作业参数
    pub parameters: HashMap<String, String>,

    /// Job listener
    /// 作业监听器
    pub listener: Option<Arc<dyn JobListener>>,
}

/// Type-erased step trait
/// 类型擦除的步骤trait
///
/// Note: This trait uses `&mut StepExecution` which is not `Send` across await.
/// For production use, consider using `Arc<RwLock<StepExecution>>` instead.
#[async_trait]
pub trait AnyStep: Send + Sync
{
    /// Get step name
    /// 获取步骤名称
    fn name(&self) -> &str;

    /// Execute the step
    /// 执行步骤
    async fn execute(
        &self,
        step_execution: &mut StepExecution,
        context: &StepContext,
    ) -> BatchResult<ExitStatus>;
}

#[async_trait]
impl<R, P, W> AnyStep for Step<R, P, W>
where
    R: crate::reader::ItemReader + Send + Sync + 'static,
    R::Item: Send + Sync + 'static,
    P: crate::processor::ItemProcessor<Input = R::Item> + Send + Sync + 'static,
    P::Output: Send + Sync + 'static,
    W: crate::writer::ItemWriter<Item = P::Output> + Send + Sync + 'static,
{
    fn name(&self) -> &str
    {
        &self.name
    }

    async fn execute(
        &self,
        step_execution: &mut StepExecution,
        context: &StepContext,
    ) -> BatchResult<ExitStatus>
    {
        self.execute(step_execution, context).await
    }
}

impl Job
{
    /// Execute the job with given repository
    /// 使用给定存储库执行作业
    pub async fn execute_with_repository(
        &self,
        repository: &InMemoryJobRepository,
    ) -> BatchResult<JobExecution>
    {
        // Create job execution
        let mut job_execution = repository
            .create_job_execution(self.name.clone(), self.parameters.clone())
            .await?;

        job_execution.set_status(JobStatus::Started);
        job_execution.set_start_time(Utc::now());

        // Save initial execution
        repository.save_job_execution(&job_execution).await?;

        // Create job context
        let job_context = JobContext::new();

        // Notify listener
        if let Some(listener) = &self.listener
        {
            listener.before_job(&job_execution, &job_context).await?;
        }

        // Execute steps
        let mut final_status = ExitStatus::completed();

        for step in &self.steps
        {
            // Create step execution
            let mut step_execution = repository
                .create_step_execution(step.name().to_string(), job_execution.id)
                .await?;

            // Create step context
            let step_context = StepContext::new(job_context.clone());

            // Execute step
            let step_result = step.execute(&mut step_execution, &step_context).await;

            // Save step execution
            repository.update_step_execution(&step_execution).await?;
            job_execution.add_step_execution(step_execution);

            match step_result
            {
                Ok(status) =>
                {
                    final_status = status;
                },
                Err(e) =>
                {
                    final_status = ExitStatus::failed();
                    job_execution.failures.push(e.to_string());
                    break;
                },
            }
        }

        // Update job execution
        job_execution.set_end_time(Utc::now());

        if final_status.code == "COMPLETED"
        {
            job_execution.set_status(JobStatus::Completed);
            job_execution.set_exit_status(ExitStatus::completed());
        }
        else
        {
            job_execution.set_status(JobStatus::Failed);
            job_execution.set_exit_status(final_status);
        }

        repository.update_job_execution(&job_execution).await?;

        // Notify listener
        if let Some(listener) = &self.listener
        {
            listener.after_job(&job_execution, &job_context).await?;
        }

        Ok(job_execution)
    }
}

/// Job builder
/// 作业构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// JobBuilder builder = new JobBuilder("jobName", jobRepository);
/// ```
pub struct JobBuilder
{
    name: String,
    steps: Vec<Arc<dyn AnyStep>>,
    allow_restart: bool,
    parameters: HashMap<String, String>,
    listener: Option<Arc<dyn JobListener>>,
}

impl JobBuilder
{
    /// Create new job builder
    /// 创建新作业构建器
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            steps: Vec::new(),
            allow_restart: true,
            parameters: HashMap::new(),
            listener: None,
        }
    }

    /// Add step to job
    /// 向作业添加步骤
    pub fn add_step<R, P, W>(mut self, step: Step<R, P, W>) -> Self
    where
        R: crate::reader::ItemReader + Send + Sync + 'static,
        R::Item: Send + Sync + 'static,
        P: crate::processor::ItemProcessor<Input = R::Item> + Send + Sync + 'static,
        P::Output: Send + Sync + 'static,
        W: crate::writer::ItemWriter<Item = P::Output> + Send + Sync + 'static,
    {
        self.steps.push(Arc::new(step));
        self
    }

    /// Set job parameters
    /// 设置作业参数
    pub fn with_parameters(mut self, params: HashMap<String, String>) -> Self
    {
        self.parameters = params;
        self
    }

    /// Add a single parameter
    /// 添加单个参数
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Set listener
    /// 设置监听器
    pub fn with_listener(mut self, listener: Arc<dyn JobListener>) -> Self
    {
        self.listener = Some(listener);
        self
    }

    /// Build job
    /// 构建作业
    pub fn build(self) -> BatchResult<Job>
    {
        if self.steps.is_empty()
        {
            return Err(BatchError::ValidationError {
                message: "Job must have at least one step".to_string(),
            });
        }

        Ok(Job {
            name: self.name,
            steps: self.steps,
            allow_restart: self.allow_restart,
            parameters: self.parameters,
            listener: self.listener,
        })
    }
}

/// Job listener
/// 作业监听器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface JobExecutionListener {
///     void beforeJob(JobExecution jobExecution);
///     void afterJob(JobExecution jobExecution);
/// }
/// ```
#[async_trait]
pub trait JobListener: Send + Sync
{
    /// Called before job execution
    /// 作业执行前调用
    async fn before_job(
        &self,
        job_execution: &JobExecution,
        context: &JobContext,
    ) -> BatchResult<()>
    {
        let _ = (job_execution, context);
        Ok(())
    }

    /// Called after job execution
    /// 作业执行后调用
    async fn after_job(&self, job_execution: &JobExecution, context: &JobContext)
    -> BatchResult<()>
    {
        let _ = (job_execution, context);
        Ok(())
    }

    /// Called on job error
    /// 作业错误时调用
    async fn on_error(&self, job_execution: &JobExecution, error: &BatchError) -> BatchResult<()>
    {
        let _ = (job_execution, error);
        Ok(())
    }
}

/// Simple job listener for logging
/// 用于日志记录的简单作业监听器
pub struct LoggingJobListener
{
    pub log_prefix: String,
}

impl LoggingJobListener
{
    pub fn new(prefix: impl Into<String>) -> Self
    {
        Self {
            log_prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl JobListener for LoggingJobListener
{
    async fn before_job(
        &self,
        job_execution: &JobExecution,
        _context: &JobContext,
    ) -> BatchResult<()>
    {
        tracing::info!("[{}] Starting job: {}", self.log_prefix, job_execution.job_name);
        Ok(())
    }

    async fn after_job(
        &self,
        job_execution: &JobExecution,
        _context: &JobContext,
    ) -> BatchResult<()>
    {
        tracing::info!(
            "[{}] Completed job: {} - status: {:?}",
            self.log_prefix,
            job_execution.job_name,
            job_execution.status
        );
        Ok(())
    }

    async fn on_error(&self, job_execution: &JobExecution, error: &BatchError) -> BatchResult<()>
    {
        tracing::error!("[{}] Error in job {}: {}", self.log_prefix, job_execution.job_name, error);
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use super::*;
    use crate::{reader::ItemStreamReader, step::StepBuilder, writer::ItemStreamWriter};

    #[tokio::test]
    async fn test_job_builder()
    {
        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("step1")
            .with_chunk_size(10)
            .build(reader, writer);

        let job = JobBuilder::new("test-job").add_step(step).build().unwrap();

        assert_eq!(job.name, "test-job");
        assert_eq!(job.steps.len(), 1);
    }

    #[tokio::test]
    async fn test_job_execute()
    {
        let reader = ItemStreamReader::new(vec![1, 2, 3, 4, 5]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("step1")
            .with_chunk_size(2)
            .build(reader, writer);

        let repository = InMemoryJobRepository::new();

        let job = JobBuilder::new("test-job").add_step(step).build().unwrap();

        let execution = job.execute_with_repository(&repository).await.unwrap();

        assert_eq!(execution.job_name, "test-job");
        assert_eq!(execution.status, JobStatus::Completed);
        assert_eq!(execution.step_executions.len(), 1);
    }

    #[tokio::test]
    async fn test_job_with_listener()
    {
        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("step1").build(reader, writer);

        let listener = Arc::new(LoggingJobListener::new("TEST"));

        let job = JobBuilder::new("test-job")
            .add_step(step)
            .with_listener(listener)
            .build()
            .unwrap();

        let repository = InMemoryJobRepository::new();
        let execution = job.execute_with_repository(&repository).await.unwrap();

        assert_eq!(execution.status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn test_job_builder_no_steps()
    {
        let result = JobBuilder::new("empty-job").build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_job_with_parameters()
    {
        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("step1").build(reader, writer);

        let mut params = HashMap::new();
        params.insert("input".to_string(), "data.csv".to_string());
        params.insert("output".to_string(), "result.csv".to_string());

        let job = JobBuilder::new("test-job")
            .add_step(step)
            .with_parameters(params.clone())
            .build()
            .unwrap();

        assert_eq!(job.parameters.get("input"), Some(&"data.csv".to_string()));
        assert_eq!(job.parameters.get("output"), Some(&"result.csv".to_string()));
    }
}
