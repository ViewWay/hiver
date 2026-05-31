//! Transaction isolation level
//! 事务隔离级别

use serde::{Deserialize, Serialize};

/// Transaction isolation level
/// 事务隔离级别
///
/// Equivalent to Spring's Isolation enum.
/// `等价于Spring的Isolation枚举`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Transactional(isolation = Isolation.SERIALIZABLE)
/// public void transferMoney() { ... }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum IsolationLevel {
    /// Read uncommitted
    /// 读未提交
    ///
    /// Lowest isolation level. Dirty reads, non-repeatable reads, and phantom reads can occur.
    /// 最低隔离级别。可能发生脏读、不可重复读和幻读。
    ReadUncommitted = 1,

    /// Read committed
    /// 读已提交
    ///
    /// Prevents dirty reads. Non-repeatable reads and phantom reads can occur.
    /// 防止脏读。可能发生不可重复读和幻读。
    ReadCommitted = 2,

    /// Repeatable read
    /// 可重复读
    ///
    /// Prevents dirty reads and non-repeatable reads. Phantom reads can occur.
    /// 防止脏读和不可重复读。可能发生幻读。
    RepeatableRead = 3,

    /// Serializable
    /// 串行化
    ///
    /// Highest isolation level. Prevents all concurrency issues.
    /// 最高隔离级别。防止所有并发问题。
    Serializable = 4,

    /// Default isolation level
    /// 默认隔离级别
    ///
    /// Uses the database's default isolation level (usually `ReadCommitted`).
    /// 使用数据库的默认隔离级别（通常是ReadCommitted）。
    #[default]
    Default = 0,
}

impl IsolationLevel {
    /// Get isolation level from value
    /// 从值获取隔离级别
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(IsolationLevel::Default),
            1 => Some(IsolationLevel::ReadUncommitted),
            2 => Some(IsolationLevel::ReadCommitted),
            3 => Some(IsolationLevel::RepeatableRead),
            4 => Some(IsolationLevel::Serializable),
            _ => None,
        }
    }

    /// Get the numeric value
    /// 获取数字值
    pub fn value(&self) -> i32 {
        *self as i32
    }

    /// Check if this is the default isolation level
    /// 检查是否为默认隔离级别
    pub fn is_default(&self) -> bool {
        matches!(self, IsolationLevel::Default)
    }

    /// Get description
    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "Read Uncommitted - allows dirty reads",
            IsolationLevel::ReadCommitted => "Read Committed - prevents dirty reads",
            IsolationLevel::RepeatableRead => {
                "Repeatable Read - prevents dirty and non-repeatable reads"
            },
            IsolationLevel::Serializable => "Serializable - full isolation",
            IsolationLevel::Default => "Default - uses database default",
        }
    }
}


impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationLevel::ReadUncommitted => write!(f, "READ_UNCOMMITTED"),
            IsolationLevel::ReadCommitted => write!(f, "READ_COMMITTED"),
            IsolationLevel::RepeatableRead => write!(f, "REPEATABLE_READ"),
            IsolationLevel::Serializable => write!(f, "SERIALIZABLE"),
            IsolationLevel::Default => write!(f, "DEFAULT"),
        }
    }
}

impl std::str::FromStr for IsolationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "READ_UNCOMMITTED" | "READ-UNCOMMITTED" => Ok(IsolationLevel::ReadUncommitted),
            "READ_COMMITTED" | "READ-COMMITTED" => Ok(IsolationLevel::ReadCommitted),
            "REPEATABLE_READ" | "REPEATABLE-READ" => Ok(IsolationLevel::RepeatableRead),
            "SERIALIZABLE" => Ok(IsolationLevel::Serializable),
            "DEFAULT" => Ok(IsolationLevel::Default),
            _ => Err(format!("Unknown isolation level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_from_str() {
        assert_eq!("SERIALIZABLE".parse::<IsolationLevel>().unwrap(), IsolationLevel::Serializable);
        assert_eq!(
            "read_committed".parse::<IsolationLevel>().unwrap(),
            IsolationLevel::ReadCommitted
        );
    }

    #[test]
    fn test_isolation_display() {
        assert_eq!(IsolationLevel::Serializable.to_string(), "SERIALIZABLE");
        assert_eq!(IsolationLevel::Default.to_string(), "DEFAULT");
    }
}
