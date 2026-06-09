//! Model trait and metadata
//! Model trait 和元数据
//!
//! # Overview / 概述
//!
//! This module provides the Model trait and related metadata for ORM operations.
//! 本模块提供 Model trait 和相关的 ORM 操作元数据。

use crate::{Error, Result};

/// Column type
/// 列类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Boolean type
    Bool,
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 128-bit signed integer
    I128,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// String type
    String,
    /// Text type (long string)
    Text,
    /// Bytes type
    Bytes,
    /// UUID type
    Uuid,
    /// Date type
    Date,
    /// Time type
    Time,
    /// Timestamp type
    Timestamp,
    /// JSON type
    Json,
    /// Decimal type
    Decimal,
    /// Enum type
    Enum,
    /// Array type
    Array,
    /// Custom type
    Custom(String),
}

impl ColumnType {
    /// Get the SQL type name for a given database
    pub fn as_sql(&self, dialect: SqlDialect) -> &str {
        match (self, dialect) {
            (ColumnType::Bool, _) => "BOOLEAN",
            (ColumnType::I32, SqlDialect::PostgreSQL) => "INTEGER",
            (ColumnType::I32, SqlDialect::MySQL) => "INT",
            (ColumnType::I64, SqlDialect::PostgreSQL) => "BIGINT",
            (ColumnType::I64, SqlDialect::MySQL) => "BIGINT",
            (ColumnType::String, SqlDialect::PostgreSQL) => "VARCHAR",
            (ColumnType::String, SqlDialect::MySQL) => "VARCHAR",
            (ColumnType::Text, _) => "TEXT",
            (ColumnType::Json, SqlDialect::PostgreSQL) => "JSONB",
            (ColumnType::Json, _) => "JSON",
            _ => "TEXT", // Default fallback
        }
    }
}

/// SQL dialect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    /// PostgreSQL dialect
    PostgreSQL,
    /// MySQL dialect
    MySQL,
    /// SQLite dialect
    SQLite,
}

/// Column metadata
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Column type
    pub type_: ColumnType,
    /// Whether this is a primary key
    pub is_primary_key: bool,
    /// Whether this column is nullable
    pub is_nullable: bool,
    /// Whether this column has a unique constraint
    pub is_unique: bool,
    /// Default value expression
    pub default: Option<String>,
    /// Maximum length for string types
    pub max_length: Option<usize>,
}

impl Column {
    /// Create a new column definition
    pub fn new(name: impl Into<String>, type_: ColumnType) -> Self {
        Self {
            name: name.into(),
            type_,
            is_primary_key: false,
            is_nullable: false,
            is_unique: false,
            default: None,
            max_length: None,
        }
    }

    /// Mark this column as a primary key
    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self
    }

    /// Mark this column as nullable
    pub fn nullable(mut self) -> Self {
        self.is_nullable = true;
        self
    }

    /// Mark this column as unique
    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    /// Set the default value for this column
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// Model metadata
#[derive(Debug, Clone)]
pub struct ModelMeta {
    /// Table name
    pub table_name: String,
    /// Column definitions
    pub columns: Vec<Column>,
}

impl ModelMeta {
    /// Create new model metadata with the given table name
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            columns: Vec::new(),
        }
    }

    /// Add a column to the model metadata
    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Get the table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}

/// Model trait
/// Model trait
pub trait Model: Send + Sync {
    /// Get the model metadata
    fn meta() -> ModelMeta;

    /// Get the table name
    fn table_name() -> String
    where
        Self: Sized,
    {
        Self::meta().table_name().to_string()
    }

    /// Get the primary key value (placeholder)
    fn primary_key(&self) -> Result<String> {
        Err(Error::unknown("Primary key not implemented"))
    }

    /// Set the primary key value (placeholder)
    fn set_primary_key(&mut self, _value: String) -> Result<()> {
        Err(Error::unknown("Set primary key not implemented"))
    }

    /// Validate the model
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Get all column names in declaration order.
    /// 获取声明顺序的所有列名。
    ///
    /// Returns a static slice of column name strings.
    /// 返回列名字符串的静态切片。
    fn column_names() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &[]
    }

    /// Get the number of columns.
    /// 获取列数量。
    fn column_count() -> usize
    where
        Self: Sized,
    {
        Self::column_names().len()
    }

    /// Construct the model from a database row.
    /// 从数据库行构造模型实例。
    ///
    /// Uses `Row::get_as` to extract typed values by column name.
    /// 使用 `Row::get_as` 按列名提取类型化值。
    fn from_row(row: &hiver_data_rdbc::Row) -> Result<Self>
    where
        Self: Sized,
    {
        let _ = row;
        Err(Error::unknown("from_row not implemented"))
    }

    /// Convert the model to column-value pairs for INSERT/UPDATE.
    /// 将模型转换为列-值对，用于 INSERT/UPDATE。
    ///
    /// Returns a vector of (column_name, boxed value) pairs.
    /// 返回 (列名, 装箱值) 对的向量。
    fn to_row(&self) -> Vec<(&'static str, Box<dyn std::any::Any + Send>)>
    where
        Self: Sized,
    {
        Vec::new()
    }

    /// Get the relationship definitions for this model.
    /// 获取此模型的关系定义。
    ///
    /// Override this method to declare cascading relationships.
    /// 重写此方法以声明级联关系。
    fn relations() -> Vec<crate::Relation>
    where
        Self: Sized,
    {
        Vec::new()
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
    fn test_column_type_sql() {
        assert_eq!(ColumnType::I32.as_sql(SqlDialect::PostgreSQL), "INTEGER");
        assert_eq!(ColumnType::String.as_sql(SqlDialect::PostgreSQL), "VARCHAR");
    }

    #[test]
    fn test_model_meta() {
        let meta =
            ModelMeta::new("users").add_column(Column::new("id", ColumnType::I64).primary_key());

        assert_eq!(meta.table_name(), "users");
        assert_eq!(meta.columns.len(), 1);
    }
}
