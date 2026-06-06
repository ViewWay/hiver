//! Transaction status
//! 事务状态

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Transaction status
/// 事务状态
///
/// Represents the current state of a transaction.
/// 表示事务的当前状态。
///
/// Equivalent to Spring's `TransactionStatus` interface.
/// `等价于Spring的TransactionStatus接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface TransactionStatus {
///     boolean isNewTransaction();
///     boolean hasSavepoint();
///     void setRollbackOnly();
///     boolean isRollbackOnly();
///     boolean isCompleted();
///     void flush();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TransactionStatus
{
    /// Whether this is a new transaction
    /// 是否为新事务
    new_transaction: Arc<AtomicBool>,

    /// Whether rollback only
    /// 是否仅回滚
    rollback_only: Arc<AtomicBool>,

    /// Whether completed
    /// 是否已完成
    completed: Arc<AtomicBool>,

    /// Whether has savepoint
    /// 是否有保存点
    has_savepoint: Arc<AtomicBool>,

    /// Transaction name
    /// 事务名称
    name: String,
}

impl TransactionStatus
{
    /// Create a new transaction status
    /// 创建新的事务状态
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            new_transaction: Arc::new(AtomicBool::new(true)),
            rollback_only: Arc::new(AtomicBool::new(false)),
            completed: Arc::new(AtomicBool::new(false)),
            has_savepoint: Arc::new(AtomicBool::new(false)),
            name: name.into(),
        }
    }

    /// Create with existing transaction
    /// 使用现有事务创建
    pub fn existing(name: impl Into<String>) -> Self
    {
        let status = Self::new(name);
        status.new_transaction.store(false, Ordering::SeqCst);
        status
    }

    /// Check if this is a new transaction
    /// 检查是否为新事务
    pub fn is_new_transaction(&self) -> bool
    {
        self.new_transaction.load(Ordering::SeqCst)
    }

    /// Check if transaction has savepoint
    /// 检查事务是否有保存点
    pub fn has_savepoint(&self) -> bool
    {
        self.has_savepoint.load(Ordering::SeqCst)
    }

    /// Set rollback only
    /// 设置仅回滚
    pub fn set_rollback_only(&self)
    {
        self.rollback_only.store(true, Ordering::SeqCst);
    }

    /// Check if rollback only
    /// 检查是否仅回滚
    pub fn is_rollback_only(&self) -> bool
    {
        self.rollback_only.load(Ordering::SeqCst)
    }

    /// Check if transaction is completed
    /// 检查事务是否已完成
    pub fn is_completed(&self) -> bool
    {
        self.completed.load(Ordering::SeqCst)
    }

    /// Mark as completed
    /// 标记为已完成
    pub fn mark_completed(&self)
    {
        self.completed.store(true, Ordering::SeqCst);
    }

    /// Get transaction name
    /// 获取事务名称
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// Set has savepoint
    /// 设置有保存点
    pub fn set_has_savepoint(&self)
    {
        self.has_savepoint.store(true, Ordering::SeqCst);
    }

    /// Flush the transaction (if applicable)
    /// 刷新事务（如果适用）
    pub fn flush(&self)
    {
        // Default implementation does nothing
        // Subclasses can override for specific behavior
    }
}

/// Transaction state enum
/// 事务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransactionState
{
    /// Active transaction
    /// 活动事务
    Active,

    /// Committed
    /// 已提交
    Committed,

    /// Rolled back
    /// 已回滚
    RolledBack,

    /// Unknown state
    /// 未知状态
    Unknown,
}

impl TransactionState
{
    /// Check if transaction is active
    /// 检查事务是否活动
    pub(crate) fn is_active(self) -> bool
    {
        matches!(self, TransactionState::Active)
    }

    /// Check if transaction is completed
    /// 检查事务是否已完成
    pub(crate) fn is_completed(self) -> bool
    {
        matches!(self, TransactionState::Committed | TransactionState::RolledBack)
    }
}

/// Transaction savepoint
/// 事务保存点
///
/// Equivalent to Spring's Savepoint.
/// `等价于Spring的Savepoint`。
#[derive(Debug, Clone)]
pub(crate) struct Savepoint
{
    /// Savepoint name
    /// 保存点名称
    pub name: String,

    /// Savepoint ID
    /// 保存点ID
    pub id: Option<u64>,
}

impl Savepoint
{
    /// Create a new savepoint
    /// 创建新的保存点
    pub(crate) fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            id: None,
        }
    }

    /// Create with ID
    /// 使用ID创建
    pub(crate) fn with_id(name: impl Into<String>, id: u64) -> Self
    {
        Self {
            name: name.into(),
            id: Some(id),
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_transaction_status()
    {
        let status = TransactionStatus::new("test_tx");

        assert!(status.is_new_transaction());
        assert!(!status.is_completed());
        assert!(!status.is_rollback_only());

        status.set_rollback_only();
        assert!(status.is_rollback_only());

        status.mark_completed();
        assert!(status.is_completed());
    }

    #[test]
    fn test_transaction_state()
    {
        assert!(TransactionState::Active.is_active());
        assert!(!TransactionState::Active.is_completed());
        assert!(TransactionState::Committed.is_completed());
        assert!(TransactionState::RolledBack.is_completed());
    }
}
