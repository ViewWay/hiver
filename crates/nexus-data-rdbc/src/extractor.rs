//! ResultSet extraction traits
//! 结果集提取 trait
//!
//! # Overview / 概述
//!
//! This module provides Spring Data R2DBC-equivalent result extraction traits.
//! These traits enable transforming raw Row data into domain entities.
//!
//! 本模块提供 Spring Data R2DBC 等价的结果提取 trait。
//! 这些 trait 支持将原始 Row 数据转换为领域实体。
//!
//! # Traits / Trait
//!
//! | Nexus | Spring Data R2DBC |
//! |-------|-------------------|
//! | `RowMapper` | `RowMapper<T>` |
//! | `ResultSetExtractor` | `ResultExtractor<T>` |

use crate::error::Error as CrateError;

/// Row Mapper trait
/// Row Mapper trait
///
/// Maps a single database row to a domain entity.
/// Equivalent to Spring's `RowMapper<T>`.
///
/// 将单个数据库行映射到领域实体。
/// 等价于 Spring 的 `RowMapper<T>`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_rdbc::RowMapper;
/// use nexus_data_rdbc::error::Error;
///
/// #[derive(Debug, Clone)]
/// struct User {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// // Row type from the database backend
/// type DbRow = (); // placeholder
///
/// struct UserRowMapper;
///
/// impl RowMapper<DbRow, User> for UserRowMapper {
///     fn map_row(&self, _row: &DbRow, _row_num: usize) -> Result<User, Error> {
///         // Map your row here
///         todo!()
///     }
/// }
/// ```
pub trait RowMapper<R, T>: Send + Sync {
    /// Map a single row to an entity
    /// 将单行映射为实体
    ///
    /// # Parameters / 参数
    ///
    /// - `row`: The database row to map / 要映射的数据库行
    /// - `row_num`: The row number (0-indexed) / 行号（从0开始）
    ///
    /// # Returns / 返回
    ///
    /// The mapped entity / 映射后的实体
    fn map_row(&self, row: &R, row_num: usize) -> Result<T, CrateError>;
}

/// ResultSet Extractor trait
/// 结果集提取器 trait
///
/// Extracts a complete result from a set of database rows.
/// Equivalent to Spring's `ResultSetExtractor<T>`.
///
/// 从数据库行集合中提取完整结果。
/// 等价于 Spring 的 `ResultSetExtractor<T>`。
pub trait ResultSetExtractor<R, T>: Send + Sync {
    /// Extract data from a set of rows
    /// 从行集合中提取数据
    ///
    /// # Parameters / 参数
    ///
    /// - `rows`: The result rows to extract from / 要从中提取的结果行
    ///
    /// # Returns / 返回
    ///
    /// The extracted result / 提取的结果
    fn extract_data(&self, rows: &[R]) -> Result<T, CrateError>;
}

/// Rows trait — represents a set of database rows
/// Rows trait — 表示数据库行集合
///
/// Equivalent to Spring's `Result` (the query result, not Rust's Result).
/// 等价于 Spring 的 `Result`（查询结果，不是 Rust 的 Result）。
pub trait Rows: Send + Sync {
    /// The row type
    /// 行类型
    type Row;

    /// Get the number of rows
    /// 获取行数
    fn len(&self) -> usize;

    /// Check if the result set is empty
    /// 检查结果集是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a row by index
    /// 根据索引获取行
    fn get(&self, index: usize) -> Result<&Self::Row, CrateError>;

    /// Map rows using a RowMapper
    /// 使用 RowMapper 映射所有行
    fn map_all<T>(
        &self,
        mapper: &dyn RowMapper<Self::Row, T>,
    ) -> Result<Vec<T>, CrateError> {
        let mut results = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            let row = self.get(i)?;
            results.push(mapper.map_row(row, i)?);
        }
        Ok(results)
    }
}

/// Convenience: create a RowMapper from a closure
/// 便捷函数：从闭包创建 RowMapper
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_rdbc::{row_mapper, error::Error};
///
/// type DbRow = (); // placeholder
///
/// let mapper = row_mapper(|_row: &DbRow, _row_num: usize| -> Result<User, Error> {
///     // map your row
///     todo!()
/// });
/// ```
pub fn row_mapper<R, T, F>(f: F) -> impl RowMapper<R, T>
where
    F: Fn(&R, usize) -> Result<T, CrateError> + Send + Sync,
    R: Send + Sync,
    T: Send + Sync,
{
    struct ClosureRowMapper<R, T, F> {
        f: F,
        _phantom: std::marker::PhantomData<(R, T)>,
    }

    impl<R, T, F> RowMapper<R, T> for ClosureRowMapper<R, T, F>
    where
        F: Fn(&R, usize) -> Result<T, CrateError> + Send + Sync,
        R: Send + Sync,
        T: Send + Sync,
    {
        fn map_row(&self, row: &R, row_num: usize) -> Result<T, CrateError> {
            (self.f)(row, row_num)
        }
    }

    ClosureRowMapper {
        f,
        _phantom: std::marker::PhantomData,
    }
}
