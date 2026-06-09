//! Sorting support
//! 排序支持
//!
//! # Overview / 概述
//!
//! This module provides sorting types for repository queries.
//! 本模块提供 repository 查询的排序类型。

use std::fmt;

use serde::{Deserialize, Serialize};

/// Sort definition for queries
/// 查询的排序定义
///
/// Represents sorting options for query results.
/// 表示查询结果的排序选项。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Sort;
///
/// // Sort by single field ascending
/// let sort = Sort::by(&["name"]);
///
/// // Sort by multiple fields
/// let sort = Sort::by(&["name", "email"]);
///
/// // Sort with custom orders
/// let sort = Sort::new(vec![
///     Order::asc("name"),
///     Order::desc("age"),
/// ]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    /// Sort orders
    /// 排序规则
    pub orders: Vec<Order>,
}

impl Sort {
    /// Create a new sort from orders
    /// 从排序规则创建新的排序
    pub fn new(orders: Vec<Order>) -> Self {
        Self { orders }
    }

    /// Create a sort by field names (all ascending)
    /// 通过字段名创建排序（全部升序）
    pub fn by(fields: &[&str]) -> Self {
        Self {
            orders: fields.iter().map(|f| Order::asc(f)).collect(),
        }
    }

    /// Create a sort by single field ascending
    /// 通过单个字段创建升序排序
    pub fn asc(field: &str) -> Self {
        Self {
            orders: vec![Order::asc(field)],
        }
    }

    /// Create a sort by single field descending
    /// 通过单个字段创建降序排序
    pub fn desc(field: &str) -> Self {
        Self {
            orders: vec![Order::desc(field)],
        }
    }

    /// Add another sort (combine with AND)
    /// 添加另一个排序（使用 AND 组合）
    pub fn and(mut self, sort: Sort) -> Self {
        self.orders.extend(sort.orders);
        self
    }

    /// Check if sort is empty
    /// 检查排序是否为空
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Get iterator over orders
    /// 获取排序规则的迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Order> {
        self.orders.iter()
    }

    /// Unsort (no sorting)
    /// 无排序
    pub fn unsorted() -> Self {
        Self { orders: Vec::new() }
    }
}

impl Default for Sort {
    fn default() -> Self {
        Self::unsorted()
    }
}

/// Sort order (direction + property)
/// 排序规则（方向 + 属性）
///
/// Represents a single sort criterion.
/// 表示单个排序条件。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Order;
///
/// // Ascending by name
/// let order = Order::asc("name");
///
/// // Descending by age
/// let order = Order::desc("age");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Sort direction
    /// 排序方向
    pub direction: Direction,

    /// Property name to sort by
    /// 排序属性名
    pub property: String,
}

impl Order {
    /// Create an ascending order
    /// 创建升序规则
    pub fn asc(property: &str) -> Self {
        Self {
            direction: Direction::ASC,
            property: property.to_string(),
        }
    }

    /// Create a descending order
    /// 创建降序规则
    pub fn desc(property: &str) -> Self {
        Self {
            direction: Direction::DESC,
            property: property.to_string(),
        }
    }

    /// Create an order with explicit direction
    /// 创建带有显式方向的排序规则
    pub fn new(property: &str, direction: Direction) -> Self {
        Self {
            direction,
            property: property.to_string(),
        }
    }

    /// Check if ascending
    /// 检查是否为升序
    pub fn is_ascending(&self) -> bool {
        matches!(self.direction, Direction::ASC)
    }

    /// Check if descending
    /// 检查是否为降序
    pub fn is_descending(&self) -> bool {
        matches!(self.direction, Direction::DESC)
    }

    /// Ignore case (for sorting)
    /// 忽略大小写（用于排序）
    pub fn ignore_case() -> bool {
        true
    }

    /// Get null handling (default: nulls first)
    /// 获取空值处理（默认：空值在前）
    pub fn null_handling() -> NullHandling {
        NullHandling::NullsFirst
    }
}

/// Sort direction
/// 排序方向
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Direction;
///
/// let dir = Direction::ASC;
/// let dir = Direction::DESC;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Ascending order
    /// 升序
    ASC,

    /// Descending order
    /// 降序
    DESC,
}

impl Direction {
    /// Reverse the direction
    /// 反转方向
    pub fn reverse(&self) -> Self {
        match self {
            Self::ASC => Self::DESC,
            Self::DESC => Self::ASC,
        }
    }

    /// Check if ascending
    /// 检查是否为升序
    pub fn is_ascending(&self) -> bool {
        matches!(self, Self::ASC)
    }

    /// Check if descending
    /// 检查是否为降序
    pub fn is_descending(&self) -> bool {
        matches!(self, Self::DESC)
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ASC => write!(f, "ASC"),
            Self::DESC => write!(f, "DESC"),
        }
    }
}

/// Null handling for sorting
/// 排序的空值处理
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullHandling {
    /// Null values first
    /// 空值在前
    NullsFirst,

    /// Null values last
    /// 空值在后
    NullsLast,
}

impl fmt::Display for NullHandling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullsFirst => write!(f, "NULLS_FIRST"),
            Self::NullsLast => write!(f, "NULLS_LAST"),
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
mod tests {
    use super::*;

    #[test]
    fn test_sort_by() {
        let sort = Sort::by(&["name", "email"]);
        assert_eq!(sort.orders.len(), 2);
        assert_eq!(sort.orders[0].property, "name");
        assert_eq!(sort.orders[0].direction, Direction::ASC);
    }

    #[test]
    fn test_sort_asc() {
        let sort = Sort::asc("age");
        assert_eq!(sort.orders.len(), 1);
        assert_eq!(sort.orders[0].property, "age");
        assert!(sort.orders[0].is_ascending());
    }

    #[test]
    fn test_sort_desc() {
        let sort = Sort::desc("age");
        assert_eq!(sort.orders.len(), 1);
        assert!(sort.orders[0].is_descending());
    }

    #[test]
    fn test_sort_and() {
        let sort1 = Sort::asc("name");
        let sort2 = Sort::desc("age");
        let combined = sort1.and(sort2);
        assert_eq!(combined.orders.len(), 2);
    }

    #[test]
    fn test_order_asc() {
        let order = Order::asc("username");
        assert_eq!(order.property, "username");
        assert!(order.is_ascending());
    }

    #[test]
    fn test_order_desc() {
        let order = Order::desc("created_at");
        assert_eq!(order.property, "created_at");
        assert!(order.is_descending());
    }

    #[test]
    fn test_direction_reverse() {
        assert_eq!(Direction::ASC.reverse(), Direction::DESC);
        assert_eq!(Direction::DESC.reverse(), Direction::ASC);
    }
}
