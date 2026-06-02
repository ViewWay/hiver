//! Transaction synchronization manager (task-local active transactions).
//! 事务同步管理器（任务本地活跃事务）。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::manager::TransactionDefinition;
use crate::{Propagation, TransactionError, TransactionResult, TransactionStatus};

/// Holds a live database transaction behind a mutex.
/// 在互斥锁后持有活动的数据库事务。
pub trait LiveTransaction: Send {
    /// Commit the underlying transaction.
    fn commit_boxed(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn Future<Output = TransactionResult<()>> + Send>>;
    /// Roll back the underlying transaction.
    fn rollback_boxed(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn Future<Output = TransactionResult<()>> + Send>>;
}

tokio::task_local! {
    static ACTIVE_TX: Arc<Mutex<HashMap<String, (TransactionStatus, Box<dyn LiveTransaction>)>>>;
}

fn active_map() -> Arc<Mutex<HashMap<String, (TransactionStatus, Box<dyn LiveTransaction>)>>> {
    ACTIVE_TX.try_with(Clone::clone).expect(
        "No active transaction scope: wrap your transactional code with `with_transaction_scope`",
    )
}

/// Initialize the task-local transaction map for the duration of `f`.
/// 为 `f` 的执行期间初始化任务本地事务映射。
///
/// The `#[transactional]` macro wraps the annotated method body with this call so
/// that `bind_transaction` / `take_transaction` share the same `Arc<Mutex<HashMap>>`
/// throughout the whole begin → commit/rollback lifecycle.
///
/// `#[transactional]` 宏将带注解的方法体包裹在此调用中，使
/// `bind_transaction` / `take_transaction` 在整个 begin → commit/rollback
/// 生命周期中共享同一个 `Arc<Mutex<HashMap>>`。
pub async fn with_transaction_scope<F, R>(f: F) -> R
where
    F: Future<Output = R>,
{
    ACTIVE_TX
        .scope(Arc::new(Mutex::new(HashMap::new())), f)
        .await
}

/// Bind a live transaction to a status name in the current task.
/// 将活动事务绑定到当前任务中的状态名称。
pub async fn bind_transaction(status: TransactionStatus, tx: Box<dyn LiveTransaction>) {
    let map = active_map();
    let mut guard = map.lock().await;
    guard.insert(status.name().to_string(), (status, tx));
}

/// Take and remove a live transaction for commit/rollback.
/// 取出并移除用于提交/回滚的活动事务。
pub async fn take_transaction(status: &TransactionStatus) -> Option<Box<dyn LiveTransaction>> {
    let map = active_map();
    let mut guard = map.lock().await;
    guard.remove(status.name()).map(|(_, tx)| tx)
}

/// Returns an existing active status for REQUIRED propagation, if any.
/// 对于 REQUIRED 传播，返回现有活动状态（如有）。
pub async fn current_status() -> Option<TransactionStatus> {
    let map = active_map();
    let guard = map.lock().await;
    guard.values().next().map(|(s, _)| s.clone())
}

/// Resolve propagation: returns existing status or indicates a new transaction is needed.
/// 解析传播行为：返回现有状态或指示需要新事务。
pub async fn resolve_propagation(
    definition: &TransactionDefinition,
) -> Result<Option<TransactionStatus>, TransactionError> {
    match definition.propagation {
        Propagation::Required | Propagation::Supports => Ok(current_status().await),
        Propagation::Mandatory => match current_status().await {
            Some(s) => Ok(Some(s)),
            None => {
                Err(TransactionError::InvalidState("No existing transaction for MANDATORY".into()))
            },
        },
        Propagation::Never => {
            if current_status().await.is_some() {
                Err(TransactionError::InvalidState("Existing transaction present for NEVER".into()))
            } else {
                Ok(None)
            }
        },
        Propagation::NotSupported | Propagation::RequiresNew | Propagation::Nested => Ok(None),
    }
}
