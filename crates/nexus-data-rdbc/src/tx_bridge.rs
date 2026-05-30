//! Transaction bridge between nexus-data-rdbc and nexus-tx.
//! nexus-data-rdbc 与 nexus-tx 之间的桥接事务模块。
//!
//! Provides bidirectional `IsolationLevel` conversions and a `RdbcTransactionManager`
//! that implements `nexus_tx::TransactionManager` by delegating to a `DatabaseClient`.
//!
//! 提供双向 `IsolationLevel` 转换和 `RdbcTransactionManager`，
//! 通过委托给 `DatabaseClient` 来实现 `nexus_tx::TransactionManager`。
//!
//! # Feature / 特性
//!
//! This module is only available when the `tx-bridge` feature is enabled:
//! 此模块仅在启用 `tx-bridge` feature 时可用：
//!
//! ```toml
//! [dependencies]
//! nexus-data-rdbc = { version = "0.1", features = ["tx-bridge"] }
//! ```

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use nexus_tx::synchronization::{LiveTransaction, bind_transaction, take_transaction};
use nexus_tx::{
    TransactionDefinition, TransactionError, TransactionManager, TransactionResult,
    TransactionStatus,
};

use crate::client::DatabaseClient;
use crate::transaction::IsolationLevel as RdbcIsolation;

// ──────────────────────────────────────────────────────────────────────────────
// Current Transaction Context / 当前事务上下文
// ──────────────────────────────────────────────────────────────────────────────

tokio::task_local! {
    static CURRENT_RDBC_TX: std::cell::RefCell<Option<crate::Transaction>>;
}

/// Store a transaction in the current task-local context.
/// 将事务存储到当前 task-local 上下文中。
pub(crate) fn set_current_tx(tx: &crate::Transaction) {
    // best-effort: if we're not inside a CURRENT_RDBC_TX scope, this is a no-op
    let _ = CURRENT_RDBC_TX.try_with(|cell| {
        *cell.borrow_mut() = Some(tx.clone());
    });
}

/// Clear the transaction from the current task-local context.
/// 从当前 task-local 上下文中清除事务。
pub(crate) fn clear_current_tx() {
    let _ = CURRENT_RDBC_TX.try_with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Try to get the current transaction from the task-local context.
/// 尝试从 task-local 上下文获取当前事务。
pub fn try_current_transaction() -> Option<crate::Transaction> {
    CURRENT_RDBC_TX
        .try_with(|cell| cell.borrow().clone())
        .ok()
        .flatten()
}

// ──────────────────────────────────────────────────────────────────────────────
// IsolationLevel Conversions / 隔离级别转换
// ──────────────────────────────────────────────────────────────────────────────

/// Convert from nexus-tx IsolationLevel to nexus-data-rdbc IsolationLevel.
/// 从 nexus-tx IsolationLevel 转换为 nexus-data-rdbc IsolationLevel。
impl From<nexus_tx::IsolationLevel> for RdbcIsolation {
    fn from(level: nexus_tx::IsolationLevel) -> Self {
        match level {
            nexus_tx::IsolationLevel::ReadUncommitted => RdbcIsolation::ReadUncommitted,
            nexus_tx::IsolationLevel::ReadCommitted => RdbcIsolation::ReadCommitted,
            nexus_tx::IsolationLevel::RepeatableRead => RdbcIsolation::RepeatableRead,
            nexus_tx::IsolationLevel::Serializable => RdbcIsolation::Serializable,
            nexus_tx::IsolationLevel::Default => RdbcIsolation::ReadCommitted,
        }
    }
}

/// Convert from nexus-data-rdbc IsolationLevel to nexus-tx IsolationLevel.
/// 从 nexus-data-rdbc IsolationLevel 转换为 nexus-tx IsolationLevel。
impl From<RdbcIsolation> for nexus_tx::IsolationLevel {
    fn from(level: RdbcIsolation) -> Self {
        match level {
            RdbcIsolation::ReadUncommitted => nexus_tx::IsolationLevel::ReadUncommitted,
            RdbcIsolation::ReadCommitted => nexus_tx::IsolationLevel::ReadCommitted,
            RdbcIsolation::RepeatableRead => nexus_tx::IsolationLevel::RepeatableRead,
            RdbcIsolation::Serializable => nexus_tx::IsolationLevel::Serializable,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// RdbcTransactionManager / RDBC 事务管理器
// ──────────────────────────────────────────────────────────────────────────────

/// Wraps a nexus-data-rdbc `Transaction` so it can be stored in the
/// nexus-tx synchronization map via `LiveTransaction`.
/// 包装 nexus-data-rdbc 的 `Transaction`，使其可以通过 `LiveTransaction` 存储在 nexus-tx 同步映射中。
struct RdbcLiveTx {
    tx: crate::Transaction,
}

impl LiveTransaction for RdbcLiveTx {
    fn commit_boxed(self: Box<Self>) -> Pin<Box<dyn std::future::Future<Output = TransactionResult<()>> + Send>> {
        Box::pin(async move {
            self.tx
                .commit()
                .await
                .map_err(|e| TransactionError::CommitFailed(format!("{e}")))
        })
    }

    fn rollback_boxed(self: Box<Self>) -> Pin<Box<dyn std::future::Future<Output = TransactionResult<()>> + Send>> {
        Box::pin(async move {
            self.tx
                .rollback()
                .await
                .map_err(|e| TransactionError::RollbackFailed(format!("{e}")))
        })
    }
}

/// A `TransactionManager` implementation that delegates to a `DatabaseClient`.
/// 通过委托给 `DatabaseClient` 实现的 `TransactionManager`。
///
/// This allows nexus-tx's `TransactionTemplate` and `#[Transactional]` to work with
/// any database backend supported by nexus-data-rdbc.
///
/// 使 nexus-tx 的 `TransactionTemplate` 和 `#[Transactional]` 能够与
/// nexus-data-rdbc 支持的任何数据库后端一起工作。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_rdbc::{DatabaseClient, tx_bridge::RdbcTransactionManager};
/// use nexus_tx::TransactionTemplate;
///
/// let client = SqlxPoolClient::new(pool).await?;
/// let tx_mgr = RdbcTransactionManager::new(client, "default");
/// let template = TransactionTemplate::new(tx_mgr);
///
/// let result = template.execute(|| async {
///     // Database operations here run within the rdbc transaction
///     Ok(())
/// }).await?;
/// ```
pub struct RdbcTransactionManager<C> {
    client: Arc<C>,
    name: String,
}

impl<C> RdbcTransactionManager<C>
where
    C: DatabaseClient + 'static,
{
    /// Create a new RdbcTransactionManager wrapping a DatabaseClient.
    /// 创建新的 RdbcTransactionManager 包装 DatabaseClient。
    pub fn new(client: C, name: impl Into<String>) -> Self {
        Self {
            client: Arc::new(client),
            name: name.into(),
        }
    }

    /// Get a reference to the underlying client.
    /// 获取底层客户端的引用。
    pub fn client(&self) -> &C {
        &self.client
    }
}

#[async_trait]
impl<C> TransactionManager for RdbcTransactionManager<C>
where
    C: DatabaseClient + 'static,
{
    async fn begin(&self, definition: &TransactionDefinition) -> TransactionResult<TransactionStatus> {
        let status = TransactionStatus::new(definition.name.clone());
        let tx = self
            .client
            .begin_transaction()
            .await
            .map_err(|e| TransactionError::CreationFailed(format!("{e}")))?;
        set_current_tx(&tx);
        bind_transaction(status.clone(), Box::new(RdbcLiveTx { tx })).await;
        Ok(status)
    }

    async fn commit(&self, status: TransactionStatus) -> TransactionResult<()> {
        clear_current_tx();
        if let Some(live) = take_transaction(&status).await {
            live.commit_boxed().await
        } else {
            Err(TransactionError::InvalidState(
                "no active transaction found for commit".to_string(),
            ))
        }
    }

    async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()> {
        clear_current_tx();
        if let Some(live) = take_transaction(&status).await {
            live.rollback_boxed().await
        } else {
            Err(TransactionError::InvalidState(
                "no active transaction found for rollback".to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests / 测试
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_to_rdbc_isolation() {
        assert_eq!(
            RdbcIsolation::from(nexus_tx::IsolationLevel::ReadUncommitted),
            RdbcIsolation::ReadUncommitted,
        );
        assert_eq!(
            RdbcIsolation::from(nexus_tx::IsolationLevel::Serializable),
            RdbcIsolation::Serializable,
        );
    }

    #[test]
    fn test_rdbc_to_tx_isolation() {
        assert_eq!(
            nexus_tx::IsolationLevel::from(RdbcIsolation::RepeatableRead),
            nexus_tx::IsolationLevel::RepeatableRead,
        );
    }

    #[test]
    fn test_tx_default_maps_to_read_committed() {
        assert_eq!(
            RdbcIsolation::from(nexus_tx::IsolationLevel::Default),
            RdbcIsolation::ReadCommitted,
        );
    }
}
