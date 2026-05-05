//! LDAP query builder
//! LDAP查询构建器
//!
//! Provides a fluent API for building LDAP search filters.
//! Equivalent to Spring LDAP's `LdapQueryBuilder`.
//!
//! 提供用于构建LDAP搜索过滤器的流畅API。
//! 等价于 Spring LDAP 的 `LdapQueryBuilder`。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_ldap::query::LdapQueryBuilder;
//!
//! // Build: (&(objectClass=person)(sn=Smith))
//! let filter = LdapQueryBuilder::new()
//!     .where_attr("objectClass").is("person")
//!     .and()
//!     .where_attr("sn").is("Smith")
//!     .build();
//! assert_eq!(filter, "(&(objectClass=person)(sn=Smith))");
//!
//! // Build: (|(cn=John)(cn=Jane))
//! let filter = LdapQueryBuilder::new()
//!     .or_query()
//!     .where_attr("cn").is("John")
//!     .or()
//!     .where_attr("cn").is("Jane")
//!     .build();
//! assert_eq!(filter, "(|(cn=John)(cn=Jane))");
//! ```

/// LDAP search scope
/// LDAP搜索范围
///
/// Determines how deep the search traverses the directory tree.
/// 决定搜索遍历目录树的深度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    /// Search only the base entry / 仅搜索基础条目
    Base,
    /// Search one level below the base / 搜索基础下一级
    OneLevel,
    /// Search the entire subtree / 搜索整个子树
    Subtree,
    /// Search all children below the base / 搜索基础下所有子项
    Children,
}

/// Build the final LDAP filter string from this scope
/// 从此范围构建最终的LDAP过滤器字符串
pub fn scope_to_filter(scope: SearchScope) -> &'static str {
    match scope {
        SearchScope::Base => "base",
        SearchScope::OneLevel => "one",
        SearchScope::Subtree => "sub",
        SearchScope::Children => "children",
    }
}

/// LDAP query builder for constructing search filters
/// 用于构建搜索过滤器的LDAP查询构建器
///
/// Equivalent to Spring LDAP's `LdapQueryBuilder`.
/// Provides a fluent, type-safe API for building complex LDAP filter expressions.
///
/// 等价于 Spring LDAP 的 `LdapQueryBuilder`。
/// 提供流畅的类型安全API来构建复杂的LDAP过滤器表达式。
#[derive(Debug, Clone)]
pub struct LdapQueryBuilder {
    /// The filter expression components / 过滤器表达式组件
    components: Vec<String>,
    /// The top-level logical operator / 顶层逻辑运算符
    operator: FilterOperator,
}

/// Logical filter operator / 逻辑过滤器运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterOperator {
    /// AND (&) / 与
    And,
    /// OR (|) / 或
    Or,
}

impl Default for LdapQueryBuilder {
    fn default() -> Self {
        Self {
            components: Vec::new(),
            operator: FilterOperator::And,
        }
    }
}

impl LdapQueryBuilder {
    /// Create a new query builder (defaults to AND logic)
    /// 创建新的查询构建器（默认为AND逻辑）
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new query builder with OR logic
    /// 创建使用OR逻辑的查询构建器
    pub fn or_query() -> Self {
        Self {
            components: Vec::new(),
            operator: FilterOperator::Or,
        }
    }

    /// Start a condition on an attribute / 开始对某个属性的条件
    ///
    /// Returns an `AttributeCondition` for further chaining.
    /// 返回 `AttributeCondition` 以进一步链式操作。
    pub fn where_attr(self, attr: &str) -> AttributeCondition {
        AttributeCondition {
            builder: self,
            attr: attr.to_string(),
        }
    }

    /// Add an AND between conditions / 在条件之间添加AND
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use nexus_ldap::query::LdapQueryBuilder;
    ///
    /// let filter = LdapQueryBuilder::new()
    ///     .where_attr("cn").is("John")
    ///     .and()
    ///     .where_attr("objectClass").is("person")
    ///     .build();
    /// assert_eq!(filter, "(&(cn=John)(objectClass=person))");
    /// ```
    pub fn and(self) -> Self {
        // No-op separator for readability; the operator handles logic
        self
    }

    /// Add an OR between conditions / 在条件之间添加OR
    pub fn or(self) -> Self {
        self
    }

    /// Add a raw filter component / 添加原始过滤器组件
    ///
    /// Allows inserting pre-built filter expressions.
    /// 允许插入预构建的过滤器表达式。
    pub fn raw(mut self, filter: &str) -> Self {
        self.components.push(filter.to_string());
        self
    }

    /// Add an equality filter component / 添加等值过滤器组件
    ///
    /// Shorthand for `where_attr(attr).is(value)` without chaining.
    /// 不链式调用时 `where_attr(attr).is(value)` 的简写。
    pub fn eq(mut self, attr: &str, value: &str) -> Self {
        self.components.push(format!("({}={})", attr, value));
        self
    }

    /// Add a presence filter (attribute exists) / 添加存在性过滤器（属性存在）
    pub fn present(mut self, attr: &str) -> Self {
        self.components.push(format!("({}=*)", attr));
        self
    }

    /// Add a substring filter / 添加子串过滤器
    ///
    /// Example: `substring("cn", Some("Jo"), None, Some("n"))` → `(cn=Jo*n)`
    pub fn substring(
        mut self,
        attr: &str,
        initial: Option<&str>,
        any: Option<&str>,
        final_: Option<&str>,
    ) -> Self {
        let init = initial.unwrap_or("");
        let mid = any.unwrap_or("");
        let fin = final_.unwrap_or("");
        self.components.push(format!("({}={}*{}*{})", attr, init, mid, fin));
        self
    }

    /// Add a greater-than-or-equal filter / 添加大于等于过滤器
    pub fn gte(mut self, attr: &str, value: &str) -> Self {
        self.components.push(format!("({}>={})", attr, value));
        self
    }

    /// Add a less-than-or-equal filter / 添加小于等于过滤器
    pub fn lte(mut self, attr: &str, value: &str) -> Self {
        self.components.push(format!("({}<={})", attr, value));
        self
    }

    /// Add an approximate match filter / 添加近似匹配过滤器
    pub fn approx(mut self, attr: &str, value: &str) -> Self {
        self.components.push(format!("({}~={})", attr, value));
        self
    }

    /// Build the final LDAP filter string / 构建最终的LDAP过滤器字符串
    ///
    /// Returns a well-formed LDAP filter expression.
    /// 返回格式良好的LDAP过滤器表达式。
    pub fn build(self) -> String {
        if self.components.is_empty() {
            return "(objectClass=*)".to_string();
        }
        if self.components.len() == 1 {
            return self.components.into_iter().next().unwrap_or_default();
        }
        let op = match self.operator {
            FilterOperator::And => '&',
            FilterOperator::Or => '|',
        };
        let inner: String = self.components.join("");
        format!("({}{})", op, inner)
    }
}

/// Attribute condition builder / 属性条件构建器
///
/// Intermediate builder state for setting the comparison operator
/// on an attribute.
///
/// 中间构建器状态，用于设置属性上的比较运算符。
#[derive(Debug)]
pub struct AttributeCondition {
    builder: LdapQueryBuilder,
    attr: String,
}

impl AttributeCondition {
    /// Equality condition / 等值条件
    ///
    /// Matches entries where the attribute equals the value.
    /// 匹配属性等于值的条目。
    pub fn is(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}={})", self.attr, value));
        b
    }

    /// Not-equal condition (using negation) / 不等条件（使用否定）
    ///
    /// Matches entries where the attribute does NOT equal the value.
    /// 匹配属性不等于值的条目。
    pub fn is_not(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components
            .push(format!("(!({}={}))", self.attr, value));
        b
    }

    /// Presence condition (attribute has any value) / 存在性条件（属性有任意值）
    pub fn exists(self) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}=*)", self.attr));
        b
    }

    /// Absence condition (attribute has no value) / 缺失性条件（属性没有值）
    pub fn not_exists(self) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("(!({}=*))", self.attr));
        b
    }

    /// Like/contains condition / 包含条件
    ///
    /// Matches entries where the attribute contains the value as a substring.
    /// 匹配属性包含值作为子串的条目。
    pub fn like(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}=*{}*)", self.attr, value));
        b
    }

    /// Starts-with condition / 开头匹配条件
    pub fn starts_with(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}={}*)", self.attr, value));
        b
    }

    /// Ends-with condition / 结尾匹配条件
    pub fn ends_with(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}=*{})", self.attr, value));
        b
    }

    /// Greater than or equal / 大于等于
    pub fn gte(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}>={})", self.attr, value));
        b
    }

    /// Less than or equal / 小于等于
    pub fn lte(self, value: &str) -> LdapQueryBuilder {
        let mut b = self.builder;
        b.components.push(format!("({}<={})", self.attr, value));
        b
    }
}

/// Convenience function to build a filter matching objectClass
/// 构建匹配 objectClass 的过滤器的便捷函数
pub fn object_class(class: &str) -> String {
    format!("(objectClass={})", class)
}

/// Convenience function to build a filter matching a DN
/// 构建匹配DN的过滤器的便捷函数
pub fn dn_filter(attr: &str, dn: &str) -> String {
    format!("({}={})", attr, dn)
}

/// Build the wildcard filter (matches everything)
/// 构建通配符过滤器（匹配所有内容）
pub fn wildcard() -> String {
    "(objectClass=*)".to_string()
}

#[cfg(test)]
mod query_tests {
    use super::*;

    #[test]
    fn test_empty_builder_returns_wildcard() {
        let filter = LdapQueryBuilder::new().build();
        assert_eq!(filter, "(objectClass=*)");
    }

    #[test]
    fn test_single_eq_condition() {
        let filter = LdapQueryBuilder::new()
            .eq("objectClass", "person")
            .build();
        assert_eq!(filter, "(objectClass=person)");
    }

    #[test]
    fn test_and_filter() {
        let filter = LdapQueryBuilder::new()
            .where_attr("objectClass").is("person")
            .and()
            .where_attr("sn").is("Smith")
            .build();
        assert_eq!(filter, "(&(objectClass=person)(sn=Smith))");
    }

    #[test]
    fn test_or_filter() {
        let filter = LdapQueryBuilder::or_query()
            .where_attr("cn").is("John")
            .or()
            .where_attr("cn").is("Jane")
            .build();
        assert_eq!(filter, "(|(cn=John)(cn=Jane))");
    }

    #[test]
    fn test_present_filter() {
        let filter = LdapQueryBuilder::new()
            .present("mail")
            .build();
        assert_eq!(filter, "(mail=*)");
    }

    #[test]
    fn test_like_filter() {
        let filter = LdapQueryBuilder::new()
            .where_attr("cn").like("John")
            .build();
        assert_eq!(filter, "(cn=*John*)");
    }

    #[test]
    fn test_starts_with_filter() {
        let filter = LdapQueryBuilder::new()
            .where_attr("cn").starts_with("J")
            .build();
        assert_eq!(filter, "(cn=J*)");
    }

    #[test]
    fn test_ends_with_filter() {
        let filter = LdapQueryBuilder::new()
            .where_attr("mail").ends_with("@example.com")
            .build();
        assert_eq!(filter, "(mail=*@example.com)");
    }

    #[test]
    fn test_not_equal() {
        let filter = LdapQueryBuilder::new()
            .where_attr("status").is_not("inactive")
            .build();
        assert_eq!(filter, "(!(status=inactive))");
    }

    #[test]
    fn test_gte_lte() {
        let filter = LdapQueryBuilder::new()
            .gte("uidNumber", "1000")
            .lte("uidNumber", "2000")
            .build();
        assert_eq!(filter, "(&(uidNumber>=1000)(uidNumber<=2000))");
    }

    #[test]
    fn test_raw_filter() {
        let filter = LdapQueryBuilder::new()
            .raw("(&(cn=admin)(objectClass=*))")
            .build();
        assert_eq!(filter, "(&(cn=admin)(objectClass=*))");
    }

    #[test]
    fn test_complex_nested() {
        let filter = LdapQueryBuilder::new()
            .eq("objectClass", "person")
            .raw("(|(cn=John)(cn=Jane))")
            .build();
        assert_eq!(
            filter,
            "(&(objectClass=person)(|(cn=John)(cn=Jane)))"
        );
    }

    #[test]
    fn test_exists_not_exists() {
        let filter = LdapQueryBuilder::new()
            .where_attr("mail").exists()
            .and()
            .where_attr("password").not_exists()
            .build();
        assert_eq!(filter, "(&(mail=*)(!(password=*)))");
    }

    #[test]
    fn test_convenience_functions() {
        assert_eq!(object_class("person"), "(objectClass=person)");
        assert_eq!(wildcard(), "(objectClass=*)");
        assert_eq!(dn_filter("member", "cn=John,dc=example,dc=com"), "(member=cn=John,dc=example,dc=com)");
    }
}
