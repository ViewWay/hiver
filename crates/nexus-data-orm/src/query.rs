//! Query Builder
//! 查询构建器
//!
//! # Overview / 概述
//!
//! This module provides a fluent query builder for constructing database queries.
//! 本模块提供用于构建数据库查询的流畅查询构建器。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring/JPA |
//! |-------|------------|
//! | `QueryBuilder::where_()` | `Specification` / `CriteriaBuilder.where()` |
//! | `QueryBuilder::order_by()` | `Sort` / `OrderBy` |
//! | `QueryBuilder::limit()` | `Pageable` / `setMaxResults()` |
//! | `QueryBuilder::join()` | `EntityGraph` / `JOIN` |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_orm::QueryBuilder;
//!
//! let users = User::query()
//!     .where_("age > ?", &["18"])
//!     .where_("status = ?", &["active"])
//!     .order_by("created_at DESC")
//!     .limit(10)
//!     .offset(20)
//!     .all().await?;
//! ```

use crate::{Error, Model, Result};
use nexus_data_rdbc::{DatabaseClient, QueryParam};
use std::marker::PhantomData;

fn validate_identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Trait for SQL parameter conversion
/// SQL 参数转换的 trait
pub trait ToSql: Send + Sync {
    /// Convert to SQL value
    /// 转换为 SQL 值
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for i64 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for u32 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for u64 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for f64 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for &str {
    fn to_sql(&self) -> String {
        // SECURITY: Escape single-quotes for SQL safety: ' → ''
        // and strip null bytes which can cause truncation attacks.
        // NOTE: This is a fallback escaper — parameterized queries are preferred.
        format!("'{}'", self.replace('\'', "''").replace('\0', ""))
    }
}

impl ToSql for String {
    fn to_sql(&self) -> String {
        // SECURITY: Same as &str — escape quotes and strip null bytes.
        format!("'{}'", self.replace('\'', "''").replace('\0', ""))
    }
}

impl ToSql for bool {
    fn to_sql(&self) -> String {
        if *self {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }
    }
}

/// Where clause
/// WHERE 子句
///
/// Represents a condition in a WHERE clause.
/// 表示 WHERE 子句中的条件。
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// The condition SQL (with `?` placeholders)
    /// 条件 SQL（使用 `?` 占位符）
    pub condition: String,

    /// Parameters for the condition
    /// 条件的参数
    pub params: Vec<QueryParam>,
}

impl WhereClause {
    /// Create a new where clause
    /// 创建新的 WHERE 子句
    pub fn new(condition: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
            params: Vec::new(),
        }
    }

    /// Add a parameter
    /// 添加参数
    pub fn param(mut self, value: impl Into<QueryParam>) -> Self {
        self.params.push(value.into());
        self
    }

    /// Add multiple parameters
    /// 添加多个参数
    pub fn params(mut self, values: &[QueryParam]) -> Self {
        self.params.extend(values.iter().cloned());
        self
    }
}

/// Order by clause
/// ORDER BY 子句
///
/// Represents sorting in a query.
/// 表示查询中的排序。
#[derive(Debug, Clone)]
pub struct OrderBy {
    /// Column name
    /// 列名
    pub column: String,

    /// Direction (ASC or DESC)
    /// 方向（ASC 或 DESC）
    pub direction: OrderDirection,
}

impl OrderBy {
    /// Create a new order by clause
    /// 创建新的 ORDER BY 子句
    pub fn new(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: OrderDirection::Asc,
        }
    }

    /// Set direction to ascending
    /// 设置方向为升序
    pub fn asc(mut self) -> Self {
        self.direction = OrderDirection::Asc;
        self
    }

    /// Set direction to descending
    /// 设置方向为降序
    pub fn desc(mut self) -> Self {
        self.direction = OrderDirection::Desc;
        self
    }
}

/// Order direction
/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    /// Ascending
    /// 升序
    Asc,

    /// Descending
    /// 降序
    Desc,
}

impl OrderDirection {
    /// Get the SQL keyword
    /// 获取 SQL 关键字
    pub fn as_sql(&self) -> &str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// Limit clause
/// LIMIT 子句
///
/// Represents the LIMIT and OFFSET in a query.
/// 表示查询中的 LIMIT 和 OFFSET。
#[derive(Debug, Clone)]
pub struct Limit {
    /// Maximum number of rows to return
    /// 要返回的最大行数
    pub limit: Option<usize>,

    /// Number of rows to skip
    /// 要跳过的行数
    pub offset: Option<usize>,
}

impl Limit {
    /// Create a new limit clause
    /// 创建新的 LIMIT 子句
    pub fn new() -> Self {
        Self {
            limit: None,
            offset: None,
        }
    }

    /// Set the limit
    /// 设置限制
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset
    /// 设置偏移
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl Default for Limit {
    fn default() -> Self {
        Self::new()
    }
}

/// Join clause
/// JOIN 子句
///
/// Represents a JOIN in a query.
/// 表示查询中的 JOIN。
#[derive(Debug, Clone)]
pub struct Join {
    /// Join type (INNER, LEFT, RIGHT)
    /// JOIN 类型（INNER, LEFT, RIGHT）
    pub join_type: JoinType,

    /// Table to join
    /// 要连接的表
    pub table: String,

    /// Join condition
    /// 连接条件
    pub on: String,

    /// Alias for the joined table
    /// 连接表的别名
    pub alias: Option<String>,
}

impl Join {
    /// Create a new join clause
    /// 创建新的 JOIN 子句
    pub fn new(join_type: JoinType, table: impl Into<String>, on: impl Into<String>) -> Self {
        Self {
            join_type,
            table: table.into(),
            on: on.into(),
            alias: None,
        }
    }

    /// Set an alias for the joined table
    /// 为连接的表设置别名
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }
}

/// Join type
/// JOIN 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// Inner join
    /// 内连接
    Inner,

    /// Left join
    /// 左连接
    Left,

    /// Right join
    /// 右连接
    Right,

    /// Cross join
    /// 交叉连接
    Cross,
}

impl JoinType {
    /// Get the SQL keyword
    /// 获取 SQL 关键字
    pub fn as_sql(&self) -> &str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
    }
}

/// Query builder
/// 查询构建器
///
/// Provides a fluent interface for building database queries.
/// 提供用于构建数据库查询的流畅接口。
///
/// # Safety / 安全性
///
/// Condition values passed to `where_()` are escaped via [`ToSql`] before
/// insertion into the generated SQL. For string parameters single-quotes
/// are escaped (`'` → `''`). Nevertheless, **do not concatenate untrusted
/// user input into condition strings** — use the `?` placeholder pattern
/// and pass values separately through the params slice.
///
/// Column names in `order_by()`, `select()`, `group_by()` are interpolated
/// directly — they should come from trusted sources only (column enums or
/// the [`ModelMeta`](crate::ModelMeta), not raw user input).
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_orm::QueryBuilder;
///
/// let query = User::query()
///     .where_("age > ?", &["18"])
///     .order_by("created_at DESC")
///     .limit(10);
///
/// let sql = query.to_sql();
/// // SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 10
/// ```
pub struct QueryBuilder<M: Model> {
    /// Model type
    /// 模型类型
    _phantom: PhantomData<M>,

    /// Where clauses
    /// WHERE 子句
    wheres: Vec<WhereClause>,

    /// Order by clauses
    /// ORDER BY 子句
    order_by: Vec<OrderBy>,

    /// Limit and offset
    /// LIMIT 和 OFFSET
    limit: Limit,

    /// Joins
    /// JOIN
    joins: Vec<Join>,

    /// Selected columns (empty means *)
    /// 选择的列（空表示 *）
    select: Vec<String>,

    /// Group by columns
    /// GROUP BY 列
    group_by: Vec<String>,

    /// Having clause
    /// HAVING 子句
    having: Option<String>,

    /// Distinct flag
    /// DISTINCT 标志
    distinct: bool,
}

impl<M: Model> QueryBuilder<M> {
    /// Create a new query builder
    /// 创建新的查询构建器
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            wheres: Vec::new(),
            order_by: Vec::new(),
            limit: Limit::default(),
            joins: Vec::new(),
            select: Vec::new(),
            group_by: Vec::new(),
            having: None,
            distinct: false,
        }
    }

    /// Add a where clause
    /// 添加 WHERE 子句
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let users = User::query()
    ///     .where_("age > ?", &[QueryParam::I32(18)])
    ///     .where_("status = ?", &[QueryParam::Text("active".into())])
    ///     .all().await?;
    /// ```
    // SAFETY: `condition` is a raw SQL fragment that gets interpolated directly.
    // Callers MUST use the `?` placeholder pattern for values and never concatenate
    // untrusted user input into the condition string.
    #[must_use = "QueryBuilder is consumed by each method; chain calls or use the final result"]
    pub fn where_(mut self, condition: &str, params: &[QueryParam]) -> Self {
        self.wheres.push(WhereClause {
            condition: condition.to_string(),
            params: params.to_vec(),
        });
        self
    }

    /// Add an order by clause
    /// 添加 ORDER BY 子句
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let users = User::query()
    ///     .order_by("created_at DESC")
    ///     .all().await?;
    /// ```
    pub fn order_by(self, column: &str) -> OrderByBuilder<M> {
        assert!(
            validate_identifier(column),
            "order_by column must contain only alphanumeric characters and underscores, got: {column}"
        );
        OrderByBuilder {
            query_builder: self,
            column: column.to_string(),
        }
    }

    /// Set the limit
    /// 设置 LIMIT
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let users = User::query()
    ///     .limit(10)
    ///     .all().await?;
    /// ```
    #[must_use]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit.limit = Some(limit);
        self
    }

    /// Set the offset
    /// 设置 OFFSET
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let users = User::query()
    ///     .limit(10)
    ///     .offset(20)
    ///     .all().await?;
    /// ```
    #[must_use]
    pub fn offset(mut self, offset: usize) -> Self {
        self.limit.offset = Some(offset);
        self
    }

    /// Add a join
    /// 添加 JOIN
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let users = User::query()
    ///     .join(JoinType::Inner, "posts", "users.id = posts.user_id")
    ///     .all().await?;
    /// ```
    #[must_use]
    pub fn join(mut self, join_type: JoinType, table: &str, on: &str) -> Self {
        assert!(
            validate_identifier(table),
            "join table must contain only alphanumeric characters and underscores, got: {table}"
        );
        self.joins.push(Join::new(join_type, table, on));
        self
    }

    /// Select specific columns
    /// 选择特定列
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let users = User::query()
    ///     .select(&["id", "name"])
    ///     .all().await?;
    /// ```
    #[must_use]
    pub fn select(mut self, columns: &[&str]) -> Self {
        for col in columns {
            assert!(
                validate_identifier(col),
                "select column must contain only alphanumeric characters and underscores, got: {col}"
            );
        }
        self.select = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a group by clause
    /// 添加 GROUP BY 子句
    #[must_use]
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        for col in columns {
            assert!(
                validate_identifier(col),
                "group_by column must contain only alphanumeric characters and underscores, got: {col}"
            );
        }
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a having clause
    /// 添加 HAVING 子句
    #[must_use]
    pub fn having(mut self, condition: &str) -> Self {
        assert!(
            validate_identifier(condition),
            "having column must contain only alphanumeric characters and underscores, got: {condition}"
        );
        self.having = Some(condition.to_string());
        self
    }

    /// Set distinct flag
    /// 设置 DISTINCT 标志
    #[must_use]
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Build parameterized SQL query
    /// 构建参数化 SQL 查询
    ///
    /// Returns `(sql, params)` where `sql` uses `$1, $2, ...` placeholders
    /// and `params` contains the corresponding parameter values.
    pub fn build(&self) -> (String, Vec<QueryParam>) {
        let mut sql = String::new();
        let mut params = Vec::new();
        let mut param_idx = 1u32;

        // SELECT clause
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }
        if self.select.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.select.join(", "));
        }

        // FROM clause
        sql.push_str(" FROM ");
        sql.push_str(&M::table_name());

        // JOINs
        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join.join_type.as_sql());
            sql.push(' ');
            sql.push_str(&join.table);
            if let Some(alias) = &join.alias {
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
            sql.push_str(" ON ");
            sql.push_str(&join.on);
        }

        // WHERE clause — replace ? with $N
        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .wheres
                .iter()
                .map(|w| {
                    let mut condition = w.condition.clone();
                    for param in &w.params {
                        condition = condition.replacen('?', &format!("${param_idx}"), 1);
                        param_idx += 1;
                    }
                    condition
                })
                .collect();
            params.extend(self.wheres.iter().flat_map(|w| w.params.iter().cloned()));
            sql.push_str(&conditions.join(" AND "));
        }

        // GROUP BY clause
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }

        // HAVING clause
        if let Some(having) = &self.having {
            sql.push_str(" HAVING ");
            sql.push_str(having);
        }

        // ORDER BY clause
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let orderings: Vec<String> = self
                .order_by
                .iter()
                .map(|o| format!("{} {}", o.column, o.direction.as_sql()))
                .collect();
            sql.push_str(&orderings.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.limit.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }

    /// Build SQL with inline values (backward compatible)
    /// 构建带内联值的 SQL（向后兼容）
    pub fn to_sql(&self) -> String {
        let (sql, params) = self.build();
        let mut result = sql;
        for (i, param) in params.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, &param.to_sql_literal());
        }
        result
    }

    /// Execute the query and return all results.
    /// 执行查询并返回所有结果。
    pub async fn all<C: DatabaseClient>(&self, client: &C) -> Result<Vec<M>>
    where
        M: serde::de::DeserializeOwned,
    {
        let (sql, params) = self.build();
        let rows = client
            .fetch_all_params(&sql, &params)
            .await
            .map_err(|e| Error::query_build(format!("Query failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            results.push(
                row.deserialize()
                    .map_err(|e| Error::query_build(format!("Row deserialization failed: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Execute the query and return the first result, if any.
    /// 执行查询并返回第一个结果（如果存在）。
    pub async fn first<C: DatabaseClient>(&self, client: &C) -> Result<Option<M>>
    where
        M: serde::de::DeserializeOwned,
    {
        let (sql, params) = self.build();
        let row = client
            .fetch_one_params(&sql, &params)
            .await
            .map_err(|e| Error::query_build(format!("Query failed: {e}")))?;
        match row {
            Some(r) => r
                .deserialize()
                .map(Some)
                .map_err(|e| Error::query_build(format!("Row deserialization failed: {e}"))),
            None => Ok(None),
        }
    }

    /// Execute a COUNT query and return the count.
    /// 执行 COUNT 查询并返回计数。
    pub async fn count<C: DatabaseClient>(&self, client: &C) -> Result<i64> {
        let (_, where_params) = self.build();

        let mut count_sql = String::from("SELECT COUNT(*) AS cnt FROM ");
        count_sql.push_str(&M::table_name());

        if !self.wheres.is_empty() {
            count_sql.push_str(" WHERE ");
            let mut param_idx = 1u32;
            let conditions: Vec<String> = self
                .wheres
                .iter()
                .map(|w| {
                    let mut condition = w.condition.clone();
                    for _ in &w.params {
                        condition = condition.replacen('?', &format!("${param_idx}"), 1);
                        param_idx += 1;
                    }
                    condition
                })
                .collect();
            count_sql.push_str(&conditions.join(" AND "));
        }

        let rows = client
            .fetch_all_params(&count_sql, &where_params)
            .await
            .map_err(|e| Error::query_build(format!("Count query failed: {e}")))?;
        let cnt = rows
            .first()
            .and_then(|r| r.get_as::<i64>("cnt").ok())
            .unwrap_or(0);
        Ok(cnt)
    }

    /// Execute the query with pagination and return a page of results.
    /// 执行分页查询并返回一页结果。
    pub async fn paginate<C: DatabaseClient>(
        &self,
        client: &C,
        page: u32,
        per_page: u32,
    ) -> Result<nexus_data_commons::Page<M>>
    where
        M: serde::de::DeserializeOwned,
    {
        let total = self.count(client).await?;
        let offset = ((page.max(1) - 1) * per_page) as usize;
        let (base_sql, params) = self.build();
        let sql = if base_sql.contains("LIMIT") {
            base_sql
        } else {
            format!("{} LIMIT {} OFFSET {}", base_sql, per_page, offset)
        };

        let rows = client
            .fetch_all_params(&sql, &params)
            .await
            .map_err(|e| Error::query_build(format!("Pagination query failed: {e}")))?;
        let records: Vec<M> = rows
            .iter()
            .map(|r| {
                r.deserialize()
                    .map_err(|e| Error::query_build(format!("Row deserialization failed: {e}")))
            })
            .collect::<Result<Vec<M>>>()?;

        Ok(nexus_data_commons::Page::new(
            records,
            page,
            per_page,
            total as u64,
        ))
    }

}

impl<M: Model> Default for QueryBuilder<M> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for order by clause
/// ORDER BY 子句的构建器
pub struct OrderByBuilder<M: Model> {
    query_builder: QueryBuilder<M>,
    column: String,
}

impl<M: Model> OrderByBuilder<M> {
    /// Set direction to ascending and return the query builder
    /// 设置方向为升序并返回查询构建器
    pub fn asc(self) -> QueryBuilder<M> {
        let mut builder = self.query_builder;
        builder.order_by.push(OrderBy {
            column: self.column,
            direction: OrderDirection::Asc,
        });
        builder
    }

    /// Set direction to descending and return the query builder
    /// 设置方向为降序并返回查询构建器
    pub fn desc(self) -> QueryBuilder<M> {
        let mut builder = self.query_builder;
        builder.order_by.push(OrderBy {
            column: self.column,
            direction: OrderDirection::Desc,
        });
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock model for testing
    #[derive(Debug, Clone)]
    struct User;

    impl Model for User {
        fn meta() -> crate::ModelMeta {
            let mut meta = crate::ModelMeta::new("users");
            meta.columns.push(crate::Column::new("id", crate::ColumnType::I64));
            meta.columns.push(crate::Column::new("name", crate::ColumnType::String));
            meta.columns.push(crate::Column::new("email", crate::ColumnType::String));
            meta
        }

        fn primary_key(&self) -> Result<String> {
            Ok("1".to_string())
        }

        fn set_primary_key(&mut self, _value: String) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_query_builder_basic() {
        let query = QueryBuilder::<User>::new()
            .where_("age > ?", &[QueryParam::I32(18)])
            .to_sql();

        assert!(query.contains("SELECT * FROM users"));
        assert!(query.contains("WHERE"));
        assert!(query.contains("age > 18"));
    }

    #[test]
    fn test_query_builder_build_parameterized() {
        let (sql, params) = QueryBuilder::<User>::new()
            .where_("age > ?", &[QueryParam::I32(18)])
            .where_("name = ?", &[QueryParam::Text("Alice".into())])
            .build();

        assert!(sql.contains("$1"));
        assert!(sql.contains("$2"));
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], QueryParam::I32(18));
        assert_eq!(params[1], QueryParam::Text("Alice".into()));
    }

    #[test]
    fn test_query_builder_order_by() {
        let query = QueryBuilder::<User>::new()
            .order_by("created_at")
            .desc()
            .to_sql();

        assert!(query.contains("ORDER BY"));
        assert!(query.contains("created_at DESC"));
    }

    #[test]
    fn test_query_builder_limit_offset() {
        let query = QueryBuilder::<User>::new()
            .limit(10)
            .offset(20)
            .to_sql();

        assert!(query.contains("LIMIT 10"));
        assert!(query.contains("OFFSET 20"));
    }

    #[test]
    fn test_query_builder_join() {
        let query = QueryBuilder::<User>::new()
            .join(JoinType::Inner, "posts", "users.id = posts.user_id")
            .to_sql();

        assert!(query.contains("INNER JOIN posts"));
        assert!(query.contains("ON users.id = posts.user_id"));
    }

    #[test]
    fn test_to_sql_for_various_types() {
        assert_eq!(QueryParam::I32(42).to_sql_literal(), "42");
        assert_eq!(QueryParam::Text("hello".into()).to_sql_literal(), "'hello'");
        assert_eq!(QueryParam::Text("it's".into()).to_sql_literal(), "'it''s'");
        assert_eq!(QueryParam::Bool(true).to_sql_literal(), "TRUE");
        assert_eq!(QueryParam::Bool(false).to_sql_literal(), "FALSE");
    }

    #[test]
    fn test_build_sql_injection_prevention() {
        let malicious = "'; DROP TABLE users; --";
        let (sql, params) = QueryBuilder::<User>::new()
            .where_("name = ?", &[QueryParam::Text(malicious.into())])
            .build();

        // SQL uses $1 placeholder, not inline value
        assert!(sql.contains("$1"));
        assert!(!sql.contains("DROP TABLE"));
        // The malicious content is only in params, never in SQL
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], QueryParam::Text(malicious.into()));
    }
}
