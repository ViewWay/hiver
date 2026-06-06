//! Query wrapper types (MyBatis-Plus style)
//! 查询包装器类型（MyBatis-Plus 风格）
//!
//! # Overview / 概述
//!
//! This module provides query builder types similar to MyBatis-Plus QueryWrapper.
//! 本模块提供类似 MyBatis-Plus QueryWrapper 的查询构建器类型。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Query wrapper for building queries
/// 查询包装器，用于构建查询
///
/// This is similar to MyBatis-Plus QueryWrapper, providing a fluent API
/// for building query conditions.
///
/// 这类似 MyBatis-Plus QueryWrapper，提供流畅的 API 用于构建查询条件。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::QueryWrapper;
///
/// let query = QueryWrapper::new()
///     .eq("status", "active")
///     .ge("age", 18)
///     .like("name", "Alice")
///     .in_("city", vec!["Beijing", "Shanghai"])
///     .order_by_asc("created_at");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryWrapper
{
    /// WHERE conditions
    /// WHERE 条件
    pub conditions: Vec<Condition>,

    /// ORDER BY clauses (internal QueryOrder)
    /// ORDER BY 子句（内部 QueryOrder）
    pub orders: Vec<QueryOrder>,

    /// LIMIT clause
    /// LIMIT 子句
    pub limit: Option<u64>,

    /// OFFSET clause
    /// OFFSET 子句
    pub offset: Option<u64>,

    /// SELECT columns (None means *)
    /// SELECT 列（None 表示 *）
    pub select: Option<Vec<String>>,
}

impl QueryWrapper
{
    /// Create a new empty query wrapper
    /// 创建新的空查询包装器
    pub fn new() -> Self
    {
        Self {
            conditions: Vec::new(),
            orders: Vec::new(),
            limit: None,
            offset: None,
            select: None,
        }
    }

    /// Add an equality condition
    /// 添加相等条件
    ///
    /// Equivalent to: `field = value`
    pub fn eq(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Eq {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add a not-equal condition
    /// 添加不等条件
    ///
    /// Equivalent to: `field != value`
    pub fn ne(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Ne {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add a greater-than condition
    /// 添加大于条件
    ///
    /// Equivalent to: `field > value`
    pub fn gt(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Gt {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add a greater-than-or-equal condition
    /// 添加大于等于条件
    ///
    /// Equivalent to: `field >= value`
    pub fn ge(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Ge {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add a less-than condition
    /// 添加小于条件
    ///
    /// Equivalent to: `field < value`
    pub fn lt(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Lt {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add a less-than-or-equal condition
    /// 添加小于等于条件
    ///
    /// Equivalent to: `field <= value`
    pub fn le(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Le {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add a LIKE condition
    /// 添加 LIKE 条件
    ///
    /// Equivalent to: `field LIKE pattern`
    pub fn like(mut self, field: &str, pattern: &str) -> Self
    {
        self.conditions.push(Condition::Like {
            field: field.to_string(),
            pattern: pattern.to_string(),
        });
        self
    }

    /// Add a NOT LIKE condition
    /// 添加 NOT LIKE 条件
    pub fn not_like(mut self, field: &str, pattern: &str) -> Self
    {
        self.conditions.push(Condition::NotLike {
            field: field.to_string(),
            pattern: pattern.to_string(),
        });
        self
    }

    /// Add an IN condition
    /// 添加 IN 条件
    ///
    /// Equivalent to: `field IN (values...)`
    pub fn in_<T: ToValue>(mut self, field: &str, values: Vec<T>) -> Self
    {
        let values = values.into_iter().map(|v| v.to_value()).collect();
        self.conditions.push(Condition::In {
            field: field.to_string(),
            values,
        });
        self
    }

    /// Add a NOT IN condition
    /// 添加 NOT IN 条件
    ///
    /// Equivalent to: `field NOT IN (values...)`
    pub fn not_in<T: ToValue>(mut self, field: &str, values: Vec<T>) -> Self
    {
        let values = values.into_iter().map(|v| v.to_value()).collect();
        self.conditions.push(Condition::NotIn {
            field: field.to_string(),
            values,
        });
        self
    }

    /// Add a BETWEEN condition
    /// 添加 BETWEEN 条件
    ///
    /// Equivalent to: `field BETWEEN low AND high`
    pub fn between(mut self, field: &str, low: Value, high: Value) -> Self
    {
        self.conditions.push(Condition::Between {
            field: field.to_string(),
            low,
            high,
        });
        self
    }

    /// Add a NOT BETWEEN condition
    /// 添加 NOT BETWEEN 条件
    pub fn not_between(mut self, field: &str, low: Value, high: Value) -> Self
    {
        self.conditions.push(Condition::NotBetween {
            field: field.to_string(),
            low,
            high,
        });
        self
    }

    /// Add an IS NULL condition
    /// 添加 IS NULL 条件
    pub fn is_null(mut self, field: &str) -> Self
    {
        self.conditions.push(Condition::IsNull {
            field: field.to_string(),
        });
        self
    }

    /// Add an IS NOT NULL condition
    /// 添加 IS NOT NULL 条件
    pub fn is_not_null(mut self, field: &str) -> Self
    {
        self.conditions.push(Condition::IsNotNull {
            field: field.to_string(),
        });
        self
    }

    /// Add a nested AND condition
    /// 添加嵌套 AND 条件
    pub fn and(mut self, wrapper: QueryWrapper) -> Self
    {
        self.conditions
            .push(Condition::And(Box::new(wrapper.conditions)));
        self
    }

    /// Add a nested OR condition
    /// 添加嵌套 OR 条件
    pub fn or(mut self, wrapper: QueryWrapper) -> Self
    {
        self.conditions
            .push(Condition::Or(Box::new(wrapper.conditions)));
        self
    }

    /// Add ORDER BY ASC
    /// 添加升序排序
    pub fn order_by_asc(mut self, field: &str) -> Self
    {
        self.orders.push(QueryOrder::Asc(field.to_string()));
        self
    }

    /// Add ORDER BY DESC
    /// 添加降序排序
    pub fn order_by_desc(mut self, field: &str) -> Self
    {
        self.orders.push(QueryOrder::Desc(field.to_string()));
        self
    }

    /// Set LIMIT
    /// 设置 LIMIT
    pub fn limit(mut self, limit: u64) -> Self
    {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    /// 设置 OFFSET
    pub fn offset(mut self, offset: u64) -> Self
    {
        self.offset = Some(offset);
        self
    }

    /// Set SELECT columns
    /// 设置 SELECT 列
    pub fn select<T: ToString>(mut self, columns: Vec<T>) -> Self
    {
        self.select = Some(columns.into_iter().map(|c| c.to_string()).collect());
        self
    }

    /// Check if has any conditions
    /// 检查是否有任何条件
    pub fn has_conditions(&self) -> bool
    {
        !self.conditions.is_empty()
    }

    /// Build the WHERE clause as SQL
    /// 构建 WHERE 子句为 SQL
    pub fn build_where(&self) -> String
    {
        if self.conditions.is_empty()
        {
            return String::new();
        }

        let mut sql = String::from("WHERE ");
        let mut first = true;

        for condition in &self.conditions
        {
            if !first
            {
                sql.push_str(" AND ");
            }
            first = false;

            sql.push_str(&condition.to_sql());
        }

        sql
    }
}

impl Default for QueryWrapper
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Update wrapper for building UPDATE statements
/// 更新包装器，用于构建 UPDATE 语句
///
/// This is similar to MyBatis-Plus UpdateWrapper.
/// 这类似 MyBatis-Plus UpdateWrapper。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::UpdateWrapper;
///
/// let update = UpdateWrapper::new()
///     .set("name", "Alice")
///     .set("age", 25)
///     .eq("id", 1);
/// ```
#[derive(Debug, Clone)]
pub struct UpdateWrapper
{
    /// SET clauses (field -> value)
    /// SET 子句（字段 -> 值）
    pub sets: HashMap<String, Value>,

    /// WHERE conditions
    /// WHERE 条件
    pub conditions: Vec<Condition>,
}

impl UpdateWrapper
{
    /// Create a new update wrapper
    /// 创建新的更新包装器
    pub fn new() -> Self
    {
        Self {
            sets: HashMap::new(),
            conditions: Vec::new(),
        }
    }

    /// Set a field value
    /// 设置字段值
    pub fn set<T>(mut self, field: &str, value: T) -> Self
    where
        T: ToValue,
    {
        self.sets.insert(field.to_string(), value.to_value());
        self
    }

    /// Set multiple fields from a struct
    /// 从结构体设置多个字段
    pub fn set_from<T>(mut self, entity: &T) -> Self
    where
        T: ToValueMap,
    {
        if let Some(map) = entity.to_value_map()
        {
            for (k, v) in map
            {
                self.sets.insert(k, v);
            }
        }
        self
    }

    /// Add a WHERE condition (delegates to QueryWrapper)
    /// 添加 WHERE 条件（委托给 QueryWrapper）
    pub fn eq(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Eq {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add ne condition
    pub fn ne(mut self, field: &str, value: Value) -> Self
    {
        self.conditions.push(Condition::Ne {
            field: field.to_string(),
            value,
        });
        self
    }

    /// Add in condition
    pub fn in_<T: ToValue>(mut self, field: &str, values: Vec<T>) -> Self
    {
        let values = values.into_iter().map(|v| v.to_value()).collect();
        self.conditions.push(Condition::In {
            field: field.to_string(),
            values,
        });
        self
    }

    /// Check if has any SET clauses
    /// 检查是否有任何 SET 子句
    pub fn has_sets(&self) -> bool
    {
        !self.sets.is_empty()
    }
}

impl Default for UpdateWrapper
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Query condition
/// 查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition
{
    /// Equal: field = value
    /// 相等：field = value
    Eq
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Not equal: field != value
    /// 不等：field != value
    Ne
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Greater than: field > value
    /// 大于：field > value
    Gt
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Greater than or equal: field >= value
    /// 大于等于：field >= value
    Ge
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Less than: field < value
    /// 小于：field < value
    Lt
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Less than or equal: field <= value
    /// 小于等于：field <= value
    Le
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// LIKE: field LIKE pattern
    Like
    {
        /// Field name / 字段名
        field: String,
        /// LIKE pattern / 匹配模式
        pattern: String,
    },
    /// NOT LIKE: field NOT LIKE pattern
    NotLike
    {
        /// Field name / 字段名
        field: String,
        /// NOT LIKE pattern / 不匹配模式
        pattern: String,
    },
    /// IN: field IN (values...)
    In
    {
        /// Field name / 字段名
        field: String,
        /// Values to compare / 比较值列表
        values: Vec<Value>,
    },
    /// NOT IN: field NOT IN (values...)
    NotIn
    {
        /// Field name / 字段名
        field: String,
        /// Values to exclude / 排除值列表
        values: Vec<Value>,
    },
    /// BETWEEN: field BETWEEN low AND high
    Between
    {
        /// Field name / 字段名
        field: String,
        /// Low value / 下限值
        low: Value,
        /// High value / 上限值
        high: Value,
    },
    /// NOT BETWEEN: field NOT BETWEEN low AND high
    NotBetween
    {
        /// Field name / 字段名
        field: String,
        /// Low value / 下限值
        low: Value,
        /// High value / 上限值
        high: Value,
    },
    /// IS NULL: field IS NULL
    IsNull
    {
        /// Field name / 字段名
        field: String,
    },
    /// IS NOT NULL: field IS NOT NULL
    IsNotNull
    {
        /// Field name / 字段名
        field: String,
    },
    /// AND: (condition1 AND condition2 AND ...)
    And(Box<Vec<Condition>>),
    /// OR: (condition1 OR condition2 OR ...)
    Or(Box<Vec<Condition>>),
}

impl Condition
{
    /// Convert condition to SQL fragment
    /// 将条件转换为 SQL 片段
    /// Validate that a field/identifier contains only safe characters.
    /// 验证字段/标识符仅包含安全字符。
    fn validate_field(field: &str)
    {
        assert!(
            !field.is_empty()
                && field
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.'),
            "Invalid SQL identifier: {field}"
        );
    }

    /// Convert this criterion to a SQL WHERE clause fragment.
    /// 将此条件转换为 SQL WHERE 子句片段。
    pub fn to_sql(&self) -> String
    {
        match self
        {
            Self::Eq { field, value } =>
            {
                Self::validate_field(field);
                format!("{} = {}", field, value.to_sql())
            },
            Self::Ne { field, value } =>
            {
                Self::validate_field(field);
                format!("{} != {}", field, value.to_sql())
            },
            Self::Gt { field, value } =>
            {
                Self::validate_field(field);
                format!("{} > {}", field, value.to_sql())
            },
            Self::Ge { field, value } =>
            {
                Self::validate_field(field);
                format!("{} >= {}", field, value.to_sql())
            },
            Self::Lt { field, value } =>
            {
                Self::validate_field(field);
                format!("{} < {}", field, value.to_sql())
            },
            Self::Le { field, value } =>
            {
                Self::validate_field(field);
                format!("{} <= {}", field, value.to_sql())
            },
            Self::Like { field, pattern } =>
            {
                Self::validate_field(field);
                format!("{} LIKE '{}'", field, pattern.replace('\'', "''").replace('\0', ""))
            },
            Self::NotLike { field, pattern } =>
            {
                Self::validate_field(field);
                format!("{} NOT LIKE '{}'", field, pattern.replace('\'', "''").replace('\0', ""))
            },
            Self::In { field, values } =>
            {
                Self::validate_field(field);
                let vals: Vec<String> = values.iter().map(|v| v.to_sql()).collect();
                format!("{} IN ({})", field, vals.join(", "))
            },
            Self::NotIn { field, values } =>
            {
                Self::validate_field(field);
                let vals: Vec<String> = values.iter().map(|v| v.to_sql()).collect();
                format!("{} NOT IN ({})", field, vals.join(", "))
            },
            Self::Between { field, low, high } =>
            {
                Self::validate_field(field);
                format!("{} BETWEEN {} AND {}", field, low.to_sql(), high.to_sql())
            },
            Self::NotBetween { field, low, high } =>
            {
                Self::validate_field(field);
                format!("{} NOT BETWEEN {} AND {}", field, low.to_sql(), high.to_sql())
            },
            Self::IsNull { field } =>
            {
                Self::validate_field(field);
                format!("{} IS NULL", field)
            },
            Self::IsNotNull { field } =>
            {
                Self::validate_field(field);
                format!("{} IS NOT NULL", field)
            },
            Self::And(conditions) =>
            {
                let conds: Vec<String> = conditions.iter().map(|c| c.to_sql()).collect();
                format!("({})", conds.join(" AND "))
            },
            Self::Or(conditions) =>
            {
                let conds: Vec<String> = conditions.iter().map(|c| c.to_sql()).collect();
                format!("({})", conds.join(" OR "))
            },
        }
    }
}

/// Order clause for QueryWrapper
/// QueryWrapper 的 ORDER BY 子句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryOrder
{
    /// Ascending order
    Asc(String),
    /// Descending order
    Desc(String),
}

impl QueryOrder
{
    /// Convert to sort::Order
    /// 转换为 sort::Order
    pub fn to_order(&self) -> super::Order
    {
        match self
        {
            Self::Asc(field) => super::Order {
                property: field.clone(),
                direction: super::Direction::ASC,
            },
            Self::Desc(field) => super::Order {
                property: field.clone(),
                direction: super::Direction::DESC,
            },
        }
    }
}

/// Query value
/// 查询值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value
{
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    I32(i32),
    /// Long integer value
    I64(i64),
    /// Float value
    F32(f32),
    /// Double value
    F64(f64),
    /// String value
    String(String),
    /// Bytes value
    Bytes(Vec<u8>),
}

impl Value
{
    /// Convert value to SQL fragment
    /// 将值转换为 SQL 片段
    pub fn to_sql(&self) -> String
    {
        match self
        {
            Self::Null => "NULL".to_string(),
            Self::Bool(b) => (if *b { "TRUE" } else { "FALSE" }).to_string(),
            Self::I32(n) => n.to_string(),
            Self::I64(n) => n.to_string(),
            Self::F32(n) => n.to_string(),
            Self::F64(n) => n.to_string(),
            Self::String(s) => format!("'{}'", s.replace('\'', "''").replace('\0', "")),
            Self::Bytes(b) => format!("x'{}'", hex::encode(b)),
        }
    }
}

/// Trait for converting types to Value
/// 将类型转换为 Value 的 trait
pub trait ToValue
{
    /// Convert self to Value
    /// 将 self 转换为 Value
    fn to_value(&self) -> Value;
}

// Implement ToValue for common types
impl ToValue for String
{
    fn to_value(&self) -> Value
    {
        Value::String(self.clone())
    }
}

impl ToValue for &str
{
    fn to_value(&self) -> Value
    {
        Value::String(self.to_string())
    }
}

impl ToValue for bool
{
    fn to_value(&self) -> Value
    {
        Value::Bool(*self)
    }
}

impl ToValue for i32
{
    fn to_value(&self) -> Value
    {
        Value::I32(*self)
    }
}

impl ToValue for i64
{
    fn to_value(&self) -> Value
    {
        Value::I64(*self)
    }
}

impl ToValue for u32
{
    fn to_value(&self) -> Value
    {
        Value::I64(*self as i64)
    }
}

impl ToValue for u64
{
    fn to_value(&self) -> Value
    {
        Value::I64(*self as i64)
    }
}

impl ToValue for f32
{
    fn to_value(&self) -> Value
    {
        Value::F32(*self)
    }
}

impl ToValue for f64
{
    fn to_value(&self) -> Value
    {
        Value::F64(*self)
    }
}

/// Trait for converting structs to value maps
/// 将结构体转换为值映射的 trait
pub trait ToValueMap
{
    /// Convert self to a map of field names to values
    /// 将 self 转换为字段名到值的映射
    fn to_value_map(&self) -> Option<HashMap<String, Value>>;
}

/// Lambda query wrapper (type-safe query builder)
/// Lambda 查询包装器（类型安全的查询构建器）
///
/// This provides type-safe query building using lambda expressions.
/// Similar to MyBatis-Plus LambdaQueryWrapper.
///
/// 这提供使用 lambda 表达式的类型安全查询构建。
/// 类似 MyBatis-Plus LambdaQueryWrapper。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::LambdaQueryWrapper;
///
/// let query = LambdaQueryWrapper::new()
///     .eq(User::getName, "Alice")
///     .ge(User::getAge, 18);
/// ```
#[derive(Debug, Clone)]
pub struct LambdaQueryWrapper
{
    /// Inner query wrapper
    /// 内部查询包装器
    inner: QueryWrapper,
}

impl LambdaQueryWrapper
{
    /// Create a new lambda query wrapper
    /// 创建新的 lambda 查询包装器
    pub fn new() -> Self
    {
        Self {
            inner: QueryWrapper::new(),
        }
    }

    /// Add equality condition using a getter function
    /// 使用 getter 函数添加相等条件
    pub fn eq<T, F>(mut self, _getter: F, value: T) -> Self
    where
        T: ToValue,
        F: Fn(&T) -> &str,
    {
        // In a real implementation, this would use the getter to extract the field name
        // 在实际实现中，这将使用 getter 提取字段名
        self.inner = self.inner.eq("field", value.to_value());
        self
    }
}

impl Default for LambdaQueryWrapper
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Dynamic query specification
/// 动态查询规范
///
/// Similar to JPA Specification or MyBatis-Plus query wrapper.
/// 类似于 JPA Specification 或 MyBatis-Plus 查询包装器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::Specification;
///
/// let spec = Specification::and(
///     Specification::eq("status", "active"),
///     Specification::ge("age", 18)
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Specification
{
    /// Predicate for filtering
    /// 用于过滤的谓词
    pub predicate: Option<Predicate>,
}

impl Specification
{
    /// Create a new specification
    /// 创建新的规范
    pub fn new() -> Self
    {
        Self { predicate: None }
    }

    /// Create an AND specification
    /// 创建 AND 规范
    pub fn and(spec1: Specification, spec2: Specification) -> Self
    {
        let pred1 = spec1.predicate;
        let pred2 = spec2.predicate;

        let predicate = match (pred1, pred2)
        {
            (Some(p1), Some(p2)) => Some(Predicate::And(Box::new(p1), Box::new(p2))),
            (Some(p), None) => Some(p),
            (None, Some(p)) => Some(p),
            (None, None) => None,
        };

        Self { predicate }
    }

    /// Create an OR specification
    /// 创建 OR 规范
    pub fn or(spec1: Specification, spec2: Specification) -> Self
    {
        let pred1 = spec1.predicate;
        let pred2 = spec2.predicate;

        let predicate = match (pred1, pred2)
        {
            (Some(p1), Some(p2)) => Some(Predicate::Or(Box::new(p1), Box::new(p2))),
            (Some(p), None) => Some(p),
            (None, Some(p)) => Some(p),
            (None, None) => None,
        };

        Self { predicate }
    }

    /// Create an equality predicate
    /// 创建相等谓词
    pub fn eq(field: &str, value: impl ToValue) -> Self
    {
        Self {
            predicate: Some(Predicate::Eq {
                field: field.to_string(),
                value: value.to_value(),
            }),
        }
    }

    /// Create a greater-than predicate
    /// 创建大于谓词
    pub fn gt(field: &str, value: impl ToValue) -> Self
    {
        Self {
            predicate: Some(Predicate::Gt {
                field: field.to_string(),
                value: value.to_value(),
            }),
        }
    }

    /// Create a greater-than-or-equal predicate
    /// 创建大于等于谓词
    pub fn ge(field: &str, value: impl ToValue) -> Self
    {
        Self {
            predicate: Some(Predicate::Ge {
                field: field.to_string(),
                value: value.to_value(),
            }),
        }
    }

    /// Create a less-than predicate
    /// 创建小于谓词
    pub fn lt(field: &str, value: impl ToValue) -> Self
    {
        Self {
            predicate: Some(Predicate::Lt {
                field: field.to_string(),
                value: value.to_value(),
            }),
        }
    }

    /// Create a less-than-or-equal predicate
    /// 创建小于等于谓词
    pub fn le(field: &str, value: impl ToValue) -> Self
    {
        Self {
            predicate: Some(Predicate::Le {
                field: field.to_string(),
                value: value.to_value(),
            }),
        }
    }

    /// Create a like predicate
    /// 创建 LIKE 谓词
    pub fn like(field: &str, pattern: &str) -> Self
    {
        Self {
            predicate: Some(Predicate::Like {
                field: field.to_string(),
                pattern: pattern.to_string(),
            }),
        }
    }

    /// Create an IN predicate
    /// 创建 IN 谓词
    pub fn in_<T>(field: &str, values: Vec<T>) -> Self
    where
        T: ToValue,
    {
        Self {
            predicate: Some(Predicate::In {
                field: field.to_string(),
                values: values.into_iter().map(|v| v.to_value()).collect(),
            }),
        }
    }

    /// Check if specification has any predicate
    /// 检查规范是否有任何谓词
    pub fn has_predicate(&self) -> bool
    {
        self.predicate.is_some()
    }
}

impl Default for Specification
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Query predicate
/// 查询谓词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate
{
    /// Equal: field = value
    /// 相等：field = value
    Eq
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Not equal: field != value
    /// 不等：field != value
    Ne
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Greater than: field > value
    /// 大于：field > value
    Gt
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Greater than or equal: field >= value
    /// 大于等于：field >= value
    Ge
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Less than: field < value
    /// 小于：field < value
    Lt
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// Less than or equal: field <= value
    /// 小于等于：field <= value
    Le
    {
        /// Field name / 字段名
        field: String,
        /// Field value / 字段值
        value: Value,
    },
    /// LIKE: field LIKE pattern
    Like
    {
        /// Field name / 字段名
        field: String,
        /// LIKE pattern / 匹配模式
        pattern: String,
    },
    /// IN: field IN (values...)
    In
    {
        /// Field name / 字段名
        field: String,
        /// Values to compare / 比较值列表
        values: Vec<Value>,
    },
    /// NOT IN: field NOT IN (values...)
    NotIn
    {
        /// Field name / 字段名
        field: String,
        /// Values to exclude / 排除值列表
        values: Vec<Value>,
    },
    /// AND: (predicate1 AND predicate2)
    And(Box<Predicate>, Box<Predicate>),
    /// OR: (predicate1 OR predicate2)
    Or(Box<Predicate>, Box<Predicate>),
    /// NOT: NOT predicate
    Not(Box<Predicate>),
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_query_wrapper_new()
    {
        let qw = QueryWrapper::new();
        assert!(!qw.has_conditions());
        assert_eq!(qw.conditions.len(), 0);
    }

    #[test]
    fn test_query_wrapper_eq()
    {
        let qw = QueryWrapper::new().eq("status", Value::String("active".to_string()));
        assert!(qw.has_conditions());
        assert_eq!(qw.conditions.len(), 1);
    }

    #[test]
    fn test_query_wrapper_chain()
    {
        let qw = QueryWrapper::new()
            .eq("status", Value::String("active".to_string()))
            .ge("age", Value::I32(18))
            .like("name", "Alice");
        assert_eq!(qw.conditions.len(), 3);
    }

    #[test]
    fn test_query_wrapper_in()
    {
        let qw = QueryWrapper::new().in_("city", vec!["Beijing", "Shanghai"]);
        assert_eq!(qw.conditions.len(), 1);
    }

    #[test]
    fn test_query_wrapper_order()
    {
        let qw = QueryWrapper::new()
            .order_by_asc("name")
            .order_by_desc("created_at");
        assert_eq!(qw.orders.len(), 2);
    }

    #[test]
    fn test_query_wrapper_limit_offset()
    {
        let qw = QueryWrapper::new().limit(10).offset(20);
        assert_eq!(qw.limit, Some(10));
        assert_eq!(qw.offset, Some(20));
    }

    #[test]
    fn test_update_wrapper()
    {
        let uw = UpdateWrapper::new().set("name", "Alice").set("age", 25);
        assert!(uw.has_sets());
        assert_eq!(uw.sets.len(), 2);
    }

    #[test]
    fn test_specification_and()
    {
        let spec1 = Specification::eq("status", "active");
        let spec2 = Specification::ge("age", 18);
        let combined = Specification::and(spec1, spec2);
        assert!(combined.has_predicate());
    }

    #[test]
    fn test_value_to_sql()
    {
        assert_eq!(Value::String("test".to_string()).to_sql(), "'test'");
        assert_eq!(Value::I32(42).to_sql(), "42");
        assert_eq!(Value::Bool(true).to_sql(), "TRUE");
        assert_eq!(Value::Null.to_sql(), "NULL");
    }
}
