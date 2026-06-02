//! Job and step execution tracking
//! 作业和步骤执行跟踪

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

/// Batch execution status
/// 批处理执行状态
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// enum BatchStatus {
///     COMPLETED, STARTING, STARTED, STOPPING, STOPPED,
///     FAILED, ABANDONED, UNKNOWN
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {
    /// Job/Step is starting
    /// 作业/步骤正在启动
    Starting,

    /// Job/Step has started
    /// 作业/步骤已启动
    Started,

    /// Job/Step is executing
    /// 作业/步骤正在执行
    Executing,

    /// Job/Step completed successfully
    /// 作业/步骤成功完成
    Completed,

    /// Job/Step failed
    /// 作业/步骤失败
    Failed,

    /// Job/Step was stopped
    /// 作业/步骤被停止
    Stopped,

    /// Job/Step was abandoned
    /// 作业/步骤被放弃
    Abandoned,

    /// Status is unknown
    /// 状态未知
    Unknown,
}

impl BatchStatus {
    /// Check if status is terminal (no further transitions)
    /// 检查状态是否为终止状态（无法再转换）
    pub fn is_terminal(self) -> bool {
        matches!(self, BatchStatus::Completed | BatchStatus::Failed | BatchStatus::Abandoned)
    }

    /// Check if status is running
    /// 检查状态是否为运行中
    pub fn is_running(self) -> bool {
        matches!(self, BatchStatus::Starting | BatchStatus::Started | BatchStatus::Executing)
    }

    /// Check if status is unsuccessful
    /// 检查状态是否为不成功
    pub fn is_unsuccessful(self) -> bool {
        matches!(self, BatchStatus::Failed | BatchStatus::Abandoned)
    }
}

/// Job execution status
/// 作业执行状态
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// enum JobExecutionStatus {
///     STARTING, IN_PROGRESS, COMPLETED, FAILED, STOPPED
/// }
/// ```
pub type JobStatus = BatchStatus;

/// Exit status for job/step
/// 作业/步骤的退出状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitStatus {
    /// Exit code
    /// 退出代码
    pub code: String,

    /// Exit description
    /// 退出描述
    pub description: Option<String>,
}

impl ExitStatus {
    /// Create new exit status
    /// 创建新退出状态
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: None,
        }
    }

    /// Set description
    /// 设置描述
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Completed exit status
    /// 完成退出状态
    pub fn completed() -> Self {
        Self::new("COMPLETED").with_description("Job completed successfully")
    }

    /// Failed exit status
    /// 失败退出状态
    pub fn failed() -> Self {
        Self::new("FAILED").with_description("Job failed")
    }

    /// Stopped exit status
    /// 停止退出状态
    pub fn stopped() -> Self {
        Self::new("STOPPED").with_description("Job was stopped")
    }

    /// Unknown exit status
    /// 未知退出状态
    pub fn unknown() -> Self {
        Self::new("UNKNOWN").with_description("Unknown exit status")
    }
}

/// Job execution instance
/// 作业执行实例
///
/// Tracks the execution of a single job instance.
/// 跟踪单个作业实例的执行。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// JobExecution jobExecution = jobRepository.createJobExecution("jobName", jobParameters);
/// jobExecution.setStatus(BatchStatus.STARTED);
/// jobExecution.setStartTime(new Date());
/// ```
#[derive(Debug, Clone)]
pub struct JobExecution {
    /// Unique execution ID
    /// 唯一执行ID
    pub id: Uuid,

    /// Job name
    /// 作业名称
    pub job_name: String,

    /// Job instance ID
    /// 作业实例ID
    pub job_instance_id: Uuid,

    /// Current status
    /// 当前状态
    pub status: JobStatus,

    /// Exit status
    /// 退出状态
    pub exit_status: ExitStatus,

    /// Start time
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,

    /// End time
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// Step executions
    /// 步骤执行列表
    pub step_executions: Vec<StepExecution>,

    /// Job parameters
    /// 作业参数
    pub parameters: HashMap<String, String>,

    /// Failure exceptions
    /// 失败异常
    pub failures: Vec<String>,
}

impl JobExecution {
    /// Create new job execution
    /// 创建新作业执行
    pub fn new(job_name: impl Into<String>, job_instance_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_name: job_name.into(),
            job_instance_id,
            status: JobStatus::Starting,
            exit_status: ExitStatus::unknown(),
            start_time: None,
            end_time: None,
            step_executions: Vec::new(),
            parameters: HashMap::new(),
            failures: Vec::new(),
        }
    }

    /// Set status
    /// 设置状态
    pub fn set_status(&mut self, status: JobStatus) {
        self.status = status;
    }

    /// Set start time
    /// 设置开始时间
    pub fn set_start_time(&mut self, time: DateTime<Utc>) {
        self.start_time = Some(time);
    }

    /// Set end time
    /// 设置结束时间
    pub fn set_end_time(&mut self, time: DateTime<Utc>) {
        self.end_time = Some(time);
    }

    /// Set exit status
    /// 设置退出状态
    pub fn set_exit_status(&mut self, exit_status: ExitStatus) {
        self.exit_status = exit_status;
    }

    /// Add step execution
    /// 添加步骤执行
    pub fn add_step_execution(&mut self, step_execution: StepExecution) {
        self.step_executions.push(step_execution);
    }

    /// Add failure
    /// 添加失败
    pub fn add_failure(&mut self, error: String) {
        self.failures.push(error);
    }

    /// Get duration
    /// 获取执行时长
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Get all step executions
    /// 获取所有步骤执行
    pub fn step_executions(&self) -> &[StepExecution] {
        &self.step_executions
    }

    /// Check if execution is running
    /// 检查执行是否正在运行
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }
}

/// Step execution instance
/// 步骤执行实例
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// StepExecution stepExecution = new StepExecution("stepName", jobExecution);
/// stepExecution.setStartTime(new Date());
/// stepExecution.setReadCount(100);
/// stepExecution.setWriteCount(100);
/// ```
#[derive(Debug, Clone)]
pub struct StepExecution {
    /// Unique execution ID
    /// 唯一执行ID
    pub id: Uuid,

    /// Step name
    /// 步骤名称
    pub step_name: String,

    /// Job execution ID
    /// 作业执行ID
    pub job_execution_id: Uuid,

    /// Current status
    /// 当前状态
    pub status: BatchStatus,

    /// Exit status
    /// 退出状态
    pub exit_status: ExitStatus,

    /// Start time
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,

    /// End time
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// Number of items read
    /// 读取的项目数
    pub read_count: Arc<AtomicUsize>,

    /// Number of items written
    /// 写入的项目数
    pub write_count: Arc<AtomicUsize>,

    /// Number of items processed
    /// 处理的项目数
    pub process_count: Arc<AtomicUsize>,

    /// Number of items skipped
    /// 跳过的项目数
    pub skip_count: Arc<AtomicUsize>,

    /// Number of filter/commit counts
    /// 过滤/提交计数
    pub filter_count: Arc<AtomicUsize>,

    /// Number of rollbacks
    /// 回滚次数
    pub rollback_count: Arc<AtomicUsize>,
}

impl StepExecution {
    /// Create new step execution
    /// 创建新步骤执行
    pub fn new(step_name: impl Into<String>, job_execution_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            step_name: step_name.into(),
            job_execution_id,
            status: BatchStatus::Starting,
            exit_status: ExitStatus::unknown(),
            start_time: None,
            end_time: None,
            read_count: Arc::new(AtomicUsize::new(0)),
            write_count: Arc::new(AtomicUsize::new(0)),
            process_count: Arc::new(AtomicUsize::new(0)),
            skip_count: Arc::new(AtomicUsize::new(0)),
            filter_count: Arc::new(AtomicUsize::new(0)),
            rollback_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Set status
    /// 设置状态
    pub fn set_status(&mut self, status: BatchStatus) {
        self.status = status;
    }

    /// Set start time
    /// 设置开始时间
    pub fn set_start_time(&mut self, time: DateTime<Utc>) {
        self.start_time = Some(time);
    }

    /// Set end time
    /// 设置结束时间
    pub fn set_end_time(&mut self, time: DateTime<Utc>) {
        self.end_time = Some(time);
    }

    /// Set exit status
    /// 设置退出状态
    pub fn set_exit_status(&mut self, exit_status: ExitStatus) {
        self.exit_status = exit_status;
    }

    /// Increment read count
    /// 增加读取计数
    pub fn increment_read_count(&self) -> usize {
        self.read_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Increment write count
    /// 增加写入计数
    pub fn increment_write_count(&self, amount: usize) -> usize {
        self.write_count.fetch_add(amount, Ordering::SeqCst) + amount
    }

    /// Increment process count
    /// 增加处理计数
    pub fn increment_process_count(&self) -> usize {
        self.process_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Increment skip count
    /// 增加跳过计数
    pub fn increment_skip_count(&self) -> usize {
        self.skip_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Increment filter count
    /// 增加过滤计数
    pub fn increment_filter_count(&self) -> usize {
        self.filter_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Increment rollback count
    /// 增加回滚计数
    pub fn increment_rollback_count(&self) -> usize {
        self.rollback_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Get read count
    /// 获取读取计数
    pub fn read_count(&self) -> usize {
        self.read_count.load(Ordering::SeqCst)
    }

    /// Get write count
    /// 获取写入计数
    pub fn write_count(&self) -> usize {
        self.write_count.load(Ordering::SeqCst)
    }

    /// Get process count
    /// 获取处理计数
    pub fn process_count(&self) -> usize {
        self.process_count.load(Ordering::SeqCst)
    }

    /// Get skip count
    /// 获取跳过计数
    pub fn skip_count(&self) -> usize {
        self.skip_count.load(Ordering::SeqCst)
    }

    /// Get filter count
    /// 获取过滤计数
    pub fn filter_count(&self) -> usize {
        self.filter_count.load(Ordering::SeqCst)
    }

    /// Get rollback count
    /// 获取回滚计数
    pub fn rollback_count(&self) -> usize {
        self.rollback_count.load(Ordering::SeqCst)
    }

    /// Get duration
    /// 获取执行时长
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Check if execution is running
    /// 检查执行是否正在运行
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_status() {
        assert!(!BatchStatus::Starting.is_terminal());
        assert!(BatchStatus::Starting.is_running());
        assert!(!BatchStatus::Starting.is_unsuccessful());

        assert!(BatchStatus::Completed.is_terminal());
        assert!(!BatchStatus::Completed.is_running());
        assert!(!BatchStatus::Completed.is_unsuccessful());

        assert!(BatchStatus::Failed.is_terminal());
        assert!(!BatchStatus::Failed.is_running());
        assert!(BatchStatus::Failed.is_unsuccessful());
    }

    #[test]
    fn test_exit_status() {
        let completed = ExitStatus::completed();
        assert_eq!(completed.code, "COMPLETED");
        assert!(completed.description.is_some());

        let failed = ExitStatus::failed().with_description("Custom failure");
        assert_eq!(failed.code, "FAILED");
        assert_eq!(failed.description, Some("Custom failure".to_string()));
    }

    #[test]
    fn test_job_execution() {
        let instance_id = Uuid::new_v4();
        let mut execution = JobExecution::new("test-job", instance_id);

        assert_eq!(execution.job_name, "test-job");
        assert_eq!(execution.job_instance_id, instance_id);
        assert!(execution.is_running());

        execution.set_status(JobStatus::Completed);
        execution.set_start_time(Utc::now());
        execution.set_end_time(Utc::now());

        assert!(!execution.is_running());
        assert!(execution.duration().is_some());
    }

    #[test]
    fn test_step_execution_counters() {
        let job_exec_id = Uuid::new_v4();
        let step_execution = StepExecution::new("test-step", job_exec_id);

        assert_eq!(step_execution.read_count(), 0);
        assert_eq!(step_execution.write_count(), 0);

        step_execution.increment_read_count();
        step_execution.increment_write_count(10);

        assert_eq!(step_execution.read_count(), 1);
        assert_eq!(step_execution.write_count(), 10);
    }
}
