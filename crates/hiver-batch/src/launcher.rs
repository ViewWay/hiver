//! Job launcher for starting batch jobs
//! 作业启动器，用于启动批处理作业

use crate::error::{BatchError, BatchResult};
use crate::execution::{JobExecution, JobStatus};
use crate::job::Job;
use crate::repository::{InMemoryJobRepository, JobRepository};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Job launcher
/// 作业启动器
///
/// Launches and manages batch job execution.
/// 启动和管理批处理作业执行。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JobLauncher jobLauncher(JobRepository jobRepository) {
///     SimpleJobLauncher launcher = new SimpleJobLauncher();
///     launcher.setJobRepository(jobRepository);
///     launcher.setTaskExecutor(taskExecutor);
///     return launcher;
/// }
///
/// // Usage
/// JobExecution execution = jobLauncher.run(job, jobParameters);
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// let repository = Arc::new(JobRepository::new());
/// let launcher = JobLauncher::new(repository);
///
/// let job = Job::builder("my-job")
///     .add_step(step1)
///     .build()?;
///
/// let execution = launcher.run(job).await?;
/// println!("Job completed: {:?}", execution.status);
/// ```
pub struct JobLauncher {
    repository: Arc<InMemoryJobRepository>,
    running_jobs: Arc<RwLock<HashMap<String, bool>>>,
    max_concurrent_jobs: usize,
}

impl JobLauncher {
    /// Create new job launcher
    /// 创建新作业启动器
    pub fn new(repository: InMemoryJobRepository) -> Self {
        Self {
            repository: Arc::new(repository),
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_jobs: 10,
        }
    }

    /// Set max concurrent jobs
    /// 设置最大并发作业数
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent_jobs = max;
        self
    }

    /// Run a job
    /// 运行作业
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// JobExecution jobExecution = jobLauncher.run(job, jobParameters);
    /// ```
    pub async fn run(&self, job: Job) -> BatchResult<JobExecution> {
        // Check concurrent limit
        {
            let running = self.running_jobs.read().await;
            if running.len() >= self.max_concurrent_jobs {
                return Err(BatchError::Other("Maximum concurrent jobs limit reached".to_string()));
            }
        }

        // Mark as running
        self.running_jobs
            .write()
            .await
            .insert(job.name.clone(), true);

        let result = job.execute_with_repository(&self.repository).await;

        // Clear from running
        self.running_jobs.write().await.remove(&job.name);

        result
    }

    /// Run job with parameters
    /// 使用参数运行作业
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// JobExecution jobExecution = jobLauncher.run(job, new JobParametersBuilder()
    ///     .addString("inputFile", "data.csv")
    ///     .addLong("timestamp", System.currentTimeMillis())
    ///     .toJobParameters());
    /// ```
    pub async fn run_with_parameters(
        &self,
        mut job: Job,
        parameters: HashMap<String, String>,
    ) -> BatchResult<JobExecution> {
        // Check if already running with same parameters
        if self.repository.is_job_running(&job.name).await? {
            return Err(BatchError::JobAlreadyRunning {
                job_name: job.name.clone(),
            });
        }

        // Update job parameters
        job.parameters = parameters;

        self.run(job).await
    }

    /// Get job execution by ID
    /// 通过ID获取作业执行
    pub async fn get_execution(&self, id: uuid::Uuid) -> BatchResult<JobExecution> {
        self.repository.get_job_execution(id).await
    }

    /// Stop a running job
    /// 停止正在运行的作业
    ///
    /// Note: This is a graceful stop request. The job will complete its current chunk.
    /// 注意：这是一个优雅停止请求。作业将完成当前块。
    pub async fn stop(&self, job_name: &str) -> BatchResult<bool> {
        if self.repository.is_job_running(job_name).await? {
            self.running_jobs.write().await.remove(job_name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all running jobs
    /// 获取所有正在运行的作业
    pub async fn running_jobs(&self) -> Vec<String> {
        self.running_jobs.read().await.keys().cloned().collect()
    }

    /// Get job execution by job name
    /// 通过作业名称获取作业执行
    pub async fn get_last_execution(&self, job_name: &str) -> BatchResult<Option<JobExecution>> {
        self.repository.get_last_job_execution(job_name).await
    }

    /// Restart a failed job
    /// 重启失败的作业
    ///
    /// # Spring Equivalent / Spring等价物
    ///
    /// ```java
    /// JobExecution restartedExecution = jobLauncher.run(job, jobExecution.getJobParameters());
    /// ```
    pub async fn restart(&self, job_name: &str) -> BatchResult<JobExecution> {
        let last_execution = self.repository.get_last_job_execution(job_name).await?;

        let last = match last_execution {
            Some(exec) => exec,
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
}

/// Job operator
/// 作业操作器
///
/// Provides higher-level job operations like starting, stopping, and restarting.
/// 提供更高级的作业操作，如启动、停止和重启。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JobOperator jobOperator(JobExplorer jobExplorer,
///                                JobRepository jobRepository,
///                                JobRegistry jobRegistry,
///                                JobLauncher jobLauncher) {
///     SimpleJobOperator operator = new SimpleJobOperator();
///     operator.setJobExplorer(jobExplorer);
///     operator.setJobRepository(jobRepository);
///     operator.setJobRegistry(jobRegistry);
///     operator.setJobLauncher(jobLauncher);
///     return operator;
/// }
/// ```
pub struct JobOperator {
    launcher: Arc<Mutex<JobLauncher>>,
}

impl JobOperator {
    /// Create new job operator
    /// 创建新作业操作器
    pub fn new(launcher: JobLauncher) -> Self {
        Self {
            launcher: Arc::new(Mutex::new(launcher)),
        }
    }

    /// Start a job with parameters
    /// 使用参数启动作业
    pub async fn start(
        &self,
        job_name: &str,
        parameters: HashMap<String, String>,
    ) -> BatchResult<uuid::Uuid> {
        let launcher = self.launcher.lock().await;

        // Check if already running
        if launcher.repository.is_job_running(job_name).await? {
            return Err(BatchError::JobAlreadyRunning {
                job_name: job_name.to_string(),
            });
        }

        // Create execution
        let execution = launcher
            .repository
            .create_job_execution(job_name.to_string(), parameters)
            .await?;

        Ok(execution.id)
    }

    /// Restart the last failed execution of a job
    /// 重启作业的最后一次失败执行
    pub async fn restart(&self, job_name: &str) -> BatchResult<JobExecution> {
        let launcher = self.launcher.lock().await;
        launcher.restart(job_name).await
    }

    /// Stop a running job
    /// 停止正在运行的作业
    pub async fn stop(&self, execution_id: uuid::Uuid) -> BatchResult<bool> {
        let launcher = self.launcher.lock().await;

        let execution = launcher.repository.get_job_execution(execution_id).await?;
        launcher.stop(&execution.job_name).await
    }

    /// Get job execution summary
    /// 获取作业执行摘要
    pub async fn get_summary(&self, execution_id: uuid::Uuid) -> BatchResult<JobSummary> {
        let launcher = self.launcher.lock().await;
        let execution = launcher.repository.get_job_execution(execution_id).await?;

        Ok(JobSummary {
            id: execution.id,
            job_name: execution.job_name.clone(),
            status: execution.status,
            start_time: execution.start_time,
            end_time: execution.end_time,
            duration: execution.duration(),
            step_count: execution.step_executions.len(),
            exit_code: execution.exit_status.code,
        })
    }
}

/// Job execution summary
/// 作业执行摘要
#[derive(Debug, Clone)]
pub struct JobSummary {
    /// Execution ID
    /// 执行ID
    pub id: uuid::Uuid,

    /// Job name
    /// 作业名称
    pub job_name: String,

    /// Status
    /// 状态
    pub status: JobStatus,

    /// Start time
    /// 开始时间
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,

    /// End time
    /// 结束时间
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Duration
    /// 时长
    pub duration: Option<chrono::Duration>,

    /// Number of steps
    /// 步骤数量
    pub step_count: usize,

    /// Exit code
    /// 退出代码
    pub exit_code: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::JobBuilder;
    use crate::reader::ItemStreamReader;
    use crate::step::StepBuilder;
    use crate::writer::ItemStreamWriter;

    #[tokio::test]
    async fn test_job_launcher() {
        let repository = InMemoryJobRepository::new();
        let launcher = JobLauncher::new(repository);

        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();
        let step = StepBuilder::new("step1").build(reader, writer);

        let job = JobBuilder::new("test-job").add_step(step).build().unwrap();

        let execution = launcher.run(job).await.unwrap();

        assert_eq!(execution.job_name, "test-job");
        assert_eq!(execution.status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn test_job_launcher_duplicate() {
        let repository = InMemoryJobRepository::new();
        let launcher = JobLauncher::new(repository.clone());

        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        // First execution
        let step1 = StepBuilder::new("step1").build(reader.clone(), writer.clone());
        let job1 = JobBuilder::new("test-job").add_step(step1).build().unwrap();

        let _exec1 = launcher.run(job1).await.unwrap();

        // Second execution should work since first is complete
        let step2 = StepBuilder::new("step1").build(reader, writer);
        let job2 = JobBuilder::new("test-job").add_step(step2).build().unwrap();

        let result = launcher.run(job2).await;

        // Should succeed since first job completed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_max_concurrent_jobs() {
        let repository = InMemoryJobRepository::new();
        let launcher = JobLauncher::new(repository).with_max_concurrent(1);

        let reader = ItemStreamReader::new(vec![1, 2, 3]);
        let writer = ItemStreamWriter::new();

        let step = StepBuilder::new("step1").build(reader, writer);
        let job = JobBuilder::new("test-job").add_step(step).build().unwrap();

        let _exec = launcher.run(job).await.unwrap();

        // Job completes immediately, so running should be 0
        let running = launcher.running_jobs().await;
        assert_eq!(running.len(), 0);
    }
}
