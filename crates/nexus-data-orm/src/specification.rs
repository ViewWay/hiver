//! Specification pattern for composable queries
//! Specification 模式，用于可组合查询
//!
//! Equivalent to Spring Data JPA's Specification / Criteria API.
//! Allows building complex queries from reusable, composable predicates.
//!
//! 等价于 Spring Data JPA 的 Specification / Criteria API。
//! 允许从可重用、可组合的谓词构建复杂查询。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_orm::specification::{Specification, Spec, CompositeSpec};
//!
//! let active_users = Spec::new("status = ?", vec!["active"]);
//! let admins = Spec::new("role = ?", vec!["ADMIN"]);
//! let active_admins = active_users.and(admins);
//! ```

/// A single specification predicate
/// 单个规约谓词
#[derive(Debug, Clone)]
pub struct Spec {
    /// SQL where clause fragment (e.g. "status = ?")
    /// SQL WHERE 子句片段
    pub clause: String,
    /// Parameter values
    /// 参数值
    pub params: Vec<String>,
}

impl Spec {
    /// Create a new specification
    /// 创建新规约
    pub fn new(clause: impl Into<String>, params: Vec<&str>) -> Self {
        Self {
            clause: clause.into(),
            params: params.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create an empty (always-true) specification
    /// 创建空规约（恒真）
    pub fn none() -> Self {
        Self {
            clause: "1=1".to_string(),
            params: Vec::new(),
        }
    }

    /// Get the clause
    /// 获取子句
    pub fn clause(&self) -> &str {
        &self.clause
    }

    /// Get the parameters
    /// 获取参数
    pub fn params(&self) -> &[String] {
        &self.params
    }
}

/// Composite specification with AND/OR logic
/// 带有 AND/OR 逻辑的组合规约
#[derive(Debug, Clone)]
pub enum CompositeSpec {
    /// A single predicate
    /// 单个谓词
    Leaf(Spec),
    /// AND combination
    /// AND 组合
    And(Box<CompositeSpec>, Box<CompositeSpec>),
    /// OR combination
    /// OR 组合
    Or(Box<CompositeSpec>, Box<CompositeSpec>),
    /// NOT negation
    /// NOT 取反
    Not(Box<CompositeSpec>),
}

impl CompositeSpec {
    /// Create from a simple spec
    /// 从简单规约创建
    pub fn from_spec(spec: Spec) -> Self {
        Self::Leaf(spec)
    }

    /// Combine with AND
    /// AND 组合
    pub fn and(self, other: CompositeSpec) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    /// Combine with OR
    /// OR 组合
    pub fn or(self, other: CompositeSpec) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }

    /// Negate
    /// 取反
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Convert to SQL WHERE clause with parameters
    /// 转换为 SQL WHERE 子句和参数
    pub fn to_sql(&self) -> (String, Vec<String>) {
        match self {
            CompositeSpec::Leaf(spec) => {
                (spec.clause.clone(), spec.params.clone())
            }
            CompositeSpec::And(left, right) => {
                let (l_clause, l_params) = left.to_sql();
                let (r_clause, r_params) = right.to_sql();
                let mut params = l_params;
                params.extend(r_params);
                (format!("({} AND {})", l_clause, r_clause), params)
            }
            CompositeSpec::Or(left, right) => {
                let (l_clause, l_params) = left.to_sql();
                let (r_clause, r_params) = right.to_sql();
                let mut params = l_params;
                params.extend(r_params);
                (format!("({} OR {})", l_clause, r_clause), params)
            }
            CompositeSpec::Not(inner) => {
                let (clause, params) = inner.to_sql();
                (format!("NOT ({})", clause), params)
            }
        }
    }
}

/// Trait for implementing custom specifications
/// 实现自定义规约的 trait
///
/// Equivalent to Spring Data JPA's `Specification<T>` interface.
/// 等价于 Spring Data JPA 的 `Specification<T>` 接口。
pub trait Specification: Send + Sync {
    /// Convert this specification to a composite spec
    /// 将此规约转换为组合规约
    fn to_spec(&self) -> CompositeSpec;
}

/// Trait for repositories supporting Specification queries
/// 支持 Specification 查询的 Repository trait
pub trait JpaSpecificationExecutor<T: Send>: Send + Sync {
    /// Find all entities matching the specification
    /// 查找所有匹配规约的实体
    fn find_by_spec(&self, spec: &(dyn Specification + Send + Sync)) -> impl std::future::Future<Output = Result<Vec<T>, crate::OrmError>> + Send;

    /// Count entities matching the specification
    /// 计算匹配规约的实体数
    fn count_by_spec(&self, spec: &(dyn Specification + Send + Sync)) -> impl std::future::Future<Output = Result<i64, crate::OrmError>> + Send;

    /// Check if any entity matches
    /// 检查是否有实体匹配
    fn exists_by_spec(&self, spec: &(dyn Specification + Send + Sync)) -> impl std::future::Future<Output = Result<bool, crate::OrmError>> + Send where Self: Sync {
        async move { Ok(self.count_by_spec(spec).await? > 0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_creation() {
        let spec = Spec::new("status = ?", vec!["active"]);
        assert_eq!(spec.clause(), "status = ?");
        assert_eq!(spec.params().len(), 1);
    }

    #[test]
    fn test_spec_none() {
        let spec = Spec::none();
        assert_eq!(spec.clause(), "1=1");
        assert!(spec.params().is_empty());
    }

    #[test]
    fn test_composite_and() {
        let left = CompositeSpec::from_spec(Spec::new("age > ?", vec!["18"]));
        let right = CompositeSpec::from_spec(Spec::new("status = ?", vec!["active"]));
        let combined = left.and(right);
        let (sql, params) = combined.to_sql();
        assert!(sql.contains("AND"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_composite_or() {
        let left = CompositeSpec::from_spec(Spec::new("role = ?", vec!["ADMIN"]));
        let right = CompositeSpec::from_spec(Spec::new("role = ?", vec!["MODERATOR"]));
        let combined = left.or(right);
        let (sql, params) = combined.to_sql();
        assert!(sql.contains("OR"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_composite_not() {
        let inner = CompositeSpec::from_spec(Spec::new("deleted = ?", vec!["true"]));
        let negated = inner.not();
        let (sql, params) = negated.to_sql();
        assert!(sql.starts_with("NOT"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_nested_composition() {
        let a = CompositeSpec::from_spec(Spec::new("a = ?", vec!["1"]));
        let b = CompositeSpec::from_spec(Spec::new("b = ?", vec!["2"]));
        let c = CompositeSpec::from_spec(Spec::new("c = ?", vec!["3"]));
        let combined = a.and(b).or(c);
        let (sql, params) = combined.to_sql();
        assert!(sql.contains("AND"));
        assert!(sql.contains("OR"));
        assert_eq!(params.len(), 3);
    }
}
