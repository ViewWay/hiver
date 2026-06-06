//! SQL query generation utilities
//! SQL 查询生成工具
//!
//! # Overview / 概述
//!
//! Provides standalone functions for building SQL queries from wrapper types.
//! 提供从包装类型构建 SQL 查询的独立函数。
//!
//! These functions are used by `QueryExecutor` and can also be used directly
//! for custom query building scenarios.
//! 这些函数被 `QueryExecutor` 使用，也可直接用于自定义查询构建场景。

use hiver_data_commons::{Condition, PageRequest, QueryOrder, QueryWrapper, UpdateWrapper};

use crate::client::QueryParam;

/// Build a SELECT query from a QueryWrapper and table name.
/// 从 QueryWrapper 和表名构建 SELECT 查询。
pub fn build_select_query(wrapper: &QueryWrapper, table: &str) -> (String, Vec<QueryParam>)
{
    let cols = wrapper
        .select
        .as_ref()
        .map(|v| v.join(", "))
        .unwrap_or_else(|| "*".to_string());

    let mut sql = format!("SELECT {} FROM {}", cols, table);

    let (where_clause, params) = build_where_clause(&wrapper.conditions);
    if !where_clause.is_empty()
    {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clause);
    }

    if !wrapper.orders.is_empty()
    {
        let order_clauses: Vec<String> = wrapper
            .orders
            .iter()
            .map(|o| match o
            {
                QueryOrder::Asc(field) => format!("{} ASC", field),
                QueryOrder::Desc(field) => format!("{} DESC", field),
            })
            .collect();
        sql.push_str(" ORDER BY ");
        sql.push_str(&order_clauses.join(", "));
    }

    if let Some(limit) = wrapper.limit
    {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    (sql, params)
}

/// Build a COUNT query from a QueryWrapper and table name.
/// 从 QueryWrapper 和表名构建 COUNT 查询。
pub fn build_count_query(wrapper: &QueryWrapper, table: &str) -> (String, Vec<QueryParam>)
{
    let mut sql = format!("SELECT COUNT(*) AS cnt FROM {}", table);

    let (where_clause, params) = build_where_clause(&wrapper.conditions);
    if !where_clause.is_empty()
    {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clause);
    }

    (sql, params)
}

/// Build a paginated SELECT query from a PageRequest, QueryWrapper, and table name.
/// 从 PageRequest、QueryWrapper 和表名构建分页 SELECT 查询。
pub fn build_page_query(
    page: &PageRequest,
    wrapper: &QueryWrapper,
    table: &str,
) -> (String, Vec<QueryParam>)
{
    let cols = wrapper
        .select
        .as_ref()
        .map(|v| v.join(", "))
        .unwrap_or_else(|| "*".to_string());
    let mut sql = format!("SELECT {} FROM {}", cols, table);

    let (where_clause, params) = build_where_clause(&wrapper.conditions);
    if !where_clause.is_empty()
    {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clause);
    }

    if !wrapper.orders.is_empty()
    {
        let order_clauses: Vec<String> = wrapper
            .orders
            .iter()
            .map(|o| match o
            {
                QueryOrder::Asc(field) => format!("{} ASC", field),
                QueryOrder::Desc(field) => format!("{} DESC", field),
            })
            .collect();
        sql.push_str(" ORDER BY ");
        sql.push_str(&order_clauses.join(", "));
    }

    let offset = (page.page.saturating_sub(1)) * page.size;
    sql.push_str(&format!(" LIMIT {} OFFSET {}", page.size, offset));

    (sql, params)
}

/// Build an UPDATE query from an UpdateWrapper and table name.
/// 从 UpdateWrapper 和表名构建 UPDATE 查询。
pub fn build_update_query(wrapper: &UpdateWrapper, table: &str) -> (String, Vec<QueryParam>)
{
    let mut set_parts = Vec::new();
    let mut params = Vec::new();

    for (idx, (column, value)) in (1u32..).zip(wrapper.sets.iter())
    {
        set_parts.push(format!("{} = ${}", column, idx));
        params.push(QueryParam::from(value.clone()));
    }

    let mut sql = format!("UPDATE {} SET {}", table, set_parts.join(", "));

    let (where_clause, _where_params) = build_where_clause(&wrapper.conditions);
    if !where_clause.is_empty()
    {
        let offset = params.len();
        let (where_sql, where_prms) = build_where_clause_offset(&wrapper.conditions, offset);
        sql.push_str(" WHERE ");
        sql.push_str(&where_sql);
        params.extend(where_prms);
    }

    (sql, params)
}

/// Build a DELETE query from a QueryWrapper and table name.
/// 从 QueryWrapper 和表名构建 DELETE 查询。
pub fn build_delete_query(wrapper: &QueryWrapper, table: &str) -> (String, Vec<QueryParam>)
{
    let mut sql = format!("DELETE FROM {}", table);

    let (where_clause, params) = build_where_clause(&wrapper.conditions);
    if !where_clause.is_empty()
    {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clause);
    }

    (sql, params)
}

/// Build a WHERE clause from conditions with parameter offset 0.
/// 从条件构建 WHERE 子句，参数偏移量为 0。
pub fn build_where_clause(conditions: &[Condition]) -> (String, Vec<QueryParam>)
{
    build_where_clause_offset(conditions, 0)
}

/// Build a WHERE clause from conditions with a parameter offset.
/// 从条件构建带参数偏移量的 WHERE 子句。
pub fn build_where_clause_offset(
    conditions: &[Condition],
    start_idx: usize,
) -> (String, Vec<QueryParam>)
{
    if conditions.is_empty()
    {
        return (String::new(), Vec::new());
    }

    let mut sql = String::new();
    let mut params = Vec::new();
    let mut idx = (start_idx + 1) as u32;

    for (i, condition) in conditions.iter().enumerate()
    {
        if i > 0
        {
            sql.push_str(" AND ");
        }

        match condition
        {
            Condition::Eq { field, value } =>
            {
                sql.push_str(&format!("{} = ${}", field, idx));
                params.push(QueryParam::from(value.clone()));
                idx += 1;
            },
            Condition::Ne { field, value } =>
            {
                sql.push_str(&format!("{} != ${}", field, idx));
                params.push(QueryParam::from(value.clone()));
                idx += 1;
            },
            Condition::Gt { field, value } =>
            {
                sql.push_str(&format!("{} > ${}", field, idx));
                params.push(QueryParam::from(value.clone()));
                idx += 1;
            },
            Condition::Ge { field, value } =>
            {
                sql.push_str(&format!("{} >= ${}", field, idx));
                params.push(QueryParam::from(value.clone()));
                idx += 1;
            },
            Condition::Lt { field, value } =>
            {
                sql.push_str(&format!("{} < ${}", field, idx));
                params.push(QueryParam::from(value.clone()));
                idx += 1;
            },
            Condition::Le { field, value } =>
            {
                sql.push_str(&format!("{} <= ${}", field, idx));
                params.push(QueryParam::from(value.clone()));
                idx += 1;
            },
            Condition::Like { field, pattern } =>
            {
                sql.push_str(&format!("{} LIKE ${}", field, idx));
                params.push(QueryParam::Text(pattern.clone()));
                idx += 1;
            },
            Condition::NotLike { field, pattern } =>
            {
                sql.push_str(&format!("{} NOT LIKE ${}", field, idx));
                params.push(QueryParam::Text(pattern.clone()));
                idx += 1;
            },
            Condition::In { field, values } =>
            {
                let placeholders: Vec<String> = values
                    .iter()
                    .map(|v| {
                        let ph = format!("${}", idx);
                        params.push(QueryParam::from(v.clone()));
                        idx += 1;
                        ph
                    })
                    .collect();
                sql.push_str(&format!("{} IN ({})", field, placeholders.join(", ")));
            },
            Condition::NotIn { field, values } =>
            {
                let placeholders: Vec<String> = values
                    .iter()
                    .map(|v| {
                        let ph = format!("${}", idx);
                        params.push(QueryParam::from(v.clone()));
                        idx += 1;
                        ph
                    })
                    .collect();
                sql.push_str(&format!("{} NOT IN ({})", field, placeholders.join(", ")));
            },
            Condition::IsNull { field } =>
            {
                sql.push_str(&format!("{} IS NULL", field));
            },
            Condition::IsNotNull { field } =>
            {
                sql.push_str(&format!("{} IS NOT NULL", field));
            },
            Condition::Between { field, low, high } =>
            {
                sql.push_str(&format!("{} BETWEEN ${} AND ${}", field, idx, idx + 1));
                params.push(QueryParam::from(low.clone()));
                params.push(QueryParam::from(high.clone()));
                idx += 2;
            },
            Condition::NotBetween { field, low, high } =>
            {
                sql.push_str(&format!("{} NOT BETWEEN ${} AND ${}", field, idx, idx + 1));
                params.push(QueryParam::from(low.clone()));
                params.push(QueryParam::from(high.clone()));
                idx += 2;
            },
            Condition::And(inner) =>
            {
                let (inner_sql, inner_params) =
                    build_where_clause_offset(inner, (idx - 1) as usize);
                idx += inner_params.len() as u32;
                sql.push_str(&format!("({})", inner_sql));
                params.extend(inner_params);
            },
            Condition::Or(inner) =>
            {
                let (inner_sql, inner_params) =
                    build_where_clause_offset(inner, (idx - 1) as usize);
                idx += inner_params.len() as u32;
                sql.push_str(&format!("({})", inner_sql));
                params.extend(inner_params);
            },
        }
    }

    (sql, params)
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use hiver_data_commons::Value;

    use super::*;

    #[test]
    fn test_build_where_clause_eq()
    {
        let conditions = vec![Condition::Eq {
            field: "id".into(),
            value: Value::I64(42),
        }];
        let (sql, params) = build_where_clause(&conditions);
        assert_eq!(sql, "id = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_build_where_clause_multiple()
    {
        let conditions = vec![
            Condition::Eq {
                field: "id".into(),
                value: Value::I64(1),
            },
            Condition::Gt {
                field: "age".into(),
                value: Value::I64(18),
            },
        ];
        let (sql, params) = build_where_clause(&conditions);
        assert_eq!(sql, "id = $1 AND age > $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_like()
    {
        let conditions = vec![Condition::Like {
            field: "name".into(),
            pattern: "%test%".into(),
        }];
        let (sql, params) = build_where_clause(&conditions);
        assert_eq!(sql, "name LIKE $1");
        assert_eq!(params[0], QueryParam::Text("%test%".into()));
    }

    #[test]
    fn test_build_where_clause_in()
    {
        let conditions = vec![Condition::In {
            field: "status".into(),
            values: vec![
                Value::String("active".into()),
                Value::String("pending".into()),
            ],
        }];
        let (sql, params) = build_where_clause(&conditions);
        assert_eq!(sql, "status IN ($1, $2)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_between()
    {
        let conditions = vec![Condition::Between {
            field: "age".into(),
            low: Value::I64(18),
            high: Value::I64(65),
        }];
        let (sql, params) = build_where_clause(&conditions);
        assert_eq!(sql, "age BETWEEN $1 AND $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_where_clause_nested()
    {
        let conditions = vec![
            Condition::Eq {
                field: "a".into(),
                value: Value::I64(1),
            },
            Condition::Or(Box::new(vec![
                Condition::Gt {
                    field: "b".into(),
                    value: Value::I64(2),
                },
                Condition::Lt {
                    field: "c".into(),
                    value: Value::I64(3),
                },
            ])),
        ];
        let (sql, params) = build_where_clause(&conditions);
        assert_eq!(sql, "a = $1 AND (b > $2 AND c < $3)");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_sql_escape()
    {
        assert_eq!("hello".replace('\'', "''"), "hello");
        assert_eq!("it's".replace('\'', "''"), "it''s");
    }
}
