//! Query runtime execution engine (re-exports from split modules)
//! 查询运行时执行引擎（从拆分模块重新导出）
//!
//! # Overview / 概述
//!
//! This module re-exports types from the split modules (`query_metadata`, `mapper`, `executor`)
//! for backward compatibility. New code should import directly from the split modules.
//!
//! 本模块从拆分模块（`query_metadata`、`mapper`、`executor`）重新导出类型，
//! 以保持向后兼容。新代码应直接从拆分模块导入。

// Re-export from split modules for backward compatibility
// 从拆分模块重新导出以保持向后兼容
pub use crate::executor::{AnnotatedQueryExecutor, QueryExecutor};
pub use crate::mapper::{
    BeanRowMapper, FirstRowExtractor, MappingResultSetExtractor, ResultSetExtractor, RowMapper,
};
pub use crate::query_metadata::{ParamStyle, QueryMetadata, QueryType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_query_type_select() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type(), QueryType::Select);
    }

    #[test]
    fn test_detect_query_type_insert() {
        let sql = "INSERT INTO users (name) VALUES (:name)";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type(), QueryType::Insert);
    }

    #[test]
    fn test_detect_query_type_update() {
        let sql = "UPDATE users SET name = :name WHERE id = :id";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type(), QueryType::Update);
    }

    #[test]
    fn test_detect_query_type_delete() {
        let sql = "DELETE FROM users WHERE id = :id";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type(), QueryType::Delete);
    }

    #[test]
    fn test_extract_named_params() {
        let sql = "SELECT * FROM users WHERE id = :id AND name = :name";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.param_names(), &["id", "name"]);
    }

    #[test]
    fn test_extract_mybatis_params() {
        let sql = "SELECT * FROM users WHERE id = #{id} AND name = #{name}";
        let metadata = QueryMetadata::new(sql, ParamStyle::MyBatis);
        assert_eq!(metadata.param_names(), &["id", "name"]);
    }

    #[test]
    fn test_bind_named_params() {
        use std::collections::HashMap;

        let metadata = QueryMetadata::new(
            "SELECT * FROM users WHERE id = :id AND name = :name",
            ParamStyle::Named,
        );

        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(1));
        params.insert("name".to_string(), serde_json::json!("Alice"));

        let (sql, values) = metadata.bind_params(&params).unwrap();

        assert!(sql.contains("$1"));
        assert!(sql.contains("$2"));
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], hiver_data_rdbc::QueryParam::I64(1));
        assert_eq!(values[1], hiver_data_rdbc::QueryParam::Text("Alice".into()));
    }

    #[test]
    fn test_bind_mybatis_params() {
        use std::collections::HashMap;

        let metadata =
            QueryMetadata::new("SELECT * FROM users WHERE id = #{id}", ParamStyle::MyBatis);

        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(1));

        let (sql, values) = metadata.bind_params(&params).unwrap();

        assert!(sql.contains("$1"));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], hiver_data_rdbc::QueryParam::I64(1));
    }

    #[test]
    fn test_missing_param_error() {
        use std::collections::HashMap;

        let metadata = QueryMetadata::new(
            "SELECT * FROM users WHERE id = :id AND name = :name",
            ParamStyle::Named,
        );

        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(1));
        // Missing "name" parameter

        let result = metadata.bind_params(&params);
        assert!(result.is_err());
    }
}
