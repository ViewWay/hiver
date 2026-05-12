//! Query runtime execution engine
//! 查询运行时执行引擎
//!
//! # Overview / 概述
//!
//! This module provides runtime execution support for SQL queries defined in annotation macros.
//! 本模块提供注解宏中定义的 SQL 查询的运行时执行支持。
//!
//! # Features / 功能
//!
//! - Supports multiple parameter binding styles (:param, #{param}, $1, $2)
//!   支持多种参数绑定样式
//! - Automatic row-to-entity mapping
//!   自动将行映射到实体
//! - Query execution with type safety
//!   类型安全的查询执行

use crate::{
    client::DatabaseClient,
    error::{R2dbcError, R2dbcResult},
};
use serde::Deserialize;
use std::collections::HashMap;

/// Query metadata extracted from annotation macros
/// 从注解宏中提取的查询元数据
///
/// This struct stores the SQL query and parameter binding information
/// 这个结构体存储 SQL 查询和参数绑定信息
#[derive(Debug, Clone)]
pub struct QueryMetadata {
    /// The SQL query string / SQL 查询字符串
    pub sql: String,

    /// Parameter binding style / 参数绑定样式
    pub param_style: ParamStyle,

    /// Parameter names in order of appearance
    /// 参数名称（按出现顺序）
    pub param_names: Vec<String>,

    /// Query type / 查询类型
    pub query_type: QueryType,
}

/// Parameter binding style / 参数绑定样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamStyle {
    /// Named parameter with colon: :param
    /// 带冒号的命名参数：:param
    Named,

    /// MyBatis-Plus style: #{param}
    /// MyBatis-Plus 风格：#{param}
    MyBatis,

    /// Positional parameter: $1, $2
    /// 位置参数：$1, $2
    Positional,

    /// Question mark style: ?
    /// 问号样式：?
    QuestionMark,
}

/// Query type / 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// SELECT query / SELECT 查询
    Select,

    /// INSERT query / INSERT 查询
    Insert,

    /// UPDATE query / UPDATE 查询
    Update,

    /// DELETE query / DELETE 查询
    Delete,
}

impl QueryMetadata {
    /// Create new query metadata
    /// 创建新的查询元数据
    pub fn new(sql: impl Into<String>, param_style: ParamStyle) -> Self {
        let sql = sql.into();
        let query_type = Self::detect_query_type(&sql);
        let param_names = Self::extract_param_names(&sql, param_style);

        Self {
            sql,
            param_style,
            param_names,
            query_type,
        }
    }

    /// Detect query type from SQL string
    /// 从 SQL 字符串检测查询类型
    fn detect_query_type(sql: &str) -> QueryType {
        let sql_upper = sql.trim().to_uppercase();

        if sql_upper.starts_with("SELECT") {
            QueryType::Select
        } else if sql_upper.starts_with("INSERT") {
            QueryType::Insert
        } else if sql_upper.starts_with("UPDATE") {
            QueryType::Update
        } else if sql_upper.starts_with("DELETE") {
            QueryType::Delete
        } else {
            // Default to SELECT for unknown queries
            QueryType::Select
        }
    }

    /// Extract parameter names from SQL query
    /// 从 SQL 查询中提取参数名称
    fn extract_param_names(sql: &str, style: ParamStyle) -> Vec<String> {
        match style {
            ParamStyle::Named => {
                // Extract :param style
                // 提取 :param 样式
                let mut params = Vec::new();
                let mut chars = sql.chars().peekable();
                let mut current_param = String::new();

                while let Some(c) = chars.next() {
                    if c == ':' {
                        // Start of parameter
                        // 参数开始
                        while let Some(&next_c) = chars.peek() {
                            if next_c.is_alphanumeric() || next_c == '_' {
                                current_param.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        if !current_param.is_empty() {
                            params.push(current_param.clone());
                            current_param.clear();
                        }
                    }
                }

                params
            },
            ParamStyle::MyBatis => {
                // Extract #{param} style
                // 提取 #{param} 样式
                let mut params = Vec::new();
                let mut chars = sql.chars().peekable();
                let mut in_param = false;
                let mut current_param = String::new();

                while let Some(c) = chars.next() {
                    if c == '#' {
                        if let Some(&'{') = chars.peek() {
                            chars.next(); // consume '{'
                            in_param = true;
                        }
                    } else if c == '}' && in_param {
                        in_param = false;
                        if !current_param.is_empty() {
                            params.push(current_param.clone());
                            current_param.clear();
                        }
                    } else if in_param {
                        current_param.push(c);
                    }
                }

                params
            },
            ParamStyle::Positional => {
                // Extract $1, $2 style parameters
                // 提取 $1, $2 样式参数
                let mut params = Vec::new();
                let mut chars = sql.chars().peekable();
                let mut current_num = String::new();

                while let Some(c) = chars.next() {
                    if c == '$' {
                        while let Some(&next_c) = chars.peek() {
                            if next_c.is_numeric() {
                                if let Some(c) = chars.next() {
                                    current_num.push(c);
                                }
                            } else {
                                break;
                            }
                        }
                        if !current_num.is_empty() {
                            let param_num: usize = current_num.parse().unwrap_or(1);
                            let index = param_num.saturating_sub(1);
                            // Ensure vector is large enough
                            // 确保向量足够大
                            if params.len() <= index {
                                params.resize(index + 1, format!("param{}", index + 1));
                            }
                            current_num.clear();
                        }
                    }
                }

                params
            },
            ParamStyle::QuestionMark => {
                // Count ? placeholders
                // 统计 ? 占位符
                let count = sql.matches('?').count();
                (0..count).map(|i| format!("param{}", i + 1)).collect()
            },
        }
    }

    /// Bind parameters to the query
    /// 绑定参数到查询
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let metadata = QueryMetadata::new(
    ///     "SELECT * FROM users WHERE id = :id AND name = :name",
    ///     ParamStyle::Named
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("id".to_string(), serde_json::json!(1));
    /// params.insert("name".to_string(), serde_json::json!("Alice"));
    ///
    /// let (sql, values) = metadata.bind_params(&params)?;
    /// ```
    pub fn bind_params(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<(String, Vec<serde_json::Value>)> {
        let mut sql = self.sql.clone();
        let mut values = Vec::new();

        match self.param_style {
            ParamStyle::Named => {
                // Replace :param with $1, $2, etc.
                // 将 :param 替换为 $1, $2 等
                let mut param_index = 1;
                let mut offset = 0;

                for param_name in &self.param_names {
                    let placeholder = format!(":{}", param_name);

                    if let Some(pos) = sql[offset..].find(&placeholder) {
                        let replacement = format!("${}", param_index);
                        sql.replace_range(
                            offset + pos..offset + pos + placeholder.len(),
                            &replacement,
                        );

                        if let Some(value) = params.get(param_name) {
                            values.push(value.clone());
                        } else {
                            return Err(R2dbcError::sql(format!(
                                "Missing parameter: {}",
                                param_name
                            )));
                        }

                        param_index += 1;
                        offset += pos + replacement.len();
                    }
                }

                Ok((sql, values))
            },
            ParamStyle::MyBatis => {
                // Replace #{param} with $1, $2, etc.
                // 将 #{param} 替换为 $1, $2 等
                let _param_index = 1;

                for (param_index, param_name) in (1..).zip(self.param_names.iter()) {
                    let placeholder = format!("#{{{}}}", param_name);
                    let replacement = format!("${}", param_index);

                    sql = sql.replace(&placeholder, &replacement);

                    if let Some(value) = params.get(param_name) {
                        values.push(value.clone());
                    } else {
                        return Err(R2dbcError::sql(format!("Missing parameter: {}", param_name)));
                    }
                }

                Ok((sql, values))
            },
            ParamStyle::Positional => {
                // Already in positional format, just order the values
                // 已经是位置格式，只需排序值
                let mut ordered_values = Vec::new();

                for param_name in &self.param_names {
                    if let Some(value) = params.get(param_name) {
                        ordered_values.push(value.clone());
                    } else {
                        return Err(R2dbcError::sql(format!("Missing parameter: {}", param_name)));
                    }
                }

                Ok((sql, ordered_values))
            },
            ParamStyle::QuestionMark => {
                // Map params by order
                // 按顺序映射参数
                let mut ordered_values = Vec::new();

                for param_name in &self.param_names {
                    if let Some(value) = params.get(param_name) {
                        ordered_values.push(value.clone());
                    } else {
                        return Err(R2dbcError::sql(format!("Missing parameter: {}", param_name)));
                    }
                }

                Ok((sql, ordered_values))
            },
        }
    }
}

/// Query executor for annotated queries
/// 注解查询的执行器
///
/// Provides runtime execution support for queries defined in annotation macros.
/// 为注解宏中定义的查询提供运行时执行支持。
pub struct AnnotatedQueryExecutor<E>
where
    E: DatabaseClient,
{
    /// The underlying executor / 底层执行器
    executor: E,
}

impl<E> AnnotatedQueryExecutor<E>
where
    E: DatabaseClient,
{
    /// Create a new annotated query executor
    /// 创建新的注解查询执行器
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    /// Execute a query and return a single result
    /// 执行查询并返回单个结果
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use nexus_data_rdbc::query_runtime::{AnnotatedQueryExecutor, QueryMetadata, ParamStyle};
    ///
    /// let executor = AnnotatedQueryExecutor::new(db_executor);
    ///
    /// let metadata = QueryMetadata::new(
    ///     "SELECT * FROM users WHERE id = :id",
    ///     ParamStyle::Named
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("id".to_string(), serde_json::json!(1));
    ///
    /// let user: Option<User> = executor.fetch_one(&metadata, &params).await?;
    /// ```
    pub async fn fetch_one<T>(
        &self,
        metadata: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let (sql, _values) = metadata.bind_params(params)?;
        match self.executor.fetch_one(&sql).await? {
            Some(row) => {
                let entity = row.deserialize::<T>()
                    .map_err(|e| R2dbcError::RowMapping(format!("failed to map row: {}", e)))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    /// Execute a query and return all results
    /// 执行查询并返回所有结果
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let metadata = QueryMetadata::new(
    ///     "SELECT * FROM users WHERE age > :min_age",
    ///     ParamStyle::Named
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("min_age".to_string(), serde_json::json!(18));
    ///
    /// let users: Vec<User> = executor.fetch_all(&metadata, &params).await?;
    /// ```
    pub async fn fetch_all<T>(
        &self,
        metadata: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let (sql, _values) = metadata.bind_params(params)?;
        let rows = self.executor.fetch_all(&sql).await?;
        rows.into_iter()
            .map(|row| {
                row.deserialize::<T>()
                    .map_err(|e| R2dbcError::RowMapping(format!("failed to map row: {}", e)))
            })
            .collect()
    }

    /// Execute an INSERT/UPDATE/DELETE query
    /// 执行 INSERT/UPDATE/DELETE 查询
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let metadata = QueryMetadata::new(
    ///     "UPDATE users SET email = :email WHERE id = :id",
    ///     ParamStyle::Named
    /// );
    ///
    /// let mut params = HashMap::new();
    /// params.insert("email".to_string(), serde_json::json!("new@example.com"));
    /// params.insert("id".to_string(), serde_json::json!(1));
    ///
    /// let affected = executor.execute(&metadata, &params).await?;
    /// ```
    pub async fn execute(
        &self,
        metadata: &QueryMetadata,
        params: &HashMap<String, serde_json::Value>,
    ) -> R2dbcResult<u64> {
        let (sql, _values) = metadata.bind_params(params)?;

        self.executor.execute_cmd(&sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_query_type_select() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type, QueryType::Select);
    }

    #[test]
    fn test_detect_query_type_insert() {
        let sql = "INSERT INTO users (name) VALUES (:name)";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type, QueryType::Insert);
    }

    #[test]
    fn test_detect_query_type_update() {
        let sql = "UPDATE users SET name = :name WHERE id = :id";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type, QueryType::Update);
    }

    #[test]
    fn test_detect_query_type_delete() {
        let sql = "DELETE FROM users WHERE id = :id";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.query_type, QueryType::Delete);
    }

    #[test]
    fn test_extract_named_params() {
        let sql = "SELECT * FROM users WHERE id = :id AND name = :name";
        let metadata = QueryMetadata::new(sql, ParamStyle::Named);
        assert_eq!(metadata.param_names, vec!["id", "name"]);
    }

    #[test]
    fn test_extract_mybatis_params() {
        let sql = "SELECT * FROM users WHERE id = #{id} AND name = #{name}";
        let metadata = QueryMetadata::new(sql, ParamStyle::MyBatis);
        assert_eq!(metadata.param_names, vec!["id", "name"]);
    }

    #[test]
    fn test_bind_named_params() {
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
    }

    #[test]
    fn test_bind_mybatis_params() {
        let metadata =
            QueryMetadata::new("SELECT * FROM users WHERE id = #{id}", ParamStyle::MyBatis);

        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(1));

        let (sql, values) = metadata.bind_params(&params).unwrap();

        assert!(sql.contains("$1"));
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_missing_param_error() {
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
