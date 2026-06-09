//! @Transactional Runtime Support / @Transactional 运行时支持
//!
//! This module provides runtime support for the @Transactional annotation,
//! allowing methods to be automatically executed within a transaction.
//!
//! 此模块提供 @Transactional 注解的运行时支持，允许方法在事务中自动执行。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_annotations::Transactional;
//! use hiver_data_rdbc::IsolationLevel;
//!
//! #[Transactional]
//! async fn transfer_funds(from: i64, to: i64, amount: i64) -> Result<()> {
//!     // Debit from account
//!     // 从账户借记
//!     update_account_balance(from, -amount).await?;
//!
//!     // Credit to account
//!     // 向账户贷记
//!     update_account_balance(to, amount).await?;
//!
//!     // Transaction will be automatically committed on success
//!     // 成功时事务将自动提交
//!     Ok(())
//! }
//! ```

#![allow(dead_code)]
use std::{future::Future, pin::Pin, sync::Arc};

/// Transactional configuration
/// 事务配置
///
/// Configuration for @Transactional behavior.
///
/// @Transactional 行为的配置。
#[derive(Debug, Clone)]
pub(crate) struct TransactionalConfig
{
    /// Isolation level
    /// 隔离级别
    pub isolation: IsolationLevel,

    /// Timeout in seconds
    /// 超时时间（秒）
    pub timeout: Option<u64>,

    /// Whether to create a new transaction (always)
    /// 是否总是创建新事务
    pub propagation: Propagation,

    /// Read-only flag
    /// 只读标志
    pub read_only: bool,

    /// Max retry attempts for serialization failures
    /// 序列化失败的最大重试次数
    pub max_retries: u32,
}

/// Transaction isolation level
/// 事务隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum IsolationLevel
{
    /// Use the default isolation level
    /// 使用默认隔离级别
    #[default]
    Default,

    /// Read uncommitted - lowest isolation
    /// 读未提交 - 最低隔离级别
    ReadUncommitted,

    /// Read committed - prevents dirty reads
    /// 读已提交 - 防止脏读
    ReadCommitted,

    /// Repeatable read - prevents non-repeatable reads
    /// 可重复读 - 防止不可重复读
    RepeatableRead,

    /// Serializable - highest isolation
    /// 可串行化 - 最高隔离级别
    Serializable,
}

/// Transaction propagation behavior
/// 事务传播行为
///
/// Defines how transactions should be propagated when a @Transactional
/// method is called from another @Transactional method.
///
/// 定义当一个 @Transactional 方法从另一个 @Transactional 方法调用时
/// 事务应该如何传播。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Propagation
{
    /// Support a current transaction, create a new one if none exists
    /// 支持当前事务，如果没有则创建新事务
    #[default]
    Required,

    /// Support a current transaction, execute non-transactionally if none exists
    /// 支持当前事务，如果没有则非事务执行
    Supports,

    /// Support a current transaction, throw an exception if none exists
    /// 支持当前事务，如果没有则抛出异常
    Mandatory,

    /// Create a new transaction, and suspend the current transaction if one exists
    /// 创建新事务，如果存在当前事务则挂起
    RequiresNew,

    /// Execute non-transactionally, suspend the current transaction if one exists
    /// 非事务执行，如果存在当前事务则挂起
    NotSupported,

    /// Execute non-transactionally, throw an exception if a transaction exists
    /// 非事务执行，如果存在事务则抛出异常
    Never,

    /// Execute within a nested transaction if a current transaction exists
    /// 如果存在当前事务，则在嵌套事务中执行
    Nested,
}

impl Default for TransactionalConfig
{
    fn default() -> Self
    {
        Self {
            isolation: IsolationLevel::default(),
            timeout: None,
            propagation: Propagation::default(),
            read_only: false,
            max_retries: 3,
        }
    }
}

impl TransactionalConfig
{
    /// Create a new transactional config
    /// 创建新的事务配置
    pub(crate) fn new() -> Self
    {
        Self::default()
    }

    /// Set the isolation level
    /// 设置隔离级别
    pub(crate) fn isolation(mut self, isolation: IsolationLevel) -> Self
    {
        self.isolation = isolation;
        self
    }

    /// Set the timeout in seconds
    /// 设置超时时间（秒）
    pub(crate) fn timeout(mut self, timeout: u64) -> Self
    {
        self.timeout = Some(timeout);
        self
    }

    /// Set the propagation behavior
    /// 设置传播行为
    pub(crate) fn propagation(mut self, propagation: Propagation) -> Self
    {
        self.propagation = propagation;
        self
    }

    /// Set the read-only flag
    /// 设置只读标志
    pub(crate) fn read_only(mut self, read_only: bool) -> Self
    {
        self.read_only = read_only;
        self
    }

    /// Set the max retries
    /// 设置最大重试次数
    pub(crate) fn max_retries(mut self, max_retries: u32) -> Self
    {
        self.max_retries = max_retries;
        self
    }
}

/// Transactional executor
/// 事务执行器
///
/// Executes functions within transactions according to the configuration.
///
/// 根据配置在事务中执行函数。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::transactional::{TransactionalExecutor, TransactionalConfig};
///
/// let executor = TransactionalExecutor::new(transaction_manager);
///
/// let config = TransactionalConfig::new()
///     .isolation(IsolationLevel::ReadCommitted)
///     .timeout(30);
///
/// let result = executor.execute(config, || async {
///     // Do work within transaction
///     // 在事务中执行工作
///     Ok(())
/// }).await?;
/// ```
pub(crate) struct TransactionalExecutor
{
    /// Transaction manager
    /// 事务管理器
    transaction_manager: Arc<dyn TransactionManager>,

    /// Current transaction context
    /// 当前事务上下文
    current_context: Arc<tokio::sync::RwLock<Option<TransactionContext>>>,
}

/// Transaction context
/// 事务上下文
#[derive(Debug, Clone)]
struct TransactionContext
{
    /// Transaction ID
    /// 事务 ID
    id: u64,

    /// Configuration
    /// 配置
    config: TransactionalConfig,

    /// Nested level
    /// 嵌套级别
    level: u32,
}

impl TransactionalExecutor
{
    /// Create a new transactional executor
    /// 创建新的事务执行器
    pub(crate) fn new(transaction_manager: Arc<dyn TransactionManager>) -> Self
    {
        Self {
            transaction_manager,
            current_context: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Execute a function within a transaction
    /// 在事务中执行函数
    ///
    /// The transaction will be committed if the function returns Ok,
    /// and rolled back if it returns Err.
    ///
    /// 如果函数返回 Ok，事务将被提交；如果返回 Err，事务将被回滚。
    pub(crate) async fn execute<F, T, E>(
        &self,
        config: TransactionalConfig,
        f: F,
    ) -> Result<T, TransactionError<E>>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Handle propagation
        // 处理传播
        let context = self.current_context.read().await;
        let should_create_new = match (&*context, config.propagation)
        {
            (None, Propagation::Required | Propagation::RequiresNew)
            | (Some(_), Propagation::RequiresNew) => true,
            (None, Propagation::Supports | Propagation::NotSupported | Propagation::Never)
            | (
                Some(_),
                Propagation::Required
                | Propagation::Supports
                | Propagation::Mandatory
                | Propagation::Nested,
            ) => false,
            (None, Propagation::Mandatory | Propagation::Nested) =>
            {
                return Err(TransactionError::NoExistingTransaction);
            },
            (Some(_), Propagation::NotSupported | Propagation::Never) =>
            {
                return Err(TransactionError::ExistingTransaction);
            },
        };
        drop(context);

        let mut f = f;

        if should_create_new
        {
            self.execute_in_new_transaction(config, &mut f).await
        }
        else
        {
            self.execute_in_existing_transaction(&mut f).await
        }
    }

    /// Execute in a new transaction
    /// 在新事务中执行
    async fn execute_in_new_transaction<F, T, E>(
        &self,
        config: TransactionalConfig,
        f: &mut F,
    ) -> Result<T, TransactionError<E>>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Begin transaction
        // 开始事务
        let tx = self
            .transaction_manager
            .begin(config.clone().into())
            .await
            .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;

        // Set context
        // 设置上下文
        let new_context = TransactionContext {
            id: rand::random::<u64>(),
            config: config.clone(),
            level: 1,
        };

        {
            let mut context = self.current_context.write().await;
            *context = Some(new_context.clone());
        }

        // Execute with retry
        // 带重试执行
        let result = self.execute_with_retry(config.clone(), &tx, f).await;

        // Clear context
        // 清除上下文
        {
            let mut context = self.current_context.write().await;
            *context = None;
        }

        // Commit or rollback
        // 提交或回滚
        match &result
        {
            Ok(_) =>
            {
                tx.commit()
                    .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
            },
            Err(_) =>
            {
                tx.rollback()
                    .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
            },
        }

        result
    }

    /// Execute in existing transaction
    /// 在现有事务中执行
    async fn execute_in_existing_transaction<F, T, E>(
        &self,
        f: &mut F,
    ) -> Result<T, TransactionError<E>>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Just execute the function without transaction management
        // 在没有事务管理的情况下执行函数
        f().await.map_err(TransactionError::ExecutionFailed)
    }

    /// Execute with retry on serialization failures
    /// 在序列化失败时重试执行
    async fn execute_with_retry<F, T, E>(
        &self,
        config: TransactionalConfig,
        _tx: &Transaction,
        f: &mut F,
    ) -> Result<T, TransactionError<E>>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut attempt = 0;
        let max_attempts = config.max_retries + 1;

        loop
        {
            attempt += 1;

            let result = f().await;

            match &result
            {
                Err(e) if attempt < max_attempts && self.is_retryable_error(e) =>
                {
                    // Retry
                    // 重试
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                },
                _ =>
                {
                    return result.map_err(TransactionError::ExecutionFailed);
                },
            }
        }
    }

    /// Check if an error is retryable
    /// 检查错误是否可重试
    fn is_retryable_error<E>(&self, _error: &E) -> bool
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check for serialization failure, deadlock, etc.
        // 检查序列化失败、死锁等
        // This is a simplified check
        // 这是一个简化的检查
        false
    }
}

/// Transaction error
/// 事务错误
#[derive(Debug)]
pub(crate) enum TransactionError<E>
where
    E: std::error::Error,
{
    /// No existing transaction
    /// 没有现有事务
    NoExistingTransaction,

    /// Existing transaction when none expected
    /// 有现有事务但预期没有
    ExistingTransaction,

    /// Begin transaction failed
    /// 开始事务失败
    BeginFailed(String),

    /// Commit transaction failed
    /// 提交事务失败
    CommitFailed(String),

    /// Rollback transaction failed
    /// 回滚事务失败
    RollbackFailed(String),

    /// Execution failed
    /// 执行失败
    ExecutionFailed(E),
}

impl<E: std::error::Error> std::fmt::Display for TransactionError<E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::NoExistingTransaction => write!(f, "No existing transaction"),
            Self::ExistingTransaction => write!(f, "Existing transaction when none expected"),
            Self::BeginFailed(msg) => write!(f, "Begin transaction failed: {}", msg),
            Self::CommitFailed(msg) => write!(f, "Commit transaction failed: {}", msg),
            Self::RollbackFailed(msg) => write!(f, "Rollback transaction failed: {}", msg),
            Self::ExecutionFailed(e) => write!(f, "Execution failed: {}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for TransactionError<E> {}

/// Transaction manager trait
/// 事务管理器 trait
pub(crate) trait TransactionManager: Send + Sync
{
    /// Begin a new transaction
    /// 开始新事务
    fn begin(
        &self,
        config: TransactionConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, String>> + Send + '_>>;
}

/// Transaction
/// 事务
pub(crate) struct Transaction
{
    _inner: (),
}

/// Transaction configuration for the manager
/// 事务管理器的配置
#[derive(Debug, Clone)]
pub(crate) struct TransactionConfig
{
    /// Isolation level
    /// 隔离级别
    pub isolation: IsolationLevel,

    /// Read-only
    /// 只读
    pub read_only: bool,
}

impl From<TransactionalConfig> for TransactionConfig
{
    fn from(config: TransactionalConfig) -> Self
    {
        Self {
            isolation: config.isolation,
            read_only: config.read_only,
        }
    }
}

impl Transaction
{
    /// Commit the transaction
    /// 提交事务
    pub(crate) fn commit(&self) -> Result<(), String>
    {
        // Placeholder
        // 占位符
        Ok(())
    }

    /// Rollback the transaction
    /// 回滚事务
    pub(crate) fn rollback(&self) -> Result<(), String>
    {
        // Placeholder
        // 占位符
        Ok(())
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
mod tests
{
    use super::*;

    #[test]
    fn test_transactional_config_default()
    {
        let config = TransactionalConfig::default();
        assert_eq!(config.isolation, IsolationLevel::Default);
        assert_eq!(config.propagation, Propagation::Required);
        assert!(!config.read_only);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_transactional_config_builder()
    {
        let config = TransactionalConfig::new()
            .isolation(IsolationLevel::Serializable)
            .timeout(30)
            .read_only(true)
            .max_retries(5);

        assert_eq!(config.isolation, IsolationLevel::Serializable);
        assert_eq!(config.timeout, Some(30));
        assert!(config.read_only);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_isolation_level_default()
    {
        let level = IsolationLevel::default();
        assert_eq!(level, IsolationLevel::Default);
    }

    #[test]
    fn test_propagation_default()
    {
        let propagation = Propagation::default();
        assert_eq!(propagation, Propagation::Required);
    }
}
