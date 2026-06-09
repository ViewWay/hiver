//! Transaction implementation
//! 事务实现

#![allow(dead_code)]
use std::sync::Arc;

use crate::{Propagation, TransactionStatus};

/// Transaction
/// 事务
///
/// Represents an active transaction.
/// 表示活动事务。
///
/// Equivalent to Spring's `TransactionRepresentation`.
/// `等价于Spring的TransactionRepresentation`。
#[derive(Debug, Clone)]
pub struct Transaction
{
    /// Transaction status
    /// 事务状态
    status: TransactionStatus,

    /// Transaction manager name
    /// 事务管理器名称
    manager_name: String,

    /// Propagation behavior
    /// 传播行为
    propagation: Propagation,

    /// Nested transactions
    /// 嵌套事务
    nested: Vec<Transaction>,
}

impl Transaction
{
    /// Create a new transaction
    /// 创建新事务
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            status: TransactionStatus::new(name),
            manager_name: "default".to_string(),
            propagation: Propagation::Required,
            nested: Vec::new(),
        }
    }

    /// Create with existing status
    /// 使用现有状态创建
    pub fn with_status(status: TransactionStatus) -> Self
    {
        Self {
            status,
            manager_name: "default".to_string(),
            propagation: Propagation::Required,
            nested: Vec::new(),
        }
    }

    /// Get transaction status
    /// 获取事务状态
    pub fn status(&self) -> &TransactionStatus
    {
        &self.status
    }

    /// Get mutable transaction status
    /// 获取可变事务状态
    pub fn status_mut(&mut self) -> &mut TransactionStatus
    {
        &mut self.status
    }

    /// Get manager name
    /// 获取管理器名称
    pub fn manager_name(&self) -> &str
    {
        &self.manager_name
    }

    /// Set manager name
    /// 设置管理器名称
    pub fn set_manager_name(&mut self, name: impl Into<String>)
    {
        self.manager_name = name.into();
    }

    /// Get propagation behavior
    /// 获取传播行为
    pub fn propagation(&self) -> Propagation
    {
        self.propagation
    }

    /// Set propagation behavior
    /// 设置传播行为
    pub fn set_propagation(&mut self, propagation: Propagation)
    {
        self.propagation = propagation;
    }

    /// Create a nested transaction
    /// 创建嵌套事务
    pub fn create_nested(&mut self, name: impl Into<String>) -> Transaction
    {
        let mut nested = Transaction::new(name);
        nested.propagation = Propagation::Nested;
        nested
    }

    /// Get nested transactions
    /// 获取嵌套事务
    pub fn nested(&self) -> &[Transaction]
    {
        &self.nested
    }

    /// Check if transaction is active
    /// 检查事务是否活动
    pub fn is_active(&self) -> bool
    {
        !self.status.is_completed()
    }

    /// Check if transaction is rollback only
    /// 检查事务是否仅回滚
    pub fn is_rollback_only(&self) -> bool
    {
        self.status.is_rollback_only()
    }

    /// Mark for rollback only
    /// 标记为仅回滚
    pub fn mark_rollback_only(&self)
    {
        self.status.set_rollback_only();
    }
}

/// Transaction holder for thread-local storage
/// 事务的线程本地存储持有者
///
/// Equivalent to Spring's `TransactionSynchronizationManager`.
/// `等价于Spring的TransactionSynchronizationManager`。
pub(crate) struct TransactionHolder
{
    /// Current transaction
    /// 当前事务
    current: Arc<tokio::sync::RwLock<Option<Transaction>>>,

    /// Transaction stack
    /// 事务栈
    stack: Arc<tokio::sync::RwLock<Vec<Transaction>>>,
}

impl TransactionHolder
{
    /// Create a new transaction holder
    /// 创建新的事务持有者
    pub(crate) fn new() -> Self
    {
        Self {
            current: Arc::new(tokio::sync::RwLock::new(None)),
            stack: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Get current transaction
    /// 获取当前事务
    pub(crate) async fn current(&self) -> Option<Transaction>
    {
        self.current.read().await.clone()
    }

    /// Set current transaction
    /// 设置当前事务
    pub(crate) async fn set_current(&self, tx: Transaction)
    {
        let mut current = self.current.write().await;
        *current = Some(tx);
    }

    /// Clear current transaction
    /// 清除当前事务
    pub(crate) async fn clear(&self)
    {
        let mut current = self.current.write().await;
        *current = None;
    }

    /// Push transaction onto stack
    /// 将事务压入栈
    pub(crate) async fn push(&self, tx: Transaction)
    {
        let mut stack = self.stack.write().await;
        stack.push(tx);
    }

    /// Pop transaction from stack
    /// 从栈弹出事务
    pub(crate) async fn pop(&self) -> Option<Transaction>
    {
        let mut stack = self.stack.write().await;
        stack.pop()
    }

    /// Get stack depth
    /// 获取栈深度
    pub(crate) async fn depth(&self) -> usize
    {
        self.stack.read().await.len()
    }
}

impl Default for TransactionHolder
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Global transaction holder
/// 全局事务持有者
static GLOBAL_HOLDER: std::sync::LazyLock<TransactionHolder> =
    std::sync::LazyLock::new(TransactionHolder::new);

/// Get global transaction holder
/// 获取全局事务持有者
pub(crate) fn global_holder() -> &'static TransactionHolder
{
    &GLOBAL_HOLDER
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

    #[tokio::test]
    async fn test_transaction()
    {
        let tx = Transaction::new("test");

        assert!(tx.is_active());
        assert!(!tx.is_rollback_only());
        assert!(tx.status().is_new_transaction());
    }

    #[tokio::test]
    async fn test_transaction_holder()
    {
        let holder = TransactionHolder::new();
        let tx = Transaction::new("test");

        holder.set_current(tx.clone()).await;
        let retrieved = holder.current().await;

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().status().name(), "test");

        holder.clear().await;
        assert!(holder.current().await.is_none());
    }
}
