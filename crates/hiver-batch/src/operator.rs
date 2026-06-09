//! Advanced Job Operator for managing running jobs
//! 高级作业操作器，用于管理运行中的作业
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Batch |
//! |-------|-------------|
//! | `JobOperator` | `JobOperator` |
//! | `JobExplorer` | `JobExplorer` |
//! | `JobRegistry` | `JobRegistry` |
//!
//! The JobOperator provides higher-level operations for managing batch jobs,
//! including inspecting execution history, stopping/restarting jobs,
//! and abandoning failed executions.
//!
//! JobOperator 提供更高级的批处理作业管理操作，
//! 包括检查执行历史、停止/重启作业以及放弃失败的执行。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_batch::operator::AdvancedJobOperator;
//!
//! let operator = AdvancedJobOperator::new(repository);
//!
//! // Get execution summary
//! let summary = operator.get_job_summary("import-users").await?;
//!
//! // Abandon a failed execution
//! operator.abandon(execution_id).await?;
//! ```

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::{BatchError, BatchResult},
    execution::{BatchStatus, ExitStatus, JobExecution, JobStatus},
    repository::{InMemoryJobRepository, JobRepository},
};

// ──────────────────────────────────────────────────────────────────────────────
// Advanced Job Operator
// ──────────────────────────────────────────────────────────────────────────────

/// Advanced job operator — manage running jobs with full lifecycle control.
/// 高级作业操作器 — 通过完整生命周期控制管理运行中的作业。
///
/// Extends the basic `JobOperator` with job exploration and registry features.
/// Equivalent to Spring Batch's `SimpleJobOperator` combined with `JobExplorer`.
///
/// 扩展基本 `JobOperator`，增加作业探索和注册功能。
/// 等价于 Spring Batch 的 `SimpleJobOperator` 与 `JobExplorer` 的组合。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JobOperator jobOperator(
///     JobExplorer jobExplorer,
///     JobRepository jobRepository,
///     JobRegistry jobRegistry,
///     JobLauncher jobLauncher
/// ) {
///     SimpleJobOperator operator = new SimpleJobOperator();
///     operator.setJobExplorer(jobExplorer);
///     operator.setJobRepository(jobRepository);
///     operator.setJobRegistry(jobRegistry);
///     operator.setJobLauncher(jobLauncher);
///     return operator;
/// }
/// ```
pub struct AdvancedJobOperator {
    repository: Arc<InMemoryJobRepository>,
    running_jobs: Arc<RwLock<HashMap<Uuid, JobExecution>>>,
    job_registry: Arc<RwLock<HashMap<String, RegisteredJob>>>,
}

/// A registered job entry.
/// 已注册的作业条目。
#[derive(Debug, Clone)]
pub struct RegisteredJob {
    /// Job name
    /// 作业名称
    pub name: String,

    /// Whether the job is restartable
    /// 作业是否可重启
    pub restartable: bool,

    /// Registration time
    /// 注册时间
    pub registered_at: DateTime<Utc>,
}

impl AdvancedJobOperator {
    /// Create a new advanced job operator.
    /// 创建新的高级作业操作器。
    pub fn new(repository: InMemoryJobRepository) -> Self {
        Self {
            repository: Arc::new(repository),
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            job_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create from an existing shared repository.
    /// 从现有共享存储库创建。
    pub fn with_shared(repository: Arc<InMemoryJobRepository>) -> Self {
        Self {
            repository,
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            job_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ── Job Registry operations ─────────────────────────────────────────

    /// Register a job name so it can be restarted later.
    /// 注册作业名称，以便稍后可以重启。
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// jobRegistry.register(new ReferenceJobFactory(job));
    /// ```
    pub async fn register_job(&self, name: &str, restartable: bool) {
        let mut registry = self.job_registry.write().await;
        registry.insert(
            name.to_string(),
            RegisteredJob {
                name: name.to_string(),
                restartable,
                registered_at: Utc::now(),
            },
        );
    }

    /// Unregister a job by name.
    /// 按名称取消注册作业。
    pub async fn unregister_job(&self, name: &str) -> bool {
        let mut registry = self.job_registry.write().await;
        registry.remove(name).is_some()
    }

    /// List all registered job names.
    /// 列出所有已注册的作业名称。
    pub async fn registered_jobs(&self) -> Vec<String> {
        let registry = self.job_registry.read().await;
        registry.keys().cloned().collect()
    }

    /// Check if a job is registered.
    /// 检查作业是否已注册。
    pub async fn is_registered(&self, name: &str) -> bool {
        let registry = self.job_registry.read().await;
        registry.contains_key(name)
    }

    // ── Job Exploration ─────────────────────────────────────────────────

    /// Get a job execution by its ID.
    /// 通过 ID 获取作业执行。
    pub async fn get_execution(&self, execution_id: Uuid) -> BatchResult<JobExecution> {
        self.repository.get_job_execution(execution_id).await
    }

    /// Get the last execution of a job by name.
    /// 通过名称获取作业的最后一次执行。
    pub async fn get_last_execution(&self, job_name: &str) -> BatchResult<Option<JobExecution>> {
        self.repository.get_last_job_execution(job_name).await
    }

    /// Get a summary of a job execution.
    /// 获取作业执行的摘要。
    pub async fn get_job_summary(
        &self,
        job_name: &str,
    ) -> BatchResult<Option<JobExecutionSummary>> {
        let execution = self.repository.get_last_job_execution(job_name).await?;
        Ok(execution.map(|e| JobExecutionSummary::from_execution(&e)))
    }

    /// Get summaries for all executions of a job.
    /// 获取作业所有执行的摘要。
    pub async fn get_job_executions(
        &self,
        job_name: &str,
    ) -> BatchResult<Vec<JobExecutionSummary>> {
        // In the in-memory repo, we only store the last execution per job name
        let last = self.repository.get_last_job_execution(job_name).await?;
        Ok(last
            .map(|e| vec![JobExecutionSummary::from_execution(&e)])
            .unwrap_or_default())
    }

    // ── Lifecycle Operations ────────────────────────────────────────────

    /// Stop a running job execution.
    /// 停止运行中的作业执行。
    ///
    /// Requests a graceful stop. The job will complete its current chunk
    /// before stopping.
    ///
    /// 请求优雅停止。作业将在停止前完成当前块。
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// jobOperator.stop(executionId);
    /// ```
    pub async fn stop(&self, execution_id: Uuid) -> BatchResult<()> {
        let mut running = self.running_jobs.write().await;
        if let Some(mut execution) = running.remove(&execution_id) {
            execution.set_status(JobStatus::Stopped);
            execution.set_end_time(Utc::now());
            execution.set_exit_status(ExitStatus::stopped());
            Ok(())
        } else {
            Err(BatchError::NotFound {
                resource: "JobExecution".to_string(),
                id: execution_id.to_string(),
            })
        }
    }

    /// Restart a failed or stopped job execution.
    /// 重启失败或已停止的作业执行。
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// jobOperator.restart(executionId);
    /// ```
    pub async fn restart(&self, job_name: &str) -> BatchResult<JobExecution> {
        let last = self.repository.get_last_job_execution(job_name).await?;
        let last = match last {
            Some(e) => e,
            None => {
                return Err(BatchError::NotFound {
                    resource: "JobExecution".to_string(),
                    id: job_name.to_string(),
                });
            },
        };

        if last.status == JobStatus::Completed {
            return Err(BatchError::JobAlreadyComplete {
                job_name: job_name.to_string(),
            });
        }

        if last.status.is_running() {
            return Err(BatchError::JobAlreadyRunning {
                job_name: job_name.to_string(),
            });
        }

        // Create new execution with same parameters
        let execution = self
            .repository
            .create_job_execution(job_name.to_string(), last.parameters)
            .await?;

        Ok(execution)
    }

    /// Abandon a failed job execution (mark as abandoned, no longer restartable).
    /// 放弃失败的作业执行（标记为已放弃，不再可重启）。
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// jobOperator.abandon(executionId);
    /// ```
    pub async fn abandon(&self, execution_id: Uuid) -> BatchResult<()> {
        let mut execution = self.repository.get_job_execution(execution_id).await?;

        if execution.status.is_running() {
            return Err(BatchError::Other(
                "Cannot abandon a running execution; stop it first".to_string(),
            ));
        }

        execution.set_status(BatchStatus::Abandoned);
        execution.set_end_time(Utc::now());
        execution.set_exit_status(ExitStatus::new("ABANDONED").with_description("Job abandoned"));
        Ok(())
    }

    /// Get the status of a job execution.
    /// 获取作业执行的状态。
    pub async fn get_status(&self, execution_id: Uuid) -> BatchResult<BatchStatus> {
        let execution = self.repository.get_job_execution(execution_id).await?;
        Ok(execution.status)
    }

    /// List all currently running job names.
    /// 列出所有当前运行中的作业名称。
    pub async fn running_job_names(&self) -> Vec<String> {
        let running = self.running_jobs.read().await;
        running.values().map(|e| e.job_name.clone()).collect()
    }

    /// Count the number of running jobs.
    /// 计算运行中的作业数量。
    pub async fn running_count(&self) -> usize {
        let running = self.running_jobs.read().await;
        running.len()
    }

    /// Register a running job execution for tracking.
    /// 注册运行中的作业执行以进行跟踪。
    pub async fn mark_running(&self, execution: JobExecution) {
        let mut running = self.running_jobs.write().await;
        running.insert(execution.id, execution);
    }

    /// Remove a job from the running set (e.g., after completion).
    /// 从运行集合中移除作业（例如完成后）。
    pub async fn mark_completed(&self, execution_id: Uuid) {
        let mut running = self.running_jobs.write().await;
        running.remove(&execution_id);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Job Execution Summary
// ──────────────────────────────────────────────────────────────────────────────

/// Summary of a job execution for quick inspection.
/// 作业执行摘要，用于快速检查。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Equivalent to querying JobExplorer for execution details
/// JobExecution execution = jobExplorer.getJobExecution(executionId);
/// ```
#[derive(Debug, Clone)]
pub struct JobExecutionSummary {
    /// Execution ID
    /// 执行ID
    pub id: Uuid,

    /// Job name
    /// 作业名称
    pub job_name: String,

    /// Job instance ID
    /// 作业实例ID
    pub job_instance_id: Uuid,

    /// Current status
    /// 当前状态
    pub status: BatchStatus,

    /// Exit code
    /// 退出代码
    pub exit_code: String,

    /// Start time
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,

    /// End time
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// Duration
    /// 持续时间
    pub duration: Option<chrono::Duration>,

    /// Number of step executions
    /// 步骤执行数量
    pub step_count: usize,

    /// Total items read across all steps
    /// 所有步骤中读取的总项目数
    pub total_read_count: usize,

    /// Total items written across all steps
    /// 所有步骤中写入的总项目数
    pub total_write_count: usize,

    /// Total items skipped across all steps
    /// 所有步骤中跳过的总项目数
    pub total_skip_count: usize,
}

impl JobExecutionSummary {
    /// Create a summary from a JobExecution.
    /// 从 JobExecution 创建摘要。
    pub fn from_execution(execution: &JobExecution) -> Self {
        let total_read_count: usize = execution
            .step_executions
            .iter()
            .map(|s| s.read_count())
            .sum();
        let total_write_count: usize = execution
            .step_executions
            .iter()
            .map(|s| s.write_count())
            .sum();
        let total_skip_count: usize = execution
            .step_executions
            .iter()
            .map(|s| s.skip_count())
            .sum();

        Self {
            id: execution.id,
            job_name: execution.job_name.clone(),
            job_instance_id: execution.job_instance_id,
            status: execution.status,
            exit_code: execution.exit_status.code.clone(),
            start_time: execution.start_time,
            end_time: execution.end_time,
            duration: execution.duration(),
            step_count: execution.step_executions.len(),
            total_read_count,
            total_write_count,
            total_skip_count,
        }
    }

    /// Check if the execution is running.
    /// 检查执行是否正在运行。
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    /// Check if the execution is finished (terminal state).
    /// 检查执行是否已完成（终止状态）。
    pub fn is_finished(&self) -> bool {
        self.status.is_terminal()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Chunk Processing
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for chunk-oriented processing with fault tolerance.
/// 带容错的面向块处理配置。
///
/// Equivalent to Spring Batch's `chunk()` processing with skip and retry.
/// 等价于 Spring Batch 的 `chunk()` 处理（带跳过和重试）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// stepBuilderFactory.get("step1")
///     .<User, User>chunk(100)
///     .reader(reader)
///     .processor(processor)
///     .writer(writer)
///     .faultTolerant()
///     .skipLimit(10)
///     .skip(ValidationException.class)
///     .retryLimit(3)
///     .retry(TransientException.class)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct FaultTolerantStep {
    /// Step name
    /// 步骤名称
    pub name: String,

    /// Chunk size (number of items per read-process-write cycle)
    /// 块大小（每次读-处理-写循环的项目数量）
    pub chunk_size: usize,

    /// Maximum number of items to skip on read/process/write errors
    /// 读取/处理/写入错误时允许跳过的最大项目数
    pub skip_limit: usize,

    /// Maximum number of retry attempts per item
    /// 每个项目的最大重试次数
    pub retry_limit: usize,

    /// Whether to skip on read errors
    /// 是否在读取错误时跳过
    pub skip_on_read_error: bool,

    /// Whether to skip on process errors
    /// 是否在处理错误时跳过
    pub skip_on_process_error: bool,

    /// Whether to skip on write errors
    /// 是否在写入错误时跳过
    pub skip_on_write_error: bool,

    /// Timeout in seconds for the entire step
    /// 整个步骤的超时时间（秒）
    pub timeout_secs: Option<u64>,
}

impl FaultTolerantStep {
    /// Create a new fault-tolerant step configuration.
    /// 创建新的容错步骤配置。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chunk_size: 100,
            skip_limit: 0,
            retry_limit: 0,
            skip_on_read_error: false,
            skip_on_process_error: true,
            skip_on_write_error: false,
            timeout_secs: None,
        }
    }

    /// Set the chunk size.
    /// 设置块大小。
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the skip limit.
    /// 设置跳过限制。
    pub fn with_skip_limit(mut self, limit: usize) -> Self {
        self.skip_limit = limit;
        self
    }

    /// Set the retry limit.
    /// 设置重试限制。
    pub fn with_retry_limit(mut self, limit: usize) -> Self {
        self.retry_limit = limit;
        self
    }

    /// Enable skipping on read errors.
    /// 启用读取错误时跳过。
    pub fn skip_on_read_error(mut self) -> Self {
        self.skip_on_read_error = true;
        self
    }

    /// Enable skipping on write errors.
    /// 启用写入错误时跳过。
    pub fn skip_on_write_error(mut self) -> Self {
        self.skip_on_write_error = true;
        self
    }

    /// Set step timeout.
    /// 设置步骤超时。
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_job_operator_creation() {
        let repo = InMemoryJobRepository::new();
        let _operator = AdvancedJobOperator::new(repo);
    }

    #[tokio::test]
    async fn test_job_registry() {
        let repo = InMemoryJobRepository::new();
        let operator = AdvancedJobOperator::new(repo);

        assert!(!operator.is_registered("import-users").await);

        operator.register_job("import-users", true).await;
        assert!(operator.is_registered("import-users").await);

        let jobs = operator.registered_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert!(jobs.contains(&"import-users".to_string()));

        let removed = operator.unregister_job("import-users").await;
        assert!(removed);
        assert!(!operator.is_registered("import-users").await);
    }

    #[tokio::test]
    async fn test_running_jobs_tracking() {
        let repo = InMemoryJobRepository::new();
        let operator = AdvancedJobOperator::new(repo);

        assert_eq!(operator.running_count().await, 0);

        let _instance_id = Uuid::new_v4();
        let mut execution = JobExecution::new("test-job", _instance_id);
        execution.set_status(JobStatus::Executing);
        let exec_id = execution.id;

        operator.mark_running(execution).await;
        assert_eq!(operator.running_count().await, 1);

        let names = operator.running_job_names().await;
        assert!(names.contains(&"test-job".to_string()));

        operator.mark_completed(exec_id).await;
        assert_eq!(operator.running_count().await, 0);
    }

    #[tokio::test]
    async fn test_abandon_non_running() {
        let repo = InMemoryJobRepository::new();
        let operator = AdvancedJobOperator::new(repo);

        // Create a failed execution
        let _instance_id = Uuid::new_v4();
        let mut execution = operator
            .repository
            .create_job_execution("test-job".to_string(), HashMap::new())
            .await
            .unwrap();

        // Manually mark as failed so abandon can proceed
        execution.set_status(JobStatus::Failed);
        operator
            .repository
            .update_job_execution(&execution)
            .await
            .unwrap();

        // Abandon should succeed for non-running failed executions
        let result = operator.abandon(execution.id).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_fault_tolerant_step_defaults() {
        let step = FaultTolerantStep::new("test-step");
        assert_eq!(step.chunk_size, 100);
        assert_eq!(step.skip_limit, 0);
        assert_eq!(step.retry_limit, 0);
        assert!(!step.skip_on_read_error);
        assert!(step.skip_on_process_error);
        assert!(!step.skip_on_write_error);
    }

    #[test]
    fn test_fault_tolerant_step_builder() {
        let step = FaultTolerantStep::new("test-step")
            .with_chunk_size(50)
            .with_skip_limit(10)
            .with_retry_limit(3)
            .skip_on_read_error()
            .skip_on_write_error()
            .with_timeout(300);

        assert_eq!(step.chunk_size, 50);
        assert_eq!(step.skip_limit, 10);
        assert_eq!(step.retry_limit, 3);
        assert!(step.skip_on_read_error);
        assert!(step.skip_on_write_error);
        assert_eq!(step.timeout_secs, Some(300));
    }

    #[test]
    fn test_job_execution_summary() {
        let instance_id = Uuid::new_v4();
        let execution = JobExecution::new("test-job", instance_id);

        let summary = JobExecutionSummary::from_execution(&execution);
        assert_eq!(summary.job_name, "test-job");
        assert_eq!(summary.step_count, 0);
        assert_eq!(summary.total_read_count, 0);
        assert!(!summary.is_finished());
        assert!(summary.is_running());
    }
}
