//! Job repository for storing job metadata and execution history
//! 作业存储库，用于存储作业元数据和执行历史

use async_trait::async_trait;
use crate::error::{BatchError, BatchResult};
use crate::execution::{JobExecution, StepExecution, JobStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Job instance
/// 作业实例
///
/// Represents a unique job instance (distinguished by parameters).
/// 表示唯一的作业实例（由参数区分）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// JobInstance jobInstance = new JobInstance(jobName, jobParameters);
/// ```
#[derive(Debug, Clone)]
pub struct JobInstance {
    /// Unique instance ID
    /// 唯一实例ID
    pub id: Uuid,

    /// Job name
    /// 作业名称
    pub job_name: String,

    /// Job parameters (as string map for simplicity)
    /// 作业参数（简化为字符串映射）
    pub parameters: HashMap<String, String>,
}

impl JobInstance {
    /// Create new job instance
    /// 创建新作业实例
    pub fn new(job_name: impl Into<String>, parameters: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_name: job_name.into(),
            parameters,
        }
    }
}

/// Job repository
/// 作业存储库
///
/// Stores and retrieves job metadata and execution history.
/// 存储和检索作业元数据和执行历史。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JobRepository jobRepository() {
///     return new SimpleJobRepository();
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// let repository = JobRepository::new();
///
/// // Create job execution
/// let execution = repository.create_job_execution("my-job", params).await?;
///
/// // Save execution
/// repository.save_job_execution(execution).await?;
///
/// // Get last execution
/// let last = repository.get_last_job_execution("my-job").await?;
/// ```
#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Create new job execution
    /// 创建新作业执行
    async fn create_job_execution(
        &self,
        job_name: String,
        parameters: HashMap<String, String>,
    ) -> BatchResult<JobExecution>;

    /// Save job execution
    /// 保存作业执行
    async fn save_job_execution(&self, execution: &JobExecution) -> BatchResult<()>;

    /// Update job execution
    /// 更新作业执行
    async fn update_job_execution(&self, execution: &JobExecution) -> BatchResult<()>;

    /// Get job execution by ID
    /// 通过ID获取作业执行
    async fn get_job_execution(&self, id: Uuid) -> BatchResult<JobExecution>;

    /// Get last job execution for a job
    /// 获取作业的最后一次执行
    async fn get_last_job_execution(&self, job_name: &str) -> BatchResult<Option<JobExecution>>;

    /// Get all job executions for a job instance
    /// 获取作业实例的所有执行
    async fn get_job_executions(
        &self,
        job_instance_id: Uuid,
    ) -> BatchResult<Vec<JobExecution>>;

    /// Create step execution
    /// 创建步骤执行
    async fn create_step_execution(
        &self,
        step_name: String,
        job_execution_id: Uuid,
    ) -> BatchResult<StepExecution>;

    /// Save step execution
    /// 保存步骤执行
    async fn save_step_execution(&self, execution: &StepExecution) -> BatchResult<()>;

    /// Update step execution
    /// 更新步骤执行
    async fn update_step_execution(&self, execution: &StepExecution) -> BatchResult<()>;

    /// Get step execution by ID
    /// 通过ID获取步骤执行
    async fn get_step_execution(&self, id: Uuid) -> BatchResult<StepExecution>;

    /// Check if job is already running
    /// 检查作业是否已在运行
    async fn is_job_running(&self, job_name: &str) -> BatchResult<bool>;

    /// Check if job instance is complete
    /// 检查作业实例是否已完成
    async fn is_job_complete(&self, job_instance_id: Uuid) -> BatchResult<bool>;
}

/// In-memory job repository
/// 内存作业存储库
///
/// Simple in-memory implementation of JobRepository.
/// JobRepository 的简单内存实现。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JobRepository jobRepository() {
///     return new MapJobRepositoryFactoryBean().getObject();
/// }
/// ```
#[derive(Clone)]
pub struct InMemoryJobRepository {
    job_instances: Arc<RwLock<HashMap<Uuid, JobInstance>>>,
    job_executions: Arc<RwLock<HashMap<Uuid, JobExecution>>>,
    step_executions: Arc<RwLock<HashMap<Uuid, StepExecution>>>,
    job_name_to_instance: Arc<RwLock<HashMap<String, Uuid>>>,
    running_jobs: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl Default for InMemoryJobRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryJobRepository {
    /// Create new in-memory repository
    /// 创建新内存存储库
    pub fn new() -> Self {
        Self {
            job_instances: Arc::new(RwLock::new(HashMap::new())),
            job_executions: Arc::new(RwLock::new(HashMap::new())),
            step_executions: Arc::new(RwLock::new(HashMap::new())),
            job_name_to_instance: Arc::new(RwLock::new(HashMap::new())),
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clear all data
    /// 清除所有数据
    pub async fn clear(&self) {
        self.job_instances.write().await.clear();
        self.job_executions.write().await.clear();
        self.step_executions.write().await.clear();
        self.job_name_to_instance.write().await.clear();
        self.running_jobs.write().await.clear();
    }

    /// Get all job executions
    /// 获取所有作业执行
    pub async fn get_all_job_executions(&self) -> Vec<JobExecution> {
        self.job_executions.read().await.values().cloned().collect()
    }

    /// Get all step executions
    /// 获取所有步骤执行
    pub async fn get_all_step_executions(&self) -> Vec<StepExecution> {
        self.step_executions.read().await.values().cloned().collect()
    }
}

#[async_trait::async_trait]
impl JobRepository for InMemoryJobRepository {
    async fn create_job_execution(
        &self,
        job_name: String,
        parameters: HashMap<String, String>,
    ) -> BatchResult<JobExecution> {
        // Check if already running
        if self.is_job_running(&job_name).await? {
            return Err(BatchError::JobAlreadyRunning { job_name });
        }

        // Create or get job instance
        let mut name_to_instance = self.job_name_to_instance.write().await;
        let mut instances = self.job_instances.write().await;

        let instance_id = if let Some(id) = name_to_instance.get(&job_name) {
            *id
        } else {
            let instance = JobInstance::new(job_name.clone(), parameters);
            let id = instance.id;
            instances.insert(id, instance);
            name_to_instance.insert(job_name.clone(), id);
            id
        };

        let execution = JobExecution::new(job_name, instance_id);

        // Mark as running
        self.running_jobs.write().await.insert(execution.job_name.clone(), execution.id);

        Ok(execution)
    }

    async fn save_job_execution(&self, execution: &JobExecution) -> BatchResult<()> {
        self.job_executions
            .write()
            .await
            .insert(execution.id, execution.clone());
        Ok(())
    }

    async fn update_job_execution(&self, execution: &JobExecution) -> BatchResult<()> {
        self.job_executions
            .write()
            .await
            .insert(execution.id, execution.clone());

        // Clear from running if terminal
        if execution.status.is_terminal() {
            self.running_jobs
                .write()
                .await
                .remove(&execution.job_name);
        }

        Ok(())
    }

    async fn get_job_execution(&self, id: Uuid) -> BatchResult<JobExecution> {
        self.job_executions
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| BatchError::NotFound {
                resource: "JobExecution".to_string(),
                id: id.to_string(),
            })
    }

    async fn get_last_job_execution(&self, job_name: &str) -> BatchResult<Option<JobExecution>> {
        let executions = self.job_executions.read().await;

        let last = executions
            .values()
            .filter(|e| e.job_name == job_name)
            .max_by_key(|e| e.start_time);

        Ok(last.cloned())
    }

    async fn get_job_executions(
        &self,
        job_instance_id: Uuid,
    ) -> BatchResult<Vec<JobExecution>> {
        let executions = self.job_executions.read().await;

        let filtered = executions
            .values()
            .filter(|e| e.job_instance_id == job_instance_id)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn create_step_execution(
        &self,
        step_name: String,
        job_execution_id: Uuid,
    ) -> BatchResult<StepExecution> {
        Ok(StepExecution::new(step_name, job_execution_id))
    }

    async fn save_step_execution(&self, execution: &StepExecution) -> BatchResult<()> {
        self.step_executions
            .write()
            .await
            .insert(execution.id, execution.clone());
        Ok(())
    }

    async fn update_step_execution(&self, execution: &StepExecution) -> BatchResult<()> {
        self.step_executions
            .write()
            .await
            .insert(execution.id, execution.clone());
        Ok(())
    }

    async fn get_step_execution(&self, id: Uuid) -> BatchResult<StepExecution> {
        self.step_executions
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| BatchError::NotFound {
                resource: "StepExecution".to_string(),
                id: id.to_string(),
            })
    }

    async fn is_job_running(&self, job_name: &str) -> BatchResult<bool> {
        let running = self.running_jobs.read().await;
        Ok(running.contains_key(job_name))
    }

    async fn is_job_complete(&self, job_instance_id: Uuid) -> BatchResult<bool> {
        let executions = self.get_job_executions(job_instance_id).await?;

        // Check if any execution completed successfully
        Ok(executions
            .iter()
            .any(|e| e.status == JobStatus::Completed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::ExitStatus;

    #[tokio::test]
    async fn test_create_job_execution() {
        let repository = InMemoryJobRepository::new();
        let params = HashMap::new();

        let execution = repository
            .create_job_execution("test-job".to_string(), params)
            .await
            .unwrap();

        assert_eq!(execution.job_name, "test-job");
        assert!(repository.is_job_running("test-job").await.unwrap());
    }

    #[tokio::test]
    async fn test_job_already_running() {
        let repository = InMemoryJobRepository::new();
        let params = HashMap::new();

        repository
            .create_job_execution("test-job".to_string(), params.clone())
            .await
            .unwrap();

        let result = repository
            .create_job_execution("test-job".to_string(), params)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BatchError::JobAlreadyRunning { .. }
        ));
    }

    #[tokio::test]
    async fn test_save_and_get_job_execution() {
        let repository = InMemoryJobRepository::new();

        let execution = JobExecution::new("test-job", Uuid::new_v4());
        let id = execution.id;

        repository.save_job_execution(&execution).await.unwrap();

        let retrieved = repository.get_job_execution(id).await.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.job_name, "test-job");
    }

    #[tokio::test]
    async fn test_update_job_execution() {
        let repository = InMemoryJobRepository::new();

        let execution = repository
            .create_job_execution("test-job".to_string(), HashMap::new())
            .await
            .unwrap();
        let _id = execution.id;

        let mut updated = execution.clone();
        updated.set_status(JobStatus::Completed);
        updated.set_exit_status(ExitStatus::completed());

        repository.update_job_execution(&updated).await.unwrap();

        // No longer running
        assert!(!repository.is_job_running("test-job").await.unwrap());
    }

    #[tokio::test]
    async fn test_step_execution() {
        let repository = InMemoryJobRepository::new();

        let step = repository
            .create_step_execution("step1".to_string(), Uuid::new_v4())
            .await
            .unwrap();

        repository.save_step_execution(&step).await.unwrap();

        let retrieved = repository.get_step_execution(step.id).await.unwrap();
        assert_eq!(retrieved.step_name, "step1");
    }

    #[tokio::test]
    async fn test_clear_repository() {
        let repository = InMemoryJobRepository::new();

        repository
            .create_job_execution("test-job".to_string(), HashMap::new())
            .await
            .unwrap();

        repository.clear().await;

        assert!(!repository.is_job_running("test-job").await.unwrap());
        assert!(repository
            .get_all_job_executions()
            .await
            .is_empty());
    }

    #[tokio::test]
    async fn test_get_last_job_execution() {
        let repository = InMemoryJobRepository::new();

        let execution = repository
            .create_job_execution("test-job".to_string(), HashMap::new())
            .await
            .unwrap();
        repository.save_job_execution(&execution).await.unwrap();

        let last = repository
            .get_last_job_execution("test-job")
            .await
            .unwrap();

        assert!(last.is_some());
        assert_eq!(last.unwrap().job_name, "test-job");

        let none = repository
            .get_last_job_execution("non-existent")
            .await
            .unwrap();

        assert!(none.is_none());
    }
}
