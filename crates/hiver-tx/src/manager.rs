//! Transaction manager
//! 事务管理器

use async_trait::async_trait;

use crate::{IsolationLevel, Propagation, TransactionError, TransactionResult, TransactionStatus};

/// Transaction manager trait
/// 事务管理器trait
///
/// Equivalent to Spring's `PlatformTransactionManager` interface.
/// `等价于Spring的PlatformTransactionManager接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface PlatformTransactionManager {
///     TransactionStatus getTransaction(TransactionDefinition definition);
///     void commit(TransactionStatus status);
///     void rollback(TransactionStatus status);
/// }
/// ```
#[async_trait]
pub trait TransactionManager: Send + Sync
{
    /// Begin a new transaction
    /// 开始新事务
    ///
    /// Returns a transaction status that can be used for commit/rollback.
    /// 返回可用于提交/回滚的事务状态。
    async fn begin(
        &self,
        definition: &TransactionDefinition,
    ) -> TransactionResult<TransactionStatus>;

    /// Commit the transaction
    /// 提交事务
    async fn commit(&self, status: TransactionStatus) -> TransactionResult<()>;

    /// Rollback the transaction
    /// 回滚事务
    async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()>;

    /// Get transaction manager name
    /// 获取事务管理器名称
    fn name(&self) -> &str;
}

/// Transaction definition
/// 事务定义
///
/// Defines transaction attributes.
/// 定义事务属性。
///
/// Equivalent to Spring's `TransactionDefinition`.
/// `等价于Spring的TransactionDefinition`。
#[derive(Debug, Clone)]
pub struct TransactionDefinition
{
    /// Transaction name
    /// 事务名称
    pub name: String,

    /// Propagation behavior
    /// 传播行为
    pub propagation: Propagation,

    /// Isolation level
    /// 隔离级别
    pub isolation: IsolationLevel,

    /// Timeout in seconds
    /// 超时时间（秒）
    pub timeout_secs: Option<u64>,

    /// Read-only flag
    /// 只读标志
    pub read_only: bool,
}

impl TransactionDefinition
{
    /// Create a new transaction definition
    /// 创建新的事务定义
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            propagation: Propagation::default(),
            isolation: IsolationLevel::default(),
            timeout_secs: Some(crate::DEFAULT_TX_TIMEOUT_SECS),
            read_only: false,
        }
    }

    /// Set propagation
    /// 设置传播行为
    pub fn propagation(mut self, propagation: Propagation) -> Self
    {
        self.propagation = propagation;
        self
    }

    /// Set isolation
    /// 设置隔离级别
    pub fn isolation(mut self, isolation: IsolationLevel) -> Self
    {
        self.isolation = isolation;
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
}

impl Default for TransactionDefinition
{
    fn default() -> Self
    {
        Self::new(crate::DEFAULT_TX_NAME)
    }
}

/// Transaction manager builder
/// 事务管理器构建器
///
/// Equivalent to Spring's `TransactionManagerCustomizer`.
/// `等价于Spring的TransactionManagerCustomizer`。
#[derive(Debug, Clone)]
pub struct TransactionManagerBuilder
{
    /// Default timeout
    /// 默认超时
    default_timeout_secs: u64,

    /// Default isolation level
    /// 默认隔离级别
    default_isolation: IsolationLevel,

    /// Default propagation
    /// 默认传播行为
    default_propagation: Propagation,

    /// Enable transactions
    /// 启用事务
    enable_transactions: bool,
}

impl TransactionManagerBuilder
{
    /// Create a new builder
    /// 创建新的构建器
    pub fn new() -> Self
    {
        Self {
            default_timeout_secs: crate::DEFAULT_TX_TIMEOUT_SECS,
            default_isolation: IsolationLevel::Default,
            default_propagation: Propagation::Required,
            enable_transactions: true,
        }
    }

    /// Set default timeout
    /// 设置默认超时
    pub fn default_timeout_secs(mut self, timeout: u64) -> Self
    {
        self.default_timeout_secs = timeout;
        self
    }

    /// Set default isolation level
    /// 设置默认隔离级别
    pub fn default_isolation(mut self, isolation: IsolationLevel) -> Self
    {
        self.default_isolation = isolation;
        self
    }

    /// Set default propagation
    /// 设置默认传播行为
    pub fn default_propagation(mut self, propagation: Propagation) -> Self
    {
        self.default_propagation = propagation;
        self
    }

    /// Enable or disable transactions
    /// 启用或禁用事务
    pub fn enable_transactions(mut self, enable: bool) -> Self
    {
        self.enable_transactions = enable;
        self
    }

    /// Build transaction definition
    /// 构建事务定义
    pub fn build_definition(&self, name: impl Into<String>) -> TransactionDefinition
    {
        TransactionDefinition {
            name: name.into(),
            propagation: self.default_propagation,
            isolation: self.default_isolation,
            timeout_secs: Some(self.default_timeout_secs),
            read_only: false,
        }
    }
}

impl Default for TransactionManagerBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Simple in-memory transaction manager
/// 简单内存事务管理器
///
/// For demonstration purposes. In production, use a database-specific implementation.
/// 用于演示目的。在生产中，使用特定数据库的实现。
#[derive(Debug)]
pub(crate) struct SimpleTransactionManager
{
    name: String,
}

impl SimpleTransactionManager
{
    /// Create a new simple transaction manager
    /// 创建新的简单事务管理器
    pub(crate) fn new() -> Self
    {
        Self {
            name: "simple".to_string(),
        }
    }

    /// Create with name
    /// 使用名称创建
    pub(crate) fn with_name(name: impl Into<String>) -> Self
    {
        Self { name: name.into() }
    }

    /// Build a transaction definition
    /// 构建事务定义
    pub(crate) fn build_definition(&self, name: impl Into<String>) -> TransactionDefinition
    {
        TransactionDefinition::new(name)
            .propagation(Propagation::Required)
            .isolation(IsolationLevel::ReadCommitted)
            .timeout_secs(crate::DEFAULT_TX_TIMEOUT_SECS)
            .read_only(false)
    }
}

impl Default for SimpleTransactionManager
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl TransactionManager for SimpleTransactionManager
{
    async fn begin(
        &self,
        definition: &TransactionDefinition,
    ) -> TransactionResult<TransactionStatus>
    {
        // Create a new transaction status
        Ok(TransactionStatus::new(&definition.name))
    }

    async fn commit(&self, status: TransactionStatus) -> TransactionResult<()>
    {
        if status.is_rollback_only()
        {
            return Err(TransactionError::InvalidState(
                "Cannot commit transaction marked as rollback-only".to_string(),
            ));
        }
        status.mark_completed();
        Ok(())
    }

    async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()>
    {
        status.mark_completed();
        Ok(())
    }

    fn name(&self) -> &str
    {
        &self.name
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_transaction_definition()
    {
        let def = TransactionDefinition::new("test")
            .propagation(Propagation::RequiresNew)
            .isolation(IsolationLevel::Serializable)
            .timeout_secs(60)
            .read_only(true);

        assert_eq!(def.name, "test");
        assert_eq!(def.propagation, Propagation::RequiresNew);
        assert_eq!(def.isolation, IsolationLevel::Serializable);
        assert_eq!(def.timeout_secs, Some(60));
        assert!(def.read_only);
    }

    #[tokio::test]
    async fn test_simple_transaction_manager()
    {
        let manager = SimpleTransactionManager::new();
        let def = manager.build_definition("test");

        let status = manager.begin(&def).await.unwrap();
        assert!(status.is_new_transaction());

        manager.commit(status).await.unwrap();
    }
}

use std::sync::{Arc, OnceLock};

/// Global transaction manager installed at application startup.
/// 应用启动时安装的全局事务管理器。
static GLOBAL_TX_MANAGER: OnceLock<Arc<dyn TransactionManager>> = OnceLock::new();

/// Install the global `TransactionManager`.
/// 安装全局 `TransactionManager`。
pub fn set_global_tx_manager(mgr: Arc<dyn TransactionManager>)
{
    let _ = GLOBAL_TX_MANAGER.set(mgr);
}

/// Get the global `TransactionManager` if registered.
/// 获取已注册的全局 `TransactionManager`。
pub fn global_tx_manager() -> Option<Arc<dyn TransactionManager>>
{
    GLOBAL_TX_MANAGER.get().cloned()
}

/// No-op transaction manager used when no datasource is configured.
/// 未配置数据源时使用的空操作事务管理器。
#[derive(Debug, Default)]
pub struct NoopTransactionManager;

#[async_trait::async_trait]
impl TransactionManager for NoopTransactionManager
{
    async fn begin(
        &self,
        definition: &TransactionDefinition,
    ) -> TransactionResult<TransactionStatus>
    {
        Ok(TransactionStatus::new(&definition.name))
    }

    async fn commit(&self, status: TransactionStatus) -> TransactionResult<()>
    {
        status.mark_completed();
        Ok(())
    }

    async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()>
    {
        status.mark_completed();
        Ok(())
    }

    fn name(&self) -> &'static str
    {
        "noop"
    }
}
