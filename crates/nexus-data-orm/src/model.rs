//! Model trait and metadata
//! Model trait 和元数据
//!
//! # Overview / 概述
//!
//! This module provides the Model trait and related metadata for ORM operations.
//! 本模块提供 Model trait 和相关的 ORM 操作元数据。

use crate::{Error, Result};
use std::collections::HashMap;

/// Column type
/// 列类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Boolean type
    Bool,
    /// Integer types
    I8, I16, I32, I64, I128,
    /// Unsigned integer types
    U8, U16, U32, U64,
    /// Float types
    F32, F64,
    /// String type
    String,
    /// Text type (long string)
    Text,
    /// Bytes type
    Bytes,
    /// UUID type
    Uuid,
    /// Date/Time types
    Date, Time, Timestamp,
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
    PostgreSQL,
    MySQL,
    SQLite,
}

/// Column metadata
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: ColumnType,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub default: Option<String>,
    pub max_length: Option<usize>,
}

impl Column {
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

    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.is_nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// Model metadata
#[derive(Debug, Clone)]
pub struct ModelMeta {
    pub table_name: String,
    pub columns: Vec<Column>,
}

impl ModelMeta {
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_type_sql() {
        assert_eq!(ColumnType::I32.as_sql(SqlDialect::PostgreSQL), "INTEGER");
        assert_eq!(ColumnType::String.as_sql(SqlDialect::PostgreSQL), "VARCHAR");
    }

    #[test]
    fn test_model_meta() {
        let meta = ModelMeta::new("users")
            .add_column(Column::new("id", ColumnType::I64).primary_key());

        assert_eq!(meta.table_name(), "users");
        assert_eq!(meta.columns.len(), 1);
    }
}
