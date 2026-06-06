//! Diesel Bridge — integration with Diesel for schema DSL and QueryDSL
//! Diesel 桥接 — 与 Diesel 集成以支持 schema DSL 和 QueryDSL
//!
//! # Overview / 概述
//!
//! Provides a bridge between Hiver Data ORM and Diesel ORM.
//! Follows Diesel's schema definition and query building conventions
//! while delegating actual execution to the Hiver `DatabaseClient`.
//! 遵循 Diesel 的 schema 定义和查询构建约定，
//! 同时将实际执行委托给 Hiver `DatabaseClient`。
//!
//! Requires the `diesel` feature flag.
//! 需要 `diesel` 特性标志。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::diesel::{DieselSchema, DieselQuery, DieselColumnType};
//!
//! // Schema DSL — define your table
//! let schema = DieselSchema::new("users")
//!     .column("id", DieselColumnType::BigInt, true)  // primary key
//!     .column("name", DieselColumnType::Text, false)
//!     .column("email", DieselColumnType::Text, false);
//!
//! let create_sql = schema.to_create_sql();
//!
//! // QueryDSL — build type-safe queries
//! let users = DieselQuery::<User>::new()
//!     .filter("active = ?")
//!     .order("created_at", OrderDirection::Desc)
//!     .limit(10)
//!     .load(&client).await?;
//! ```

use std::marker::PhantomData;

use hiver_data_rdbc::DatabaseClient;

use crate::{Model, Result};

// ── Schema DSL ──

/// Diesel-compatible column type.
/// Diesel 兼容的列类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DieselColumnType
{
    /// 32-bit integer
    Integer,
    /// 64-bit integer (BIGINT)
    BigInt,
    /// Variable-length text (VARCHAR / TEXT)
    Text,
    /// Boolean (TRUE / FALSE)
    Boolean,
    /// 32-bit floating point
    Float,
    /// 64-bit floating point (DOUBLE PRECISION)
    Double,
    /// Date-time with timezone (TIMESTAMP WITH TIME ZONE)
    Timestamp,
    /// Binary data (BYTEA / BLOB)
    Bytes,
    /// UUID type
    Uuid,
}

impl DieselColumnType
{
    /// Convert to SQL type string.
    /// 转换为 SQL 类型字符串。
    pub fn to_sql(&self) -> &'static str
    {
        match self
        {
            Self::Integer => "INTEGER",
            Self::BigInt => "BIGINT",
            Self::Text => "TEXT",
            Self::Boolean => "BOOLEAN",
            Self::Float => "REAL",
            Self::Double => "DOUBLE PRECISION",
            Self::Timestamp => "TIMESTAMP WITH TIME ZONE",
            Self::Bytes => "BYTEA",
            Self::Uuid => "UUID",
        }
    }
}

/// A single column in a Diesel-style schema definition.
/// Diesel 风格 schema 定义中的单个列。
#[derive(Debug, Clone)]
pub struct DieselColumn
{
    /// Column name / 列名
    pub name: String,
    /// Column data type / 列数据类型
    pub type_: DieselColumnType,
    /// Whether this column is a primary key / 是否为主键
    pub is_primary_key: bool,
    /// Whether this column allows NULL / 是否允许 NULL
    pub is_nullable: bool,
    /// Whether this column has a UNIQUE constraint / 是否具有 UNIQUE 约束
    pub is_unique: bool,
}

impl DieselColumn
{
    /// Create a new column definition.
    /// 创建新的列定义。
    pub fn new(name: impl Into<String>, type_: DieselColumnType) -> Self
    {
        Self {
            name: name.into(),
            type_,
            is_primary_key: false,
            is_nullable: false,
            is_unique: false,
        }
    }

    /// Mark this column as the primary key.
    /// 将此列标记为主键。
    #[must_use]
    pub fn primary_key(mut self) -> Self
    {
        self.is_primary_key = true;
        self
    }

    /// Allow NULL values for this column.
    /// 允许此列为 NULL。
    #[must_use]
    pub fn nullable(mut self) -> Self
    {
        self.is_nullable = true;
        self
    }

    /// Add a UNIQUE constraint to this column.
    /// 为此列添加 UNIQUE 约束。
    #[must_use]
    pub fn unique(mut self) -> Self
    {
        self.is_unique = true;
        self
    }

    /// Generate the SQL fragment for this column in a CREATE TABLE statement.
    fn to_create_column_sql(&self) -> String
    {
        let mut sql = format!("{} {}", self.name, self.type_.to_sql());
        if !self.is_nullable
        {
            sql.push_str(" NOT NULL");
        }
        if self.is_unique
        {
            sql.push_str(" UNIQUE");
        }
        sql
    }
}

/// Diesel-style schema builder.
/// Diesel 风格的 schema 构建器。
///
/// Equivalent to Diesel's `table!` macro.
/// 等价于 Diesel 的 `table!` 宏。
#[derive(Debug, Clone)]
pub struct DieselSchema
{
    table_name: String,
    columns: Vec<DieselColumn>,
}

impl DieselSchema
{
    /// Create a new schema definition for a table.
    /// 为表创建新的 schema 定义。
    pub fn new(table_name: impl Into<String>) -> Self
    {
        Self {
            table_name: table_name.into(),
            columns: Vec::new(),
        }
    }

    /// Add a column to the schema.
    /// 向 schema 中添加列。
    #[must_use]
    pub fn column(
        mut self,
        name: impl Into<String>,
        type_: DieselColumnType,
        primary_key: bool,
    ) -> Self
    {
        let col = if primary_key
        {
            DieselColumn::new(name, type_).primary_key()
        }
        else
        {
            DieselColumn::new(name, type_)
        };
        self.columns.push(col);
        self
    }

    /// Add a column with full configuration.
    /// 添加一个完整配置的列。
    #[must_use]
    pub fn column_full(mut self, column: DieselColumn) -> Self
    {
        self.columns.push(column);
        self
    }

    /// Generate the CREATE TABLE SQL statement.
    /// 生成 CREATE TABLE SQL 语句。
    pub fn to_create_sql(&self) -> String
    {
        let col_defs: Vec<String> = self
            .columns
            .iter()
            .map(|c| c.to_create_column_sql())
            .collect();

        let pk_columns: Vec<&str> = self
            .columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .collect();

        let mut sql =
            format!("CREATE TABLE IF NOT EXISTS {} ({}", self.table_name, col_defs.join(", "));

        if !pk_columns.is_empty()
        {
            sql.push_str(&format!(", PRIMARY KEY ({})", pk_columns.join(", ")));
        }

        sql.push(')');
        sql
    }

    /// Generate the DROP TABLE SQL statement.
    /// 生成 DROP TABLE SQL 语句。
    pub fn to_drop_sql(&self) -> String
    {
        format!("DROP TABLE IF EXISTS {}", self.table_name)
    }

    /// Get the table name.
    /// 获取表名。
    pub fn table_name(&self) -> &str
    {
        &self.table_name
    }

    /// Get the column definitions.
    /// 获取列定义。
    pub fn columns(&self) -> &[DieselColumn]
    {
        &self.columns
    }
}

// ── QueryDSL ──

/// Sort direction for ORDER BY.
/// ORDER BY 的排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection
{
    /// Ascending / 升序
    Asc,
    /// Descending / 降序
    Desc,
}

impl OrderDirection
{
    fn to_sql(&self) -> &'static str
    {
        match self
        {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Diesel-style query builder.
/// Diesel 风格的查询构建器。
///
/// Equivalent to Diesel's `QueryDsl` — builds a type-safe query
/// that is executed via the Hiver `DatabaseClient`.
/// 等价于 Diesel 的 `QueryDsl` — 构建通过 Hiver`DatabaseClient` 执行的类型安全查询。
#[derive(Debug, Clone)]
pub struct DieselQuery<M>
{
    _phantom: PhantomData<M>,
    table: String,
    filters: Vec<String>,
    orders: Vec<String>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
}

impl<M: Model + serde::de::DeserializeOwned> DieselQuery<M>
{
    /// Create a new query against the model's table.
    /// 创建针对模型表的新查询。
    pub fn new() -> Self
    {
        Self {
            _phantom: PhantomData,
            table: M::table_name().to_string(),
            filters: Vec::new(),
            orders: Vec::new(),
            limit_val: None,
            offset_val: None,
        }
    }

    /// Create a query against a specific table.
    /// 针对特定表创建查询。
    pub fn from_table(table: impl Into<String>) -> Self
    {
        Self {
            _phantom: PhantomData,
            table: table.into(),
            filters: Vec::new(),
            orders: Vec::new(),
            limit_val: None,
            offset_val: None,
        }
    }

    /// Add a WHERE condition.
    /// 添加 WHERE 条件。
    ///
    /// Conditions should use `?` placeholders for parameter binding
    /// when using the SQLx-style execution path.
    #[must_use]
    pub fn filter(mut self, condition: impl Into<String>) -> Self
    {
        self.filters.push(condition.into());
        self
    }

    /// Add an optional filter — only if `Some`.
    /// 添加可选过滤 — 仅在 `Some` 时。
    #[must_use]
    pub fn filter_optional(self, condition: Option<impl Into<String>>) -> Self
    {
        match condition
        {
            Some(cond) => self.filter(cond),
            None => self,
        }
    }

    /// Add an ORDER BY clause.
    /// 添加 ORDER BY 子句。
    #[must_use]
    pub fn order(mut self, column: impl Into<String>, direction: OrderDirection) -> Self
    {
        self.orders
            .push(format!("{} {}", column.into(), direction.to_sql()));
        self
    }

    /// Set the LIMIT.
    /// 设置 LIMIT。
    #[must_use]
    pub fn limit(mut self, n: usize) -> Self
    {
        self.limit_val = Some(n);
        self
    }

    /// Set the OFFSET.
    /// 设置 OFFSET。
    #[must_use]
    pub fn offset(mut self, n: usize) -> Self
    {
        self.offset_val = Some(n);
        self
    }

    /// Build the SELECT SQL statement from the current query.
    /// 从当前查询构建 SELECT SQL 语句。
    pub fn to_sql(&self) -> String
    {
        let mut sql = format!("SELECT * FROM {}", self.table);

        if !self.filters.is_empty()
        {
            sql.push_str(" WHERE ");
            sql.push_str(&self.filters.join(" AND "));
        }

        if !self.orders.is_empty()
        {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.orders.join(", "));
        }

        if let Some(limit) = self.limit_val
        {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        if let Some(offset) = self.offset_val
        {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        sql
    }

    /// Execute the query and load all matching records.
    /// 执行查询并加载所有匹配记录。
    ///
    /// Delegates to `DatabaseClient::fetch_all`.
    pub async fn load<C: DatabaseClient>(self, client: &C) -> Result<Vec<M>>
    {
        let sql = self.to_sql();
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Diesel query load failed: {e}")))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in &rows
        {
            let model: M = row
                .deserialize()
                .map_err(|e| crate::Error::validation(format!("Diesel deserialize: {e}")))?;
            results.push(model);
        }
        Ok(results)
    }

    /// Execute the query and return the first matching record.
    /// 执行查询并返回第一个匹配记录。
    pub async fn first<C: DatabaseClient>(self, client: &C) -> Result<Option<M>>
    {
        let sql = format!("{} LIMIT 1", self.to_sql());
        let row = client
            .fetch_one(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Diesel first failed: {e}")))?;

        match row
        {
            Some(row) =>
            {
                let model: M = row
                    .deserialize()
                    .map_err(|e| crate::Error::validation(format!("Diesel deserialize: {e}")))?;
                Ok(Some(model))
            },
            None => Ok(None),
        }
    }

    /// Execute a count query.
    /// 执行计数查询。
    pub async fn count<C: DatabaseClient>(self, client: &C) -> Result<i64>
    {
        let mut sql = format!("SELECT COUNT(*) AS cnt FROM {}", self.table);

        if !self.filters.is_empty()
        {
            sql.push_str(" WHERE ");
            sql.push_str(&self.filters.join(" AND "));
        }

        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Diesel count failed: {e}")))?;

        let count = rows
            .first()
            .and_then(|r| r.get_as::<i64>("cnt").ok())
            .unwrap_or(0);

        Ok(count)
    }
}

impl<M: Model + serde::de::DeserializeOwned> Default for DieselQuery<M>
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::model::*;

    /// A simple test model
    #[derive(Debug, Clone)]
    struct User;

    impl Model for User
    {
        fn meta() -> ModelMeta
        {
            let mut meta = ModelMeta::new("users");
            meta.columns.push(Column::new("id", ColumnType::I64));
            meta.columns.push(Column::new("name", ColumnType::String));
            meta
        }

        fn primary_key(&self) -> crate::Result<String>
        {
            Ok("id".to_string())
        }

        fn set_primary_key(&mut self, _: String) -> crate::Result<()>
        {
            Ok(())
        }
    }

    impl<'de> serde::Deserialize<'de> for User
    {
        fn deserialize<D: serde::Deserializer<'de>>(_d: D) -> std::result::Result<Self, D::Error>
        {
            Ok(User)
        }
    }

    #[test]
    fn test_diesel_schema_builder()
    {
        let schema = DieselSchema::new("users")
            .column("id", DieselColumnType::BigInt, true)
            .column("name", DieselColumnType::Text, false)
            .column("email", DieselColumnType::Text, false);

        let sql = schema.to_create_sql();
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(sql.contains("id BIGINT NOT NULL"));
        assert!(sql.contains("name TEXT NOT NULL"));
        assert!(sql.contains("PRIMARY KEY (id)"));
    }

    #[test]
    fn test_diesel_schema_drop_sql()
    {
        let schema = DieselSchema::new("products");
        assert_eq!(schema.to_drop_sql(), "DROP TABLE IF EXISTS products");
    }

    #[test]
    fn test_diesel_column_nullable()
    {
        let col = DieselColumn::new("description", DieselColumnType::Text).nullable();
        assert!(col.is_nullable);
        assert!(col.to_create_column_sql().contains("TEXT"));
        assert!(!col.to_create_column_sql().contains("NOT NULL"));
    }

    #[test]
    fn test_diesel_column_unique()
    {
        let col = DieselColumn::new("email", DieselColumnType::Text).unique();
        assert!(col.is_unique);
        assert!(col.to_create_column_sql().contains("UNIQUE"));
    }

    #[test]
    fn test_diesel_column_type_to_sql()
    {
        assert_eq!(DieselColumnType::Integer.to_sql(), "INTEGER");
        assert_eq!(DieselColumnType::BigInt.to_sql(), "BIGINT");
        assert_eq!(DieselColumnType::Text.to_sql(), "TEXT");
        assert_eq!(DieselColumnType::Boolean.to_sql(), "BOOLEAN");
        assert_eq!(DieselColumnType::Double.to_sql(), "DOUBLE PRECISION");
        assert_eq!(DieselColumnType::Uuid.to_sql(), "UUID");
    }

    #[test]
    fn test_diesel_query_to_sql()
    {
        let sql = DieselQuery::<User>::new()
            .filter("active = true")
            .order("created_at", OrderDirection::Desc)
            .limit(10)
            .offset(20)
            .to_sql();

        assert!(sql.contains("SELECT * FROM users"));
        assert!(sql.contains("WHERE active = true"));
        assert!(sql.contains("ORDER BY created_at DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
    }

    #[test]
    fn test_diesel_query_multiple_filters()
    {
        let sql = DieselQuery::<User>::new()
            .filter("active = true")
            .filter("age > 18")
            .to_sql();

        assert!(sql.contains("active = true AND age > 18"));
    }

    #[test]
    fn test_diesel_query_filter_optional_some()
    {
        let sql = DieselQuery::<User>::new()
            .filter_optional(Some("name LIKE '%test%'"))
            .to_sql();

        assert!(sql.contains("WHERE name LIKE '%test%'"));
    }

    #[test]
    fn test_diesel_query_filter_optional_none()
    {
        let sql = DieselQuery::<User>::new()
            .filter_optional(None::<&str>)
            .to_sql();

        assert!(!sql.contains("WHERE"));
    }

    #[test]
    fn test_diesel_query_default()
    {
        let q: DieselQuery<User> = DieselQuery::default();
        let sql = q.to_sql();
        assert_eq!(sql, "SELECT * FROM users");
    }
}
