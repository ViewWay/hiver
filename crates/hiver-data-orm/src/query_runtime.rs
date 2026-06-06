//! Query runtime execution engine (re-exports from split modules)
//! 查询运行时执行引擎（从拆分模块重新导出）
//!
//! # Deprecation Notice / 弃用通知
//!
//! This module is deprecated. Import directly from the split modules:
//! 本模块已弃用。请直接从拆分模块导入：
//!
//! - `query_metadata` → `QueryMetadata`, `ParamStyle`, `QueryType`
//! - `mapper` → `RowMapper`, `ResultSetExtractor`, `BeanRowMapper`, …
//! - `executor` → `QueryExecutor`, `AnnotatedQueryExecutor`

#[deprecated(
    since = "0.1.0-alpha.6",
    note = "Import directly from query_metadata, mapper, or executor modules instead"
)]
// Re-export from split modules for backward compatibility
// 从拆分模块重新导出以保持向后兼容
pub use crate::{
    executor::{AnnotatedQueryExecutor, QueryExecutor},
    mapper::{
        BeanRowMapper, FirstRowExtractor, MappingResultSetExtractor, ResultSetExtractor, RowMapper,
    },
    query_metadata::{ParamStyle, QueryMetadata, QueryType},
};
