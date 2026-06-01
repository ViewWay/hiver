//! Query metadata and parameter binding
//! 查询元数据和参数绑定
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `@Query("SELECT ... WHERE name = :name")` annotation metadata
//! - Spring Data JPA named parameter resolution
//! - MyBatis `#{param}` parameter parsing

use hiver_data_rdbc::QueryParam;
use hiver_data_rdbc::{Error as R2dbcError, Result as R2dbcResult};
use std::collections::HashMap;

/// Query metadata extracted from annotation macros.
/// 从注解宏中提取的查询元数据。
#[derive(Debug, Clone)]
pub struct QueryMetadata {
    sql: String,
    param_style: ParamStyle,
    param_names: Vec<String>,
    query_type: QueryType,
}

impl QueryMetadata {
    /// Create new query metadata.
    /// 创建新的查询元数据。
    pub fn new(sql: impl Into<String>, param_style: ParamStyle) -> Self {
        let sql = sql.into();
        let query_type = Self::detect_query_type(&sql);
        let param_names = Self::extract_param_names(&sql, param_style);

        Self { sql, param_style, param_names, query_type }
    }

    /// Returns the raw SQL string.
    /// 返回原始 SQL 字符串。
    pub fn sql(&self) -> &str { &self.sql }

    /// Returns the parameter binding style.
    /// 返回参数绑定样式。
    pub fn param_style(&self) -> ParamStyle { self.param_style }

    /// Returns the parameter names in order of appearance.
    /// 返回按出现顺序排列的参数名称。
    pub fn param_names(&self) -> &[String] { &self.param_names }

    /// Returns the detected query type.
    /// 返回检测到的查询类型。
    pub fn query_type(&self) -> QueryType { self.query_type }

    fn detect_query_type(sql: &str) -> QueryType {
        let sql_upper = sql.trim().to_uppercase();
        if sql_upper.starts_with("SELECT") { QueryType::Select }
        else if sql_upper.starts_with("INSERT") { QueryType::Insert }
        else if sql_upper.starts_with("UPDATE") { QueryType::Update }
        else if sql_upper.starts_with("DELETE") { QueryType::Delete }
        else { QueryType::Select }
    }

    fn extract_param_names(sql: &str, style: ParamStyle) -> Vec<String> {
        match style {
            ParamStyle::Named => Self::extract_named(sql),
            ParamStyle::MyBatis => Self::extract_mybatis(sql),
            ParamStyle::Positional => Self::extract_positional(sql),
            ParamStyle::QuestionMark => Self::extract_question(sql),
        }
    }

    fn extract_named(sql: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = sql.chars().peekable();
        let mut cur = String::new();
        while let Some(c) = chars.next() {
            if c == ':' {
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' { cur.push(chars.next().unwrap()); }
                    else { break; }
                }
                if !cur.is_empty() { params.push(cur.clone()); cur.clear(); }
            }
        }
        params
    }

    fn extract_mybatis(sql: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = sql.chars().peekable();
        let mut in_p = false;
        let mut cur = String::new();
        while let Some(c) = chars.next() {
            if c == '#' {
                if let Some(&'{') = chars.peek() { chars.next(); in_p = true; }
            } else if c == '}' && in_p {
                in_p = false;
                if !cur.is_empty() { params.push(cur.clone()); cur.clear(); }
            } else if in_p { cur.push(c); }
        }
        params
    }

    fn extract_positional(sql: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = sql.chars().peekable();
        let mut cur = String::new();
        while let Some(c) = chars.next() {
            if c == '$' {
                while let Some(&nc) = chars.peek() {
                    if nc.is_numeric() { cur.push(chars.next().unwrap()); }
                    else { break; }
                }
                if !cur.is_empty() {
                    let n: usize = cur.parse().unwrap_or(1);
                    let idx = n.saturating_sub(1);
                    if params.len() <= idx { params.resize(idx + 1, format!("param{}", idx + 1)); }
                    cur.clear();
                }
            }
        }
        params
    }

    fn extract_question(sql: &str) -> Vec<String> {
        (0..sql.matches('?').count()).map(|i| format!("param{}", i + 1)).collect()
    }

    /// Bind parameters to the query.
    /// 绑定参数到查询。
    pub fn bind_params(&self, params: &HashMap<String, serde_json::Value>) -> R2dbcResult<(String, Vec<QueryParam>)> {
        let mut sql = self.sql.clone();
        let mut values = Vec::new();
        match self.param_style {
            ParamStyle::Named => {
                let mut idx = 1; let mut off = 0;
                for name in &self.param_names {
                    let ph = format!(":{}", name);
                    if let Some(pos) = sql[off..].find(&ph) {
                        let rep = format!("${}", idx);
                        sql.replace_range(off + pos..off + pos + ph.len(), &rep);
                        values.push(params.get(name).cloned().map(QueryParam::from)
                            .ok_or_else(|| R2dbcError::sql(format!("Missing parameter: {}", name)))?);
                        idx += 1; off += pos + rep.len();
                    }
                }
                Ok((sql, values))
            }
            ParamStyle::MyBatis => {
                for (idx, name) in (1..).zip(&self.param_names) {
                    sql = sql.replace(&format!("#{{{}}}", name), &format!("${}", idx));
                    values.push(params.get(name).cloned().map(QueryParam::from)
                        .ok_or_else(|| R2dbcError::sql(format!("Missing parameter: {}", name)))?);
                }
                Ok((sql, values))
            }
            ParamStyle::Positional => {
                for name in &self.param_names {
                    values.push(params.get(name).cloned().map(QueryParam::from)
                        .ok_or_else(|| R2dbcError::sql(format!("Missing parameter: {}", name)))?);
                }
                Ok((sql, values))
            }
            ParamStyle::QuestionMark => {
                for (i, name) in self.param_names.iter().enumerate() {
                    values.push(params.get(name).cloned().map(QueryParam::from)
                        .ok_or_else(|| R2dbcError::sql(format!("Missing parameter: {}", name)))?);
                    sql = sql.replacen('?', &format!("${}", i + 1), 1);
                }
                Ok((sql, values))
            }
        }
    }
}

/// Parameter binding style.
/// 参数绑定样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamStyle {
    /// `:param` / 带冒号的命名参数
    Named,
    /// `#{param}` / MyBatis 风格
    MyBatis,
    /// `$1`, `$2` / 位置参数
    Positional,
    /// `?` / 问号样式
    QuestionMark,
}

/// Query type.
/// 查询类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType { Select, Insert, Update, Delete }

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_detect_select() {
        assert_eq!(QueryMetadata::new("SELECT 1", ParamStyle::Named).query_type(), QueryType::Select);
    }
    #[test] fn test_detect_insert() {
        assert_eq!(QueryMetadata::new("INSERT INTO t VALUES(:v)", ParamStyle::Named).query_type(), QueryType::Insert);
    }
    #[test] fn test_detect_update() {
        assert_eq!(QueryMetadata::new("UPDATE t SET a=1", ParamStyle::Named).query_type(), QueryType::Update);
    }
    #[test] fn test_detect_delete() {
        assert_eq!(QueryMetadata::new("DELETE FROM t", ParamStyle::Named).query_type(), QueryType::Delete);
    }
    #[test] fn test_named_params() {
        assert_eq!(QueryMetadata::new("WHERE a=:x AND b=:y", ParamStyle::Named).param_names(), &["x","y"]);
    }
    #[test] fn test_mybatis_params() {
        assert_eq!(QueryMetadata::new("WHERE a=#{x}", ParamStyle::MyBatis).param_names(), &["x"]);
    }
    #[test] fn test_bind_named() {
        let m = QueryMetadata::new("WHERE id=:id AND n=:name", ParamStyle::Named);
        let mut p = HashMap::new();
        p.insert("id".into(), serde_json::json!(1)); p.insert("name".into(), serde_json::json!("A"));
        let (sql, v) = m.bind_params(&p).unwrap();
        assert!(sql.contains("$1")); assert!(sql.contains("$2"));
        assert_eq!(v.len(), 2);
    }
    #[test] fn test_missing_param() {
        let m = QueryMetadata::new("WHERE id=:id AND n=:name", ParamStyle::Named);
        let mut p = HashMap::new(); p.insert("id".into(), serde_json::json!(1));
        assert!(m.bind_params(&p).is_err());
    }
    #[test] fn test_accessors() {
        let m = QueryMetadata::new("SELECT * FROM t WHERE a=:x", ParamStyle::Named);
        assert_eq!(m.sql(), "SELECT * FROM t WHERE a=:x");
        assert_eq!(m.param_style(), ParamStyle::Named);
        assert_eq!(m.query_type(), QueryType::Select);
    }
}
