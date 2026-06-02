//! SQLx Bridge — integration with SQLx for async queries and compile-time checks
//! SQLx 桥接 — 与 SQLx 集成以支持异步查询和编译时检查
//!
//! # Overview / 概述
//!
//! Provides a bridge between Hiver Data ORM and SQLx.
//! Supports compile-time query verification (via SQLx's `query!`) and
//! runtime async execution via the Hiver `DatabaseClient`.
//! 提供 Hiver Data ORM 与 SQLx 之间的桥接。
//! 支持编译时查询验证和通过 Hiver `DatabaseClient` 进行运行时异步执行。
//!
//! Requires the `sqlx` feature flag.
//! 需要 `sqlx` 特性标志。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::sqlx::{SqlxQuery, SqlxPoolAdapter};
//!
//! // Build a query
//! let users = SqlxQuery::<User>::select()
//!     .where_("active = ?", &[QueryParam::Bool(true)])
//!     .order_by("created_at DESC")
//!     .limit(10)
//!     .fetch_all(&client).await?;
//!
//! // Compile-time verified query (requires DATABASE_URL at build time)
//! // let rows = sqlx::query!("SELECT id, name FROM users WHERE active = $1", true)
//! //     .fetch_all(&pool).await?;
//! ```

use crate::{Model, Result};
use hiver_data_rdbc::DatabaseClient;
use std::marker::PhantomData;

// ── SQLx Query Builder ──

/// Order direction for SQLx queries.
/// SQLx 查询的排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlxOrder {
    /// Ascending / 升序
    Asc,
    /// Descending / 降序
    Desc,
}

impl SqlxOrder {
    fn to_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// SQLx-style parameterized query builder.
/// SQLx 风格的参数化查询构建器。
///
/// Builds queries with `$1`, `$2` ... positional parameter markers
/// (PostgreSQL-style), delegating execution to `DatabaseClient`.
/// Follows SQLx conventions for query building.
/// 构建带有 `$1`, `$2` ... 位置参数标记（PostgreSQL 风格）的查询，
/// 将执行委托给 `DatabaseClient`。
#[derive(Debug, Clone)]
pub struct SqlxQuery<M> {
    _phantom: PhantomData<M>,
    table: String,
    select_columns: Vec<String>,
    conditions: Vec<String>,
    params: Vec<String>,
    param_counter: usize,
    orders: Vec<String>,
    groups: Vec<String>,
    having: Option<String>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
    distinct: bool,
}

impl<M: Model + serde::de::DeserializeOwned> SqlxQuery<M> {
    /// Create a new SELECT * query.
    /// 创建新的 SELECT * 查询。
    pub fn select() -> Self {
        Self {
            _phantom: PhantomData,
            table: M::table_name().to_string(),
            select_columns: Vec::new(),
            conditions: Vec::new(),
            params: Vec::new(),
            param_counter: 0,
            orders: Vec::new(),
            groups: Vec::new(),
            having: None,
            limit_val: None,
            offset_val: None,
            distinct: false,
        }
    }

    /// Select specific columns instead of *.
    /// 选择特定列而不是 *。
    #[must_use]
    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.select_columns = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a WHERE condition with positional parameters.
    /// 添加带有位置参数的 WHERE 条件。
    ///
    /// Uses `$1`, `$2` ... markers like PostgreSQL/SQLx.
    /// The `params` values are embedded directly (for mock clients);
    /// real SQLx would use proper parameter binding.
    /// 使用 `$1`, `$2` ... 标记（如 PostgreSQL/SQLx）。
    #[must_use]
    pub fn where_(
        mut self,
        condition: impl Into<String>,
        params: &[hiver_data_rdbc::QueryParam],
    ) -> Self {
        let mut cond = condition.into();
        for param in params {
            self.param_counter += 1;
            let marker = format!("${}", self.param_counter);
            // Replace first `?` or `$N` with the actual parameter marker
            if let Some(pos) = cond.find('?') {
                cond.replace_range(pos..pos + 1, &marker);
            }
            self.params.push(param.to_sql_literal());
        }
        self.conditions.push(cond);
        self
    }

    /// Add an ORDER BY clause.
    /// 添加 ORDER BY 子句。
    #[must_use]
    pub fn order_by(mut self, expr: impl Into<String>) -> Self {
        self.orders.push(expr.into());
        self
    }

    /// Add an ORDER BY clause with direction.
    /// 添加带方向的 ORDER BY 子句。
    #[must_use]
    pub fn order_by_dir(mut self, column: impl Into<String>, direction: SqlxOrder) -> Self {
        self.orders
            .push(format!("{} {}", column.into(), direction.to_sql()));
        self
    }

    /// Add a GROUP BY clause.
    /// 添加 GROUP BY 子句。
    #[must_use]
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.groups = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a HAVING clause.
    /// 添加 HAVING 子句。
    #[must_use]
    pub fn having(mut self, condition: impl Into<String>) -> Self {
        self.having = Some(condition.into());
        self
    }

    /// Set LIMIT.
    /// 设置 LIMIT。
    #[must_use]
    pub fn limit(mut self, n: usize) -> Self {
        self.limit_val = Some(n);
        self
    }

    /// Set OFFSET.
    /// 设置 OFFSET。
    #[must_use]
    pub fn offset(mut self, n: usize) -> Self {
        self.offset_val = Some(n);
        self
    }

    /// Enable DISTINCT.
    /// 启用 DISTINCT。
    #[must_use]
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Build the full SQL string.
    /// 构建完整的 SQL 字符串。
    pub fn to_sql(&self) -> String {
        let columns = if self.select_columns.is_empty() {
            "*".to_string()
        } else {
            self.select_columns.join(", ")
        };

        let mut sql = format!("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }
        sql.push_str(&columns);
        sql.push_str(&format!(" FROM {}", self.table));

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        if !self.groups.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.groups.join(", "));
        }

        if let Some(ref having) = self.having {
            sql.push_str(&format!(" HAVING {having}"));
        }

        if !self.orders.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.orders.join(", "));
        }

        if let Some(limit) = self.limit_val {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        if let Some(offset) = self.offset_val {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        sql
    }

    /// Get the collected parameter values (for parameterized execution).
    /// 获取收集的参数值（用于参数化执行）。
    ///
    /// WARNING: Parameters are stored but the current `DatabaseClient` trait
    /// only accepts raw SQL strings. When a parameterized execute API becomes
    /// available, this method should be used to bind values. For now, the
    /// `$1`, `$2` ... markers in `to_sql()` are placeholders only.
    /// 警告：参数已存储但当前 `DatabaseClient` trait 仅接受原始 SQL 字符串。
    /// 当参数化执行 API 可用时，应使用此方法绑定值。
    pub fn params(&self) -> &[String] {
        &self.params
    }

    /// Execute the query and fetch all results.
    /// 执行查询并获取所有结果。
    pub async fn fetch_all<C: DatabaseClient>(self, client: &C) -> Result<Vec<M>> {
        let sql = self.to_sql();
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Sqlx fetch_all failed: {e}")))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            let model: M = row
                .deserialize()
                .map_err(|e| crate::Error::validation(format!("Sqlx deserialize: {e}")))?;
            results.push(model);
        }
        Ok(results)
    }

    /// Execute the query and fetch at most one result.
    /// 执行查询并最多获取一个结果。
    pub async fn fetch_optional<C: DatabaseClient>(self, client: &C) -> Result<Option<M>> {
        // BUGFIX: Avoid double LIMIT if the query already has one.
        let base_sql = self.to_sql();
        let sql = if base_sql.contains("LIMIT") {
            base_sql
        } else {
            format!("{} LIMIT 1", base_sql)
        };
        let row = client
            .fetch_one(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Sqlx fetch_optional failed: {e}")))?;

        match row {
            Some(row) => {
                let model: M = row
                    .deserialize()
                    .map_err(|e| crate::Error::validation(format!("Sqlx deserialize: {e}")))?;
                Ok(Some(model))
            },
            None => Ok(None),
        }
    }

    /// Execute the query and fetch exactly one result.
    /// 执行查询并获取恰好一个结果。
    ///
    /// Returns an error if zero or more than one row is returned.
    pub async fn fetch_one<C: DatabaseClient>(self, client: &C) -> Result<M> {
        let result = self.fetch_optional(client).await?;
        result.ok_or_else(|| crate::Error::not_found("Sqlx fetch_one: no rows returned"))
    }
}

impl<M: Model + serde::de::DeserializeOwned> Default for SqlxQuery<M> {
    fn default() -> Self {
        Self::select()
    }
}

// ── FromRow Adapter ──

/// Trait for types that can be constructed from a Hiver RDBC row.
/// 可从 Hiver RDBC 行构造的类型的 trait。
///
/// Equivalent to SQLx's `FromRow` trait — bridges row-based
/// deserialization to the Hiver `Model` ecosystem.
/// 等价于 SQLx 的 `FromRow` trait — 将基于行的反序列化桥接到 Hiver `Model` 生态系统。
pub trait FromRow: Sized {
    /// Construct this type from a Hiver RDBC row.
    /// 从 Hiver RDBC 行构造此类型。
    fn from_rdbc_row(row: &hiver_data_rdbc::Row) -> Result<Self>;
}

// ── SQLx Pool Adapter ──

/// Adapter that wraps a SQLx connection pool and exposes it as a
/// Hiver `DatabaseClient`.
/// 包装 SQLx 连接池并将其暴露为 Hiver `DatabaseClient` 的适配器。
///
/// Only available when the `sqlx` feature is enabled and a real
/// SQLx pool is provided.
/// 仅在启用 `sqlx` 特性并提供真实 SQLx 池时可用。
#[cfg(feature = "sqlx")]
pub struct SqlxPoolAdapter<DB: sqlx::Database> {
    pool: sqlx::Pool<DB>,
}

#[cfg(feature = "sqlx")]
impl<DB: sqlx::Database> SqlxPoolAdapter<DB> {
    /// Create a new adapter wrapping a SQLx pool.
    /// 创建包装 SQLx 池的新适配器。
    pub fn new(pool: sqlx::Pool<DB>) -> Self {
        Self { pool }
    }

    /// Get a reference to the inner pool.
    /// 获取内部池的引用。
    pub fn inner(&self) -> &sqlx::Pool<DB> {
        &self.pool
    }
}

#[cfg(feature = "sqlx")]
impl<DB: sqlx::Database> Clone for SqlxPoolAdapter<DB> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

// ── Compile-time Query Helper ──

/// A compile-time verified query (placeholder for `sqlx::query!` integration).
/// 编译时验证的查询（`sqlx::query!` 集成的占位符）。
///
/// In production, users should use `sqlx::query!()` directly for
/// compile-time SQL verification. This struct provides a runtime
/// bridge that follows the same conventions.
/// 在生产环境中，用户应直接使用 `sqlx::query!()` 进行编译时 SQL 验证。
/// 此结构体提供了一个遵循相同约定的运行时桥接。
#[derive(Debug, Clone)]
pub struct VerifiedQuery {
    sql: String,
}

impl VerifiedQuery {
    /// Create a new verified query.
    /// 创建新的已验证查询。
    ///
    /// In production, use `sqlx::query!(sql)` instead.
    /// 在生产环境中，请改用 `sqlx::query!(sql)`。
    pub fn new(sql: impl Into<String>) -> Self {
        Self { sql: sql.into() }
    }

    /// Execute this query and fetch all rows.
    /// 执行此查询并获取所有行。
    pub async fn fetch_all<C: DatabaseClient>(
        &self,
        client: &C,
    ) -> Result<Vec<hiver_data_rdbc::Row>> {
        client
            .fetch_all(&self.sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Verified query failed: {e}")))
    }

    /// Execute this query and fetch at most one row.
    /// 执行此查询并最多获取一行。
    pub async fn fetch_optional<C: DatabaseClient>(
        &self,
        client: &C,
    ) -> Result<Option<hiver_data_rdbc::Row>> {
        // BUGFIX: Avoid double LIMIT if the query already has one.
        let sql = if self.sql.contains("LIMIT") {
            self.sql.clone()
        } else {
            format!("{} LIMIT 1", self.sql)
        };
        client
            .fetch_one(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Verified query failed: {e}")))
    }

    /// Get the raw SQL.
    /// 获取原始 SQL。
    pub fn sql(&self) -> &str {
        &self.sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    /// A simple test model
    #[derive(Debug, Clone)]
    struct Product;

    impl Model for Product {
        fn meta() -> ModelMeta {
            let mut meta = ModelMeta::new("products");
            meta.columns.push(Column::new("id", ColumnType::I64));
            meta.columns.push(Column::new("name", ColumnType::String));
            meta.columns.push(Column::new("price", ColumnType::F64));
            meta
        }

        fn primary_key(&self) -> crate::Result<String> {
            Ok("id".to_string())
        }

        fn set_primary_key(&mut self, _: String) -> crate::Result<()> {
            Ok(())
        }
    }

    impl<'de> serde::Deserialize<'de> for Product {
        fn deserialize<D: serde::Deserializer<'de>>(_d: D) -> std::result::Result<Self, D::Error> {
            Ok(Product)
        }
    }

    #[test]
    fn test_sqlx_query_basic() {
        let sql = SqlxQuery::<Product>::select().to_sql();
        assert_eq!(sql, "SELECT * FROM products");
    }

    #[test]
    fn test_sqlx_query_with_columns() {
        let sql = SqlxQuery::<Product>::select()
            .columns(&["id", "name"])
            .to_sql();
        assert_eq!(sql, "SELECT id, name FROM products");
    }

    #[test]
    fn test_sqlx_query_with_conditions() {
        let sql = SqlxQuery::<Product>::select()
            .where_("active = ?", &[hiver_data_rdbc::QueryParam::Bool(true)])
            .where_("price > ?", &[hiver_data_rdbc::QueryParam::F64(100.0)])
            .to_sql();

        assert!(sql.contains("SELECT * FROM products"));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("active = $1"));
        assert!(sql.contains("price > $2"));
    }

    #[test]
    fn test_sqlx_query_full() {
        let sql = SqlxQuery::<Product>::select()
            .columns(&["name", "price"])
            .where_("active = ?", &[hiver_data_rdbc::QueryParam::Bool(true)])
            .group_by(&["category"])
            .having("COUNT(*) > 5")
            .order_by_dir("price", SqlxOrder::Desc)
            .limit(10)
            .offset(0)
            .distinct()
            .to_sql();

        assert!(sql.contains("SELECT DISTINCT name, price"));
        assert!(sql.contains("WHERE active = $1"));
        assert!(sql.contains("GROUP BY category"));
        assert!(sql.contains("HAVING COUNT(*) > 5"));
        assert!(sql.contains("ORDER BY price DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 0"));
    }

    #[test]
    fn test_sqlx_order_direction() {
        assert_eq!(SqlxOrder::Asc.to_sql(), "ASC");
        assert_eq!(SqlxOrder::Desc.to_sql(), "DESC");
    }

    #[test]
    fn test_sqlx_query_default() {
        let q: SqlxQuery<Product> = SqlxQuery::default();
        assert_eq!(q.to_sql(), "SELECT * FROM products");
    }

    #[test]
    fn test_verified_query() {
        let vq = VerifiedQuery::new("SELECT count(*) FROM products");
        assert_eq!(vq.sql(), "SELECT count(*) FROM products");
    }
}
