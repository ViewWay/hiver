//! Transaction propagation behavior
//! 事务传播行为

use serde::{Deserialize, Serialize};

/// Transaction propagation behavior
/// 事务传播行为
///
/// Defines how transactions relate to each other.
/// 定义事务之间如何关联。
///
/// Equivalent to Spring's Propagation enum.
/// `等价于Spring的Propagation枚举`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Transactional(propagation = Propagation.REQUIRES_NEW)
/// public void auditAction() { ... }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum Propagation {
    /// Required
    /// 必需
    ///
    /// Support a current transaction, create a new one if none exists.
    /// This is the default.
    /// 支持当前事务，如果不存在则创建新事务。这是默认值。
    #[default]
    Required = 0,

    /// Supports
    /// 支持
    ///
    /// Support a current transaction, execute non-transactionally if none exists.
    /// 支持当前事务，如果不存在则非事务执行。
    Supports = 1,

    /// Mandatory
    /// 强制
    ///
    /// Support a current transaction, throw an exception if none exists.
    /// 支持当前事务，如果不存在则抛出异常。
    Mandatory = 2,

    /// Requires new
    /// 需要新事务
    ///
    /// Create a new transaction, suspending the current transaction if one exists.
    /// 创建新事务，如果存在当前事务则挂起。
    RequiresNew = 3,

    /// Not supported
    /// 不支持
    ///
    /// Execute non-transactionally, suspend the current transaction if one exists.
    /// 非事务执行，如果存在当前事务则挂起。
    NotSupported = 4,

    /// Never
    /// 从不
    ///
    /// Execute non-transactionally, throw an exception if a transaction exists.
    /// 非事务执行，如果存在事务则抛出异常。
    Never = 5,

    /// Nested
    /// 嵌套
    ///
    /// Execute within a nested transaction if a current transaction exists.
    /// 如果存在当前事务，则在嵌套事务中执行。
    Nested = 6,
}

impl Propagation {
    /// Get propagation from value
    /// 从值获取传播行为
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Propagation::Required),
            1 => Some(Propagation::Supports),
            2 => Some(Propagation::Mandatory),
            3 => Some(Propagation::RequiresNew),
            4 => Some(Propagation::NotSupported),
            5 => Some(Propagation::Never),
            6 => Some(Propagation::Nested),
            _ => None,
        }
    }

    /// Get the numeric value
    /// 获取数字值
    pub fn value(&self) -> i32 {
        *self as i32
    }

    /// Check if this creates a new transaction
    /// 检查是否创建新事务
    pub fn creates_new_transaction(&self) -> bool {
        matches!(self, Propagation::Required | Propagation::RequiresNew | Propagation::Nested)
    }

    /// Get description
    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Propagation::Required => "Support current transaction, create new if none",
            Propagation::Supports => {
                "Support current transaction, execute non-transactionally if none"
            },
            Propagation::Mandatory => "Support current transaction, error if none",
            Propagation::RequiresNew => "Always create new transaction",
            Propagation::NotSupported => "Execute non-transactionally, suspend current if exists",
            Propagation::Never => "Execute non-transactionally, error if transaction exists",
            Propagation::Nested => "Execute within nested transaction if one exists",
        }
    }
}


impl std::fmt::Display for Propagation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Propagation::Required => write!(f, "REQUIRED"),
            Propagation::Supports => write!(f, "SUPPORTS"),
            Propagation::Mandatory => write!(f, "MANDATORY"),
            Propagation::RequiresNew => write!(f, "REQUIRES_NEW"),
            Propagation::NotSupported => write!(f, "NOT_SUPPORTED"),
            Propagation::Never => write!(f, "NEVER"),
            Propagation::Nested => write!(f, "NESTED"),
        }
    }
}

impl std::str::FromStr for Propagation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "REQUIRED" => Ok(Propagation::Required),
            "SUPPORTS" => Ok(Propagation::Supports),
            "MANDATORY" => Ok(Propagation::Mandatory),
            "REQUIRES_NEW" | "REQUIRES-NEW" => Ok(Propagation::RequiresNew),
            "NOT_SUPPORTED" | "NOT-SUPPORTED" => Ok(Propagation::NotSupported),
            "NEVER" => Ok(Propagation::Never),
            "NESTED" => Ok(Propagation::Nested),
            _ => Err(format!("Unknown propagation: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_from_str() {
        assert_eq!("REQUIRES_NEW".parse::<Propagation>().unwrap(), Propagation::RequiresNew);
        assert_eq!("mandatory".parse::<Propagation>().unwrap(), Propagation::Mandatory);
    }

    #[test]
    fn test_creates_new_transaction() {
        assert!(Propagation::Required.creates_new_transaction());
        assert!(Propagation::RequiresNew.creates_new_transaction());
        assert!(!Propagation::Supports.creates_new_transaction());
        assert!(!Propagation::Never.creates_new_transaction());
    }
}
