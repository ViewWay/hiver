//! Row mapper trait for entity mapping
//! 行映射器 trait，用于实体映射
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `RowMapper<T>` (Spring JDBC)
//! - `ResultSetExtractor<T>` (Spring JDBC)
//! - `BeanPropertyRowMapper` (Spring JDBC)

use hiver_data_rdbc::{Error as R2dbcError, Result as R2dbcResult, Row};

/// Trait for mapping a database row to an entity.
/// 将数据库行映射到实体的 trait。
///
/// Equivalent to Spring JDBC's `RowMapper<T>`.
/// 等价于 Spring JDBC 的 `RowMapper<T>`。
pub trait RowMapper<T> {
    /// Map a single row to an entity.
    /// 将单行映射到实体。
    fn map_row(&self, row: &Row) -> R2dbcResult<T>;
}

/// Trait for extracting results from an entire result set.
/// 从整个结果集提取结果的 trait。
///
/// Equivalent to Spring JDBC's `ResultSetExtractor<T>`.
/// 等价于 Spring JDBC 的 `ResultSetExtractor<T>`。
pub trait ResultSetExtractor<T> {
    /// Extract a result from the entire result set.
    /// 从整个结果集提取结果。
    fn extract(&self, rows: &[Row]) -> R2dbcResult<T>;
}

/// A generic `RowMapper` that deserializes using serde.
/// 使用 serde 反序列化的通用 `RowMapper`。
///
/// Equivalent to Spring's `BeanPropertyRowMapper`.
/// 等价于 Spring 的 `BeanPropertyRowMapper`。
pub struct BeanRowMapper<T>(std::marker::PhantomData<T>);

impl<T> BeanRowMapper<T> {
    /// Create a new bean row mapper.
    /// 创建新的 bean 行映射器。
    pub fn new() -> Self { Self(std::marker::PhantomData) }
}

impl<T> Default for BeanRowMapper<T> {
    fn default() -> Self { Self::new() }
}

impl<T: serde::de::DeserializeOwned> RowMapper<T> for BeanRowMapper<T> {
    fn map_row(&self, row: &Row) -> R2dbcResult<T> {
        row.deserialize::<T>()
            .map_err(|e| R2dbcError::RowMapping(format!("BeanRowMapper failed: {}", e)))
    }
}

/// A `ResultSetExtractor` that maps all rows using a `RowMapper`.
/// 使用 `RowMapper` 映射所有行的 `ResultSetExtractor`。
pub struct MappingResultSetExtractor<'a, M, T> { mapper: &'a M, _p: std::marker::PhantomData<T> }

impl<'a, M, T> MappingResultSetExtractor<'a, M, T> {
    /// Create new.
    /// 创建新实例。
    pub fn new(mapper: &'a M) -> Self { Self { mapper, _p: std::marker::PhantomData } }
}

impl<'a, M: RowMapper<T>, T> ResultSetExtractor<Vec<T>> for MappingResultSetExtractor<'a, M, T> {
    fn extract(&self, rows: &[Row]) -> R2dbcResult<Vec<T>> {
        rows.iter().map(|r| self.mapper.map_row(r)).collect()
    }
}

/// A `ResultSetExtractor` that maps the first row.
/// 映射第一行的 `ResultSetExtractor`。
pub struct FirstRowExtractor<'a, M, T> { mapper: &'a M, _p: std::marker::PhantomData<T> }

impl<'a, M, T> FirstRowExtractor<'a, M, T> {
    /// Create new.
    /// 创建新实例。
    pub fn new(mapper: &'a M) -> Self { Self { mapper, _p: std::marker::PhantomData } }
}

impl<'a, M: RowMapper<T>, T> ResultSetExtractor<Option<T>> for FirstRowExtractor<'a, M, T> {
    fn extract(&self, rows: &[Row]) -> R2dbcResult<Option<T>> {
        match rows.first() {
            Some(r) => Ok(Some(self.mapper.map_row(r)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_bean_mapper_new() { let _: BeanRowMapper<String> = BeanRowMapper::new(); }
    #[test] fn test_mapping_extractor() { let m = BeanRowMapper::<String>::new(); let _: MappingResultSetExtractor<'_, BeanRowMapper<String>, String> = MappingResultSetExtractor::new(&m); }
    #[test] fn test_first_row_extractor_empty() {
        let m = BeanRowMapper::<String>::new();
        let e = FirstRowExtractor::new(&m);
        assert!(e.extract(&[]).unwrap().is_none());
    }
}
