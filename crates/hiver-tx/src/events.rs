//! Transaction synchronization callbacks and events.
//! 事务同步回调和事件。
//!
//! Equivalent to Spring's `TransactionSynchronization` and
//! `@TransactionalEventListener`.
//!
//! 等价于 Spring 的 `TransactionSynchronization` 和
//! `@TransactionalEventListener`。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_tx::events::{TransactionPhase, TransactionSynchronization, SynchronizationRegistry};
//!
//! let registry = SynchronizationRegistry::new();
//! registry.register(MySync);
//!
//! // During commit, beforeCommit callbacks fire, then afterCommit
//! registry.trigger_before_commit(&tx_name).await;
//! registry.trigger_after_commit(&tx_name).await;
//! ```

use std::sync::Arc;

use tokio::sync::RwLock;

/// Phase at which a transaction callback fires.
/// 事务回调触发的阶段。
///
/// Equivalent to Spring's `TransactionPhase` used by `@TransactionalEventListener`.
/// 等价于 Spring 的 `@TransactionalEventListener` 使用的 `TransactionPhase`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionPhase {
    /// Before commit (can still trigger rollback).
    /// 提交前（仍可触发回滚）。
    BeforeCommit,
    /// After successful commit.
    /// 成功提交后。
    AfterCommit,
    /// After rollback.
    /// 回滚后。
    AfterRollback,
    /// After completion (commit or rollback).
    /// 完成后（提交或回滚）。
    AfterCompletion,
}

/// Callback trait for transaction lifecycle events.
/// 事务生命周期事件的回调 trait。
///
/// Equivalent to Spring's `TransactionSynchronization`.
/// 等价于 Spring 的 `TransactionSynchronization`。
#[async_trait::async_trait]
pub trait TransactionSynchronization: Send + Sync {
    /// Called before the transaction commits. Return `Err(())` to vote for rollback.
    /// 在事务提交前调用。返回 `Err(())` 投票回滚。
    async fn before_commit(&self, _tx_name: &str) -> Result<(), ()> {
        Ok(())
    }

    /// Called before completion (commit or rollback).
    /// 在完成前调用（提交或回滚）。
    async fn before_completion(&self, _tx_name: &str) {}

    /// Called after a successful commit.
    /// 在成功提交后调用。
    async fn after_commit(&self, _tx_name: &str) {}

    /// Called after a rollback.
    /// 在回滚后调用。
    async fn after_rollback(&self, _tx_name: &str) {}

    /// Called after completion (commit or rollback).
    /// 在完成后调用（提交或回滚）。
    async fn after_completion(&self, _tx_name: &str, _committed: bool) {}

    /// Human-readable name for debugging.
    /// 用于调试的可读名称。
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A phase-filtered listener — fires only on a specific phase.
/// 阶段过滤监听器 — 仅在特定阶段触发。
///
/// Equivalent to Spring's `@TransactionalEventListener(phase = ...)`.
/// 等价于 Spring 的 `@TransactionalEventListener(phase = ...)`。
pub struct PhaseListener {
    phase: TransactionPhase,
    callback: Arc<dyn Fn(&str) + Send + Sync>,
}

impl PhaseListener {
    /// Create a listener that fires at the given phase.
    /// 创建在给定阶段触发的监听器。
    pub fn new(phase: TransactionPhase, callback: impl Fn(&str) + Send + Sync + 'static) -> Self {
        Self {
            phase,
            callback: Arc::new(callback),
        }
    }
}

#[async_trait::async_trait]
impl TransactionSynchronization for PhaseListener {
    async fn before_commit(&self, tx_name: &str) -> Result<(), ()> {
        if self.phase == TransactionPhase::BeforeCommit {
            (self.callback)(tx_name);
        }
        Ok(())
    }

    async fn after_commit(&self, tx_name: &str) {
        if self.phase == TransactionPhase::AfterCommit {
            (self.callback)(tx_name);
        }
    }

    async fn after_rollback(&self, tx_name: &str) {
        if self.phase == TransactionPhase::AfterRollback {
            (self.callback)(tx_name);
        }
    }

    async fn after_completion(&self, tx_name: &str, _committed: bool) {
        if self.phase == TransactionPhase::AfterCompletion {
            (self.callback)(tx_name);
        }
    }

    fn name(&self) -> &'static str {
        match self.phase {
            TransactionPhase::BeforeCommit => "PhaseListener::BeforeCommit",
            TransactionPhase::AfterCommit => "PhaseListener::AfterCommit",
            TransactionPhase::AfterRollback => "PhaseListener::AfterRollback",
            TransactionPhase::AfterCompletion => "PhaseListener::AfterCompletion",
        }
    }
}

/// Registry of transaction synchronization callbacks.
/// 事务同步回调注册表。
///
/// Equivalent to Spring's `TransactionSynchronizationManager`.
/// 等价于 Spring 的 `TransactionSynchronizationManager`。
pub struct SynchronizationRegistry {
    synchronizations: Arc<RwLock<Vec<Arc<dyn TransactionSynchronization>>>>,
}

impl SynchronizationRegistry {
    /// Create an empty registry.
    /// 创建空注册表。
    pub fn new() -> Self {
        Self {
            synchronizations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a synchronization callback.
    /// 注册同步回调。
    pub fn register(&self, sync: impl TransactionSynchronization + 'static) {
        if let Ok(mut guard) = self.synchronizations.try_write() {
            guard.push(Arc::new(sync));
        }
    }

    /// Number of registered synchronizations.
    /// 已注册的同步数量。
    pub async fn count(&self) -> usize {
        self.synchronizations.read().await.len()
    }

    /// Clear all synchronizations.
    /// 清除所有同步。
    pub async fn clear(&self) {
        self.synchronizations.write().await.clear();
    }

    /// Trigger before-commit callbacks. Returns `Err(())` if any votes for rollback.
    /// 触发提交前回调。如果有投票回滚则返回 `Err(())`。
    pub async fn trigger_before_commit(&self, tx_name: &str) -> Result<(), ()> {
        let syncs = self.synchronizations.read().await;
        for sync in syncs.iter() {
            sync.before_commit(tx_name).await?;
        }
        Ok(())
    }

    /// Trigger before-completion callbacks.
    /// 触发完成前回调。
    pub async fn trigger_before_completion(&self, tx_name: &str) {
        let syncs = self.synchronizations.read().await;
        for sync in syncs.iter() {
            sync.before_completion(tx_name).await;
        }
    }

    /// Trigger after-commit callbacks.
    /// 触发提交后回调。
    pub async fn trigger_after_commit(&self, tx_name: &str) {
        let syncs = self.synchronizations.read().await;
        for sync in syncs.iter() {
            sync.after_commit(tx_name).await;
        }
    }

    /// Trigger after-rollback callbacks.
    /// 触发回滚后回调。
    pub async fn trigger_after_rollback(&self, tx_name: &str) {
        let syncs = self.synchronizations.read().await;
        for sync in syncs.iter() {
            sync.after_rollback(tx_name).await;
        }
    }

    /// Trigger after-completion callbacks.
    /// 触发完成后回调。
    pub async fn trigger_after_completion(&self, tx_name: &str, committed: bool) {
        let syncs = self.synchronizations.read().await;
        for sync in syncs.iter() {
            sync.after_completion(tx_name, committed).await;
        }
    }
}

impl Default for SynchronizationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SynchronizationRegistry {
    fn clone(&self) -> Self {
        Self {
            synchronizations: self.synchronizations.clone(),
        }
    }
}

/// A simple logging synchronization.
/// 简单的日志同步。
pub struct LoggingSynchronization;

#[async_trait::async_trait]
impl TransactionSynchronization for LoggingSynchronization {
    async fn before_commit(&self, tx_name: &str) -> Result<(), ()> {
        println!("[TxSync] before_commit: {}", tx_name);
        Ok(())
    }

    async fn after_commit(&self, tx_name: &str) {
        println!("[TxSync] after_commit: {}", tx_name);
    }

    async fn after_rollback(&self, tx_name: &str) {
        println!("[TxSync] after_rollback: {}", tx_name);
    }

    async fn after_completion(&self, tx_name: &str, committed: bool) {
        println!(
            "[TxSync] after_completion: {} ({})",
            tx_name,
            if committed {
                "committed"
            } else {
                "rolled back"
            }
        );
    }

    fn name(&self) -> &'static str {
        "LoggingSynchronization"
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct CountingSync {
        before_commit: AtomicUsize,
        after_commit: AtomicUsize,
        after_rollback: AtomicUsize,
        after_completion: AtomicUsize,
    }

    impl CountingSync {
        fn new() -> Self {
            Self {
                before_commit: AtomicUsize::new(0),
                after_commit: AtomicUsize::new(0),
                after_rollback: AtomicUsize::new(0),
                after_completion: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl TransactionSynchronization for CountingSync {
        async fn before_commit(&self, _: &str) -> Result<(), ()> {
            self.before_commit.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn after_commit(&self, _: &str) {
            self.after_commit.fetch_add(1, Ordering::SeqCst);
        }

        async fn after_rollback(&self, _: &str) {
            self.after_rollback.fetch_add(1, Ordering::SeqCst);
        }

        async fn after_completion(&self, _: &str, _: bool) {
            self.after_completion.fetch_add(1, Ordering::SeqCst);
        }
    }

    // Wrapper to make Arc<CountingSync> work as TransactionSynchronization
    struct W(Arc<CountingSync>);

    #[async_trait::async_trait]
    impl TransactionSynchronization for W {
        async fn before_commit(&self, tx: &str) -> Result<(), ()> {
            self.0.before_commit(tx).await
        }

        async fn after_commit(&self, tx: &str) {
            self.0.after_commit(tx).await;
        }

        async fn after_rollback(&self, tx: &str) {
            self.0.after_rollback(tx).await;
        }

        async fn after_completion(&self, tx: &str, c: bool) {
            self.0.after_completion(tx, c).await;
        }
    }

    #[tokio::test]
    async fn test_commit_lifecycle() {
        let registry = SynchronizationRegistry::new();
        let counter = Arc::new(CountingSync::new());
        registry.register(W(counter.clone()));

        registry.trigger_before_commit("tx1").await.unwrap();
        registry.trigger_before_completion("tx1").await;
        registry.trigger_after_commit("tx1").await;
        registry.trigger_after_completion("tx1", true).await;

        assert_eq!(counter.before_commit.load(Ordering::SeqCst), 1);
        assert_eq!(counter.after_commit.load(Ordering::SeqCst), 1);
        assert_eq!(counter.after_rollback.load(Ordering::SeqCst), 0);
        assert_eq!(counter.after_completion.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_rollback_lifecycle() {
        let registry = SynchronizationRegistry::new();
        let counter = Arc::new(CountingSync::new());
        registry.register(W(counter.clone()));

        registry.trigger_before_completion("tx2").await;
        registry.trigger_after_rollback("tx2").await;
        registry.trigger_after_completion("tx2", false).await;

        assert_eq!(counter.before_commit.load(Ordering::SeqCst), 0);
        assert_eq!(counter.after_commit.load(Ordering::SeqCst), 0);
        assert_eq!(counter.after_rollback.load(Ordering::SeqCst), 1);
        assert_eq!(counter.after_completion.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_phase_listener() {
        let fired = Arc::new(AtomicUsize::new(0));
        let f = fired.clone();
        let listener = PhaseListener::new(TransactionPhase::AfterCommit, move |_| {
            f.fetch_add(1, Ordering::SeqCst);
        });

        let registry = SynchronizationRegistry::new();
        registry.register(listener);

        registry.trigger_after_commit("tx3").await;
        registry.trigger_after_rollback("tx3").await;
        assert_eq!(fired.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_before_commit_veto() {
        struct VetoSync;
        #[async_trait::async_trait]
        impl TransactionSynchronization for VetoSync {
            async fn before_commit(&self, _: &str) -> Result<(), ()> {
                Err(())
            }
        }
        let registry = SynchronizationRegistry::new();
        registry.register(VetoSync);
        assert!(registry.trigger_before_commit("tx4").await.is_err());
    }

    #[tokio::test]
    async fn test_clear() {
        let registry = SynchronizationRegistry::new();
        registry.register(LoggingSynchronization);
        assert_eq!(registry.count().await, 1);
        registry.clear().await;
        assert_eq!(registry.count().await, 0);
    }
}
