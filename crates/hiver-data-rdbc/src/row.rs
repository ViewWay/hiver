//! Row and result types
//! 行和结果类型
//!
//! # Overview / 概述
//!
//! Types for representing database rows and results.
//! 表示数据库行和结果的类型。

use crate::error::Error;

/// Database row — map of column name to value
/// 数据库行 — 列名到值的映射
#[derive(Debug, Clone)]
pub struct Row
{
    columns: Vec<(String, ColumnValue)>,
}

impl Row
{
    /// Create a new empty row
    pub fn new() -> Self
    {
        Self {
            columns: Vec::new(),
        }
    }

    /// Add a column value
    pub fn with_column(mut self, name: impl Into<String>, value: ColumnValue) -> Self
    {
        self.columns.push((name.into(), value));
        self
    }

    /// Get a column value by name
    pub fn get(&self, name: &str) -> Option<&ColumnValue>
    {
        self.columns
            .iter()
            .find(|(col, _)| col == name)
            .map(|(_, v)| v)
    }

    /// Require a column value by name
    pub fn require(&self, name: &str) -> Result<&ColumnValue, Error>
    {
        self.get(name)
            .ok_or_else(|| Error::RowMapping(format!("column '{}' not found", name)))
    }

    /// Get as a specific type (with column name)
    pub fn get_as<T: FromRowValue>(&self, name: &str) -> Result<T, Error>
    {
        self.require(name)?
            .as_type()
            .ok_or_else(|| Error::RowMapping(format!("cannot convert column '{}'", name)))
    }

    /// Try to get a value, returning None if not found
    pub fn try_get<T: FromRowValue>(&self, name: &str) -> Result<Option<T>, Error>
    {
        match self.get(name)
        {
            Some(v) => v
                .as_type()
                .map(Some)
                .ok_or_else(|| Error::RowMapping(format!("cannot convert column '{}'", name))),
            None => Ok(None),
        }
    }

    /// Iterate over all columns
    pub fn columns(&self) -> impl Iterator<Item = &(String, ColumnValue)>
    {
        self.columns.iter()
    }

    /// Number of columns
    pub fn len(&self) -> usize
    {
        self.columns.len()
    }

    /// Whether the row is empty
    pub fn is_empty(&self) -> bool
    {
        self.columns.is_empty()
    }

    /// Create a row from a list of (name, value) pairs
    pub fn from_pairs(pairs: Vec<(impl Into<String>, ColumnValue)>) -> Self
    {
        Self {
            columns: pairs.into_iter().map(|(n, v)| (n.into(), v)).collect(),
        }
    }

    /// Deserialize this row into a type using serde JSON as intermediate
    /// 通过 serde JSON 作为中间格式将行反序列化为目标类型
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, Error>
    {
        let map: serde_json::Map<String, serde_json::Value> = self
            .columns
            .iter()
            .map(|(name, value)| (name.clone(), value.to_json_value()))
            .collect();
        serde_json::from_value(serde_json::Value::Object(map))
            .map_err(|e| Error::Deserialization(format!("row deserialization failed: {}", e)))
    }
}

impl Default for Row
{
    fn default() -> Self
    {
        Self::new()
    }
}

// Allow iterating rows directly as &Row → usable by RowMapper
impl AsRef<Row> for Row
{
    fn as_ref(&self) -> &Row
    {
        self
    }
}

/// Column value
/// 列值
#[derive(Debug, Clone)]
pub enum ColumnValue
{
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// 8-bit integer
    I8(i8),
    /// 16-bit integer
    I16(i16),
    /// 32-bit integer
    I32(i32),
    /// 64-bit integer
    I64(i64),
    /// 32-bit float
    F32(f32),
    /// 64-bit float
    F64(f64),
    /// String value
    String(String),
    /// Bytes value
    Bytes(Vec<u8>),
    /// UUID value
    Uuid(uuid::Uuid),
    /// NaiveDateTime
    NaiveDateTime(chrono::NaiveDateTime),
}

impl ColumnValue
{
    /// Check if value is null
    pub fn is_null(&self) -> bool
    {
        matches!(self, Self::Null)
    }

    /// Try to convert to a specific type
    pub fn as_type<T: FromRowValue>(&self) -> Option<T>
    {
        FromRowValue::from_column_value(self)
    }

    /// Convert to serde_json::Value for serialization
    pub fn to_json_value(&self) -> serde_json::Value
    {
        match self
        {
            Self::Null => serde_json::Value::Null,
            Self::Bool(v) => serde_json::json!(*v),
            Self::I8(v) => serde_json::json!(*v),
            Self::I16(v) => serde_json::json!(*v),
            Self::I32(v) => serde_json::json!(*v),
            Self::I64(v) => serde_json::json!(*v),
            Self::F32(v) => serde_json::json!(*v),
            Self::F64(v) => serde_json::json!(*v),
            Self::String(v) => serde_json::json!(v),
            Self::Bytes(_) => serde_json::Value::Null,
            Self::Uuid(v) => serde_json::json!(v.to_string()),
            Self::NaiveDateTime(v) => serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// Trait for types that can be extracted from a ColumnValue
/// 可从 ColumnValue 中提取的类型的 trait
pub trait FromRowValue: Sized
{
    /// Try to convert from ColumnValue
    fn from_column_value(val: &ColumnValue) -> Option<Self>;
}

impl FromRowValue for i32
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::I32(v) => Some(*v),
            ColumnValue::I8(v) => Some(*v as i32),
            ColumnValue::I16(v) => Some(*v as i32),
            _ => None,
        }
    }
}

impl FromRowValue for i64
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::I64(v) => Some(*v),
            ColumnValue::I32(v) => Some(*v as i64),
            ColumnValue::I8(v) => Some(*v as i64),
            ColumnValue::I16(v) => Some(*v as i64),
            _ => None,
        }
    }
}

impl FromRowValue for f64
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::F64(v) => Some(*v),
            ColumnValue::F32(v) => Some(*v as f64),
            _ => None,
        }
    }
}

impl FromRowValue for String
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::String(v) => Some(v.clone()),
            ColumnValue::I32(v) => Some(v.to_string()),
            ColumnValue::I64(v) => Some(v.to_string()),
            _ => None,
        }
    }
}

impl FromRowValue for bool
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
}

impl FromRowValue for Vec<u8>
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::Bytes(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl FromRowValue for uuid::Uuid
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::Uuid(v) => Some(*v),
            ColumnValue::String(s) => uuid::Uuid::parse_str(s).ok(),
            _ => None,
        }
    }
}

impl FromRowValue for chrono::NaiveDateTime
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::NaiveDateTime(v) => Some(*v),
            _ => None,
        }
    }
}

/// `Option<T>` extracts as `Some(T)` when the column value is present and non-null,
/// or `None` when the column is `Null` or the inner conversion fails.
/// `Option<T>` 在列值存在且非空时提取为 `Some(T)`，在列为 `Null` 或内部转换失败时为 `None`。
impl<T: FromRowValue> FromRowValue for Option<T>
{
    fn from_column_value(val: &ColumnValue) -> Option<Self>
    {
        match val
        {
            ColumnValue::Null => Some(None),
            other => T::from_column_value(other).map(Some),
        }
    }
}

/// Column metadata
/// 列元数据
#[derive(Debug, Clone)]
pub struct Column
{
    /// Column name
    pub name: String,
    /// Column type
    pub type_: ColumnType,
    /// Whether the column is nullable
    pub nullable: bool,
}

/// Column type
/// 列类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ColumnType
{
    Bool,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
    Uuid,
    Timestamp,
    Date,
    Unknown,
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_row_building()
    {
        let row = Row::new()
            .with_column("id", ColumnValue::I64(1))
            .with_column("name", ColumnValue::String("Alice".into()));

        assert_eq!(row.len(), 2);
        assert_eq!(row.get_as::<i64>("id").unwrap(), 1);
        assert_eq!(row.get_as::<String>("name").unwrap(), "Alice");
    }

    #[test]
    fn test_column_value_null()
    {
        let value = ColumnValue::Null;
        assert!(value.is_null());
    }

    #[test]
    fn test_column_value_as_type()
    {
        assert_eq!(ColumnValue::I64(42).as_type::<i64>(), Some(42));
        assert_eq!(ColumnValue::F64(3.15).as_type::<f64>(), Some(3.15));
        assert_eq!(ColumnValue::Bool(true).as_type::<bool>(), Some(true));
        assert_eq!(ColumnValue::String("hello".into()).as_type::<String>(), Some("hello".into()));
    }

    #[test]
    fn test_row_deserialize()
    {
        use serde::Deserialize;

        #[derive(Debug, Deserialize, PartialEq)]
        struct User
        {
            id: i64,
            name: String,
        }

        let row = Row::from_pairs(vec![
            ("id", ColumnValue::I64(42)),
            ("name", ColumnValue::String("Bob".into())),
        ]);

        let user: User = row.deserialize().unwrap();
        assert_eq!(user, User {
            id: 42,
            name: "Bob".into()
        });
    }
}
