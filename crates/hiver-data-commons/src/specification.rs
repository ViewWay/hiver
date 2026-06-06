//! Specification API for type-safe dynamic queries
//! 用于类型安全动态查询的 Specification API
//!
//! # Overview / 概述
//!
//! This module provides a JPA-inspired `Specification` pattern for building
//! dynamic queries in a composable, type-safe manner.
//! 本模块提供 JPA 风格的 `Specification` 模式，以可组合、类型安全的方式构建动态查询。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data JPA |
//! |-------|-----------------|
//! | `Specification` trait | `Specification<T>` |
//! | `Specifications` builder | `Specifications` |
//! | `Predicate` enum | `Predicate` |
//! | `JpaSpecificationExecutor<T, ID>` | `JpaSpecificationExecutor<T>` |
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_data_commons::specification::{Specifications, SpecValue};
//!
//! let spec = Specifications::where_clause()
//!     .eq("status", SpecValue::String("active".into()))
//!     .gt("age", SpecValue::I64(18))
//!     .build();
//! ```

use std::fmt;

use crate::{Page, PageRequest};

/// Literal value used in predicates.
/// 谓词中使用的字面值。
///
/// A simplified value type for use in specification predicates.
/// 用于 specification 谓词的简化值类型。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::specification::SpecValue;
///
/// let v = SpecValue::String("Alice".into());
/// let n = SpecValue::I64(42);
/// let b = SpecValue::Bool(true);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum SpecValue
{
    /// String value / 字符串值
    String(String),
    /// 64-bit integer / 64 位整数
    I64(i64),
    /// 64-bit float / 64 位浮点数
    F64(f64),
    /// Boolean / 布尔值
    Bool(bool),
    /// Null value / 空值
    Null,
}

impl fmt::Display for SpecValue
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::String(s) => write!(f, "'{}'", s.replace('\'', "''")),
            Self::I64(n) => write!(f, "{n}"),
            Self::F64(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Null => write!(f, "NULL"),
        }
    }
}

/// Predicate representing a single filter condition.
/// 表示单个过滤条件的谓词。
///
/// Predicates are the building blocks of specifications.
/// They describe how entities should be filtered.
/// 谓词是 specification 的构建块。它们描述实体应如何被过滤。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::specification::{Predicate, SpecValue};
///
/// let pred = Predicate::eq("name", SpecValue::String("Alice".into()));
/// let combined =
///     Predicate::and(Box::new(pred), Box::new(Predicate::gt("age", SpecValue::I64(18))));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate
{
    /// Logical AND of two predicates / 两个谓词的逻辑与
    And(Box<Predicate>, Box<Predicate>),
    /// Logical OR of two predicates / 两个谓词的逻辑或
    Or(Box<Predicate>, Box<Predicate>),
    /// Logical NOT of a predicate / 谓词的逻辑非
    Not(Box<Predicate>),
    /// Equality: field = value / 等于：field = value
    Eq(String, SpecValue),
    /// Inequality: field != value / 不等于：field != value
    Ne(String, SpecValue),
    /// Greater than: field > value / 大于：field > value
    Gt(String, SpecValue),
    /// Greater than or equal: field >= value / 大于等于：field >= value
    Gte(String, SpecValue),
    /// Less than: field < value / 小于：field < value
    Lt(String, SpecValue),
    /// Less than or equal: field <= value / 小于等于：field <= value
    Lte(String, SpecValue),
    /// LIKE: field LIKE pattern / 模糊匹配：field LIKE pattern
    Like(String, String),
    /// IN: field IN (values...) / 包含于：field IN (values...)
    In(String, Vec<SpecValue>),
    /// NOT IN: field NOT IN (values...) / 不包含于：field NOT IN (values...)
    NotIn(String, Vec<SpecValue>),
    /// IS NULL / 为空
    IsNull(String),
    /// IS NOT NULL / 非空
    IsNotNull(String),
    /// BETWEEN: field BETWEEN low AND high / 范围：field BETWEEN low AND high
    Between(String, SpecValue, SpecValue),
}

impl Predicate
{
    /// Create an equality predicate.
    /// 创建相等谓词。
    pub fn eq(field: impl Into<String>, value: SpecValue) -> Self
    {
        Self::Eq(field.into(), value)
    }

    /// Create an inequality predicate.
    /// 创建不等谓词。
    pub fn ne(field: impl Into<String>, value: SpecValue) -> Self
    {
        Self::Ne(field.into(), value)
    }

    /// Create a greater-than predicate.
    /// 创建大于谓词。
    pub fn gt(field: impl Into<String>, value: SpecValue) -> Self
    {
        Self::Gt(field.into(), value)
    }

    /// Create a less-than predicate.
    /// 创建小于谓词。
    pub fn lt(field: impl Into<String>, value: SpecValue) -> Self
    {
        Self::Lt(field.into(), value)
    }

    /// Create a greater-than-or-equal predicate.
    /// 创建大于等于谓词。
    pub fn gte(field: impl Into<String>, value: SpecValue) -> Self
    {
        Self::Gte(field.into(), value)
    }

    /// Create a less-than-or-equal predicate.
    /// 创建小于等于谓词。
    pub fn lte(field: impl Into<String>, value: SpecValue) -> Self
    {
        Self::Lte(field.into(), value)
    }

    /// Create a LIKE predicate.
    /// 创建 LIKE 谓词。
    pub fn like(field: impl Into<String>, pattern: impl Into<String>) -> Self
    {
        Self::Like(field.into(), pattern.into())
    }

    /// Create an IN predicate.
    /// 创建 IN 谓词。
    pub fn in_pred(field: impl Into<String>, values: Vec<SpecValue>) -> Self
    {
        Self::In(field.into(), values)
    }

    /// Create a NOT IN predicate.
    /// 创建 NOT IN 谓词。
    pub fn not_in(field: impl Into<String>, values: Vec<SpecValue>) -> Self
    {
        Self::NotIn(field.into(), values)
    }

    /// Create an IS NULL predicate.
    /// 创建 IS NULL 谓词。
    pub fn is_null(field: impl Into<String>) -> Self
    {
        Self::IsNull(field.into())
    }

    /// Create an IS NOT NULL predicate.
    /// 创建 IS NOT NULL 谓词。
    pub fn is_not_null(field: impl Into<String>) -> Self
    {
        Self::IsNotNull(field.into())
    }

    /// Create a BETWEEN predicate.
    /// 创建 BETWEEN 谓词。
    pub fn between(field: impl Into<String>, low: SpecValue, high: SpecValue) -> Self
    {
        Self::Between(field.into(), low, high)
    }

    /// Create an AND predicate from two predicates.
    /// 从两个谓词创建 AND 谓词。
    pub fn and(left: Box<Predicate>, right: Box<Predicate>) -> Self
    {
        Self::And(left, right)
    }

    /// Create an OR predicate from two predicates.
    /// 从两个谓词创建 OR 谓词。
    pub fn or(left: Box<Predicate>, right: Box<Predicate>) -> Self
    {
        Self::Or(left, right)
    }

    /// Create a NOT predicate.
    /// 创建 NOT 谓词。
    #[allow(clippy::should_implement_trait)]
    pub fn not(inner: Box<Predicate>) -> Self
    {
        Self::Not(inner)
    }

    /// Convert the predicate to an approximate SQL WHERE clause fragment.
    /// 将谓词转换为近似的 SQL WHERE 子句片段。
    ///
    /// This is useful for debugging and logging.
    /// 用于调试和日志记录。
    pub fn to_sql(&self) -> String
    {
        match self
        {
            Self::And(l, r) => format!("({} AND {})", l.to_sql(), r.to_sql()),
            Self::Or(l, r) => format!("({} OR {})", l.to_sql(), r.to_sql()),
            Self::Not(inner) => format!("NOT ({})", inner.to_sql()),
            Self::Eq(field, value) => format!("{} = {}", field, value),
            Self::Ne(field, value) => format!("{} != {}", field, value),
            Self::Gt(field, value) => format!("{} > {}", field, value),
            Self::Gte(field, value) => format!("{} >= {}", field, value),
            Self::Lt(field, value) => format!("{} < {}", field, value),
            Self::Lte(field, value) => format!("{} <= {}", field, value),
            Self::Like(field, pattern) => format!("{} LIKE '{}'", field, pattern),
            Self::In(field, values) =>
            {
                let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("{} IN ({})", field, vals.join(", "))
            },
            Self::NotIn(field, values) =>
            {
                let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("{} NOT IN ({})", field, vals.join(", "))
            },
            Self::IsNull(field) => format!("{} IS NULL", field),
            Self::IsNotNull(field) => format!("{} IS NOT NULL", field),
            Self::Between(field, low, high) =>
            {
                format!("{} BETWEEN {} AND {}", field, low, high)
            },
        }
    }
}

impl fmt::Display for Predicate
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.to_sql())
    }
}

/// Specification trait for composable query criteria.
/// 可组合查询条件的 Specification trait。
///
/// Implementors produce a [`Predicate`] that can be translated
/// into SQL, query parameters, or other query representations.
/// 实现者生成一个 [`Predicate`]，可转换为 SQL、查询参数或其他查询表示。
///
/// This mirrors Spring Data JPA's `Specification<T>` interface.
/// 这镜像了 Spring Data JPA 的 `Specification<T>` 接口。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::specification::{Specification, Predicate, SpecValue};
///
/// struct ActiveUserSpec;
///
/// impl Specification for ActiveUserSpec {
///     fn to_predicate(&self) -> Option<Predicate> {
///         Some(Predicate::eq("status", SpecValue::String("active".into())))
///     }
/// }
/// ```
pub trait Specification: std::fmt::Debug + Send + Sync
{
    /// Convert this specification into a predicate.
    /// 将此 specification 转换为谓词。
    ///
    /// Returns `None` if the specification has no constraints
    /// (matches all entities).
    /// 如果 specification 没有约束（匹配所有实体），返回 `None`。
    fn to_predicate(&self) -> Option<Predicate>;
}

/// Blanket implementation: any `Fn() -> Option<Predicate>` is a Specification.
/// 任何 `Fn() -> Option<Predicate>` 都是一个 Specification。
impl<F> Specification for F
where
    F: Fn() -> Option<Predicate> + std::fmt::Debug + Send + Sync,
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        self()
    }
}

/// Fluent builder for composing specifications.
/// 用于组合 specification 的流畅构建器。
///
/// This mirrors Spring Data's `Specifications` helper class.
/// 这镜像了 Spring Data 的 `Specifications` 辅助类。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::specification::{SpecValue, Specification, Specifications};
///
/// let spec = Specifications::where_clause()
///     .eq("status", SpecValue::String("active".into()))
///     .gt("age", SpecValue::I64(18))
///     .build();
///
/// assert!(spec.to_predicate().is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Specifications
{
    /// Current predicate being built.
    /// 当前正在构建的谓词。
    predicate: Option<Predicate>,
}

impl Specifications
{
    /// Start building a new specification with no constraints.
    /// 开始构建没有约束的新 specification。
    pub fn where_clause() -> Self
    {
        Self { predicate: None }
    }

    /// Add an equality condition.
    /// 添加相等条件。
    pub fn eq(mut self, field: impl Into<String>, value: SpecValue) -> Self
    {
        let pred = Predicate::eq(field, value);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add an inequality condition.
    /// 添加不等条件。
    pub fn ne(mut self, field: impl Into<String>, value: SpecValue) -> Self
    {
        let pred = Predicate::ne(field, value);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a greater-than condition.
    /// 添加大于条件。
    pub fn gt(mut self, field: impl Into<String>, value: SpecValue) -> Self
    {
        let pred = Predicate::gt(field, value);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a less-than condition.
    /// 添加小于条件。
    pub fn lt(mut self, field: impl Into<String>, value: SpecValue) -> Self
    {
        let pred = Predicate::lt(field, value);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a greater-than-or-equal condition.
    /// 添加大于等于条件。
    pub fn gte(mut self, field: impl Into<String>, value: SpecValue) -> Self
    {
        let pred = Predicate::gte(field, value);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a less-than-or-equal condition.
    /// 添加小于等于条件。
    pub fn lte(mut self, field: impl Into<String>, value: SpecValue) -> Self
    {
        let pred = Predicate::lte(field, value);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a LIKE condition.
    /// 添加 LIKE 条件。
    pub fn like(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self
    {
        let pred = Predicate::like(field, pattern);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add an IN condition.
    /// 添加 IN 条件。
    pub fn in_pred(mut self, field: impl Into<String>, values: Vec<SpecValue>) -> Self
    {
        let pred = Predicate::in_pred(field, values);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a NOT IN condition.
    /// 添加 NOT IN 条件。
    pub fn not_in(mut self, field: impl Into<String>, values: Vec<SpecValue>) -> Self
    {
        let pred = Predicate::not_in(field, values);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add an IS NULL condition.
    /// 添加 IS NULL 条件。
    pub fn is_null(mut self, field: impl Into<String>) -> Self
    {
        let pred = Predicate::is_null(field);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add an IS NOT NULL condition.
    /// 添加 IS NOT NULL 条件。
    pub fn is_not_null(mut self, field: impl Into<String>) -> Self
    {
        let pred = Predicate::is_not_null(field);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Add a BETWEEN condition.
    /// 添加 BETWEEN 条件。
    pub fn between(mut self, field: impl Into<String>, low: SpecValue, high: SpecValue) -> Self
    {
        let pred = Predicate::between(field, low, high);
        self.predicate = Some(match self.predicate.take()
        {
            Some(existing) => Predicate::and(Box::new(existing), Box::new(pred)),
            None => pred,
        });
        self
    }

    /// Combine with another specification using AND.
    /// 使用 AND 与另一个 specification 组合。
    pub fn and(self, other: impl Specification) -> Self
    {
        match (self.predicate, other.to_predicate())
        {
            (Some(left), Some(right)) => Self {
                predicate: Some(Predicate::and(Box::new(left), Box::new(right))),
            },
            (Some(p), None) => Self { predicate: Some(p) },
            (None, Some(p)) => Self { predicate: Some(p) },
            (None, None) => Self { predicate: None },
        }
    }

    /// Combine with another specification using OR.
    /// 使用 OR 与另一个 specification 组合。
    pub fn or(self, other: impl Specification) -> Self
    {
        match (self.predicate, other.to_predicate())
        {
            (Some(left), Some(right)) => Self {
                predicate: Some(Predicate::or(Box::new(left), Box::new(right))),
            },
            (Some(p), None) => Self { predicate: Some(p) },
            (None, Some(p)) => Self { predicate: Some(p) },
            (None, None) => Self { predicate: None },
        }
    }

    /// Negate the current specification.
    /// 取反当前 specification。
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self
    {
        match self.predicate
        {
            Some(p) => Self {
                predicate: Some(Predicate::not(Box::new(p))),
            },
            None => Self { predicate: None },
        }
    }

    /// Build the final `Specification` as a concrete object.
    /// 将最终的 `Specification` 构建为具体对象。
    pub fn build(self) -> BuiltSpecification
    {
        BuiltSpecification {
            predicate: self.predicate,
        }
    }
}

/// A concrete specification built by [`Specifications`].
/// 由 [`Specifications`] 构建的具体 specification。
#[derive(Debug, Clone)]
pub struct BuiltSpecification
{
    /// The composed predicate.
    /// 组合后的谓词。
    predicate: Option<Predicate>,
}

impl Specification for BuiltSpecification
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        self.predicate.clone()
    }
}

/// Factory for creating common specifications.
/// 创建常用 specification 的工厂。
///
/// Since Rust does not allow inherent `impl` blocks on traits,
/// these factory methods are provided on a separate struct.
/// 由于 Rust 不允许在 trait 上使用固有 `impl` 块，
/// 这些工厂方法在单独的结构体上提供。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::specification::{SpecFactories, SpecValue, Specification};
///
/// let spec = SpecFactories::eq("name", SpecValue::String("Alice".into()));
/// assert!(spec.to_predicate().is_some());
/// ```
pub struct SpecFactories;

impl SpecFactories
{
    /// Create a specification that always matches (no constraints).
    /// 创建总是匹配的 specification（无约束）。
    pub fn always() -> AlwaysSpec
    {
        AlwaysSpec
    }

    /// Create an equality specification.
    /// 创建相等 specification。
    pub fn eq(field: impl Into<String>, value: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::eq(field, value))
    }

    /// Create an inequality specification.
    /// 创建不等 specification。
    pub fn ne(field: impl Into<String>, value: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::ne(field, value))
    }

    /// Create a greater-than specification.
    /// 创建大于 specification。
    pub fn gt(field: impl Into<String>, value: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::gt(field, value))
    }

    /// Create a less-than specification.
    /// 创建小于 specification。
    pub fn lt(field: impl Into<String>, value: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::lt(field, value))
    }

    /// Create a greater-than-or-equal specification.
    /// 创建大于等于 specification。
    pub fn gte(field: impl Into<String>, value: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::gte(field, value))
    }

    /// Create a less-than-or-equal specification.
    /// 创建小于等于 specification。
    pub fn lte(field: impl Into<String>, value: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::lte(field, value))
    }

    /// Create a LIKE specification.
    /// 创建 LIKE specification。
    pub fn like(field: impl Into<String>, pattern: impl Into<String>) -> SimpleSpec
    {
        SimpleSpec(Predicate::like(field, pattern))
    }

    /// Create an IN specification.
    /// 创建 IN specification。
    pub fn in_pred(field: impl Into<String>, values: Vec<SpecValue>) -> SimpleSpec
    {
        SimpleSpec(Predicate::in_pred(field, values))
    }

    /// Create a NOT IN specification.
    /// 创建 NOT IN specification。
    pub fn not_in(field: impl Into<String>, values: Vec<SpecValue>) -> SimpleSpec
    {
        SimpleSpec(Predicate::not_in(field, values))
    }

    /// Create an IS NULL specification.
    /// 创建 IS NULL specification。
    pub fn is_null(field: impl Into<String>) -> SimpleSpec
    {
        SimpleSpec(Predicate::is_null(field))
    }

    /// Create an IS NOT NULL specification.
    /// 创建 IS NOT NULL specification。
    pub fn is_not_null(field: impl Into<String>) -> SimpleSpec
    {
        SimpleSpec(Predicate::is_not_null(field))
    }

    /// Create a BETWEEN specification.
    /// 创建 BETWEEN specification。
    pub fn between(field: impl Into<String>, low: SpecValue, high: SpecValue) -> SimpleSpec
    {
        SimpleSpec(Predicate::between(field, low, high))
    }

    /// Combine two specifications with AND.
    /// 使用 AND 组合两个 specification。
    pub fn and_spec(s1: impl Specification + 'static, s2: impl Specification + 'static) -> AndSpec
    {
        AndSpec {
            left: Box::new(s1),
            right: Box::new(s2),
        }
    }

    /// Combine two specifications with OR.
    /// 使用 OR 组合两个 specification。
    pub fn or_spec(s1: impl Specification + 'static, s2: impl Specification + 'static) -> OrSpec
    {
        OrSpec {
            left: Box::new(s1),
            right: Box::new(s2),
        }
    }

    /// Negate a specification.
    /// 取反一个 specification。
    pub fn not_spec(s: impl Specification + 'static) -> NotSpec
    {
        NotSpec { inner: Box::new(s) }
    }
}

/// A specification that always matches.
/// 总是匹配的 specification。
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysSpec;

impl Specification for AlwaysSpec
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        None
    }
}

/// A simple specification wrapping a single predicate.
/// 包装单个谓词的简单 specification。
#[derive(Debug, Clone)]
pub struct SimpleSpec(pub Predicate);

impl Specification for SimpleSpec
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        Some(self.0.clone())
    }
}

/// AND composition of two specifications.
/// 两个 specification 的 AND 组合。
#[derive(Debug)]
pub struct AndSpec
{
    left: Box<dyn Specification>,
    right: Box<dyn Specification>,
}

impl Specification for AndSpec
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        match (self.left.to_predicate(), self.right.to_predicate())
        {
            (Some(l), Some(r)) => Some(Predicate::and(Box::new(l), Box::new(r))),
            (Some(p), None) => Some(p),
            (None, Some(p)) => Some(p),
            (None, None) => None,
        }
    }
}

/// OR composition of two specifications.
/// 两个 specification 的 OR 组合。
#[derive(Debug)]
pub struct OrSpec
{
    left: Box<dyn Specification>,
    right: Box<dyn Specification>,
}

impl Specification for OrSpec
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        match (self.left.to_predicate(), self.right.to_predicate())
        {
            (Some(l), Some(r)) => Some(Predicate::or(Box::new(l), Box::new(r))),
            (Some(p), None) => Some(p),
            (None, Some(p)) => Some(p),
            (None, None) => None,
        }
    }
}

/// NOT composition of a specification.
/// specification 的 NOT 组合。
#[derive(Debug)]
pub struct NotSpec
{
    inner: Box<dyn Specification>,
}

impl Specification for NotSpec
{
    fn to_predicate(&self) -> Option<Predicate>
    {
        self.inner
            .to_predicate()
            .map(|p| Predicate::not(Box::new(p)))
    }
}

/// Executor for specification-based queries.
/// 基于 specification 的查询执行器。
///
/// This mirrors Spring Data JPA's `JpaSpecificationExecutor<T>` interface,
/// allowing repositories to execute queries built from specifications.
/// 这镜像了 Spring Data JPA 的 `JpaSpecificationExecutor<T>` 接口，
/// 允许 repository 执行由 specification 构建的查询。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::specification::{JpaSpecificationExecutor, Specification, SpecValue};
/// use hiver_data_commons::PageRequest;
///
/// // Find active users / 查找活跃用户
/// let spec = Specification::eq("status", SpecValue::String("active".into()));
/// let users = executor.find_all_spec(spec).await?;
///
/// // Find with pagination / 分页查找
/// let page = executor.find_all_spec_pageable(spec, PageRequest::of(0, 20)).await?;
/// ```
#[async_trait::async_trait]
pub trait JpaSpecificationExecutor<T: Send + 'static, ID: Send + Sync + 'static>:
    Send + Sync
{
    /// Error type for specification queries.
    /// specification 查询的错误类型。
    type Error: std::fmt::Debug + Send + Sync;

    /// Find a single entity matching the specification.
    /// 查找匹配 specification 的单个实体。
    ///
    /// Returns `Ok(None)` if no match is found.
    /// 如果没有找到匹配项，返回 `Ok(None)`。
    async fn find_one(&self, spec: impl Specification) -> Result<Option<T>, Self::Error>;

    /// Find all entities matching the specification.
    /// 查找所有匹配 specification 的实体。
    async fn find_all_spec(&self, spec: impl Specification) -> Result<Vec<T>, Self::Error>;

    /// Count entities matching the specification.
    /// 统计匹配 specification 的实体数量。
    async fn count_spec(&self, spec: impl Specification) -> Result<u64, Self::Error>;

    /// Find entities matching the specification with pagination.
    /// 分页查找匹配 specification 的实体。
    async fn find_all_spec_pageable(
        &self,
        spec: impl Specification,
        pageable: PageRequest,
    ) -> Result<Page<T>, Self::Error>;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_spec_value_display()
    {
        assert_eq!(SpecValue::String("hello".into()).to_string(), "'hello'");
        assert_eq!(SpecValue::I64(42).to_string(), "42");
        assert_eq!(SpecValue::Bool(true).to_string(), "true");
        assert_eq!(SpecValue::Null.to_string(), "NULL");
    }

    #[test]
    fn test_predicate_eq()
    {
        let pred = Predicate::eq("name", SpecValue::String("Alice".into()));
        assert_eq!(pred.to_sql(), "name = 'Alice'");
    }

    #[test]
    fn test_predicate_ne()
    {
        let pred = Predicate::ne("status", SpecValue::String("deleted".into()));
        assert_eq!(pred.to_sql(), "status != 'deleted'");
    }

    #[test]
    fn test_predicate_gt_lt()
    {
        let pred = Predicate::gt("age", SpecValue::I64(18));
        assert_eq!(pred.to_sql(), "age > 18");

        let pred = Predicate::lt("age", SpecValue::I64(65));
        assert_eq!(pred.to_sql(), "age < 65");
    }

    #[test]
    fn test_predicate_like()
    {
        let pred = Predicate::like("name", "%Alice%");
        assert_eq!(pred.to_sql(), "name LIKE '%Alice%'");
    }

    #[test]
    fn test_predicate_in()
    {
        let pred = Predicate::in_pred("city", vec![
            SpecValue::String("Beijing".into()),
            SpecValue::String("Shanghai".into()),
        ]);
        assert_eq!(pred.to_sql(), "city IN ('Beijing', 'Shanghai')");
    }

    #[test]
    fn test_predicate_is_null()
    {
        let pred = Predicate::is_null("deleted_at");
        assert_eq!(pred.to_sql(), "deleted_at IS NULL");
    }

    #[test]
    fn test_predicate_is_not_null()
    {
        let pred = Predicate::is_not_null("email");
        assert_eq!(pred.to_sql(), "email IS NOT NULL");
    }

    #[test]
    fn test_predicate_between()
    {
        let pred = Predicate::between("age", SpecValue::I64(18), SpecValue::I64(65));
        assert_eq!(pred.to_sql(), "age BETWEEN 18 AND 65");
    }

    #[test]
    fn test_predicate_and()
    {
        let pred = Predicate::and(
            Box::new(Predicate::eq("status", SpecValue::String("active".into()))),
            Box::new(Predicate::gt("age", SpecValue::I64(18))),
        );
        assert_eq!(pred.to_sql(), "(status = 'active' AND age > 18)");
    }

    #[test]
    fn test_predicate_or()
    {
        let pred = Predicate::or(
            Box::new(Predicate::eq("role", SpecValue::String("admin".into()))),
            Box::new(Predicate::eq("role", SpecValue::String("superadmin".into()))),
        );
        assert_eq!(pred.to_sql(), "(role = 'admin' OR role = 'superadmin')");
    }

    #[test]
    fn test_predicate_not()
    {
        let pred =
            Predicate::not(Box::new(Predicate::eq("status", SpecValue::String("deleted".into()))));
        assert_eq!(pred.to_sql(), "NOT (status = 'deleted')");
    }

    #[test]
    fn test_specification_always()
    {
        let spec = SpecFactories::always();
        assert!(spec.to_predicate().is_none());
    }

    #[test]
    fn test_specification_eq()
    {
        let spec = SpecFactories::eq("name", SpecValue::String("Alice".into()));
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "name = 'Alice'");
    }

    #[test]
    fn test_specification_and_spec()
    {
        let s1 = SpecFactories::eq("status", SpecValue::String("active".into()));
        let s2 = SpecFactories::gt("age", SpecValue::I64(18));
        let combined = SpecFactories::and_spec(s1, s2);
        let pred = combined.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "(status = 'active' AND age > 18)");
    }

    #[test]
    fn test_specification_or_spec()
    {
        let s1 = SpecFactories::eq("role", SpecValue::String("admin".into()));
        let s2 = SpecFactories::eq("role", SpecValue::String("superadmin".into()));
        let combined = SpecFactories::or_spec(s1, s2);
        let pred = combined.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "(role = 'admin' OR role = 'superadmin')");
    }

    #[test]
    fn test_specification_not_spec()
    {
        let s = SpecFactories::eq("deleted", SpecValue::Bool(true));
        let negated = SpecFactories::not_spec(s);
        let pred = negated.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "NOT (deleted = true)");
    }

    #[test]
    fn test_specifications_builder()
    {
        let spec = Specifications::where_clause()
            .eq("status", SpecValue::String("active".into()))
            .gt("age", SpecValue::I64(18))
            .build();

        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "(status = 'active' AND age > 18)");
    }

    #[test]
    fn test_specifications_builder_empty()
    {
        let spec = Specifications::where_clause().build();
        assert!(spec.to_predicate().is_none());
    }

    #[test]
    fn test_specifications_builder_and()
    {
        let s1 = Specifications::where_clause()
            .eq("status", SpecValue::String("active".into()))
            .build();
        let combined = Specifications::where_clause()
            .and(s1)
            .eq("age", SpecValue::I64(18))
            .build();
        let pred = combined.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "(status = 'active' AND age = 18)");
    }

    #[test]
    fn test_specifications_builder_or()
    {
        let s1 = Specifications::where_clause()
            .eq("role", SpecValue::String("admin".into()))
            .build();
        let s2 = Specifications::where_clause()
            .eq("role", SpecValue::String("superadmin".into()))
            .build();
        let combined = Specifications::where_clause().or(s1).or(s2).build();
        let pred = combined.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "(role = 'admin' OR role = 'superadmin')");
    }

    #[test]
    fn test_specifications_builder_not()
    {
        let spec = Specifications::where_clause()
            .eq("deleted", SpecValue::Bool(true))
            .not()
            .build();
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "NOT (deleted = true)");
    }

    #[test]
    fn test_specifications_builder_like()
    {
        let spec = Specifications::where_clause()
            .like("name", "%Alice%")
            .build();
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "name LIKE '%Alice%'");
    }

    #[test]
    fn test_specifications_builder_in()
    {
        let spec = Specifications::where_clause()
            .in_pred("city", vec![
                SpecValue::String("Beijing".into()),
                SpecValue::String("Shanghai".into()),
            ])
            .build();
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "city IN ('Beijing', 'Shanghai')");
    }

    #[test]
    fn test_specifications_builder_between()
    {
        let spec = Specifications::where_clause()
            .between("age", SpecValue::I64(18), SpecValue::I64(65))
            .build();
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "age BETWEEN 18 AND 65");
    }

    #[test]
    fn test_specifications_builder_is_null()
    {
        let spec = Specifications::where_clause().is_null("deleted_at").build();
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "deleted_at IS NULL");
    }

    #[test]
    fn test_specifications_builder_is_not_null()
    {
        let spec = Specifications::where_clause().is_not_null("email").build();
        let pred = spec.to_predicate().unwrap();
        assert_eq!(pred.to_sql(), "email IS NOT NULL");
    }
}
