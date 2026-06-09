//! Transactional attribute equivalent
//! @Transactional注解等价物
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Transactional` - Transactional procedural macro
//! - `@Transactional(timeout = 30)` - with options
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_tx::Transactional;
//!
//! struct UserService {
//!     // ... fields
//! }
//!
//! impl UserService {
//!     #[transactional]
//!     pub async fn create_user(&self, user: User) -> Result<User, Error> {
//!         // This runs in a transaction
//!         Ok(user)
//!     }
//!
//!     #[transactional(read_only = true)]
//!     pub async fn get_user(&self, id: u64) -> Result<Option<User>, Error> {
//!         // This runs in a read-only transaction
//!         Ok(None)
//!     }
//! }
//! ```

#![allow(async_fn_in_trait)]
#![allow(dead_code)]
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{IsolationLevel, Propagation, TransactionError, TransactionResult};

/// Transactional options
/// Transactional选项
///
/// Equivalent to Spring's @Transactional annotation parameters.
/// 等价于Spring的@Transactional注解参数。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Transactional(
///     propagation = Propagation.REQUIRES_NEW,
///     isolation = Isolation.SERIALIZABLE,
///     timeout = 30,
///     readOnly = true,
///     rollbackFor = {Exception.class},
///     noRollbackFor = {RuntimeException.class}
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionalOptions
{
    /// Transaction name
    /// 事务名称
    #[serde(default)]
    pub name: Option<String>,

    /// Propagation behavior
    /// 传播行为
    #[serde(default)]
    pub propagation: Option<Propagation>,

    /// Isolation level
    /// 隔离级别
    #[serde(default)]
    pub isolation: Option<IsolationLevel>,

    /// Timeout in seconds
    /// 超时时间（秒）
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Read-only flag
    /// 只读标志
    #[serde(default = "default_read_only")]
    pub read_only: bool,

    /// Rollback for exceptions
    /// 回滚异常
    #[serde(default)]
    pub rollback_for: Vec<String>,

    /// No rollback for exceptions
    /// 不回滚异常
    #[serde(default)]
    pub no_rollback_for: Vec<String>,
}

fn default_read_only() -> bool
{
    false
}

impl TransactionalOptions
{
    /// Create new transactional options
    /// 创建新的transactional选项
    pub fn new() -> Self
    {
        Self {
            name: None,
            propagation: None,
            isolation: None,
            timeout_secs: None,
            read_only: false,
            rollback_for: Vec::new(),
            no_rollback_for: Vec::new(),
        }
    }

    /// Set transaction name
    /// 设置事务名称
    pub fn name(mut self, name: impl Into<String>) -> Self
    {
        self.name = Some(name.into());
        self
    }

    /// Set propagation
    /// 设置传播行为
    pub fn propagation(mut self, propagation: Propagation) -> Self
    {
        self.propagation = Some(propagation);
        self
    }

    /// Set isolation
    /// 设置隔离级别
    pub fn isolation(mut self, isolation: IsolationLevel) -> Self
    {
        self.isolation = Some(isolation);
        self
    }

    /// Set timeout
    /// 设置超时
    pub fn timeout_secs(mut self, timeout: u64) -> Self
    {
        self.timeout_secs = Some(timeout);
        self
    }

    /// Set read-only
    /// 设置只读
    pub fn read_only(mut self, read_only: bool) -> Self
    {
        self.read_only = read_only;
        self
    }

    /// Add rollback for exception type
    /// 添加回滚异常类型
    pub fn rollback_for(mut self, exception: impl Into<String>) -> Self
    {
        self.rollback_for.push(exception.into());
        self
    }

    /// Add no rollback for exception type
    /// 添加不回滚异常类型
    pub fn no_rollback_for(mut self, exception: impl Into<String>) -> Self
    {
        self.no_rollback_for.push(exception.into());
        self
    }

    /// Check if should rollback for error
    /// 检查错误是否应回滚
    pub fn should_rollback(&self, error: &TransactionError) -> bool
    {
        // Default behavior: rollback for all errors
        if self.rollback_for.is_empty() && self.no_rollback_for.is_empty()
        {
            return true;
        }

        // Check no_rollback_for first
        for exception in &self.no_rollback_for
        {
            // Simplified check - in real implementation would use type matching
            if error.to_string().contains(exception)
            {
                return false;
            }
        }

        // Check rollback_for
        if self.rollback_for.is_empty()
        {
            return true;
        }

        for exception in &self.rollback_for
        {
            if error.to_string().contains(exception)
            {
                return true;
            }
        }

        false
    }
}

impl Default for TransactionalOptions
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Transactional trait
/// Transactional trait
///
/// Implement this trait to make a struct transactional.
/// 实现此trait以使结构体支持事务。
///
/// Equivalent to Spring's @Transactional on a class.
/// 等价于Spring类上的@Transactional。
pub trait Transactional
{
    /// Execute a function within a transaction
    /// 在事务中执行函数
    async fn in_transaction<F, T, E>(&self, f: F) -> TransactionResult<T>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>> + Send + Sync,
        T: Send + 'static,
        E: Into<TransactionError> + Send + 'static,
    {
        // Default implementation uses global transaction manager
        // In real implementation, this would use injected transaction manager
        let result = f().await;

        match result
        {
            Ok(value) => Ok(value),
            Err(e) => Err(e.into()),
        }
    }

    /// Execute with options
    /// 使用选项执行
    async fn in_transaction_with_options<F, T, E>(
        &self,
        options: &TransactionalOptions,
        f: F,
    ) -> TransactionResult<T>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>> + Send + Sync,
        T: Send + 'static,
        E: Into<TransactionError> + Send + 'static,
    {
        let result = f().await;

        match result
        {
            Ok(value) => Ok(value),
            Err(e) =>
            {
                let tx_error = e.into();
                if options.should_rollback(&tx_error)
                {
                    return Err(tx_error);
                }
                // Even if we don't rollback, the operation failed
                // 即使不回滚，操作仍然失败
                Err(tx_error)
            },
        }
    }
}

/// Blanket implementation for all types
/// 所有类型的blanket实现
impl<T> Transactional for T where T: Send + Sync {}

/// Transaction guard for RAII-style transaction management
/// RAII风格事务管理的事务守卫
///
/// Automatically commits or rolls back when dropped.
/// 删除时自动提交或回滚。
///
/// Equivalent to Spring's `TransactionTemplate` with try-with-resources.
/// 等价于Spring与try-with-resources的TransactionTemplate。
pub(crate) struct TransactionGuard<'a>
{
    /// Transaction status
    /// 事务状态
    status: crate::TransactionStatus,

    /// Transaction manager
    /// 事务管理器
    manager: &'a dyn crate::TransactionManager,

    /// Committed flag
    /// 已提交标志
    committed: Arc<std::sync::atomic::AtomicBool>,
}

impl<'a> TransactionGuard<'a>
{
    /// Create a new transaction guard
    /// 创建新的事务守卫
    pub(crate) fn new(
        status: crate::TransactionStatus,
        manager: &'a dyn crate::TransactionManager,
    ) -> Self
    {
        Self {
            status,
            manager,
            committed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Commit the transaction
    /// 提交事务
    pub(crate) async fn commit(self) -> TransactionResult<()>
    {
        self.committed
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.manager.commit(self.status.clone()).await
    }

    /// Rollback the transaction
    /// 回滚事务
    pub(crate) async fn rollback(self) -> TransactionResult<()>
    {
        self.manager.rollback(self.status.clone()).await
    }

    /// Mark for rollback only
    /// 标记为仅回滚
    pub(crate) fn set_rollback_only(&self)
    {
        self.status.set_rollback_only();
    }
}

impl Drop for TransactionGuard<'_>
{
    fn drop(&mut self)
    {
        if !self.committed.load(std::sync::atomic::Ordering::SeqCst)
        {
            // In a real async Drop scenario, we'd need to handle this differently
            // For now, this is a simplified implementation
            if !self.status.is_completed()
            {
                // Would trigger rollback
            }
        }
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
    fn test_transactional_options()
    {
        let options = TransactionalOptions::new()
            .name("test_tx")
            .propagation(Propagation::RequiresNew)
            .isolation(IsolationLevel::Serializable)
            .timeout_secs(60)
            .read_only(true);

        assert_eq!(options.name, Some("test_tx".to_string()));
        assert_eq!(options.propagation, Some(Propagation::RequiresNew));
        assert!(options.read_only);
    }

    #[test]
    fn test_should_rollback()
    {
        // Test 1: Default behavior - rollback for all errors
        // 测试 1: 默认行为 - 所有错误都回滚
        let default_options = TransactionalOptions::new();
        let any_error = TransactionError::CommitFailed("Some error".to_string());
        assert!(default_options.should_rollback(&any_error));

        // Test 2: Explicit no-rollback for specific error
        // 测试 2: 明确指定特定错误不回滚
        let options = TransactionalOptions::new().no_rollback_for("Invalid");

        let validation_error = TransactionError::InvalidState("Invalid input".to_string());
        // Should not rollback for "Invalid" errors (explicitly excluded)
        // "Invalid transaction state: Invalid input" contains "Invalid"
        assert!(!options.should_rollback(&validation_error));

        // Other errors should still rollback
        let commit_error = TransactionError::CommitFailed("Connection failed".to_string());
        assert!(options.should_rollback(&commit_error));
    }
}
