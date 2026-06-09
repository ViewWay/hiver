//! Transaction propagation behaviors
//! 事务传播行为
//!
//! Equivalent to Spring's `@Transactional(propagation = ...)`.
//! 等价于 Spring 的 `@Transactional(propagation = ...)`.

/// Transaction propagation behavior.
/// 事务传播行为。
///
/// Defines how a transactional method should behave when called
/// from within an existing transaction context.
///
/// 定义事务方法在已有事务上下文中被调用时应该如何表现。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Propagation {
    /// Support a current transaction; create a new one if none exists.
    /// 支持当前事务；如果不存在则创建新事务。
    #[default]
    Required,

    /// Support a current transaction; execute non-transactionally if none exists.
    /// 支持当前事务；如果不存在则以非事务方式执行。
    Supports,

    /// Support a current transaction; throw an exception if none exists.
    /// 支持当前事务；如果不存在则抛出异常。
    Mandatory,

    /// Create a new transaction, suspending the current one if one exists.
    /// 创建新事务，挂起当前事务（如果存在）。
    RequiresNew,

    /// Execute non-transactionally, suspending any current transaction.
    /// 以非事务方式执行，挂起任何当前事务。
    NotSupported,

    /// Execute non-transactionally; throw an exception if a transaction exists.
    /// 以非事务方式执行；如果存在事务则抛出异常。
    Never,

    /// Execute within a nested transaction if a current transaction exists.
    /// 如果当前存在事务，则在嵌套事务中执行。
    Nested,
}

/// Transaction isolation level.
/// 事务隔离级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Isolation {
    /// Use the default isolation level of the underlying datastore.
    /// 使用底层数据存储的默认隔离级别。
    #[default]
    Default,

    /// Dirty reads, non-repeatable reads, and phantom reads can occur.
    /// 可能发生脏读、不可重复读和幻读。
    ReadUncommitted,

    /// Dirty reads are prevented; non-repeatable and phantom reads can occur.
    /// 防止脏读；可能发生不可重复读和幻读。
    ReadCommitted,

    /// Dirty reads and non-repeatable reads are prevented; phantom reads can occur.
    /// 防止脏读和不可重复读；可能发生幻读。
    RepeatableRead,

    /// Dirty reads, non-repeatable reads, and phantom reads are prevented.
    /// 防止脏读、不可重复读和幻读。
    Serializable,
}

/// Transaction definition combining propagation and isolation.
/// 事务定义，结合传播和隔离。
#[derive(Debug, Clone, Copy)]
pub struct TransactionDefinition {
    /// Propagation behavior.
    /// 传播行为。
    pub propagation: Propagation,

    /// Isolation level.
    /// 隔离级别。
    pub isolation: Isolation,

    /// Timeout in seconds (0 = no timeout).
    /// 超时时间（秒，0 = 无超时）。
    pub timeout_secs: u32,

    /// Read-only flag.
    /// 只读标志。
    pub read_only: bool,
}

impl Default for TransactionDefinition {
    fn default() -> Self {
        Self {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            timeout_secs: 0,
            read_only: false,
        }
    }
}

impl TransactionDefinition {
    /// Create a new transaction definition with defaults.
    /// 创建带默认值的事务定义。
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the propagation behavior.
    /// 设置传播行为。
    pub fn propagation(mut self, propagation: Propagation) -> Self {
        self.propagation = propagation;
        self
    }

    /// Set the isolation level.
    /// 设置隔离级别。
    pub fn isolation(mut self, isolation: Isolation) -> Self {
        self.isolation = isolation;
        self
    }

    /// Set the timeout in seconds.
    /// 设置超时时间（秒）。
    pub fn timeout_secs(mut self, secs: u32) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set the read-only flag.
    /// 设置只读标志。
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
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

    #[test]
    fn test_propagation_default() {
        assert_eq!(Propagation::default(), Propagation::Required);
    }

    #[test]
    fn test_transaction_definition_builder() {
        let def = TransactionDefinition::new()
            .propagation(Propagation::RequiresNew)
            .isolation(Isolation::ReadCommitted)
            .timeout_secs(30)
            .read_only();

        assert_eq!(def.propagation, Propagation::RequiresNew);
        assert_eq!(def.isolation, Isolation::ReadCommitted);
        assert_eq!(def.timeout_secs, 30);
        assert!(def.read_only);
    }
}
