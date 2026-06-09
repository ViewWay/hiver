//! Job and step context for sharing data during execution
//! 作业和步骤上下文，用于执行期间共享数据

use std::{any::Any, collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

/// Type-erased context storage
/// 类型擦除的上下文存储
type ContextStore = HashMap<String, Box<dyn Any + Send + Sync>>;

/// Job context - shared data across all steps in a job
/// 作业上下文 - 在作业的所有步骤之间共享数据
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring Batch JobContext / JobExecutionContext
/// JobExecutionContext context = stepExecution.getJobExecutionContext();
/// context.put("key", value);
/// Object value = context.get("key");
/// ```
#[derive(Clone)]
pub struct JobContext {
    inner: Arc<RwLock<ContextStore>>,
}

impl Default for JobContext {
    fn default() -> Self {
        Self::new()
    }
}

impl JobContext {
    /// Create new job context
    /// 创建新作业上下文
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Put value into context
    /// 将值放入上下文
    pub async fn put<T: Any + Send + Sync>(&self, key: impl Into<String>, value: T) {
        let key = key.into();
        let mut store = self.inner.write().await;
        store.insert(key, Box::new(value));
    }

    /// Get value from context
    /// 从上下文获取值
    pub async fn get<T: Any + Send + Sync + Clone>(&self, key: &str) -> Option<T> {
        let store = self.inner.read().await;
        store
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .cloned()
    }

    /// Remove value from context
    /// 从上下文移除值
    pub async fn remove<T: Any + Send + Sync>(&self, key: &str) -> Option<T> {
        let mut store = self.inner.write().await;
        store
            .remove(key)
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Check if key exists
    /// 检查键是否存在
    pub async fn contains_key(&self, key: &str) -> bool {
        let store = self.inner.read().await;
        store.contains_key(key)
    }

    /// Get all keys
    /// 获取所有键
    pub async fn keys(&self) -> Vec<String> {
        let store = self.inner.read().await;
        store.keys().cloned().collect()
    }

    /// Clear all context data
    /// 清除所有上下文数据
    pub async fn clear(&self) {
        let mut store = self.inner.write().await;
        store.clear();
    }

    /// Get context size
    /// 获取上下文大小
    pub async fn len(&self) -> usize {
        let store = self.inner.read().await;
        store.len()
    }

    /// Check if context is empty
    /// 检查上下文是否为空
    pub async fn is_empty(&self) -> bool {
        let store = self.inner.read().await;
        store.is_empty()
    }
}

/// Step context - data for a single step execution
/// 步骤上下文 - 单个步骤执行的数据
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring Batch StepContext / StepExecutionContext
/// StepExecutionContext context = stepExecution.getExecutionContext();
/// context.putInt("readCount", 100);
/// int count = context.getInt("readCount");
/// ```
#[derive(Clone)]
pub struct StepContext {
    inner: Arc<RwLock<ContextStore>>,
    /// Reference to parent job context
    /// 父作业上下文的引用
    pub job_context: JobContext,
}

impl StepContext {
    /// Create new step context with parent job context
    /// 使用父作业上下文创建新步骤上下文
    pub fn new(job_context: JobContext) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            job_context,
        }
    }

    /// Create new step context without parent
    /// 创建没有父级的新步骤上下文
    pub fn standalone() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            job_context: JobContext::new(),
        }
    }

    /// Put value into step context
    /// 将值放入步骤上下文
    pub async fn put<T: Any + Send + Sync>(&self, key: impl Into<String>, value: T) {
        let key = key.into();
        let mut store = self.inner.write().await;
        store.insert(key, Box::new(value));
    }

    /// Get value from step context
    /// 从步骤上下文获取值
    pub async fn get<T: Any + Send + Sync + Clone>(&self, key: &str) -> Option<T> {
        let store = self.inner.read().await;
        store
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .cloned()
    }

    /// Remove value from step context
    /// 从步骤上下文移除值
    pub async fn remove<T: Any + Send + Sync>(&self, key: &str) -> Option<T> {
        let mut store = self.inner.write().await;
        store
            .remove(key)
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Get value from step context, fallback to job context
    /// 从步骤上下文获取值，回退到作业上下文
    pub async fn get_or_from_job<T: Any + Send + Sync + Clone>(&self, key: &str) -> Option<T> {
        match self.get(key).await {
            Some(value) => Some(value),
            None => self.job_context.get(key).await,
        }
    }

    /// Put value into both step and job context
    /// 将值放入步骤和作业上下文
    pub async fn put_to_job<T: Any + Send + Sync>(&self, key: impl Into<String>, value: T) {
        self.job_context.put(key, value).await;
    }

    /// Check if key exists in step context
    /// 检查键是否存在于步骤上下文
    pub async fn contains_key(&self, key: &str) -> bool {
        let store = self.inner.read().await;
        store.contains_key(key)
    }

    /// Get all keys from step context
    /// 从步骤上下文获取所有键
    pub async fn keys(&self) -> Vec<String> {
        let store = self.inner.read().await;
        store.keys().cloned().collect()
    }

    /// Clear all step context data
    /// 清除所有步骤上下文数据
    pub async fn clear(&self) {
        let mut store = self.inner.write().await;
        store.clear();
    }

    /// Get step context size
    /// 获取步骤上下文大小
    pub async fn len(&self) -> usize {
        let store = self.inner.read().await;
        store.len()
    }

    /// Check if step context is empty
    /// 检查步骤上下文是否为空
    pub async fn is_empty(&self) -> bool {
        let store = self.inner.read().await;
        store.is_empty()
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

    #[tokio::test]
    async fn test_job_context() {
        let context = JobContext::new();

        context.put("user_id", 42u32).await;
        context.put("name", "Alice").await;

        assert_eq!(context.get::<u32>("user_id").await, Some(42));
        assert_eq!(context.get::<&str>("name").await, Some("Alice"));
        assert!(context.contains_key("user_id").await);
        assert_eq!(context.len().await, 2);
        assert!(!context.is_empty().await);
    }

    #[tokio::test]
    async fn test_step_context() {
        let job_context = JobContext::new();
        job_context.put("job_level", 123).await;

        let step_context = StepContext::new(job_context.clone());

        step_context.put("step_level", 456).await;

        assert_eq!(step_context.get::<i32>("step_level").await, Some(456));
        assert_eq!(step_context.get::<i32>("job_level").await, None);
        assert_eq!(step_context.get_or_from_job::<i32>("job_level").await, Some(123));
        assert_eq!(step_context.get_or_from_job::<i32>("step_level").await, Some(456));
    }

    #[tokio::test]
    async fn test_context_remove() {
        let context = JobContext::new();

        context.put("temp", "value").await;
        assert_eq!(context.get::<&str>("temp").await, Some("value"));

        let removed: Option<&str> = context.remove("temp").await;
        assert_eq!(removed, Some("value"));
        assert!(context.get::<&str>("temp").await.is_none());
    }

    #[tokio::test]
    async fn test_context_clear() {
        let context = JobContext::new();

        context.put("a", 1).await;
        context.put("b", 2).await;
        assert_eq!(context.len().await, 2);

        context.clear().await;
        assert_eq!(context.len().await, 0);
        assert!(context.is_empty().await);
    }

    #[tokio::test]
    async fn test_standalone_step_context() {
        let step_context = StepContext::standalone();

        step_context.put("value", 100).await;
        assert_eq!(step_context.get::<i32>("value").await, Some(100));

        // Job context should be empty for standalone
        assert_eq!(step_context.job_context.len().await, 0);
    }
}
