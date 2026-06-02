//! Transaction template
//! 事务模板

use crate::{IsolationLevel, Propagation, TransactionError, TransactionManager, TransactionResult};
use std::sync::Arc;

/// Transaction template
/// 事务模板
///
/// Equivalent to Spring's `TransactionTemplate`.
/// `等价于Spring的TransactionTemplate`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_tx::TransactionTemplate;
///
/// let template = TransactionTemplate::new(manager);
///
/// let result = template.execute(|| async {
///     // Transactional code here
///     Ok(42)
/// }).await?;
/// ```
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Autowired
/// private TransactionTemplate transactionTemplate;
///
/// public void processData() {
///     transactionTemplate.execute(status -> {
///         // Transactional code
///         return null;
///     });
/// }
/// ```
#[derive(Clone)]
pub struct TransactionTemplate {
    /// Transaction manager
    /// 事务管理器
    manager: Arc<dyn TransactionManager>,

    /// Default propagation
    /// 默认传播行为
    propagation: Propagation,

    /// Default isolation
    /// 默认隔离级别
    isolation: IsolationLevel,

    /// Default read-only
    /// 默认只读
    read_only: bool,

    /// Default timeout
    /// 默认超时
    timeout_secs: Option<u64>,
}

impl std::fmt::Debug for TransactionTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionTemplate")
            .field("propagation", &self.propagation)
            .field("isolation", &self.isolation)
            .field("read_only", &self.read_only)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

impl TransactionTemplate {
    /// Create a new transaction template
    /// 创建新的事务模板
    pub fn new(manager: Arc<dyn TransactionManager>) -> Self {
        Self {
            manager,
            propagation: Propagation::default(),
            isolation: IsolationLevel::default(),
            read_only: false,
            timeout_secs: None,
        }
    }

    /// Set propagation
    /// 设置传播行为
    pub fn propagation(mut self, propagation: Propagation) -> Self {
        self.propagation = propagation;
        self
    }

    /// Set isolation
    /// 设置隔离级别
    pub fn isolation(mut self, isolation: IsolationLevel) -> Self {
        self.isolation = isolation;
        self
    }

    /// Set read-only
    /// 设置只读
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Set timeout
    /// 设置超时
    pub fn timeout_secs(mut self, timeout: u64) -> Self {
        self.timeout_secs = Some(timeout);
        self
    }

    /// Execute a function within a transaction
    /// 在事务中执行函数
    ///
    /// The function will be executed in a new transaction.
    /// If the function returns Ok, the transaction will be committed.
    /// If the function returns Err, the transaction will be rolled back.
    ///
    /// 函数将在新事务中执行。
    /// 如果函数返回Ok，事务将被提交。
    /// 如果函数返回Err，事务将被回滚。
    pub async fn execute<F, T, E>(&self, f: F) -> TransactionResult<T>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>> + Send + Sync,
        T: Send + 'static,
        E: Into<TransactionError> + Send + 'static,
    {
        // Create transaction definition
        let mut def = crate::manager::TransactionDefinition::new("template")
            .propagation(self.propagation)
            .isolation(self.isolation)
            .read_only(self.read_only);

        if let Some(timeout) = self.timeout_secs {
            def.timeout_secs = Some(timeout);
        }

        // Begin transaction
        let status = self.manager.begin(&def).await?;

        // Execute function
        let result = f().await;

        // Commit or rollback
        match result {
            Ok(value) => {
                self.manager.commit(status).await?;
                Ok(value)
            },
            Err(e) => {
                self.manager.rollback(status).await?;
                Err(e.into())
            },
        }
    }

    /// Execute without return value
    /// 执行无返回值的函数
    pub async fn execute_without_result<F, E>(&self, f: F) -> TransactionResult<()>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<(), E>> + Send + Sync,
        E: Into<TransactionError> + Send + 'static,
    {
        self.execute(f).await
    }

    /// Execute returning Result
    /// 执行并返回Result
    pub async fn execute_result<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>> + Send + Sync,
        T: Send + 'static,
        E: Into<TransactionError> + Send + 'static,
        TransactionError: Into<E>,
    {
        self.execute(f).await.map_err(Into::into)
    }
}

/// Callback for transaction execution
/// 事务执行的回调
///
/// Equivalent to Spring's `TransactionCallback`.
/// `等价于Spring的TransactionCallback`。
pub(crate) trait TransactionCallback<T>: Send {
    /// Execute within transaction
    /// 在事务内执行
    fn execute(&self) -> futures::future::BoxFuture<'_, TransactionResult<T>>;
}

/// Callback without return value
/// 无返回值的回调
///
/// Equivalent to Spring's `TransactionCallbackWithoutResult`.
/// `等价于Spring的TransactionCallbackWithoutResult`。
pub(crate) trait TransactionCallbackWithoutResult: Send {
    /// Execute within transaction
    /// 在事务内执行
    fn execute(&self) -> futures::future::BoxFuture<'_, TransactionResult<()>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::SimpleTransactionManager;

    #[tokio::test]
    async fn test_transaction_template() {
        let manager = Arc::new(SimpleTransactionManager::new());
        let template = TransactionTemplate::new(manager);

        let result = template
            .execute(|| Box::pin(async { Ok::<_, TransactionError>(42) }))
            .await
            .unwrap();

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_transaction_template_rollback() {
        let manager = Arc::new(SimpleTransactionManager::new());
        let template = TransactionTemplate::new(manager);

        let result = template
            .execute(|| {
                Box::pin(async {
                    Err::<(), TransactionError>(TransactionError::CommitFailed("error".to_string()))
                })
            })
            .await;

        assert!(result.is_err());
    }
}
